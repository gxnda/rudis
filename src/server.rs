use std::{net::TcpStream, sync::Arc};

use thiserror::Error;

use crate::storage::memory::StorageEngine;

#[derive(Debug, Error)]
pub enum ServerError {}

pub fn run(config: &Config, storage: Arc<StorageEngine>) -> Result<(), ServerError> {
    // create a main server loop in tokio
    todo!();
}

pub fn handle_connection(stream: TcpStream, storage: Arc<StorageEngine>) {
    todo!();
}
