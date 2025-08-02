mod infra;
mod app;

use axum::Router;
use crate::infra::axum::start_server;
use crate::app::routes::add_routes;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new();
    let app = add_routes(app);

    start_server(app).await;
}
