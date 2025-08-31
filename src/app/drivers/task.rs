use askama::Template;
use axum::{
    middleware,
    response::{IntoResponse, Redirect},
    routing, Router,
};

use crate::infra::{
    auth::{authentication_middlewared, SessionAuthState},
    error::AppError,
};

pub fn task_routes() -> axum::Router {
    Router::new()
        .route("/task", routing::get(get_task))
        .route(
            "/tasks",
            routing::get(async || Redirect::permanent("/task")),
        )
        .layer(middleware::from_fn(authentication_middlewared))
}

#[derive(Debug, Clone, Template)]
#[template(path = "task.html")]
struct TaskPage {
    is_authed: bool,
}

pub async fn get_task(auth_state: SessionAuthState) -> Result<impl IntoResponse, AppError> {
    let templ = TaskPage {
        is_authed: auth_state.is_authed(),
    };
    Ok(axum::response::Html(templ.render()?))
}
