use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::storage::memory::StorageEngine;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    IO(String),
}

pub struct RDB {
    pub path: PathBuf,
}

impl RDB {
    pub fn save(&self, storage: &StorageEngine) -> Result<(), PersistenceError> {
        todo!();
    }

    pub fn new(path: &Path) -> Result<Self, PersistenceError> {
        if path.is_dir() {
            return Err(PersistenceError::IO(
                "Path given was a directory.".to_string(),
            ));
        }

        // overwrite existing
        Ok(RDB {
            path: path.to_path_buf(),
        })
    }

    pub fn load() -> Result<Self, PersistenceError> {
        todo!();
    }
}
