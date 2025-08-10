mod app;
mod core;
mod infra;

use crate::app::driven;
use crate::app::drivers;
use crate::core::services::{self, CreateServiceParams};
use crate::infra::axum::start_server;
use axum::Router;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let (tx, rx) = oneshot::channel();
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let user_repo = driven::create_driven();
    let user_service = services::create_services(CreateServiceParams { user_repo });

    // build our application with a route
    let app = Router::new();
    let app = drivers::create_drivers(drivers::CreateDriverParams {
        app,
        rx,
        shutdown_token: shutdown_token.clone(),
        user_service,
    });

    start_server(app, tx, shutdown_token).await;
}
