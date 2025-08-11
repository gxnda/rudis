use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::Mutex};

use crate::{
    command::Command,
    resp::{ParseError, RespValue},
    storage::{memory::StorageEngine, persistence::errors::PersistenceError},
};

pub struct AOF {
    path: PathBuf,
    reader: Option<BufReader<File>>,
    buffer: Vec<u8>,
    write_mutex: Mutex<Option<tokio::fs::File>>,
}

impl AOF {
    pub async fn new(path: PathBuf) -> Result<Self, PersistenceError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        Ok(AOF {
            path,
            reader: None,
            buffer: Vec::new(),
            write_mutex: Mutex::new(Some(file)),
        })
    }

    pub fn ensure_reader(&mut self) -> Result<(), PersistenceError> {
        if self.reader.is_none() {
            let file = File::open(&self.path)?;
            self.reader = Some(BufReader::new(file));
        }
        Ok(())
    }

    pub fn reset_reader(&mut self) -> Result<(), PersistenceError> {
        self.reader = None;
        self.buffer.clear();
        self.ensure_reader()
    }

    pub async fn append_str(&self, resp_str: &str) -> Result<(), PersistenceError> {
        // resp_str is not checked if it is valid.
        let mut file_guard = self.write_mutex.lock().await;
        if file_guard.is_none() {
            *file_guard = Some(
                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&self.path)
                    .await?,
            );
        }
        let file = file_guard.as_mut().unwrap();
        file.write_all(&resp_str.as_bytes()).await?;
        file.flush().await?; // basically saves the file
        Ok(())
    }

    pub async fn append_command(&self, resp_command: RespValue) -> Result<(), PersistenceError> {
        let mut file_guard = self.write_mutex.lock().await;
        if file_guard.is_none() {
            *file_guard = Some(
                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&self.path)
                    .await?,
            );
        }
        let file = file_guard.as_mut().unwrap();
        file.write_all(&resp_command.serialize()).await?;
        file.flush().await?; // basically saves the file
        Ok(())
    }

    fn parse_buffer(&mut self) -> Result<Option<RespValue>, PersistenceError> {
        self.ensure_reader()?;
        let reader = self.reader.as_mut().unwrap();
        loop {
            match RespValue::parse(&self.buffer) {
                Ok((resp, consumed)) => {
                    // if it's successful, remove the bit we read from the buffer
                    self.buffer.drain(..consumed.len());
                    return Ok(Some(resp));
                }
                Err(ParseError::Incomplete) => {
                    // try get more of the file
                    let mut chunk = [0; 1024];
                    match reader.read(&mut chunk) {
                        Ok(0) => {
                            return if self.buffer.is_empty() {
                                Ok(None)
                            } else {
                                let remnants = String::from_utf8(self.buffer.to_vec())?;
                                Err(PersistenceError::IncompleteCommand(remnants))
                            }
                        }
                        Ok(n) => {
                            // get more
                            self.buffer.extend_from_slice(&chunk[..n]);
                        }
                        Err(e) => return Err(PersistenceError::from(e)),
                    }
                }
                Err(e) => return Err(PersistenceError::from(e)),
            }
        }
    }

    pub fn replay_into_storage(
        &mut self,
        storage: StorageEngine,
    ) -> Result<StorageEngine, PersistenceError> {
        // set up storage
        storage.clear();
        self.reset_reader()?; // move reader back to the start
                              // loop through instructions applied to storage
        loop {
            match self.parse_buffer()? {
                Some(resp) => {
                    // apply to storage
                    Command::from_resp(resp)?;
                }
                None => {
                    // EOF
                    return Ok(storage);
                }
            }
        }
    }
}
