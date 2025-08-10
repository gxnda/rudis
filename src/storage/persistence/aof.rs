use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use crate::{
    command::Command,
    resp::{ParseError, RespValue},
    storage::{memory::StorageEngine, persistence::errors::PersistenceError},
};

pub struct AOF {
    path: PathBuf,
    reader: Option<BufReader<File>>,
    buffer: Vec<u8>,
}

impl AOF {
    pub fn new(path: PathBuf) -> Self {
        AOF {
            path,
            reader: None,
            buffer: Vec::new(),
        }
    }

    pub fn ensure_reader(&mut self) -> Result<(), PersistenceError> {
        if self.reader.is_none() {
            let file = File::open(&self.path)?;
            self.reader = Some(BufReader::new(file));
        }
        Ok(())
    }

    pub fn append_str(&self, resp_str: &str) -> Result<(), PersistenceError> {
        // resp_str is not checked if it is valid.
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;
        write!(file, "{}", resp_str)?;
        Ok(())
    }
    pub fn append_command(&self, resp_command: RespValue) -> Result<(), PersistenceError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;
        let serialized: String = String::from_utf8(resp_command.serialize())?;
        write!(file, "{}", serialized)?;
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

    pub fn rewrite_storage(
        &mut self,
        storage: StorageEngine,
    ) -> Result<StorageEngine, PersistenceError> {
        // set up storage
        storage.clear();
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
