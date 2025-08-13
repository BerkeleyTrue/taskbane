use std::time::Duration;

use askama::Template;
use axum::{
    http::{Request, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::MethodRouter,
    serve, Router,
};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{compression, services::ServeDir, trace};
use tracing::{info, info_span, Span};

pub fn route<S>(path: &str, handler: MethodRouter<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route(path, handler)
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Route not found")]
    NotFound,
    #[error("Failed to rendered template")]
    Render(#[from] askama::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct Tmpl {
            err: AppError,
        }

        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let tmpl = Tmpl { err: self };

        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}

pub async fn start_server(
    app: Router,
    tx: tokio::sync::oneshot::Sender<()>,
    shutdown_token: CancellationToken,
) {
    // Initialize tracing
    tracing::info!("Starting Axum server...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Listening on port 3000");
    tx.send(()).unwrap();
    let app = app.fallback(|| async { AppError::NotFound });
    serve(
        listener,
        middleware(app).nest_service("/public", ServeDir::new("public")),
    )
    .with_graceful_shutdown(shutdown_signal(shutdown_token))
    .await
    .unwrap();
}

async fn shutdown_signal(shutdown_token: CancellationToken) {
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
    shutdown_token.cancel();
}

fn middleware(app: Router) -> Router {
    let service = ServiceBuilder::new()
        .layer(compression::CompressionLayer::new())
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(
                    |req: &Request<_>| info_span!("", status=tracing::field::Empty, method=%req.method(), path=%req.uri()),
                )
                .on_response(|res: &Response<_>, _latency: Duration, span: &Span| {
                    span.record("status", &tracing::field::display(res.status()));
                    info!("")
                }),
        );
    app.layer(service)
}
