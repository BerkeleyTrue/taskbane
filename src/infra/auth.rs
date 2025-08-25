use axum::{
    extract::{FromRequestParts, Request},
    http::{self}, middleware::Next, response::Response,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SessionAuthState {
    user_id: Uuid,
    username: String,
    is_authed: bool,
}
pub const SESSION_KEY: &str = "auth_state";

impl<S> FromRequestParts<S> for SessionAuthState
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        req: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;
        session
            .get::<SessionAuthState>(SESSION_KEY)
            .await
            .unwrap_or(None)
            .ok_or((http::StatusCode::BAD_REQUEST, "Failed to parse session"))
    }
}

impl SessionAuthState {
    pub fn new(user_id: &Uuid, username: String) -> Self {
        SessionAuthState {
            user_id: user_id.clone(),
            username,
            is_authed: false,
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn is_authed(&self) -> bool {
        self.is_authed
    }

    pub fn update_is_authed(&self, is_authed: bool) -> Self {
        SessionAuthState {
            user_id: self.user_id,
            username: self.username.clone(),
            is_authed,
        }
    }

    pub async fn update_session(&self, session: &Session) -> Result<Self, String> {
        session
            .insert(SESSION_KEY, self.clone())
            .await
            .map_err(|e| {
                info!("Failed to insert session: {}", e);
                return "Failed to insert session".to_string();
            })?;

        Ok(self.clone())
    }
}

pub async fn authentication_middlewared(
    auth_state: SessionAuthState,
    request: Request,
    next: Next,
) -> Response {
    if !auth_state.is_authed() {
        return Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body("Unauthorized".into())
            .unwrap();
    }

    next.run(request).await
}
