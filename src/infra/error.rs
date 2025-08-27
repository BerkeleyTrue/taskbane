use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Route not found")]
    NotFound,
    #[error("Failed to rendered template")]
    Render(#[from] askama::Error),
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

#[derive(Serialize)]
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
            message: err.to_string()
        }
    }
}
