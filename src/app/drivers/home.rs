use crate::infra::axum::route;
use axum::routing::get;

pub fn home_routes() -> axum::Router {
    route("/", get(get_home))
}

async fn get_home() -> &'static str {
    "Welcome to the home page!"
}
