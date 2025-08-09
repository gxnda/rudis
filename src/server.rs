use std::{io, net::SocketAddr, sync::Arc};

use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

use crate::{
    command::{Command, ParseError},
    connection::{Connection, ConnectionError},
    storage::memory::StorageEngine,
};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Disconnected")]
    Disconnected,

    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Parse Error")]
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
}

impl Server {
    pub async fn new(
        addr: SocketAddr,
        storage: Arc<StorageEngine>,
    ) -> Result<(Self, oneshot::Sender<()>), io::Error> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        Ok((
            Server {
                listener: TcpListener::bind(addr).await?,
                storage,
                shutdown_rx,
            },
            shutdown_tx,
        ))
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                conn = self.listener.accept() => match conn {
                    Ok((stream, _)) => {
                        let storage = self.storage.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, storage).await {
                                eprintln!("Connection error: {e}")
                            }
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
    ) -> Result<(), ServerError> {
        let mut conn = Connection::new(stream);
        loop {
            match conn.read_frame().await {
                Ok(Some(resp)) => {
                    let cmd: Command =
                        Command::from_resp(resp).map_err(|e| ServerError::Parse(e))?;
                    conn.write_response(cmd.execute(&storage)).await?;
                }
                Ok(None) => {
                    // Connection closed
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}
