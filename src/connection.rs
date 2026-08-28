use crate::conn_state::ConnState;
use crate::resp::{ParseError, RespValue};
use crate::storage::persistence::aof::AOF;
use bytes::BytesMut;
use std::sync::Arc;
use std::{io, time::SystemTimeError};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Notify;

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

    #[error("Error attempting to write to AOF for backups: {0}")]
    AofError(String),
}

pub struct Connection<S> {
    stream: S,
    buffer: BytesMut,
    state: Arc<ConnState>,
    aof: Option<Arc<AOF>>,
    shutdown_notify: Arc<Notify>,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S, aof: Option<Arc<AOF>>) -> Self {
        let notify = Arc::new(Notify::new());
        Connection {
            stream,
            buffer: BytesMut::new(),
            aof,
            state: Arc::new(ConnState::new(notify.clone())),
            shutdown_notify: notify,
        }
    }

    pub fn get_state(&self) -> Arc<ConnState> {
        self.state.clone()
    }

    async fn parse_buffer(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        match RespValue::rough_check(&self.buffer) {
            Ok(end_index) => {
                let frozen_buffer = self.buffer.split_to(end_index).freeze();
                match RespValue::parse_checked(&frozen_buffer) {
                    Ok((resp, consumed_len)) => {
                        if let Some(aof) = &self.aof {
                            aof.append_bytes(&frozen_buffer.slice(..consumed_len))
                                .await
                                .map_err(|e| ConnectionError::AofError(e.to_string()))?;
                        }
                        Ok(Some(resp))
                    }
                    Err(e) => Err(ConnectionError::RespParse(e.to_string())),
                }
            }
            Err(ParseError::Incomplete) => Ok(None),
            Err(e) => Err(ConnectionError::RespParse(e.to_string())),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<RespValue>, ConnectionError> {
        // Reads complete RESP objects from stream

        if !self.buffer.is_empty() {
            if let Some(frame) = self.parse_buffer().await? {
                return Ok(Some(frame));
            }
        }

        loop {
            tokio::select! {
                read_result = self.stream.read_buf(&mut self.buffer) => {
                    match read_result {
                        Ok(0) => {
                            if self.buffer.is_empty() {
                                return Ok(None);
                            } else {
                                // if we get nothing more but there's still stuff in the buffer)
                                return Err(ConnectionError::Disconnected);
                            }
                        }
                        Ok(_n) => {
                            self.state.touch();
                            if let Some(frame) = self.parse_buffer().await? {
                                return Ok(Some(frame));
                            }
                            // else continue looping, still not complete and we're still getting data
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // if kernel buffer is empty
                            if self.buffer.is_empty() {
                                return Ok(None);
                            } else {
                                // if we get nothing more but there's still stuff in the buffer
                                return Err(ConnectionError::Disconnected);
                            }
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                _ = self.shutdown_notify.notified() => {
                    return Err(ConnectionError::Disconnected)
                }


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

#[cfg(test)]
mod connection_tests {
    use std::time::Duration;

    use crate::server::TimeoutHandler;

    use super::*;
    use bytes::Bytes;
    use coarsetime::Updater;
    use tokio::io::duplex;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_read_simple_frame() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server, None);

        client.write_all(b"+OK\r\n").await.unwrap();

        let frame = conn.read_frame().await.unwrap();
        assert_eq!(frame, Some(RespValue::SimpleString("OK".into())));
    }

    #[tokio::test]
    async fn test_read_incomplete_frame() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server, None);
        Updater::new(10).start().unwrap();
        let conn_state = conn.get_state();
        let (watcher_shutdown_tx, watcher_shutdown_rx) = tokio::sync::watch::channel(false);

        tokio::spawn(async move {
            let t_h = TimeoutHandler::new(500);
            t_h.add(conn_state);
            t_h.watch(watcher_shutdown_rx).await;
        });

        client.write_all(b"*2\r\n$3\r\nGET\r\n").await.unwrap();

        tokio::time::sleep(Duration::from_millis(1000)).await;

        assert!(watcher_shutdown_tx.send(true).is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(conn.read_frame().await.is_err());

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
        let mut conn = Connection::new(server, None);

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
        let mut conn = Connection::new(server, None);

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
        let mut conn = Connection::new(server, None);

        client.write_all(b"invalid inline\r\n").await.unwrap();

        let result = conn.read_frame().await;
        assert!(matches!(result, Err(ConnectionError::RespParse(_))));
    }

    #[tokio::test]
    async fn test_timeout() {
        let (_client, server) = duplex(1024);
        let mut conn = Connection::new(server, None);

        // No data written to client
        match conn.read_frame().await {
            Err(ConnectionError::Timeout) => {} // Expected
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_clean_disconnect() {
        let (client, server) = duplex(1024);
        let mut conn = Connection::new(server, None);
        drop(client); // Close client end immediately

        match conn.read_frame().await {
            Ok(None) => {} // Expected clean disconnect
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_disconnect_with_partial_command() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server, None);

        client.write_all(b"*2\r\n$3\r\nGET\r\n$").await.unwrap();
        drop(client); // Disconnect after partial command

        // First read: partial command -> None
        let res = conn.read_frame().await;
        assert!(res.is_err());

        // Second read: detects disconnect with partial buffer
        match conn.read_frame().await {
            Err(ConnectionError::Disconnected) => {} // Expected
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_multiple_commands_in_single_read() {
        let (mut client, server) = duplex(1024);
        let mut conn = Connection::new(server, None);

        // Corrected test data - ensure commands are properly framed
        client
            .write_all(b"*1\r\n$4\r\nCMD1\r\n*1\r\n$4\r\nCMD2\r\n")
            .await
            .unwrap();

        let frame1 = conn.read_frame().await.unwrap();
        assert_eq!(
            frame1,
            Some(RespValue::Array(Some(vec![RespValue::BulkString(Some(
                Bytes::from("CMD1")
            ))])))
        );

        let frame2 = conn.read_frame().await.unwrap();
        assert_eq!(
            frame2,
            Some(RespValue::Array(Some(vec![RespValue::BulkString(Some(
                Bytes::from("CMD2")
            ))])))
        );
    }

    #[tokio::test]
    async fn test_large_command() {
        let (mut client, server) = duplex(4096);
        let mut conn = Connection::new(server, None);

        let large_value = vec![b'a'; 2048];
        let cmd = format!(
            "*2\r\n$3\r\nSET\r\n${}\r\n{}\r\n",
            large_value.len(),
            String::from_utf8_lossy(&large_value)
        );

        client.write_all(cmd.as_bytes()).await.unwrap();

        // Loop until we get the complete frame
        let frame = loop {
            match conn.read_frame().await {
                Ok(Some(frame)) => break frame,
                Ok(None) => continue,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        };
        match frame {
            RespValue::Array(Some(v)) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], RespValue::BulkString(Some(Bytes::from("SET"))));
                assert_eq!(v[1], RespValue::BulkString(Some(Bytes::from(large_value))));
            }
            other => panic!("Unexpected frame: {:?}", other),
        }
    }
}
