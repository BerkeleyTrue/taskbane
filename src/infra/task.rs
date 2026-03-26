use std::env;

use anyhow::Result;
use taskchampion::{storage::inmemory::InMemoryStorage, Replica, Server, ServerConfig};
use uuid::Uuid;

pub struct ServerConf {
    url: String,
    client_id: Uuid,
    encryption_secret: Vec<u8>,
}

pub async fn create_task_storage() -> anyhow::Result<(Replica<InMemoryStorage>, ServerConf)> {
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

    let server_config = ServerConf {
        url,
        client_id,
        encryption_secret,
    };
    // let server = server_config
    //     .into_server()
    //     .await
    //     .expect("Could not create taskserver");

    Ok((replica, server_config))
}

impl ServerConf {
    pub async fn into_server(&self) -> Result<Box<dyn Server>> {
        ServerConfig::Remote {
            url: self.url.clone(),
            client_id: self.client_id,
            encryption_secret: self.encryption_secret.clone(),
        }
        .into_server()
        .await
        .map_err(anyhow::Error::from)
    }
}
