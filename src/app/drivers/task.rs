use askama::Template;
use axum::{
    extract::State,
    middleware,
    response::{IntoResponse, Redirect},
    routing, Router,
};
use tower_sessions::Session;
use tracing::info;

use crate::{
    core::{models::task::TaskDto, services::TaskService},
    infra::{
        askama::{Globals, HtmlTemplate},
        auth::{unauth_middleware, SessionAuthState},
        error::AppError,
    },
};

pub fn task_routes(task_service: TaskService) -> axum::Router {
    Router::new()
        .route("/task", routing::get(get_task))
        .route(
            "/tasks",
            routing::get(async || Redirect::permanent("/task")),
        )
        .layer(middleware::from_fn(unauth_middleware))
        .with_state(task_service)
}

#[derive(Debug, Clone, Template)]
#[template(path = "task.html")]
struct TaskPage {
    is_authed: bool,
    tasks: Vec<TaskDto>,
    globals: Globals,
}

pub async fn get_task(
    session: Session,
    auth_state: SessionAuthState,
    task_service: State<TaskService>,
) -> Result<impl IntoResponse, AppError> {
    let tasks = task_service.list().await.map_err(|err| {
        info!("Error getting tasks: {:?}", err);
        AppError::InternalServerError
    })?;

    let templ = TaskPage {
        is_authed: auth_state.is_authed(),
        tasks,
        globals: Globals::fetch(&session).await,
    };
    Ok(HtmlTemplate(templ))
}
