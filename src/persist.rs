use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::Logger;

/// Store for persist values.
///
/// This is useful when you want to persist values between runs of the program.
/// E.g., you can use this to store user credentials.
pub struct Persist<'a> {
    kvs: Map<String, Value>,
    path: PathBuf,
    logger: &'a Logger,
}

impl<'a> Persist<'a> {
    /// Loads the persist values from default path.
    pub fn new(logger: &'a Logger, ignore_exist: bool) -> Self {
        let config_dir = directories::ProjectDirs::from("me", "lightquantum", "stars")
            .unwrap()
            .config_dir()
            .to_path_buf();
        if let Err(e) = fs::create_dir_all(&config_dir) {
            logger.warn(format!("Failed to create config directory: {}", e));
        }
        if ignore_exist {
            Self::empty(config_dir.join("persist.json"), logger)
        } else {
            Self::from_path(config_dir.join("persist.json"), logger)
        }
    }
    /// Loads the persist values from the given path.
    pub fn from_path(path: impl AsRef<Path>, logger: &'a Logger) -> Self {
        let path = path.as_ref();
        let content = fs::read(path).unwrap_or_default();
        let kvs = serde_json::from_slice(&*content).unwrap_or_default();
        Self {
            kvs,
            path: path.to_path_buf(),
            logger,
        }
    }
    /// Ignore the persist values and create a new store.
    pub fn empty(path: impl AsRef<Path>, logger: &'a Logger) -> Self {
        Self {
            kvs: Map::new(),
            path: path.as_ref().to_path_buf(),
            logger,
        }
    }
    /// Access the state from a closure and return its output.
    pub fn get_state<T>(&self, f: impl FnOnce(&Map<String, Value>) -> T) -> T {
        f(&self.kvs)
    }
    /// Mutate the state from a closure and persist the changes to disk.
    pub fn with_state(&mut self, f: impl FnOnce(&mut Map<String, Value>)) {
        f(&mut self.kvs);
        if let Err(e) = fs::write(&self.path, serde_json::to_vec(&self.kvs).unwrap()) {
            self.logger.warn(format!("Failed to persist: {}", e));
        }
    }
}
