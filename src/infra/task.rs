use std::env;

use taskchampion::{storage::inmemory::InMemoryStorage, Replica, Server, ServerConfig};
use uuid::Uuid;

pub async fn create_task_storage() -> anyhow::Result<(Replica<InMemoryStorage>, Box<dyn Server>)> {
    let storage = InMemoryStorage::new();
    let url = env::var("TASK_URL").expect("No taskserver url provided");

    let client_id = env::var("TASK_CLIENT_ID")
        .map_err(anyhow::Error::from)
        .and_then(|id| Uuid::parse_str(&id).map_err(anyhow::Error::from))
        .expect("No task client id provided");

    let encryption_secret: Vec<u8> = env::var("TASK_SECRET")
        .expect("No task secret provided")
        .into();

    let replica = Replica::new(storage);

    let server_config = ServerConfig::Remote {
        url,
        client_id,
        encryption_secret,
    };
    let server = server_config
        .into_server()
        .await
        .expect("Could not create taskserver");

    Ok((replica, server))
}
