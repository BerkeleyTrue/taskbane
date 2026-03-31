use anyhow::{anyhow, Error, Result};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request},
    http,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::TypedHeader;
use derive_more::Display;
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

#[derive(Debug, Clone, Display, Deserialize, Serialize)]
pub enum AuthState {
    Authorized,
    Authenticated,
    Not,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionAuthState {
    user_id: Uuid,
    username: String,
    auth_state: AuthState,
}

impl SessionAuthState {
    pub fn new(user_id: Uuid, username: String) -> Self {
        SessionAuthState {
            user_id,
            username,
            auth_state: AuthState::Not,
        }
    }

    pub async fn try_from_session(session: &Session) -> Result<Option<Self>> {
        session.get::<Self>(SESSION_KEY).await.map_err(Error::from)
    }

    pub async fn from_session(session: Session) -> Result<Self> {
        session
            .get::<Self>(SESSION_KEY)
            .await
            .map_err(Error::from)?
            .ok_or(anyhow!("No session found"))
    }

    pub async fn logout(self, session: &Session) -> Result<()> {
        session.flush().await?;
        Ok(())
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn auth_state(&self) -> AuthState {
        self.auth_state.clone()
    }

    pub fn is_authed(&self) -> bool {
        matches!(
            self.auth_state,
            AuthState::Authenticated | AuthState::Authorized
        )
    }

    pub fn is_authorized(&self) -> bool {
        matches!(self.auth_state, AuthState::Authorized)
    }

    pub fn authenticate(self) -> Self {
        if matches!(self.auth_state, AuthState::Not) {
            SessionAuthState {
                user_id: self.user_id,
                username: self.username.clone(),
                auth_state: AuthState::Authenticated,
            }
        } else {
            self
        }
    }

    pub fn authorize(self) -> Result<Self> {
        if matches!(self.auth_state, AuthState::Authenticated) {
            return Ok(SessionAuthState {
                user_id: self.user_id,
                username: self.username.clone(),
                auth_state: AuthState::Authenticated,
            });
        }
        Err(anyhow!("User not authenticated"))
    }

    pub async fn update_session(&self, session: &Session) -> Result<Self> {
        session.insert(SESSION_KEY, self.clone()).await?;

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
    type Rejection = Response;

    async fn from_request_parts(
        req: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let session = Session::from_request_parts(req, state)
            .await
            .map_err(|err| err.into_response())?;
        let auth_state_res = SessionAuthState::try_from_session(&session)
            .await
            .map_err(|err| {
                info!("Failed to parse optional session from store: {:?}", err);
                Redirect::temporary("/").into_response()
            });

        if auth_state_res.is_err() {
            // TODO: redirect to login from here
            session.flush().await.map_err(|err| {
                info!("Failed to flush errant session from store: {:?}", err);
                (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server Error",
                )
                    .into_response()
            })?;
        }
        auth_state_res
    }
}

enum ResponseType {
    Json,
    Text,
}

/// redirect unauth users, protecting routes
pub async fn unauth_middleware(
    accept: Option<TypedHeader<Accept>>,
    auth_state: Option<SessionAuthState>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let format = accept
        .and_then(|TypedHeader(accept)| accept.negotiate(ACCEPT_LIST))
        .map(|media_type| {
            if let ("application", "json") = (media_type.ty.as_str(), media_type.subty.as_str()) {
                return ResponseType::Json;
            }
            ResponseType::Text
        })
        .unwrap_or(ResponseType::Text);

    match auth_state {
        // we know the user, and the user has permission to view task
        Some(SessionAuthState {
            auth_state: AuthState::Authorized,
            ..
        }) => next.run(request).await,

        // we know the user, but they don't have permission
        Some(SessionAuthState {
            auth_state: AuthState::Authenticated,
            ..
        }) => match format {
            ResponseType::Json => (
                http::StatusCode::FORBIDDEN,
                Json(ErrorMessage::new("unauthorized")),
            )
                .into_response(),
            ResponseType::Text => Redirect::temporary("/authorize-user").into_response(),
        },
        // who are you?
        _ => match format {
            ResponseType::Json => (
                http::StatusCode::UNAUTHORIZED,
                Json(ErrorMessage::new("unauthenticated user")),
            )
                .into_response(),
            ResponseType::Text => Redirect::temporary("/login").into_response(),
        },
    }
}

/// redirect authenticated users
pub async fn authenticed_middleware(
    auth_state: Option<SessionAuthState>,
    request: Request,
    next: Next,
) -> Response {
    info!("authenticated auth state: {auth_state:?}");
    match auth_state {
        Some(SessionAuthState {
            auth_state: AuthState::Authorized,
            ..
        }) => Redirect::temporary("/task").into_response(),
        Some(SessionAuthState {
            auth_state: AuthState::Authenticated,
            ..
        }) => Redirect::temporary("/authorize-user").into_response(),
        _ => next.run(request).await,
    }
}

/// redirect authorized users
pub async fn authorized_middleware(
    auth_state: Option<SessionAuthState>,
    request: Request,
    next: Next,
) -> Response {
    info!("authorized auth state: {auth_state:?}");
    match auth_state {
        Some(SessionAuthState {
            auth_state: AuthState::Authorized,
            ..
        }) => Redirect::temporary("/task").into_response(),
        _ => next.run(request).await,
    }
}
