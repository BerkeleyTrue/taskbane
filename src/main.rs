mod app;
mod core;
mod infra;

use crate::app::driven;
use crate::app::drivers;
use crate::core::services::{self, CreateServiceParams};
use crate::infra::axum::start_server;
use axum::Router;
use dotenv::dotenv;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    dotenv().expect("Failed to load .env");
    let (tx, rx) = oneshot::channel();
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let webauthn = infra::webauthn::create_authn();
    let (user_repo, auth_service) = driven::create_driven(webauthn);
    let user_service = services::create_services(CreateServiceParams { user_repo });

    // build our application with a route
    let app = Router::new();
    let app = drivers::create_drivers(drivers::CreateDriverParams {
        app,
        rx,
        shutdown_token: shutdown_token.clone(),
        user_service,
        auth_service,
    });

    start_server(app, tx, shutdown_token).await;
}
