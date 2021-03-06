use std::cell::RefCell;
use std::collections::HashMap;

use url::Url;

use crate::common::{BoxedError, Package, Source, Target};
use crate::{Logger, Persist, TargetRegistry};

#[derive(Default)]
struct DebugTarget(RefCell<Vec<Package>>);

impl Target for DebugTarget {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn init(&mut self, _logger: &Logger, _persist: &mut Persist) -> bool {
        true
    }

    fn try_handle(&self, url: &Url) -> Option<String> {
        Some(url.to_string())
    }

    fn star(&self, _logger: &Logger, package: &Package) -> Result<(), BoxedError> {
        self.0.borrow_mut().push(package.clone());
        Ok(())
    }
}

pub fn test_source(
    source: &impl Source,
    files: HashMap<&str, &[u8]>,
    assert: impl FnOnce(&[Package]),
) {
    if !source.available() {
        eprintln!("{} not present, skipped", source.name());
        return;
    }

    let logger = Logger::new(false);
    let mut persist = Persist::new(&logger, false);

    let mut targets = TargetRegistry::new(&logger, &mut persist);
    targets.register(DebugTarget::default());

    let packages = source.snapshot(&logger, files, &targets).unwrap();

    assert(&*packages);
}
