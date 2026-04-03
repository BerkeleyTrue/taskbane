use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect},
    routing, Router,
};
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

use crate::{
    core::{models::task::TaskDto, services::TaskService},
    infra::{
        askama::{Globals, HtmlTemplate},
        auth::{unauth_middleware, SessionAuthState},
        error::{ApiError, AppError},
    },
};

pub fn task_routes(task_service: TaskService) -> axum::Router {
    Router::new()
        .route("/task", routing::get(get_task))
        .route(
            "/tasks",
            routing::get(async || Redirect::permanent("/task")),
        )
        .route("/task/{id}/done", routing::get(post_mark_task_down))
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

pub async fn post_mark_task_down(
    Path(id): Path<Uuid>,
    task_service: State<TaskService>,
) -> Result<impl IntoResponse, ApiError> {
    task_service
        .mark_task_done(id)
        .await
        .map_err(|err| ApiError::BadRequest {
            message: err.to_string(),
        })?;

    Ok((StatusCode::OK, "Ok"))
}
