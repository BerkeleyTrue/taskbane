use std::{env, time::Duration};

use askama::Template;
use axum::{
    http::{Request, Response, StatusCode},
    response::{Html, IntoResponse},
    serve, Json, Router,
};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{compression, services::ServeDir, trace};
use tracing::{info, info_span, Span};

use crate::infra::{
    askama::Globals,
    error::{ApiError, AppError, ErrorMessage},
    tower_session::MySession,
};

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct Tmpl {
            err: AppError,
            is_authed: bool,
            globals: Globals,
        }

        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let tmpl = Tmpl {
            err: self,
            is_authed: false,
            globals: Globals::default(),
        };

        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<axum::body::Body> {
        let (status, mess) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, ErrorMessage::from(self)),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, ErrorMessage::from(self)),
            ApiError::BadRequest { message } => {
                (StatusCode::BAD_REQUEST, ErrorMessage::new(message))
            }
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, ErrorMessage::from(self)),
            ApiError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorMessage::from(self))
            }
        };

        (status, Json(mess)).into_response()
    }
}

pub async fn start_server(
    app: Router,
    tx: tokio::sync::oneshot::Sender<()>,
    shutdown_token: CancellationToken,
    session_store: impl MySession,
) {
    session_store.run_migration().await.unwrap();
    let session_layer = session_store.create_layer();

    info!("starting Axum server...");

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    info!("listening on port {port}");
    let _ = tx.send(()).inspect_err(|err| {
        info!("tx: err sending {err:?}");
    });
    let app = app
        .fallback(|| async { AppError::NotFound })
        .layer(session_layer);
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

    info!("initiating graceful shutdown...");
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
                    span.record("status", tracing::field::display(res.status()));
                    // this is where the log actually happens
                    info!("")
                }),
        );
    app.layer(service)
}
