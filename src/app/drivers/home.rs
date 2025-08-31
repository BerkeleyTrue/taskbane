use askama::Template;
use axum::{
    middleware,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::infra::{auth::authenticed_middleware, error::AppError};

pub fn home_routes() -> axum::Router {
    Router::new()
        .route("/", get(get_home))
        .layer(middleware::from_fn(authenticed_middleware))
}

#[derive(Debug, Clone, Template)]
#[template(path = "index.html")]
struct Home {
    title: String,
}

async fn get_home() -> Result<impl IntoResponse, AppError> {
    let templ = Home {
        title: "Taskbane".to_string(),
    };
    Ok(Html(templ.render()?))
}
