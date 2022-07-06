use std::path::Path;

use super::error::Result;
use super::storage::{Bitcask, Storage};

pub struct KvStore {
    storage: Box<dyn Storage>,
}

impl KvStore {
    pub fn open(path: &Path) -> Result<KvStore> {
        let storage = Bitcask::open(path.to_path_buf())?;
        Ok(KvStore { 
            storage: Box::new(storage),
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.storage.get(key)
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.storage.set(key, value)
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        self.storage.remove(key)
    }
}