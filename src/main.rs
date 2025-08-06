mod app;
mod infra;

use crate::app::routes::add_routes;
use crate::infra::axum::start_server;
use axum::Router;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let (tx, rx) = oneshot::channel();
    let shutdown_token = tokio_util::sync::CancellationToken::new();

    // build our application with a route
    let app = Router::new();
    let app = add_routes(app, rx, shutdown_token.clone());

    start_server(app, tx, shutdown_token).await;
}
