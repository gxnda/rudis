use std::sync::Arc;

use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Semaphore};

use crate::storage::persistence::aof::AOF;
use crate::storage::persistence::errors::PersistenceError;
use crate::Config;
use crate::{
    command::{Command, ParseError},
    connection::{Connection, ConnectionError},
    storage::memory::StorageEngine,
};
use coarsetime::{Duration, Updater};

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
    shutdown_rx: oneshot::Receiver<()>,
    aof: Option<Arc<AOF>>,
    connection_semaphore: Arc<Semaphore>,
}

impl Server {
    pub async fn new(
        config: Arc<Config>,
        storage: Arc<StorageEngine>,
    ) -> Result<(Self, oneshot::Sender<()>), PersistenceError> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        Updater::new(100).start()?;
        Ok((
            Server {
                listener: TcpListener::bind(config.addr).await?,
                storage,
                shutdown_rx,
                connection_semaphore: Arc::new(Semaphore::new(config.max_connections)),
                aof: match config.aof {
                    true => Some(Arc::new(AOF::new(config).await?)),
                    false => None,
                },
            },
            shutdown_tx,
        ))
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                conn = self.listener.accept() => match conn {
                    Ok((stream, _)) => {
                        let permit = match self.connection_semaphore.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => {
                                eprintln!("Max connections reached, dropping connection");
                                continue;
                            }
                        };
                        let storage = self.storage.clone();
                        let aof = self.aof.clone();
                        // async move: it moves all variables into tokio, so permit is dropped when
                        // it completes.
                        tokio::spawn(async move {
                            let _permit = permit;
                            if let Err(e) = Self::handle_connection(stream, storage, aof).await {
                                eprintln!("Connection error: {e}");
                            };
                        });
                    }
                    Err(e) => eprintln!("Connection failed: {:?}", e),
                },
                _ = &mut self.shutdown_rx => {
                    break;
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        storage: Arc<StorageEngine>,
        aof: Option<Arc<AOF>>,
    ) -> Result<(), ServerError> {
        // all AOF is handled in parse_buffer of Connections
        let mut conn = Connection::new(stream, aof);
        const READ_TIMEOUT: Duration = Duration::from_secs(3);
        const CHUNK_SIZE: usize = 8192;
        loop {
            // TODO: Is this loop actually used? What happens if a command is incomplete here
            match conn
                .prealloced_read_frame(&READ_TIMEOUT, &CHUNK_SIZE)
                .await?
            {
                Some(resp) => {
                    let cmd: Command = Command::from_resp(resp).map_err(ServerError::Parse)?;
                    conn.write_response(cmd.execute(&storage)).await?;
                }
                None => {
                    // Connection closed
                    break;
                }
            }
        }
        Ok(())
    }
}
