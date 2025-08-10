mod app;
mod infra;
mod core;

use crate::app::drivers::add_routes;
use crate::app::driven;
use crate::infra::axum::start_server;
use crate::core::services::{self, CreateServiceParams};
use axum::Router;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let (tx, rx) = oneshot::channel();
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let user_repo = driven::create_driven();
    let user_service = services::create_services(CreateServiceParams {
        user_repo: user_repo
    });

    // build our application with a route
    let app = Router::new();
    let app = add_routes(app, rx, shutdown_token.clone());

    start_server(app, tx, shutdown_token).await;
}
