use std::{
    fs::File,
    io::{BufReader, Read},
    sync::Arc,
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::Mutex};

use crate::{
    command::Command,
    resp::{ParseError, RespValue},
    storage::{memory::StorageEngine, persistence::errors::PersistenceError},
    Config,
};

pub struct AOF {
    config: Arc<Config>,
    reader: Option<BufReader<File>>,
    buffer: Vec<u8>,
    write_mutex: Mutex<Option<tokio::fs::File>>,
}

impl AOF {
    pub async fn new(config: Arc<Config>) -> Result<Self, PersistenceError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.aof_path)
            .await?;
        Ok(AOF {
            config,
            reader: None,
            buffer: Vec::new(),
            write_mutex: Mutex::new(Some(file)),
        })
    }

    pub fn ensure_reader(&mut self) -> Result<(), PersistenceError> {
        if self.reader.is_none() {
            let file = File::open(&self.config.aof_path)?;
            self.reader = Some(BufReader::new(file));
        }
        Ok(())
    }

    pub fn reset_reader_and_buffer(&mut self) -> Result<(), PersistenceError> {
        self.reader = None;
        self.buffer.clear();
        self.ensure_reader()
    }

    pub async fn append_str(&self, resp_str: &str) -> Result<(), PersistenceError> {
        // resp_str is not checked if it is valid.
        self.append_bytes(resp_str.as_bytes()).await
    }

    pub async fn append_command(&self, resp_command: &RespValue) -> Result<(), PersistenceError> {
        self.append_bytes(&resp_command.serialize()).await
    }

    pub async fn append_bytes(&self, bytes: &[u8]) -> Result<(), PersistenceError> {
        // resp_str is not checked if it is valid.
        let mut file_guard = self.write_mutex.lock().await;
        if file_guard.is_none() {
            *file_guard = Some(
                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&self.config.aof_path)
                    .await?,
            );
        }
        let file = file_guard.as_mut().unwrap();
        file.write_all(bytes).await?;
        file.flush().await?; // basically saves the file
        Ok(())
    }

    fn parse_buffer(&mut self) -> Result<Option<RespValue>, PersistenceError> {
        self.ensure_reader()?;
        let reader = self.reader.as_mut().unwrap();
        loop {
            match RespValue::parse(&self.buffer) {
                Ok((resp, bytes_remaining)) => {
                    // if it's successful, remove the bit we read from the buffer
                    let consumed = self.buffer.len() - bytes_remaining.len();
                    self.buffer.drain(..consumed); // consumed is &[u8]
                    return Ok(Some(resp));
                }
                Err(ParseError::Incomplete) => {
                    // try get more of the file
                    let mut chunk = [0; 1024];
                    match reader.read(&mut chunk) {
                        Ok(0) => {
                            if self.buffer.is_empty() {
                                return Ok(None);
                            } else {
                                return Err(PersistenceError::IncompleteCommand(
                                    String::from_utf8_lossy(&self.buffer).to_string(),
                                ));
                            };
                        }
                        Ok(n) => {
                            // get more
                            self.buffer.extend_from_slice(&chunk[..n]);
                            continue;
                        }
                        Err(e) => {
                            return Err(PersistenceError::from(e));
                        }
                    }
                }
                Err(e) => {
                    return Err(PersistenceError::from(e));
                }
            }
        }
    }

    pub fn replay_into_storage(
        &mut self,
        storage: StorageEngine,
    ) -> Result<StorageEngine, PersistenceError> {
        // set up storage
        storage.clear();
        self.reset_reader_and_buffer()?; // move reader back to the start
                                         // loop through instructions applied to storage
        loop {
            match self.parse_buffer()? {
                Some(resp) => {
                    // apply to storage
                    let command: Command = Command::from_resp(resp)?;
                    let _resp = command.execute(&storage);
                }
                None => {
                    // EOF
                    return Ok(storage);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_aof_logging() {
        let dir = tempdir().unwrap();
        let mut config: Config = Config::default();
        config.aof_path = dir.path().join("test.aof");
        let mut aof = AOF::new(Arc::new(config.clone())).await.unwrap(); // Remove Arc

        let cmd = Command::Set {
            key: Bytes::from(b"k".to_vec()),
            value: Bytes::from(b"v".to_vec()),
            ttl: None,
            condition_type: None,
            condition_val: None,
            get: false,
            keep_ttl: false,
        };
        assert!(aof.append_command(&cmd.to_resp()).await.is_ok());

        // Verify AOF contains serialized command
        let contents = tokio::fs::read(&config.aof_path).await.unwrap();
        assert!(contents.starts_with(b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$1\r\nv\r\n"));

        let storage_to_use = StorageEngine::with_capacity(100);
        match aof.replay_into_storage(storage_to_use) {
            Ok(storage) => {
                assert!(storage.get(&Bytes::from(b"k".to_vec())).is_some())
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }
}
