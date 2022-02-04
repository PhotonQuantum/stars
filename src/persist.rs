use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// Store for persist values.
pub struct Persist {
    kvs: Map<String, Value>,
    path: PathBuf,
}

impl Persist {
    pub fn new() -> Self {
        let config_dir = directories::ProjectDirs::from("me", "lightquantum", "stars")
            .unwrap()
            .config_dir()
            .to_path_buf();
        fs::create_dir_all(&config_dir).unwrap();
        Self::from_path(config_dir.join("persist.json"))
    }
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let content = fs::read(path).unwrap_or_default();
        let kvs = serde_json::from_slice(&*content).unwrap_or_default();
        Self {
            kvs,
            path: path.to_path_buf(),
        }
    }
    pub fn get_state<T>(&self, f: impl FnOnce(&Map<String, Value>) -> T) -> T {
        f(&self.kvs)
    }
    pub fn with_state(&mut self, f: impl FnOnce(&mut Map<String, Value>)) {
        f(&mut self.kvs);
        fs::write(&self.path, serde_json::to_vec(&self.kvs).unwrap()).unwrap();
    }
}
