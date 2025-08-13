use crate::infra::axum::AppError;
use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

pub fn home_routes() -> axum::Router {
    Router::new().route("/", get(get_home))
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
