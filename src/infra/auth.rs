use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request},
    http,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::TypedHeader;
use headers_accept::Accept;
use mediatype::{names, MediaType};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

use crate::infra::error::ErrorMessage;

pub const SESSION_KEY: &str = "auth_state";
const ACCEPT_JSON: MediaType = MediaType::new(names::APPLICATION, names::JSON);
const ACCEPT_HTML: MediaType = MediaType::new(names::TEXT, names::HTML);
const ACCEPT_LIST: &[MediaType; 2] = &[ACCEPT_JSON, ACCEPT_HTML];

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

enum ResponseType {
    Json,
    Text,
}

pub async fn authentication_middlewared(
    accept: Option<TypedHeader<Accept>>,
    auth_state: Option<SessionAuthState>,
    request: Request,
    next: Next,
) -> Response {
    let format = accept
        .and_then(|TypedHeader(accept)| accept.negotiate(ACCEPT_LIST))
        .and_then(|media_type| {
            if let ("application", "json") = (media_type.ty.as_str(), media_type.subty.as_str()) {
                return Some(ResponseType::Json);
            }
            Some(ResponseType::Text)
        })
        .unwrap_or(ResponseType::Text);

    if auth_state.is_none_or(|auth| !auth.is_authed()) {
        return match format {
            ResponseType::Json => (
                http::StatusCode::UNAUTHORIZED,
                Json(ErrorMessage::new("unauthorized")),
            )
                .into_response(),
            ResponseType::Text => Redirect::temporary("/login").into_response(),
        };
    }

    next.run(request).await
}

pub async fn authenticed_middleware(
    auth_state: Option<SessionAuthState>,
    request: Request,
    next: Next,
) -> Response {
    if auth_state.is_some_and(|auth| auth.is_authed()) {
        return Redirect::temporary("/task").into_response();
    }
    next.run(request).await
}
