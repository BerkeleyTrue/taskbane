use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request},
    http,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

pub const SESSION_KEY: &str = "auth_state";

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SessionAuthState {
    user_id: Uuid,
    username: String,
    is_authed: bool,
}

impl SessionAuthState {
    pub fn new(user_id: &Uuid, username: String) -> Self {
        SessionAuthState {
            user_id: user_id.clone(),
            username,
            is_authed: false,
        }
    }

    pub async fn maybe_from_session(session: Session) -> Result<Option<Self>, String> {
        session
            .get::<Self>(SESSION_KEY)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn from_session(session: Session) -> Result<Self, String> {
        session
            .get::<Self>(SESSION_KEY)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("No session found".to_string())
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
            .map_err(|err| err.to_string())?;

        Ok(self.clone())
    }
}

// when session state is required for the handler
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
        SessionAuthState::from_session(session)
            .await
            .map_err(|err| {
                info!("Failed to pull session from store: {:?}", err);
                (http::StatusCode::BAD_REQUEST, "Failed to parse session")
            })
    }
}

impl<S> OptionalFromRequestParts<S> for SessionAuthState
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        req: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;
        SessionAuthState::maybe_from_session(session)
            .await
            .map_err(|err| {
                info!("Failed to parse optional session from store: {:?}", err);
                (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server Error",
                )
            })
    }
}

pub async fn authentication_middlewared(
    auth_state: Option<SessionAuthState>,
    request: Request,
    next: Next,
) -> Response {
    if auth_state.is_none() || !auth_state.unwrap().is_authed() {
        return Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body("Unauthorized".into())
            .unwrap();
    }

    next.run(request).await
}
