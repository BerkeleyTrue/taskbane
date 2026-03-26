use std::sync::Arc;

use tokio::sync::Mutex;

pub type ArcMut<S> = Arc<Mutex<S>>;
