use std::{env, sync::{Arc, Mutex}};

use taskchampion::{
    storage::{InMemoryStorage, Storage}, Replica, Server, ServerConfig
};
use uuid::Uuid;

struct TaskSync {
    replica: Replica<InMemoryStorage>,
    server: Arc<Mutex<dyn Server + Send>>,
}

impl TaskSync {
    fn new(url: String, client_id: Uuid, encryption_secret: Vec<u8>, replica: Replica<InMemoryStorage>) -> Self {
        
        let server = ServerConfig::Remote {
            url,
            client_id,
            encryption_secret,
        };

        TaskSync {
            server: server.into_server().expect("Could not create taskserver"),
            replica: replica
        }
    }

    fn sync(&mut self) {
        self.replica.sync(self.server.clone(), true);
    }
}

pub fn create_task_storage() -> (Replica<Storage>, TaskSync) {
    let storage = InMemoryStorage::new();
    let url = &env::var("TASK_URL").expect("No taskserver url provided").to_string();
    let client_id = &env::var("TASK_CLIENT_ID").expect("No task client id provided").to_string();
    let encryped_secret = &env::var("TASK_SECRET").expect("No task secret provided").to_string();
    let replica = Replica::new(storage);
    let task_sync = TaskSync::new(url, client_id, encryped_secret, replica);
    (replica, task_sync)
}
