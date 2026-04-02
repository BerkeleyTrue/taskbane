use std::fmt::Display;

use anyhow::Result;
use askama::Template;
use axum::response::{IntoResponse, Response};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::info;

use crate::infra::error::ApiError;

const ALERT_KEY: &str = "__alerts";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Error,
    Success,
    Warning,
    Info,
}

impl Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Error => write!(f, "error"),
            AlertLevel::Success => write!(f, "success"),
            AlertLevel::Warning => write!(f, "warning"),
            AlertLevel::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
}

pub type Alerts = Vec<Alert>;

#[derive(Debug, Clone, Template, Constructor)]
#[template(path = "partials/alert.html")]
pub struct AlertTempl {
    level: AlertLevel,
    message: String,
}

pub fn map_err_to_alert<E: std::fmt::Display>(err: E) -> Response {
    let alert = AlertTempl::new(AlertLevel::Error, err.to_string());

    match alert.render() {
        Ok(html) => html.into_response(),
        Err(err) => {
            info!("Error rendering alert: {err:?}");
            ApiError::InternalServerError.into_response()
        }
    }
}

pub async fn alert(level: AlertLevel, message: String, session: &Session) -> Result<()> {
    let mut alerts = session
        .get::<Alerts>(ALERT_KEY)
        .await?
        .unwrap_or(Alerts::new());

    alerts.push(Alert::new(level, message));

    session.insert(ALERT_KEY, alerts).await?;

    Ok(())
}

pub async fn flush_alert(session: &Session) -> Alerts {
    session
        .remove::<Alerts>(ALERT_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or(Alerts::new())
}

pub async fn alert_err(message: String, session: &Session) -> Result<()> {
    alert(AlertLevel::Error, message, session).await
}

pub async fn alert_success(message: String, session: &Session) -> Result<()> {
    alert(AlertLevel::Success, message, session).await
}

pub async fn alert_warning(message: String, session: &Session) -> Result<()> {
    alert(AlertLevel::Warning, message, session).await
}

pub async fn alert_info(message: String, session: &Session) -> Result<()> {
    alert(AlertLevel::Info, message, session).await
}
