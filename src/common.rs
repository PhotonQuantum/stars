//! Definitions of common structs, enums and traits.

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use attohttpc::header::USER_AGENT;
use attohttpc::Session;
use once_cell::sync::Lazy;
use url::Url;

use crate::registry::TargetRegistry;
use crate::{Logger, Persist};

/// Global available HTTP client.
pub static HTTP: Lazy<Session> = Lazy::new(|| {
    let mut session = Session::new();
    session.header(
        USER_AGENT,
        format!("me.lightquantum.stars/{}", env!("CARGO_PKG_VERSION")),
    );
    session
});

/// Convenient alias for boxed error.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// Source of packages.
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
    /// TODO not implemented yet
    #[allow(dead_code)]
    Local(&'static [&'static str]),
}

/// Target platform to star a package.
pub trait Target: 'static + Send + Sync {
    /// Identifier of this target.
    fn name(&self) -> &'static str;
    /// Initialize the target.
    ///
    /// This function will be called first time this target is used to star a package.
    /// `persist` struct can be used to save states across multiple runs.
    ///
    /// Return `true` to indicate a success.
    fn init(&mut self, logger: &Logger, persist: &mut Persist) -> bool;
    /// Check whether the url can be handled by this target.
    /// If can, extract identifier specific to this target from the url.
    ///
    /// This function should not touch the system or issue any network requests.
    fn try_handle(&self, url: &Url) -> Option<String>;
    /// Star the package.
    fn star(&self, logger: &Logger, package: &Package) -> Result<(), BoxedError>;
}

/// A package with star handler packed in.
/// Construct one through [`TargetRegistry::pack`](crate::registry::TargetRegistry::pack) method.
#[derive(Debug, Clone)]
pub struct Package {
    /// Name of the package.
    pub name: String,
    /// Identifier of the url.
    pub identifier: String,
    /// Target to star the package.
    pub target: &'static str,
}

impl Package {
    pub const fn new(name: String, identifier: String, target: &'static str) -> Self {
        Self {
            name,
            identifier,
            target,
        }
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
