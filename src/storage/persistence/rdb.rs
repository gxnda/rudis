use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use bincode::config::Configuration;
use bincode::serde::{decode_from_std_read, encode_into_std_write};

use crate::storage::memory::StorageEngine;
use crate::storage::persistence::errors::PersistenceError;

pub struct RDB {
    pub path: PathBuf,
    config: Configuration,
}

impl RDB {
    pub fn save(&self, storage: &StorageEngine) -> Result<(), PersistenceError> {
        let mut writer = BufWriter::new(File::create(&self.path)?);
        encode_into_std_write(storage, &mut writer, self.config)?;
        Ok(())
    }

    pub fn get_config(&self) -> Configuration {
        self.config
    }

    pub fn new(path: &Path, config: Configuration) -> Result<Self, PersistenceError> {
        // overwrite existing
        Ok(RDB {
            path: path.to_path_buf(),
            config,
        })
    }

    pub fn load(path: &Path, config: &Configuration) -> Result<StorageEngine, PersistenceError> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        decode_from_std_read(&mut buf_reader, *config)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }
}
