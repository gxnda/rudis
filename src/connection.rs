use crate::resp::{ParseError, RespValue};
use bytes::Buf;
use bytes::BytesMut;
use std::time::Duration;
use std::{io, time::SystemTimeError};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::timeout;

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

pub struct Connection<S> {
    stream: S,
    buffer: BytesMut,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S) -> Self {
        Connection {
            stream,
            buffer: BytesMut::new(),
        }
    }

    fn parse_buffer(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        let buf = self.buffer.as_ref();
        println!("Parsed: {:?}", RespValue::parse(buf));
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
        const READ_TIMEOUT: Duration = Duration::from_secs(1);
        let mut temp_ref = [0u8; 1024];
        self.buffer.reserve(1024);
        // try and parse self.buffer
        if let Some(frame) = self.parse_buffer()? {
            return Ok(Some(frame));
        }

        // if buffer is not ready yet
        match timeout(READ_TIMEOUT, self.stream.read(&mut temp_ref)).await {
            Ok(Ok(0)) => {
                // nothing else sent
                if self.buffer.is_empty() {
                    // nothing else to look at
                    return Ok(None);
                } else {
                    return Err(ConnectionError::Disconnected);
                }
            }
            Ok(Ok(n)) => {
                // add read data to buffer
                self.buffer.extend_from_slice(&temp_ref[..n]);
                return self.parse_buffer(); // uses recursion instead of while loop to stop
                                            // timeout on incomplete commands
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => {
                // handle a non blocking stream (idk what this is)
                return Ok(None);
            }
            Ok(Err(e)) => {
                println!("Error: {:?}", e);
                return Err(e.into());
            }
            Err(_elapsed) => return Err(ConnectionError::Timeout),
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

#[cfg(test)]
mod connection_tests {
    use super::*;
    use bytes::Bytes;
    use tokio::io::duplex;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_read_simple_frame() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server);

        client.write_all(b"+OK\r\n").await.unwrap();

        let frame = conn.read_frame().await.unwrap();
        assert_eq!(frame, Some(RespValue::SimpleString("OK".into())));
    }

    #[tokio::test]
    async fn test_read_incomplete_frame() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server);

        client.write_all(b"*2\r\n$3\r\nGET\r\n").await.unwrap();

        let frame = conn.read_frame().await.unwrap();
        assert!(frame.is_none());

        client.write_all(b"$3\r\nkey\r\n").await.unwrap();
        let frame = conn.read_frame().await.unwrap();
        assert_eq!(
            frame,
            Some(RespValue::Array(Some(vec![
                RespValue::BulkString(Some(Bytes::from("GET"))),
                RespValue::BulkString(Some(Bytes::from("key"))),
            ])))
        );
    }

    #[tokio::test]
    async fn test_read_multiple_frames() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server);

        client
            .write_all(b"*1\r\n$3\r\nGET\r\n*1\r\n$3\r\nSET\r\n")
            .await
            .unwrap();

        let frame1 = conn.read_frame().await.unwrap();
        assert_eq!(
            frame1,
            Some(RespValue::Array(Some(vec![RespValue::BulkString(Some(
                Bytes::from("GET")
            ))])))
        );

        let frame2 = conn.read_frame().await.unwrap();
        assert_eq!(
            frame2,
            Some(RespValue::Array(Some(vec![RespValue::BulkString(Some(
                Bytes::from("SET")
            )),])))
        );
    }

    #[tokio::test]
    async fn test_write_response() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server);

        conn.write_response(RespValue::SimpleString("OK".into()))
            .await
            .unwrap();

        let mut buf = [0u8; 5];
        client.read_exact(&mut buf[..]).await.unwrap();
        assert_eq!(&buf, b"+OK\r\n");
    }

    #[tokio::test]
    async fn test_parse_error() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server);

        client.write_all(b"invalid\r\n").await.unwrap();

        let result = conn.read_frame().await;
        assert!(matches!(result, Err(ConnectionError::RespParse(_))));
    }
}
