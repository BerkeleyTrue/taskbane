use axum::{middleware, response::IntoResponse, routing, Router};

use crate::infra::{auth::authentication_middlewared, error::AppError};

pub fn task_routes() -> axum::Router {
    Router::new()
        .route("/tasks", routing::get(get_task))
        .layer(middleware::from_fn(authentication_middlewared))
}

pub async fn get_task() -> Result<impl IntoResponse, AppError> {
    Ok("Get Task")
}
