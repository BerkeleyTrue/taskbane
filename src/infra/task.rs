#![allow(unused_variables)]
use std::{env, fmt::Display, sync::Arc};

use anyhow::{Error, Result};
use async_trait::async_trait;
use derive_more::Constructor;
use sqlx::{query, Sqlite, SqlitePool, Transaction};
use taskchampion::{
    server::VersionId,
    storage::{inmemory::InMemoryStorage, Storage, StorageTxn, TaskMap},
    Error as TcError, Operation, Replica, ServerConfig,
};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::types::ArcRw;

pub type ArcRep<S> = ArcRw<Replica<S>>;

pub async fn create_task_storage() -> Result<(ArcRep<InMemoryStorage>, ServerConfig)> {
    let storage = InMemoryStorage::new();
    let url = env::var("TASK_URL").expect("No taskserver url provided");

    let client_id = env::var("TASK_CLIENT_ID")
        .map_err(Error::from)
        .and_then(|id| Uuid::parse_str(&id).map_err(Error::from))
        .expect("No task client id provided");

    let encryption_secret: Vec<u8> = env::var("TASK_SECRET")
        .expect("No task secret provided")
        .into();

    let replica = Arc::new(RwLock::new(Replica::new(storage)));

    let server_config = ServerConfig::Remote {
        url,
        client_id,
        encryption_secret,
    };

    Ok((replica, server_config))
}

pub fn start_sync_loop<S: Storage + Sync + 'static>(replica: ArcRep<S>, config: ServerConfig) {
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
                    .write()
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

type TcResult<T> = std::result::Result<T, taskchampion::Error>;

#[derive(Constructor)]
struct SqlxStorage {
    conn: SqlitePool,
}

#[async_trait]
impl Storage for SqlxStorage {
    async fn txn<'a>(&'a mut self) -> TcResult<Box<dyn StorageTxn + Send + 'a>> {
        let tx = self.conn.begin().await.map_err(map_to_tc_err)?;
        Ok(Box::new(Txn::new(self, Some(tx))))
    }
}

#[derive(Constructor)]
struct Txn<'t> {
    storage: &'t mut SqlxStorage,
    tx: Option<Transaction<'t, Sqlite>>,
}

impl<'t> Txn<'t> {
    fn get_txn(&mut self) -> TcResult<&mut Transaction<'t, Sqlite>> {
        self.tx
            .as_mut()
            .ok_or(TcError::Database("transaction already commited".to_owned()))
    }
}

#[async_trait]
impl StorageTxn for Txn<'_> {
    /// Get an (immutable) task, if it is in the storage
    async fn get_task(&mut self, uuid: Uuid) -> TcResult<Option<TaskMap>> {
        let tx = self.get_txn()?;
        let task = query!("SELECT data FROM taskdb_tasks WHERE uuid = ? LIMIT 1", uuid)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_to_tc_err)?
            .map(|r| serde_json::from_str::<TaskMap>(&r.data))
            .transpose()
            .map_err(map_to_tc_err)?;
        Ok(task)
    }

    /// Get a vector of all pending tasks from the working_set
    async fn get_pending_tasks(&mut self) -> TcResult<Vec<(Uuid, TaskMap)>> {
        let tx = self.get_txn()?;
        let res = query!(
            r#"
            SELECT taskdb_tasks.* 
            FROM taskdb_tasks 
            JOIN taskdb_working_set ON taskdb_tasks.uuid = taskdb_working_set.uuid
            "#
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(map_to_tc_err)?
        .into_iter()
        .map(|r| -> TcResult<_> {
            let uuid = r
                .uuid
                .map(|s| Uuid::parse_str(&s))
                .expect("pk is never null??")
                .map_err(map_to_tc_err)?;
            let taskmap = serde_json::from_str::<TaskMap>(&r.data).map_err(map_to_tc_err)?;
            Ok((uuid, taskmap))
        })
        .collect::<TcResult<Vec<_>>>()?;

        Ok(res)
    }

    /// Create an (empty) task, only if it does not already exist.  Returns true if
    /// the task was created (did not already exist).
    async fn create_task(&mut self, uuid: Uuid) -> TcResult<bool> {
        todo!()
        // verify no task of uuid exists
        // query!("SELECT count(uuid) FROM taskdb_tasks WHERE uuid = ?", uuid)
        //     .fetch_optional(&mut *self.tx)
        //     .await
    }

    /// Set a task, overwriting any existing task.  If the task does not exist, this implicitly
    /// creates it (use `get_task` to check first, if necessary).
    async fn set_task(&mut self, uuid: Uuid, task: TaskMap) -> TcResult<()> {
        todo!();
    }

    /// Delete a task, if it exists.  Returns true if the task was deleted (already existed)
    async fn delete_task(&mut self, uuid: Uuid) -> TcResult<bool> {
        todo!();
    }

    /// Get the uuids and bodies of all tasks in the storage, in undefined order.
    async fn all_tasks(&mut self) -> TcResult<Vec<(Uuid, TaskMap)>> {
        todo!();
    }

    /// Get the uuids of all tasks in the storage, in undefined order.
    async fn all_task_uuids(&mut self) -> TcResult<Vec<Uuid>> {
        todo!();
    }

    /// Get the current base_version for this storage -- the last version synced from the server.
    /// If no version has been set, this returns the nil version.
    async fn base_version(&mut self) -> TcResult<VersionId> {
        todo!();
    }

    /// Set the current base_version for this storage.
    async fn set_base_version(&mut self, version: VersionId) -> TcResult<()> {
        todo!();
    }

    /// Get the set of operations for the given task.
    async fn get_task_operations(&mut self, uuid: Uuid) -> TcResult<Vec<Operation>> {
        todo!();
    }

    /// Get the current set of outstanding operations (operations that have not been synced to the
    /// server yet)
    async fn unsynced_operations(&mut self) -> TcResult<Vec<Operation>> {
        todo!();
    }

    /// Get the current set of outstanding operations (operations that have not been synced to the
    /// server yet)
    async fn num_unsynced_operations(&mut self) -> TcResult<usize> {
        todo!();
    }

    /// Add an operation to the end of the list of operations in the storage.  Note that this
    /// merely *stores* the operation; it is up to the TaskDb to apply it.
    async fn add_operation(&mut self, op: Operation) -> TcResult<()> {
        todo!();
    }

    /// Remove an operation from the end of the list of operations in the storage.  The operation
    /// must exactly match the most recent operation, and must not be synced. Note that like
    /// `add_operation` this only affects the list of operations.
    async fn remove_operation(&mut self, op: Operation) -> TcResult<()> {
        todo!();
    }

    /// A sync has been completed, so all operations should be marked as synced. The storage
    /// may perform additional cleanup at this time.
    async fn sync_complete(&mut self) -> TcResult<()> {
        todo!();
    }

    /// Get the entire working set, with each task UUID at its appropriate (1-based) index.
    /// Element 0 is always None.
    async fn get_working_set(&mut self) -> TcResult<Vec<Option<Uuid>>> {
        todo!();
    }

    /// Add a task to the working set and return its (one-based) index.  This index will be one greater
    /// than the highest used index.
    async fn add_to_working_set(&mut self, uuid: Uuid) -> TcResult<usize> {
        todo!();
    }

    /// Update the working set task at the given index.  This cannot add a new item to the
    /// working set.
    async fn set_working_set_item(&mut self, index: usize, uuid: Option<Uuid>) -> TcResult<()> {
        let tx = self.get_txn()?;
        let index = index as i64;

        match uuid {
            Some(uuid) => {
                query!(
                    "INSERT OR REPLACE INTO taskdb_working_set (id, uuid) VALUES (?, ?)",
                    index,
                    uuid
                )
                .execute(&mut **tx)
                .await
            }
            None => {
                query!("DELETE FROM taskdb_working_set WHERE id = ?", index)
                    .execute(&mut **tx)
                    .await
            }
        }
        .map_err(map_to_tc_err)?;

        Ok(())
    }

    /// Clear all tasks from the working set in preparation for a renumbering operation.
    /// Note that this is the only way items are removed from the set.
    async fn clear_working_set(&mut self) -> TcResult<()> {
        let tx = self.get_txn()?;
        query!("DELETE FROM taskdb_working_set")
            .execute(&mut **tx)
            .await
            .map_err(map_to_tc_err)?;
        Ok(())
    }

    /// Commit any changes made in the transaction.  It is an error to call this more than
    /// once.
    async fn commit(&mut self) -> TcResult<()> {
        self.tx
            .take()
            .ok_or(TcError::Database(
                "Transaction already commited!".to_owned(),
            ))?
            .commit()
            .await
            .map_err(map_to_tc_err)
        // Ok(())
    }
}

fn map_to_tc_err<E: Display>(err: E) -> TcError {
    TcError::Database(err.to_string())
}
