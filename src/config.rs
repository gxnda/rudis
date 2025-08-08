use std::{net::SocketAddr, path::PathBuf};

use thiserror::Error;

pub struct Config {
    pub addr: SocketAddr,
    pub rdb_path: PathBuf,
    pub aof_enabled: bool,
    pub aof_path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {}

impl Config {
    pub fn load() -> Result<Config, ConfigError> {
        todo!();
    }
}
