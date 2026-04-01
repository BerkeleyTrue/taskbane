use askama::Template;
use axum::response::{IntoResponse, Response};
use derive_more::Constructor;
use serde::Serialize;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Route not found")]
    NotFound,
    #[error("Failed to rendered template")]
    Render(#[from] askama::Error),
    #[error("Internal Server Error")]
    InternalServerError,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Internal server error")]
    InternalServerError,
    #[error("Bad request")]
    BadRequest { message: String },
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found")]
    NotFound,
}

#[derive(Debug, Serialize)]
pub struct ErrorMessage {
    pub message: String,
}

impl ErrorMessage {
    pub fn new(message: &str) -> Self {
        ErrorMessage {
            message: message.to_string(),
        }
    }
}

impl From<ApiError> for ErrorMessage {
    fn from(err: ApiError) -> ErrorMessage {
        ErrorMessage {
            message: err.to_string(),
        }
    }
}

pub type Flash = (String, String);

pub type Flashes = Option<Vec<Flash>>;

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "partials/alert.html")]
pub struct FlashTempl {
    level: String,
    message: String,
}

pub fn flash_err<E: std::fmt::Display>(err: E) -> Response {
    let flash = FlashTempl::new("error".to_string(), err.to_string());

    match flash.render() {
        Ok(html) => html.into_response(),
        Err(err) => {
            info!("Error rendering flash: {err:?}");
            ApiError::InternalServerError.into_response()
        }
    }
}
