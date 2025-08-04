use axum::{routing::MethodRouter, serve, Router};
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::info;

pub fn route(path: &str, handler: MethodRouter) -> Router {
    Router::new().route(path, handler)
}

pub async fn start_server(app: Router, tx: tokio::sync::oneshot::Sender<()>) {
    // Initialize tracing
    tracing::info!("Starting Axum server...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Listening on port 3000");
    serve(
        listener,
        app.nest_service("/public", ServeDir::new("public")),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    tx.send(()).unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Initiating graceful shutdown...");
}
