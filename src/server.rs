use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

use dashmap::DashMap;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{watch, Semaphore};

use crate::conn_state::ConnState;
use crate::storage::persistence::aof::AOF;
use crate::storage::persistence::errors::PersistenceError;
use crate::Config;
use crate::{
    command::{Command, ParseError},
    connection::{Connection, ConnectionError},
    storage::memory::StorageEngine,
};
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
    connection_semaphore: Arc<Semaphore>,
    timeout_handler: Arc<TimeoutHandler>,
}

impl Server {
    pub async fn new(
        config: Arc<Config>,
        storage: Arc<StorageEngine>,
    ) -> Result<(Self, watch::Sender<bool>), PersistenceError> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
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
                timeout_handler: Arc::new(TimeoutHandler::new(3000)),
            },
            shutdown_tx,
        ))
    }

    pub async fn run(&mut self) {
        let timeout_handler = self.timeout_handler.clone();
        let shutdown_rx_watcher = self.shutdown_rx.clone();
        tokio::spawn(async move {
            // double clone here :((
            timeout_handler.watch(&shutdown_rx_watcher).await;
        });
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
                        let conn = Connection::new(stream, aof);
                        let _id = self.timeout_handler.add(conn.get_state());
                        // async move: it moves all variables into tokio, so permit is dropped when
                        // it completes.
                        tokio::spawn(async move {
                            let _permit = permit;
                            if let Err(e) = Self::handle_connection(conn, storage).await {
                                eprintln!("Connection error: {e}");
                            };
                        });
                    }
                    Err(e) => eprintln!("Connection failed: {:?}", e),
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
    ) -> Result<(), ServerError> {
        // all AOF is handled in parse_buffer of Connections
        const READ_TIMEOUT_MS: u64 = 3000;
        const CHUNK_SIZE: usize = 8192;
        while let Some(resp) = conn
            .prealloced_read_frame(READ_TIMEOUT_MS, &CHUNK_SIZE)
            .await?
        {
            let cmd: Command = Command::from_resp(resp).map_err(ServerError::Parse)?;
            conn.write_response(cmd.execute(&storage)).await?;
        }
        // I don't think I need to manually remove connection here from timeout handler because of
        // the weak reference, which means it gets popped when its done
        Ok(())
    }
}

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);
pub struct TimeoutHandler {
    conns: DashMap<u64, Weak<ConnState>>,
    timeout_ms: u64,
}

impl TimeoutHandler {
    pub fn new(timeout_ms: u64) -> Self {
        TimeoutHandler {
            conns: DashMap::new(),
            timeout_ms,
        }
    }

    pub fn add(&self, conn: Arc<ConnState>) -> u64 {
        let id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        self.conns.insert(id, Arc::downgrade(&conn));
        id
    }

    pub fn remove(&self, id: u64) -> bool {
        self.conns.remove(&id).is_some()
    }

    pub async fn check_all(&self) {
        self.conns
            .retain(|_, weak_state| match weak_state.upgrade() {
                Some(state) => {
                    let expired = state.is_timed_out(self.timeout_ms);
                    if expired {
                        state.shutdown();
                    }
                    !expired
                }
                _ => false, // been dealloced already
            });
    }

    pub async fn watch(&self, shutdown_rx_watcher: &watch::Receiver<bool>) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000)); // check every second
        let mut watcher = shutdown_rx_watcher.clone();
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.check_all().await;
                }
                _ = watcher.changed() => {
                    break;
                }
            }
        }
    }
}
