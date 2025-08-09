use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    IO(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}
