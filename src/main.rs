mod app;
mod core;
mod infra;

use crate::app::driven;
use crate::app::drivers;
use crate::core::services::{self, CreateServiceParams};
use crate::infra::axum::start_server;
use crate::infra::sqlx::create_sqlx;
use crate::infra::tower_session::create_session_store;
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
    let pool = create_sqlx();
    let session_store = create_session_store(&pool);
    let webauthn = infra::webauthn::create_authn();
    let task_storage = infra::task::create_task_storage();
    let (user_repo, auth_repo, task_repo) = driven::create_driven(&pool, task_storage);
    let (user_service, task_service, auth_service) =
        services::create_services(CreateServiceParams {
            user_repo,
            auth_repo,
            task_repo,
            webauthn,
        });

    // build our application with a route
    let app = Router::new();
    let app = drivers::create_drivers(drivers::CreateDriverParams {
        app,
        rx,
        shutdown_token: shutdown_token.clone(),
        user_service,
        auth_service,
        task_service,
    });

    start_server(app, tx, shutdown_token, session_store).await;
}
