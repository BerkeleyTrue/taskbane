use crate::app::drivers::home;
use axum::routing::get;

pub fn add_routes(app: axum::Router) -> axum::Router {
    app.route("/ping", get(pong)).merge(home::home_routes())
}

async fn pong() -> &'static str {
    "pong"
}
