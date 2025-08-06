use std::{convert::Infallible, time::Duration};

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use futures::stream::{self, StreamExt};
use tokio::sync::{broadcast, oneshot};
use tokio_stream::{
    wrappers::{BroadcastStream, IntervalStream},
    Stream,
};
use tracing::info;

pub fn hot_reload(on_start_rx: oneshot::Receiver<()>) -> Router {
    // convert the oneshot receiver into a broadcast channel sender
    // so that we can send a start event when the server starts
    // to all clients that are connected to the SSE endpoint
    let (tx, _) = broadcast::channel(16);

    // We'll share the oneshot receiver through an Arc<Mutex<Option<...>>>
    // so it can be consumed when the first client connects
    let on_start_rx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(on_start_rx)));

    async fn get_hot_reload(
        State((tx, on_start_rx)): State<(
            broadcast::Sender<()>,
            std::sync::Arc<tokio::sync::Mutex<Option<oneshot::Receiver<()>>>>,
        )>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        // Start listening for the start signal when first client connects
        let tx_clone = tx.clone();
        let on_start_rx_clone = on_start_rx.clone();
        tokio::spawn(async move {
            let mut guard = on_start_rx_clone.lock().await;
            if let Some(rx) = guard.take() {
                if rx.await.is_ok() {
                    info!("Server started, sending start event to clients");
                    let _ = tx_clone.send(());
                }
            }
        });
        // tx.subscribe() creates a new receiver for each client
        let start_stream = BroadcastStream::new(tx.subscribe())
            .map(|_| Ok(Event::default().event("start").data("server started")));

        let heartbeat_stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
            .map(|_| Ok(Event::default().event("heartbeat").data("ping")));

        let combined_stream = stream::select(start_stream, heartbeat_stream);

        Sse::new(combined_stream).keep_alive(KeepAlive::default())
    }

    Router::new()
        .route("/__hotreload", get(get_hot_reload))
        // we make tx and on_start_rx shared state
        // so that they can be accessed by the handler
        .with_state((tx, on_start_rx))
}
