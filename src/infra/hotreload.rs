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

pub fn hot_reload(on_start_rx: oneshot::Receiver<()>) -> Router {
    // convert the oneshot receiver into a broadcast channel sender
    // so that we can send a start event when the server starts
    // to all clients that are connected to the SSE endpoint
    let (tx, _) = broadcast::channel(16);
    let tx_clone = tx.clone();

    // connect the oneshot receiver to the broadcast sender
    tokio::spawn(async move {
        if on_start_rx.await.is_ok() {
            let _ = tx_clone.send(());
        }
    });

    async fn get_hot_reload(
        State(tx): State<broadcast::Sender<()>>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
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
        // we make tx shared state
        // so that it can be accessed by the handler
        .with_state(tx)
}
