use askama::Template;
use axum::{middleware, response::IntoResponse, routing::get, Router};
use tower_sessions::Session;

use crate::infra::{
    askama::{Globals, HtmlTemplate},
    auth::{authenticed_middleware, SessionAuthState},
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
    globals: Globals,
}

async fn get_home(session: Session, maybe_auth: Option<SessionAuthState>) -> impl IntoResponse {
    let templ = Home {
        title: "Taskbane".to_string(),
        is_authed: maybe_auth.is_some_and(|a| a.is_authed()),
        globals: Globals::fetch(&session).await,
    };
    HtmlTemplate(templ)
}
