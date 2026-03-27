use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

pub type ArcMut<S> = Arc<Mutex<S>>;
pub type ArcRw<S> = Arc<RwLock<S>>;
