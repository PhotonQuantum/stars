use std::collections::HashMap;
use std::fs;

use tap::TapFallible;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType, Target};
use crate::Logger;

/// Registry for targets.
pub struct TargetRegistry<'a> {
    targets: HashMap<&'static str, Box<dyn Target>>,
    logger: &'a Logger,
}

impl<'a> TargetRegistry<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self {
            targets: Default::default(),
            logger,
        }
    }
    pub fn register(&mut self, target: impl Target) {
        if let Some(collided) = self.targets.insert(target.name(), Box::new(target)) {
            panic!("target collision: {}", collided.name());
        }
    }
    pub fn deregister(&mut self, name: &str) -> bool {
        self.targets.remove(name).is_some()
    }
    pub fn pack(&self, name: String, url: Url) -> Option<Package> {
        self.targets
            .iter()
            .find(|(_, target)| target.can_handle(&url))
            .map(|(id, _)| Package::new(name, url, *id))
    }
    pub fn star(&self, package: &Package) -> Result<(), BoxedError> {
        self.targets.get(&package.target).map_or_else(
            || Err(format!("no such target found: {}", package.target).into()),
            |target| target.star(self.logger, &package.url),
        )
    }
}

/// Registry for sources.
pub struct SourceRegistry<'a> {
    sources: Vec<Box<dyn Source>>,
    logger: &'a Logger,
}

impl<'a> SourceRegistry<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self {
            sources: vec![],
            logger,
        }
    }
    pub fn register(&mut self, source: impl Source) {
        if source.available() {
            self.sources.push(Box::new(source));
        }
    }
    pub fn deregister(&mut self, name: &str) -> bool {
        if let Some((idx, _)) = self
            .sources
            .iter()
            .enumerate()
            .find(|(_, s)| s.name() == name)
        {
            self.sources.remove(idx);
            true
        } else {
            false
        }
    }
    pub fn aggregate(&self, targets: &TargetRegistry) -> Vec<Package> {
        let cache: HashMap<&str, Vec<u8>> = self
            .sources
            .iter()
            .flat_map(|source| {
                if let SourceType::Local(filenames) = source.source_type() {
                    filenames
                        .iter()
                        .filter_map(|filename| fs::read(filename).map(|c| (*filename, c)).ok())
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect();
        let global_mode = cache.is_empty();

        self.sources
            .iter()
            .flat_map(|source| match source.source_type() {
                SourceType::Global if global_mode => source
                    .snapshot(self.logger, HashMap::new(), targets)
                    .tap_err(|e| {
                        self.logger
                            .warn(format!("failed to snapshot {}: {}", source.name(), e));
                    })
                    .unwrap_or_default(),
                SourceType::Local(filenames) if !global_mode => filenames
                    .iter()
                    .fold(Some(HashMap::new()), |acc, x| {
                        acc.and_then(|mut acc| {
                            cache.get(x).map(|file| {
                                acc.insert(*x, &file[..]);
                                acc
                            })
                        })
                    })
                    .map_or(vec![], |files| {
                        source
                            .snapshot(self.logger, files, targets)
                            .tap_err(|e| {
                                self.logger.warn(format!(
                                    "failed to snapshot {}: {}",
                                    source.name(),
                                    e
                                ));
                            })
                            .unwrap_or_default()
                    }),
                _ => vec![],
            })
            .collect()
    }
}
