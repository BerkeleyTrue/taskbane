use tracing::{info};

pub async fn start_server(app: axum::Router) {
    // Initialize tracing
    tracing::info!("Starting Axum server...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
