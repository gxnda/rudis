use crate::resp::RespValue;
use crate::storage::memory::StorageEngine;
use bytes::BytesMut;
use std::net::TcpStream;
use std::sync::Arc;
use std::{io, time::SystemTimeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("RESP protocol error: {0}")]
    RespParse(String),

    #[error("Connection timed out")]
    Timeout,

    #[error("Invalid command format: {0}")]
    InvalidCommand(String),

    #[error("Client disconnected")]
    Disconnected,

    #[error("System clock error: {0}")]
    ClockError(#[from] SystemTimeError),

    #[error("Server error: {0}")]
    Server(String),
}

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
    db: Arc<StorageEngine>,
}

impl Connection {
    pub fn read_frame(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        // Reads complete RESP objects from stream
        todo!();
    }

    pub fn write_response(&mut self, response: RespValue) -> Result<(), ConnectionError> {
        todo!();
    }
}
