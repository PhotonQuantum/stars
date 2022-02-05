use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// Store for persist values.
///
/// This is useful when you want to persist values between runs of the program.
/// E.g., you can use this to store user credentials.
pub struct Persist {
    kvs: Map<String, Value>,
    path: PathBuf,
}

impl Persist {
    /// Loads the persist values from default path.
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("me", "lightquantum", "stars")
            .unwrap()
            .config_dir()
            .to_path_buf();
        fs::create_dir_all(&config_dir).unwrap();
        Self::from_path(config_dir.join("persist.json"))
    }
    /// Loads the persist values from the given path.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let content = fs::read(path).unwrap_or_default();
        let kvs = serde_json::from_slice(&*content).unwrap_or_default();
        Self {
            kvs,
            path: path.to_path_buf(),
        }
    }
    /// Access the state from a closure and return its output.
    pub fn get_state<T>(&self, f: impl FnOnce(&Map<String, Value>) -> T) -> T {
        f(&self.kvs)
    }
    /// Mutate the state from a closure and persist the changes to disk.
    pub fn with_state(&mut self, f: impl FnOnce(&mut Map<String, Value>)) {
        f(&mut self.kvs);
        fs::write(&self.path, serde_json::to_vec(&self.kvs).unwrap()).unwrap();
    }
}
