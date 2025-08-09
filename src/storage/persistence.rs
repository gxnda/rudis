use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use bincode::config::Configuration;
use bincode::serde::{decode_from_std_read, encode_into_std_write};
use thiserror::Error;

use crate::storage::memory::StorageEngine;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    IO(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub struct RDB {
    pub path: PathBuf,
    config: Configuration,
}

impl RDB {
    pub fn save(&self, storage: &StorageEngine) -> Result<(), PersistenceError> {
        if self.path.is_dir() {
            return Err(PersistenceError::IO(
                "Path given is a directory".to_string(),
            ));
        }
        let mut writer = BufWriter::new(
            File::create(&self.path).map_err(|e| PersistenceError::IO(e.to_string()))?,
        );
        encode_into_std_write(storage, &mut writer, self.config)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        Ok(())
    }

    pub fn get_config(&self) -> Configuration {
        self.config
    }

    pub fn new(path: &Path, config: Configuration) -> Result<Self, PersistenceError> {
        if path.is_dir() {
            return Err(PersistenceError::IO(
                "Path given was a directory.".to_string(),
            ));
        }

        // overwrite existing
        Ok(RDB {
            path: path.to_path_buf(),
            config,
        })
    }

    pub fn load(path: &Path, config: &Configuration) -> Result<StorageEngine, PersistenceError> {
        let file = File::open(path).map_err(|e| PersistenceError::IO(e.to_string()))?;
        let mut buf_reader = BufReader::new(file);
        decode_from_std_read(&mut buf_reader, *config)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }
}
