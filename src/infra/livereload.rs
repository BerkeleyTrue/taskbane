use std::{convert::Infallible, path::Path, time::Duration};

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use derive_more::Display;
use futures::stream::{self, StreamExt};
use notify::{RecursiveMode, Watcher};
use tokio::sync::{broadcast, oneshot};
use tokio_stream::{
    wrappers::{BroadcastStream, IntervalStream},
    Stream,
};
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Debug, Clone, Display)]
enum Msg {
    ServerStart,
    StyleChange,
}

type LiveReloadState = (
    broadcast::Sender<Msg>,
    std::sync::Arc<tokio::sync::Mutex<Option<oneshot::Receiver<()>>>>,
    CancellationToken,
);

pub fn live_reload(
    on_start_rx: oneshot::Receiver<()>,
    shutdown_token: CancellationToken,
) -> Router {
    // convert the oneshot receiver into a broadcast channel sender
    // so that we can send a start event when the server starts
    // to all clients that are connected to the SSE endpoint
    let (tx, _) = broadcast::channel(16);

    // We'll share the oneshot receiver through an Arc<Mutex<Option<...>>>
    // so it can be consumed when the first client connects
    let on_start_rx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(on_start_rx)));

    let mut watcher = notify::recommended_watcher({
        let tx = tx.clone();
        move |res| match res {
            Ok(notify::Event {
                kind: notify::EventKind::Modify(_),
                ..
            }) => {
                let _ = tx.send(Msg::StyleChange);
            }
            Err(err) => info!("Err notify: {err:?}"),
            _ => (),
        }
    })
    .unwrap();

    watcher
        .watch(Path::new("./public/css"), RecursiveMode::NonRecursive)
        .unwrap();

    tokio::spawn({
        let shutdown_token = shutdown_token.clone();
        async move {
            shutdown_token.cancelled().await;
            drop(watcher);
        }
    });

    async fn get_live_reload(
        State((tx, on_start_rx, shutdown_token)): State<LiveReloadState>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        // Start listening for the start signal when first client connects
        tokio::spawn({
            let tx = tx.clone();
            let on_start_rx = on_start_rx.clone();
            async move {
                let mut guard = on_start_rx.lock().await;
                if let Some(rx) = guard.take() {
                    if rx.await.is_ok() {
                        let _ = tx.send(Msg::ServerStart);
                    }
                }
            }
        });
        // tx.subscribe() creates a new receiver for each client
        let start_stream = BroadcastStream::new(tx.subscribe())
            .take_until(shutdown_token.clone().cancelled_owned())
            .map(|msg| match msg {
                Ok(Msg::StyleChange) => {
                    Ok(Event::default().event("reload-style").data("style changed"))
                }
                Ok(Msg::ServerStart) => Ok(Event::default().event("start").data("server started")),
                Err(err) => Ok(Event::default()
                    .event("error")
                    .data(format!("err: {err:?}"))),
            });

        let heartbeat_stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
            .take_until(shutdown_token.clone().cancelled_owned())
            .map(|_| Ok(Event::default().event("heartbeat").data("ping")));

        let combined_stream = stream::select(start_stream, heartbeat_stream);

        Sse::new(combined_stream).keep_alive(KeepAlive::default())
    }

    Router::new()
        .route("/__livereload", get(get_live_reload))
        // we make tx and on_start_rx shared state
        // so that they can be accessed by the handler
        .with_state((tx, on_start_rx, shutdown_token))
}
