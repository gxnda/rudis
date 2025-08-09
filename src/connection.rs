use crate::resp::{ParseError, RespValue};
use bytes::Buf;
use bytes::BytesMut;
use std::{io, time::SystemTimeError};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            buffer: BytesMut::new(),
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
            Err(e) => Err(ConnectionError::RespParse(e.to_string())),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        // Reads complete RESP objects from stream
        let mut temp_ref = [0u8; 1024];
        self.buffer.reserve(1024);
        loop {
            // try and parse self.buffer
            if let Some(frame) = self.parse_buffer()? {
                return Ok(Some(frame));
            }

            // if buffer is not ready yet
            match self.stream.read(&mut temp_ref).await {
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
                    // TODO
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub async fn write_response(&mut self, response: RespValue) -> Result<(), ConnectionError> {
        let serialized = response.serialize();
        self.stream
            .write_all(&serialized)
            .await
            .map_err(ConnectionError::Io)?;
        self.stream.flush().await.map_err(ConnectionError::Io)?;
        Ok(())
    }
}
