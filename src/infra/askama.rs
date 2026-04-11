use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tower_sessions::Session;

use crate::infra::alerts::{flush_alert, Alert, Alerts};

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err:?}"),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Globals {
    pub alerts: Alerts,
    pub debug: bool,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            alerts: Alerts::default(),
            debug: cfg!(debug_assertions),
        }
    }
}

impl Globals {
    pub async fn fetch(session: &Session) -> Self {
        Self {
            alerts: flush_alert(session).await,
            debug: cfg!(debug_assertions),
        }
    }

    pub fn push_alert(mut self, alert: Alert) -> Self {
        self.alerts.push(alert);
        self
    }
}
