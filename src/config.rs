use clap::{builder::RangedU64ValueParser, Parser};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Port to listen on
    #[arg(long, default_value_t = 6379)]
    pub port: u16,

    /// Address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: IpAddr,

    /// Directory for persistence files
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,

    /// RDB filename
    #[arg(long, default_value = "dump.rdb")]
    pub dbfilename: PathBuf,

    /// Enable AOF persistence (yes or no)
    #[arg(
        long,
        default_value = "yes",
        action = clap::ArgAction::Set,
        value_parser = parse_yes_no
    )]
    pub appendonly: bool,

    /// AOF filename
    #[arg(long, default_value = "appendonly.aof")]
    pub appendfilename: PathBuf,

    /// Maximum number of connections
    #[arg(long, default_value_t = 100000, value_parser = RangedU64ValueParser::<usize>::new().range(1..))]
    pub maxclients: usize,

    /// Initial storage size
    #[arg(long, default_value_t = 512)]
    pub storage_init_size: usize,
}

fn parse_yes_no(value: &str) -> Result<bool, String> {
    if value.eq_ignore_ascii_case("yes") {
        Ok(true)
    } else if value.eq_ignore_ascii_case("no") {
        Ok(false)
    } else {
        Err("expected 'yes' or 'no'".to_string())
    }
}

impl Config {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind, self.port)
    }

    pub fn rdb_path(&self) -> PathBuf {
        self.dir.join(&self.dbfilename)
    }

    pub fn aof_path(&self) -> PathBuf {
        self.dir.join(&self.appendfilename)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 6379,
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            dir: PathBuf::from("."),
            dbfilename: PathBuf::from("dump.rdb"),
            appendonly: true,
            appendfilename: PathBuf::from("appendonly.aof"),
            maxclients: 100000,
            storage_init_size: 512,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigSetError {
    #[error("Unknown CONFIG parameter: {0}")]
    UnknownParameter(String),
    #[error("CONFIG parameter is not changeable at runtime: {0}")]
    ReadOnlyParameter(String),
    #[error("Invalid value for CONFIG parameter {parameter}: {value}")]
    InvalidValue { parameter: String, value: String },
}

#[derive(Debug)]
struct ConnectionState {
    max_clients: usize,
    active_connections: usize,
}

#[derive(Debug)]
pub struct RuntimeConfig {
    config: Config,
    connections: Mutex<ConnectionState>,
}

pub struct ConnectionPermit {
    runtime_config: Arc<RuntimeConfig>,
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        let mut state = self.runtime_config.connections.lock().unwrap();
        state.active_connections -= 1;
    }
}

impl RuntimeConfig {
    pub fn new(config: Config) -> Self {
        Self {
            connections: Mutex::new(ConnectionState {
                max_clients: config.maxclients,
                active_connections: 0,
            }),
            config,
        }
    }

    pub fn try_acquire(self: &Arc<Self>) -> Option<ConnectionPermit> {
        let mut state = self.connections.lock().unwrap();
        if state.active_connections >= state.max_clients {
            return None;
        }
        state.active_connections += 1;
        Some(ConnectionPermit {
            runtime_config: Arc::clone(self),
        })
    }

    pub fn parameters(&self) -> Vec<(String, String)> {
        let max_clients = self.connections.lock().unwrap().max_clients;
        vec![
            ("bind".to_string(), self.config.bind.to_string()),
            ("port".to_string(), self.config.port.to_string()),
            (
                "dir".to_string(),
                self.config.dir.to_string_lossy().into_owned(),
            ),
            (
                "dbfilename".to_string(),
                path_to_string(&self.config.dbfilename),
            ),
            (
                "appendonly".to_string(),
                if self.config.appendonly { "yes" } else { "no" }.to_string(),
            ),
            (
                "appendfilename".to_string(),
                path_to_string(&self.config.appendfilename),
            ),
            ("maxclients".to_string(), max_clients.to_string()),
            (
                "storage-init-size".to_string(),
                self.config.storage_init_size.to_string(),
            ),
        ]
    }

    pub fn set_parameter(&self, parameter: &str, value: &str) -> Result<(), ConfigSetError> {
        if parameter.eq_ignore_ascii_case("maxclients") {
            let max_clients = value
                .parse::<usize>()
                .ok()
                .filter(|max_clients| *max_clients > 0)
                .ok_or_else(|| ConfigSetError::InvalidValue {
                    parameter: parameter.to_string(),
                    value: value.to_string(),
                })?;
            self.connections.lock().unwrap().max_clients = max_clients;
            return Ok(());
        }

        if self
            .parameters()
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(parameter))
        {
            Err(ConfigSetError::ReadOnlyParameter(parameter.to_string()))
        } else {
            Err(ConfigSetError::UnknownParameter(parameter.to_string()))
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_redis_style_options_and_values() {
        let config = Config::try_parse_from([
            "rudisd",
            "--port",
            "6380",
            "--bind",
            "0.0.0.0",
            "--dir",
            "/tmp/redis",
            "--dbfilename",
            "snapshot.rdb",
            "--appendonly",
            "no",
            "--appendfilename",
            "append.aof",
            "--maxclients",
            "250",
        ])
        .unwrap();

        assert_eq!(config.port, 6380);
        assert_eq!(config.bind, "0.0.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(config.socket_addr().to_string(), "0.0.0.0:6380");
        assert_eq!(config.rdb_path(), PathBuf::from("/tmp/redis/snapshot.rdb"));
        assert_eq!(config.aof_path(), PathBuf::from("/tmp/redis/append.aof"));
        assert!(!config.appendonly);
        assert_eq!(config.maxclients, 250);
    }

    #[test]
    fn rejects_non_redis_appendonly_values() {
        assert!(Config::try_parse_from(["rudisd", "--appendonly", "true"]).is_err());
    }

    #[test]
    fn runtime_maxclients_updates_connection_limit() {
        let runtime = Arc::new(RuntimeConfig::new(Config {
            maxclients: 1,
            ..Config::default()
        }));
        let permit = runtime.try_acquire().unwrap();
        assert!(runtime.try_acquire().is_none());
        runtime.set_parameter("MAXCLIENTS", "2").unwrap();
        let second_permit = runtime.try_acquire().unwrap();
        assert!(runtime.try_acquire().is_none());
        drop(permit);
        drop(second_permit);
        assert_eq!(
            runtime
                .parameters()
                .into_iter()
                .find(|(name, _)| name == "maxclients")
                .unwrap()
                .1,
            "2"
        );
    }
}
