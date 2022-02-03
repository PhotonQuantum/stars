/// Definitions of common structs, enums and traits.
use std::fmt::{Debug, Display};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A source of packages.
pub trait Source: Debug {
    /// Type of the source.
    const TYPE: SourceType;
    /// Check whether the source is available on this system.
    fn available() -> bool;
    /// Snapshot of the source.
    fn snapshot(&self) -> Result<Vec<Box<dyn Package>>, Error>;
}

/// Type of the source.
#[derive(Debug)]
pub enum SourceType {
    /// Global type. Suitable for system package managers (e.g., apt, pacman).
    Global,
    /// Local type. Suitable for project lockfile (e.g., Cargo.lock). Specify filenames to read.
    Local(&'static [&'static str]),
}

pub trait Stargazer {
    /// Type of the package.
    type Package: Package;
    /// Star the package.
    fn star(&self, package: &str) -> Result<(), Error>;
}

pub trait Package: Display {}