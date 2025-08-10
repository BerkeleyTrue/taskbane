pub mod home;
pub mod layout;
pub mod auth;

use crate::infra::livereload;
use crate::core::services;
use axum::routing::get;
use tokio_util::sync::CancellationToken;

pub struct CreateDriverParams {
    pub app: axum::Router,
    pub rx: tokio::sync::oneshot::Receiver<()>,
    pub shutdown_token: CancellationToken,
    pub user_service: services::user::UserService,
}

pub fn create_drivers(params: CreateDriverParams) -> axum::Router {
    let app = params.app;
    app.route("/ping", get(pong))
        .merge(home::home_routes())
        .merge(auth::auth_routes(params.user_service))
        .merge(livereload::live_reload(params.rx, params.shutdown_token))
}

async fn pong() -> &'static str {
    "pong"
}
