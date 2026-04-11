pub mod auth;
pub mod home;
pub mod task;

use crate::core::services::{AuthService, TaskService, UserService};
#[cfg(debug_assertions)]
use crate::infra::livereload;
use axum::routing::get;
#[cfg(debug_assertions)]
use tokio_util::sync::CancellationToken;

pub struct CreateDriverParams {
    pub app: axum::Router,
    #[cfg(debug_assertions)]
    pub rx: tokio::sync::oneshot::Receiver<()>,
    #[cfg(debug_assertions)]
    pub shutdown_token: CancellationToken,
    pub user_service: UserService,
    pub auth_service: AuthService,
    pub task_service: TaskService,
}

pub fn create_drivers(params: CreateDriverParams) -> axum::Router {
    let app = params.app;
    app.route("/ping", get(pong))
        .merge(home::home_routes())
        .merge(auth::auth_routes(
            params.user_service,
            params.auth_service,
            params.task_service.clone(),
        ))
        .merge({
            #[cfg(debug_assertions)]
            {
                livereload::live_reload(params.rx, params.shutdown_token)
            }
            #[cfg(not(debug_assertions))]
            {
                axum::Router::new()
            }
        })
        .merge(task::task_routes(params.task_service))
}

async fn pong() -> &'static str {
    "pong"
}
