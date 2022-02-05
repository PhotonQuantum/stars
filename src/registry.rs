use std::collections::HashMap;
use std::fs;

use itertools::Itertools;
use tap::TapFallible;
use url::Url;

use crate::common::{Package, Source, SourceType, Target};
use crate::{Logger, Persist};

enum TargetState {
    Uninitialized,
    Initialized,
    Failed,
}

/// Registry for targets.
pub struct TargetRegistry<'a> {
    targets: HashMap<&'static str, (Box<dyn Target>, TargetState)>,
    logger: &'a Logger,
    persist: &'a mut Persist<'a>,
}

impl<'a> TargetRegistry<'a> {
    /// Create a new registry.
    pub fn new(logger: &'a Logger, persist: &'a mut Persist<'a>) -> Self {
        Self {
            targets: Default::default(),
            logger,
            persist,
        }
    }
    /// Register a target.
    pub fn register(&mut self, target: impl Target) {
        if let Some((collided, _)) = self.targets.insert(
            target.name(),
            (Box::new(target), TargetState::Uninitialized),
        ) {
            panic!("target collision: {}", collided.name());
        }
    }
    /// Deregister a target.
    pub fn deregister(&mut self, name: &str) -> bool {
        self.targets.remove(name).is_some()
    }

    /// Try to parse a URL into a package recognized by a target.
    pub fn try_parse(&self, name: String, url: &Url) -> Option<Package> {
        self.targets
            .iter()
            .find_map(|(target_id, (target, _))| Some((target_id, target.try_handle(url)?)))
            .map(|(target_id, package_id)| Package::new(name, package_id, target_id))
    }

    /// Star a package.
    ///
    /// This will attempt to star the package on its specified target.
    /// If the target is not initialized, it will be initialized.
    pub fn star(&mut self, package: &Package) {
        if let Some((target, state)) = self.targets.get_mut(&package.target) {
            loop {
                match state {
                    TargetState::Uninitialized => {
                        *state = if target.init(self.logger, self.persist) {
                            TargetState::Initialized
                        } else {
                            TargetState::Failed
                        };
                    }
                    TargetState::Initialized => {
                        if let Err(e) = target.star(self.logger, package) {
                            self.logger
                                .error(format!("error while starring {}: {}", package, e));
                        }
                        break;
                    }
                    TargetState::Failed => {
                        self.logger.warn(format!(
                            "target {} not loaded, skipped {}",
                            target.name(),
                            package
                        ));
                        break;
                    }
                }
            }
        } else {
            self.logger
                .error(format!("no such target found: {}", package.target));
        }
    }
}

/// Registry for sources.
pub struct SourceRegistry<'a> {
    sources: Vec<Box<dyn Source>>,
    logger: &'a Logger,
}

impl<'a> SourceRegistry<'a> {
    /// Create a new registry.
    pub fn new(logger: &'a Logger) -> Self {
        Self {
            sources: vec![],
            logger,
        }
    }
    /// Register a source.
    pub fn register(&mut self, source: impl Source) {
        if source.available() {
            self.sources.push(Box::new(source));
        }
    }
    /// Deregister a source.
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
    /// Aggregate packages from all sources.
    ///
    /// If a local source is matched, all global sources will be ignored.
    /// Files will be read and passed to the local source.
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
            .unique_by(|package| package.identifier.clone())
            .collect()
    }
}
