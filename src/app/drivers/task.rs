use askama::Template;
use axum::{
    extract::State,
    middleware,
    response::{IntoResponse, Redirect},
    routing, Router,
};
use taskchampion::Task;
use tracing::info;

use crate::{
    core::services::task::TaskService,
    infra::{
        auth::{authentication_middlewared, SessionAuthState},
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
        .layer(middleware::from_fn(authentication_middlewared))
        .with_state(task_service)
}

#[derive(Debug, Clone, Template)]
#[template(path = "task.html")]
struct TaskPage {
    is_authed: bool,
    tasks: Vec<Task>,
}

pub async fn get_task(
    task_service: State<TaskService>,
    auth_state: SessionAuthState,
) -> Result<impl IntoResponse, AppError> {
    let tasks = task_service.list().await.map_err(|err| {
        info!("Error getting tasks: {:?}", err);
        AppError::InternalServerError
    })?;

    let templ = TaskPage {
        is_authed: auth_state.is_authed(),
        tasks,
    };
    Ok(axum::response::Html(templ.render()?))
}
