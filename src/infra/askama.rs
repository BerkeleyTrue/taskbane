use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tower_sessions::Session;

use crate::infra::alerts::{flush, Flashes};

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

#[derive(Debug, Clone, Default)]
pub struct Globals {
    pub alerts: Flashes,
}

impl Globals {
    pub async fn fetch(session: &Session) -> Self {
        Self {
            alerts: flush(session).await,
        }
    }
}
