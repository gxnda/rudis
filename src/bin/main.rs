use std::{error::Error, sync::Arc};

use clap::Parser;
use rudis::{memory::StorageEngine, Config, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::parse();
    dbg!(&config);
    let storage: StorageEngine = StorageEngine::with_capacity(config.storage_init_size);
    let (mut server, _sender) =
        Server::new(Arc::new(config.clone()), Arc::new(storage.clone())).await?;

    server.run().await;

    Ok(())
}
