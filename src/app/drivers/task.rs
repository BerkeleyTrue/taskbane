use axum::{
    middleware,
    response::{IntoResponse, Redirect},
    routing, Router,
};

use crate::infra::{auth::authentication_middlewared, error::AppError};

pub fn task_routes() -> axum::Router {
    Router::new()
        .route("/task", routing::get(get_task))
        .route(
            "/tasks",
            routing::get(async || Redirect::permanent("/task")),
        )
        .layer(middleware::from_fn(authentication_middlewared))
}

pub async fn get_task() -> Result<impl IntoResponse, AppError> {
    Ok("Get Task")
}
