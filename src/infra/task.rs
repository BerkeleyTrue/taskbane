use std::{env, sync::Arc};

use anyhow::{Error, Result};
use taskchampion::{
    storage::{inmemory::InMemoryStorage, Storage},
    Replica, ServerConfig,
};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

pub type ArcMutRep<S> = Arc<Mutex<Replica<S>>>;

pub async fn create_task_storage() -> Result<(ArcMutRep<InMemoryStorage>, ServerConfig)> {
    let storage = InMemoryStorage::new();
    let url = env::var("TASK_URL").expect("No taskserver url provided");

    let client_id = env::var("TASK_CLIENT_ID")
        .map_err(Error::from)
        .and_then(|id| Uuid::parse_str(&id).map_err(Error::from))
        .expect("No task client id provided");

    let encryption_secret: Vec<u8> = env::var("TASK_SECRET")
        .expect("No task secret provided")
        .into();

    let replica = Arc::new(Mutex::new(Replica::new(storage)));

    let server_config = ServerConfig::Remote {
        url,
        client_id,
        encryption_secret,
    };

    Ok((replica, server_config))
}

pub fn start_sync_loop<S: Storage + 'static>(replica: ArcMutRep<S>, config: ServerConfig) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut server = config
                .into_server()
                .await
                .inspect_err(|err| {
                    info!("server err: {err:?}");
                })
                .unwrap();
            info!("sync loop setup");
            loop {
                replica
                    .lock()
                    .await
                    .sync(&mut server, false)
                    .await
                    .inspect_err(|err| {
                        info!("lock err: {err:?}");
                    })
                    .ok();
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        });
    });
}
