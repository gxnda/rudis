use crate::resp::{ParseError, RespValue};
use crate::storage::memory::StorageEngine;
use bytes::Buf;
use bytes::BytesMut;
use std::io::{Read, Write};
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
    pub fn new(db: Arc<StorageEngine>, stream: TcpStream) -> Self {
        Connection {
            stream,
            buffer: BytesMut::new(),
            db,
        }
    }

    fn parse_buffer(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        let buf = self.buffer.as_ref();
        match RespValue::parse(buf) {
            Ok((resp, consumed)) => {
                self.buffer.advance(consumed.len()); // done with these bytes now, they're boring
                Ok(Some(resp))
            }
            Err(ParseError::Incomplete) => Ok(None),
            Err(_) => Err(ConnectionError::RespParse(
                "Error reading buffer".to_string(),
            )),
        }
    }

    fn read_frame(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        // Reads complete RESP objects from stream
        let mut temp_ref = [0u8; 1024];
        loop {
            // try and parse self.buffer
            if let Some(frame) = self.parse_buffer()? {
                return Ok(Some(frame));
            }

            // if buffer is not ready yet
            match self.stream.read(&mut temp_ref) {
                Ok(0) => {
                    // nothing else sent
                    if self.buffer.is_empty() {
                        // nothing else to look at
                        return Ok(None);
                    } else {
                        return Err(ConnectionError::Disconnected);
                    }
                }
                Ok(n) => {
                    // add read data to buffer
                    self.buffer.extend_from_slice(&temp_ref[..n])
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // handle a non blocking stream (idk what this is)
                    // todo
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub fn write_response(&mut self, response: RespValue) -> Result<(), ConnectionError> {
        let serialized = response.serialize();
        self.stream
            .write_all(&serialized)
            .map_err(ConnectionError::Io)?;
        self.stream.flush().map_err(ConnectionError::Io)?;
        Ok(())
    }
}
