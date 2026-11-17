use std::sync::Arc;

use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

use crate::storage::persistence::aof::AOF;
use crate::storage::persistence::errors::PersistenceError;
use crate::{
    command::{Command, ParseError},
    connection::{Connection, ConnectionError},
    storage::memory::StorageEngine,
};
use crate::{config::RuntimeConfig, Config};
use coarsetime::Updater;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Disconnected")]
    Disconnected,

    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Parse Error: {0}")]
    Parse(ParseError),

    #[error("Error: {0}")]
    Error(String),
}

// For some reason #[from] is broken for ParseError,
impl From<ParseError> for ServerError {
    fn from(err: ParseError) -> Self {
        ServerError::Parse(err)
    }
}

pub struct Server {
    listener: TcpListener,
    storage: Arc<StorageEngine>,
    shutdown_rx: watch::Receiver<bool>,
    aof: Option<Arc<AOF>>,
    runtime_config: Arc<RuntimeConfig>,
}

impl Server {
    pub async fn new(
        config: Arc<Config>,
        storage: Arc<StorageEngine>,
    ) -> Result<(Self, watch::Sender<bool>), PersistenceError> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let runtime_config = Arc::new(RuntimeConfig::new((*config).clone()));
        Updater::new(100).start()?;
        Ok((
            Server {
                listener: TcpListener::bind(config.socket_addr()).await?,
                storage,
                shutdown_rx,
                aof: match config.appendonly {
                    true => Some(Arc::new(AOF::new(config).await?)),
                    false => None,
                },
                runtime_config,
            },
            shutdown_tx,
        ))
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                conn = self.listener.accept() => match conn {
                    Ok((stream, _)) => {
                        let permit = match self.runtime_config.try_acquire() {
                            Some(permit) => permit,
                            None => {
                                eprintln!("Max connections reached, dropping connection");
                                continue;
                            }
                        };
                        stream.set_nodelay(true).unwrap();
                        let storage = self.storage.clone();
                        let aof = self.aof.clone();
                        let runtime_config = Arc::clone(&self.runtime_config);
                        let conn = Connection::new(stream, aof);
                        // async move: it moves all variables into tokio, so permit is dropped when
                        // it completes.
                        tokio::spawn(async move {
                            let _permit = permit;
                            if let Err(e) =
                                Self::handle_connection(conn, storage, runtime_config).await
                            {
                                eprintln!("Connection error: {e}");
                            };
                        });
                    }
                    Err(e) => eprintln!("Connection failed: {e:?}"),
                },
                _ = self.shutdown_rx.changed() => {
                    break;
                }
            }
        }
    }

    async fn handle_connection(
        mut conn: Connection<TcpStream>,
        storage: Arc<StorageEngine>,
        runtime_config: Arc<RuntimeConfig>,
    ) -> Result<(), ServerError> {
        // all AOF is handled in parse_buffer of Connections TODO: Change that lmao
        while let Some(resp) = conn.read_frame().await? {
            let cmd: Command = Command::from_resp(resp).map_err(ServerError::Parse)?;
            let response = match &cmd {
                Command::Config { .. } => cmd.execute_config(&runtime_config),
                _ => cmd.execute(&storage),
            };
            conn.write_response(response).await?;
        }
        Ok(())
    }
}
