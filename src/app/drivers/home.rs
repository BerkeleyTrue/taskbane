use askama::Template;
use axum::{
    middleware,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::infra::{
    auth::{authenticed_middleware, SessionAuthState},
    error::{AppError, Flashes},
};

pub fn home_routes() -> axum::Router {
    Router::new()
        .route("/", get(get_home))
        .layer(middleware::from_fn(authenticed_middleware))
}

#[derive(Debug, Clone, Template)]
#[template(path = "index.html")]
struct Home {
    title: String,
    is_authed: bool,
    flashes: Flashes,
}

async fn get_home(maybe_auth: Option<SessionAuthState>) -> Result<impl IntoResponse, AppError> {
    let templ = Home {
        title: "Taskbane".to_string(),
        is_authed: maybe_auth.is_some_and(|a| a.is_authed()),
        flashes: None,
    };
    Ok(Html(templ.render()?))
}
