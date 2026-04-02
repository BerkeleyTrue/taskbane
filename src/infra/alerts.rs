use std::fmt::Display;

use anyhow::Result;
use askama::Template;
use axum::response::{IntoResponse, Response};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::info;

use crate::infra::error::ApiError;

pub const FLASH_KEY: &str = "__flashes";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlashLevel {
    Error,
    Success,
    Warning,
    Info,
}

impl Display for FlashLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlashLevel::Error => write!(f, "error"),
            FlashLevel::Success => write!(f, "success"),
            FlashLevel::Warning => write!(f, "warning"),
            FlashLevel::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Flash {
    pub level: FlashLevel,
    pub message: String,
}

pub type Flashes = Vec<Flash>;

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "partials/alert.html")]
pub struct FlashTempl {
    level: FlashLevel,
    message: String,
}

pub fn flash_err<E: std::fmt::Display>(err: E) -> Response {
    let flash = FlashTempl::new(FlashLevel::Error, err.to_string());

    match flash.render() {
        Ok(html) => html.into_response(),
        Err(err) => {
            info!("Error rendering flash: {err:?}");
            ApiError::InternalServerError.into_response()
        }
    }
}

pub async fn flash(level: FlashLevel, message: String, session: &Session) -> Result<()> {
    let mut flashes = session
        .get::<Flashes>(FLASH_KEY)
        .await?
        .unwrap_or(Flashes::new());

    flashes.push(Flash::new(level, message));

    session.insert(FLASH_KEY, flashes).await?;

    Ok(())
}

pub async fn flush(session: &Session) -> Flashes {
    session
        .remove::<Flashes>(FLASH_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or(Flashes::new())
}

// pub async fn flash_err
