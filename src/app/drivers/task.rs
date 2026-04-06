use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderName, HeaderValue},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing, Router,
};
use derive_more::Constructor;
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

use crate::{
    core::{models::task::TaskDto, services::TaskService},
    infra::{
        alerts::{Alert, AlertLevel},
        askama::{Globals, HtmlTemplate},
        auth::{unauth_middleware, SessionAuthState},
        error::{ApiError, AppError},
    },
};

pub fn task_routes(task_service: TaskService) -> axum::Router {
    Router::new()
        .route("/task", routing::get(get_tasks))
        .route("/task/new", routing::get(get_create_task))
        .route("/task/new", routing::post(post_create_task))
        .route(
            "/tasks",
            routing::get(async || Redirect::permanent("/task")),
        )
        .route("/task/{id}", routing::get(get_task))
        .route("/task/{id}/confirm-done", routing::get(get_confirm_done))
        .route("/task/{id}/done", routing::post(post_mark_task_down))
        .layer(middleware::from_fn(unauth_middleware))
        .with_state(task_service)
}

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "task.html")]
struct TaskListPage {
    is_authed: bool,
    tasks: Vec<TaskDto>,
    globals: Globals,
}

pub async fn get_tasks(
    session: Session,
    auth_state: SessionAuthState,
    task_service: State<TaskService>,
) -> Result<impl IntoResponse, AppError> {
    let tasks = task_service.list().await.map_err(|err| {
        info!("Error getting tasks: {:?}", err);
        AppError::InternalServerError
    })?;

    let templ = TaskListPage::new(
        auth_state.is_authed(),
        tasks,
        Globals::fetch(&session).await,
    );

    Ok(HtmlTemplate(templ))
}

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "task_create.html")]
struct CreateTaskPage {
    is_authed: bool,
    globals: Globals,
}

pub async fn get_create_task(session: Session) -> impl IntoResponse {
    let create_page = CreateTaskPage::new(true, Globals::fetch(&session).await);

    HtmlTemplate(create_page)
}

pub async fn post_create_task() -> ApiError {
    ApiError::InternalServerError
}

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "task_detail.html")]
struct TaskPage {
    is_authed: bool,
    task: TaskDto,
    globals: Globals,
}

pub async fn get_task(
    Path(id): Path<Uuid>,
    session: Session,
    auth_state: SessionAuthState,
    task_service: State<TaskService>,
) -> Result<impl IntoResponse, AppError> {
    let task = task_service.get_task(id).await.map_err(|err| {
        info!("Error getting tasks: {:?}", err);
        AppError::NotFound
    })?;

    let templ = TaskPage::new(auth_state.is_authed(), task, Globals::fetch(&session).await);

    Ok(HtmlTemplate(templ))
}

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "partials/modal-task_done.html")]
struct ConfirmDone {
    task: TaskDto,
}
pub async fn get_confirm_done(
    Path(id): Path<Uuid>,
    task_service: State<TaskService>,
) -> Result<impl IntoResponse, ApiError> {
    let task = task_service
        .get_task(id)
        .await
        .map_err(|err| ApiError::BadRequest {
            message: err.to_string(),
        })?;

    let templ = ConfirmDone::new(task);

    Ok(HtmlTemplate(templ))
}

pub async fn post_mark_task_down(
    session: Session,
    Path(id): Path<Uuid>,
    task_service: State<TaskService>,
) -> Result<impl IntoResponse, ApiError> {
    task_service
        .mark_task_done(id)
        .await
        .map_err(|err| ApiError::BadRequest {
            message: err.to_string(),
        })?;

    let tasks = task_service.list().await.map_err(|err| {
        info!("Error getting tasks: {:?}", err);
        ApiError::InternalServerError
    })?;
    let alert = Alert::new(AlertLevel::Success, "Task completed!".to_owned());

    let globals = Globals::fetch(&session).await.push_alert(alert);

    let tasks_page = TaskListPage::new(true, tasks, globals)
        .render()
        .map_err(|err| {
            info!("Error rendering alert: {err:?}");
            ApiError::InternalServerError
        })?;

    Ok((
        [(
            HeaderName::from_static("hx-replace-url"),
            HeaderValue::from_static("/task"),
        )],
        Html(tasks_page),
    ))
}
