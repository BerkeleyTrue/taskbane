pub mod auth;
pub mod home;
pub mod task;

use crate::core::services::{AuthService, TaskService, UserService};
use crate::infra::livereload;
use axum::routing::get;
use tokio_util::sync::CancellationToken;

pub struct CreateDriverParams {
    pub app: axum::Router,
    pub rx: tokio::sync::oneshot::Receiver<()>,
    pub shutdown_token: CancellationToken,
    pub user_service: UserService,
    pub auth_service: AuthService,
    pub task_service: TaskService,
}

pub fn create_drivers(params: CreateDriverParams) -> axum::Router {
    let app = params.app;
    app.route("/ping", get(pong))
        .merge(home::home_routes())
        .merge(auth::auth_routes(params.user_service, params.auth_service))
        .merge(livereload::live_reload(params.rx, params.shutdown_token))
        .merge(task::task_routes(params.task_service))
}

async fn pong() -> &'static str {
    "pong"
}
