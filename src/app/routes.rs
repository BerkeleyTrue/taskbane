use crate::app::drivers::home;
use crate::infra::hotreload;
use axum::routing::get;
use tokio_util::sync::CancellationToken;

pub fn add_routes(
    app: axum::Router,
    rx: tokio::sync::oneshot::Receiver<()>,
    shutdown_token: CancellationToken,
) -> axum::Router {
    app.route("/ping", get(pong))
        .merge(home::home_routes())
        .merge(hotreload::hot_reload(rx, shutdown_token))
}

async fn pong() -> &'static str {
    "pong"
}
