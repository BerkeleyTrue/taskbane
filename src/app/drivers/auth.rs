use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use axum::{middleware, Form, Router};
use axum::{
    routing::{get, post},
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;
use webauthn_rs::prelude::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

use crate::core::services::{AuthService, UserService};
use crate::infra::auth::{authenticed_middleware, authorized_middleware, SessionAuthState};
use crate::infra::error::{ApiError, AppError};

#[derive(Clone)]
struct AuthServices {
    user_service: UserService,
    auth_service: AuthService,
}

pub fn auth_routes<S>(user_service: UserService, auth_service: AuthService) -> axum::Router<S> {
    Router::new()
        .route("/register", get(get_register))
        .route("/auth/register", post(post_start_registration))
        .route(
            "/auth/validate-registration",
            post(post_validate_registration),
        )
        .route("/login", get(get_login))
        .route("/auth/login", post(post_authenticate))
        .route("/auth/validate-login", post(post_validate_authen))
        .route("/auth/username_validation", get(username_validation))
        // redirect authenticated users to task
        .layer(middleware::from_fn(authenticed_middleware))
        .route("/authorize-user", get(get_validate_user))
        // .route("/auth/validate-user", post(post_validate_user))
        .layer(middleware::from_fn(authorized_middleware))
        .route("/logout", get(get_logout))
        .with_state(AuthServices {
            user_service,
            auth_service,
        })
}

#[derive(Debug, Clone, Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    is_authed: bool,
}
async fn get_register() -> Result<impl IntoResponse, AppError> {
    let template = RegisterTemplate { is_authed: false };
    Ok(Html(template.render()?))
}

// 1. The first step a client (user) will carry out is requesting a credential to be
// registered. We need to provide a challenge for this. The work flow will be:
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Reg     │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//   4. Vauth_service                     │                      │
//                  │  4. Yield PubKey    │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │                      │
//                  │                     │  5. Send Reg Opts    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │         PubKey
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │─ ─ ─
//                  │                     │                      │     │ 6. Persist
//                  │                     │                      │       Credential
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
//
// In this step, we are responding to the start registration request, and providing
// the challenge to the browser.
#[derive(Deserialize)]
struct RegistrationParams {
    username: String,
}

async fn post_start_registration(
    session: Session,
    State(AuthServices {
        user_service,
        auth_service,
    }): State<AuthServices>,
    Json(payload): Json<RegistrationParams>,
) -> Result<Json<CreationChallengeResponse>, ApiError> {
    let username = payload.username;
    let user = user_service.register_user(username).await.map_err(|err| {
        info!("Error registering user: {:?}", err);
        ApiError::BadRequest {
            message: "Username already exists".to_string(),
        }
    })?;

    let challenge = auth_service
        .create_registration(user.clone())
        .await
        .map_err(|err| {
            info!("Error creating registration: {:?}", err);
            ApiError::BadRequest {
                message: "Failed to create registration".to_string(),
            }
        })?;

    SessionAuthState::new(user.id(), user.username().to_string())
        .update_session(&session)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    Ok(Json(challenge))
}

async fn post_validate_registration(
    session_auth: SessionAuthState,
    State(AuthServices {
        user_service: _user_state,
        auth_service,
    }): State<AuthServices>,
    Json(cred): Json<RegisterPublicKeyCredential>,
) -> Result<Redirect, ApiError> {
    auth_service
        .validate_registration(session_auth.user_id(), &cred)
        .await
        .or(Err(ApiError::BadRequest {
            message: "Failed to validate credentioals".to_string(),
        }))?;

    Ok(Redirect::to("/login"))
}

#[derive(Debug, Clone, Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    is_authed: bool,
}
async fn get_login() -> Result<impl IntoResponse, AppError> {
    let template = LoginTemplate { is_authed: false };
    Ok(Html(template.render()?))
}

// 2. Now that our public key has been registered, we can authenticate a user and verify
// that they are the holder of that security token. The work flow is similar to registration.
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Auth    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//  4. Verify │     │                     │                      │
//                  │    4. Yield Sig     │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │    5. Send Auth      │
//                  │                     │        Opts          │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │          Sig
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
//
// The user indicates the wish to start authentication and we need to provide a challenge.

#[derive(Deserialize)]
struct LoginParams {
    username: String,
}

async fn post_authenticate(
    session: Session,
    State(AuthServices {
        user_service,
        auth_service,
    }): State<AuthServices>,
    Json(LoginParams { username }): Json<LoginParams>,
) -> Result<Json<RequestChallengeResponse>, ApiError> {
    let user = user_service.get_user(&username).await.map_err(|err| {
        info!("get login err: {err:}");
        ApiError::BadRequest {
            message: "No user found for username".to_string(),
        }
    })?;

    let rcr = auth_service.login(user.id()).await.map_err(|err| {
        info!("Error during login: {:?}", err);
        ApiError::BadRequest {
            message: "Failed to login user".to_string(),
        }
    })?;

    SessionAuthState::new(user.id(), user.username().to_string())
        .update_session(&session)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    Ok(Json(rcr))
}

async fn post_validate_authen(
    session: Session,
    session_auth: SessionAuthState,
    State(AuthServices {
        user_service: _user_service,
        auth_service,
    }): State<AuthServices>,
    Json(pkc): Json<PublicKeyCredential>,
) -> Result<Redirect, ApiError> {
    auth_service
        .validate_login(session_auth.user_id(), &pkc)
        .await
        .map_err(|err| {
            info!("Error validating login: {:?}", err);
            ApiError::BadRequest {
                message: "Failed to validate login".to_string(),
            }
        })?;

    session_auth
        .authenticate()
        .update_session(&session)
        .await
        .or(Err(ApiError::InternalServerError))?;

    Ok(Redirect::to("/task"))
}

#[derive(Deserialize, Debug)]
struct UsernameValidationParams {
    username: String,
}

#[derive(Debug, Template)]
#[template(path = "partials/form-error.html")]
struct FormError {
    id: String,
    error: Option<String>,
}

async fn username_validation(
    State(state): State<AuthServices>,
    Form(input): Form<UsernameValidationParams>,
) -> impl IntoResponse {
    let username = input.username;
    let id = "username-error".to_string();
    let mut error = Option::None;
    if username.is_empty() {
        error = Some("Username cannot be empty");
    } else if username.len() < 3 {
        error = Some("Username must be at least 3 characters long</p>");
    } else if username.len() > 20 {
        error = Some("Username must be at most 20 characters long</p>");
    } else if username.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        error = Some("Username can only contain alphanumeric characters and underscores</p>");
    }

    if let Some(error_message) = error {
        let form_error = FormError {
            id: id.clone(),
            error: Some(error_message.to_string()),
        };
        if let Ok(body) = form_error.render() {
            return Html(body);
        } else {
            return Html(
                "<p class='text-ctp-red text-xs'>Error rendering form error</p>".to_string(),
            );
        }
    }

    let is_not_available = !(state.user_service.is_username_available(username).await);
    let form_error = FormError {
        id: id.clone(),
        error: is_not_available.then(|| "Username is already taken".to_string()),
    };

    if let Ok(body) = form_error.render() {
        Html(body)
    } else {
        Html("<p class='text-ctp-red text-xs'>Error rendering form error</p>".to_string())
    }
}

#[derive(Debug, Clone, Template)]
#[template(path = "validate-user.html")]
struct ValidateUser {
    token: Uuid,
    is_authed: bool,
}
async fn get_validate_user(
    session_auth: SessionAuthState,
    State(AuthServices { auth_service, .. }): State<AuthServices>,
) -> Result<impl IntoResponse, AppError> {
    let username = session_auth.username();

    let token = auth_service
        .get_authorization(username)
        .await
        .map_err(|err| {
            info!("auth err: {err:?}");
            AppError::InternalServerError
        })?;

    let templ = ValidateUser {
        token,
        is_authed: true,
    };

    Ok(Html(templ.render()?))
}

// async fn post_validate_user() -> Result<Json<>, ApiError> {

async fn get_logout(session: Session, session_auth: Option<SessionAuthState>) -> impl IntoResponse {
    if let Some(session_auth) = session_auth {
        let _ = session_auth.logout(&session).await.inspect_err(|err| {
            info!("Error flushing state: {err:?}");
        });
    }
    Redirect::temporary("/")
}
