use std::string::FromUtf8Error;

use bincode::{error, serde};
use thiserror::Error;

use crate::{command, resp};

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    IO(std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Incomplete command: {0}")]
    IncompleteCommand(String),
    #[error("Resp parse error: {0}")]
    RespParse(resp::ParseError),
    #[error("Error running command: {0}")]
    CommandParse(command::ParseError),
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::IO(e)
    }
}

impl From<FromUtf8Error> for PersistenceError {
    fn from(e: FromUtf8Error) -> Self {
        PersistenceError::Serialization(e.to_string())
    }
}

impl From<serde::EncodeError> for PersistenceError {
    fn from(_: serde::EncodeError) -> Self {
        PersistenceError::Serialization("Serde error encoding".to_string())
    }
}

impl From<error::EncodeError> for PersistenceError {
    fn from(e: error::EncodeError) -> Self {
        PersistenceError::Serialization(e.to_string())
    }
}

impl From<command::ParseError> for PersistenceError {
    fn from(e: command::ParseError) -> Self {
        PersistenceError::CommandParse(e)
    }
}

impl From<resp::ParseError> for PersistenceError {
    fn from(e: resp::ParseError) -> Self {
        PersistenceError::RespParse(e)
    }
}
