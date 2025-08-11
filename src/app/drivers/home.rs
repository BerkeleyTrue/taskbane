use crate::infra::axum::route;
use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
};

pub fn home_routes() -> axum::Router {
    route("/", get(get_home))
}

#[derive(Debug, Clone, Template)]
#[template(path = "index.html")]
struct Home {
    title: String,
}

async fn get_home() -> Result<impl IntoResponse, axum::Error> {
    let templ = Home {
        title: "Taskbane".to_string(),
    };
    Ok(Html(templ.render()?))
}
