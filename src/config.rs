use std::{
    net::{AddrParseError, SocketAddr},
    path::PathBuf,
};

use clap::{builder::RangedU64ValueParser, Parser};
use thiserror::Error;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Socket address to bind
    #[arg(long, default_value = "127.0.0.1:6379")]
    pub addr: SocketAddr,

    /// Path to the RDB file
    #[arg(long, default_value = "./dump.rdb")]
    pub rdb_path: PathBuf,

    /// Enable AOF persistence
    #[arg(long, default_value_t = true)]
    pub aof_enabled: bool,

    /// Path to the AOF file
    #[arg(long, default_value = "./appendonly.aof")]
    pub aof_path: PathBuf,

    /// Maximum number of connections
    #[arg(long, default_value_t = 100000, value_parser = RangedU64ValueParser::<usize>::new().range(1..))]
    pub max_connections: usize,

    /// Initial storage size
    #[arg(long, default_value_t = 512)]
    pub storage_init_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:6379".parse().unwrap(),
            aof_path: PathBuf::from("./appendonly.aof"),
            aof_enabled: true,
            rdb_path: PathBuf::from("./dump.rdb"),
            max_connections: 100000,
            storage_init_size: 512,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),
}

impl From<AddrParseError> for ConfigError {
    fn from(value: AddrParseError) -> Self {
        ConfigError::InvalidArgument(value.to_string())
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(value: std::num::ParseIntError) -> Self {
        ConfigError::InvalidArgument(value.to_string())
    }
}
