use crate::app::drivers::home;
use crate::infra::hotreload;
use axum::routing::get;

pub fn add_routes(app: axum::Router, rx: tokio::sync::oneshot::Receiver<()>) -> axum::Router {
    app.route("/ping", get(pong))
        .merge(home::home_routes())
        .merge(hotreload::hot_reload(rx))
}

async fn pong() -> &'static str {
    "pong"
}
