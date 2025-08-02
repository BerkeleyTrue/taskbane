use axum::{
    routing::{get},
};

pub fn add_routes(app: axum::Router) -> axum::Router {
    app.route("/ping", get(pong))
}

async fn pong() -> &'static str {
    "pong"
}
