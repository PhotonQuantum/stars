use std::collections::HashMap;
/// Definitions of common structs, enums and traits.
use std::fmt::{Debug, Display, Formatter};

use url::Url;

use crate::registry::TargetRegistry;
use crate::Logger;

/// Convenient alias for boxed error.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// A source of packages.
pub trait Source: 'static + Debug {
    /// Identifier of this source.
    fn name(&self) -> &'static str;
    /// Type of this source.
    fn source_type(&self) -> SourceType;
    /// Check whether the source is available on this system.
    fn available(&self) -> bool;
    /// Snapshot of the source.
    fn snapshot(
        &self,
        logger: &Logger,
        files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError>;
}

/// Type of the source.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SourceType {
    /// Global type. Suitable for system package managers (e.g., apt, pacman).
    Global,
    /// Local type. Suitable for project lockfile (e.g., Cargo.lock). Specify filenames to read.
    Local(&'static [&'static str]),
}

pub trait Target: 'static {
    /// Identifier of this target.
    fn name(&self) -> &'static str;
    /// Check whether the url can be handled by this target.
    /// This function should not touch the system or issue any network requests.
    fn can_handle(&self, url: &Url) -> bool;
    /// Star the package.
    fn star(&self, logger: &Logger, package: &Url) -> Result<(), BoxedError>;
}

/// A package with star handler packed in.
/// Construct one through [`TargetRegistry::pack`](crate::registry::TargetRegistry::pack) method.
#[derive(Debug, Clone)]
pub struct Package {
    /// Name of the package.
    pub name: String,
    /// Url of the package.
    pub url: Url,
    /// Target to star the package.
    pub target: &'static str,
}

impl Package {
    pub const fn new(name: String, url: Url, target: &'static str) -> Self {
        Self { name, url, target }
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
