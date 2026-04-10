use askama::Template;
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue};
use axum::response::{Html, IntoResponse, Redirect, Response};
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

use crate::core::models::user_auth::UserAuthorizedState;
use crate::core::services::{AuthService, TaskService, UserService};
use crate::infra::alerts::{alert_success, map_err_to_alert};
use crate::infra::askama::{Globals, HtmlTemplate};
use crate::infra::auth::{
    redirect_auth_users, redirect_authorized_users, redirect_unauthenticated_users,
    SessionAuthState,
};
use crate::infra::error::{ApiError, AppError};

#[derive(Clone)]
struct AuthServices {
    user_service: UserService,
    auth_service: AuthService,
    task_service: TaskService,
}

pub fn auth_routes<S>(
    user_service: UserService,
    auth_service: AuthService,
    task_service: TaskService,
) -> axum::Router<S> {
    let unauthen_routes = Router::new()
        .route("/register", get(get_register))
        .route("/auth/register", post(post_start_registration))
        .route(
            "/auth/validate-registration",
            post(post_validate_registration),
        )
        .route("/login", get(get_login))
        .route("/auth/login", post(post_authenticate))
        .route("/auth/validate-login", post(post_validate_authenticate))
        .route("/auth/username_validation", get(username_validation))
        // redirect authorized users to task,
        // authenticated users to validate against taskdb
        .layer(middleware::from_fn(redirect_auth_users))
        .route("/logout", get(get_logout));

    let authed_routes = Router::new()
        .route("/add-passkey", get(get_add_passkey))
        .route(
            "/auth/register-sec-passkey",
            post(post_register_sec_passkey),
        )
        .route(
            "/auth/validate-sec-passkey",
            post(post_validate_sec_passkey),
        )
        .layer(middleware::from_fn(redirect_unauthenticated_users));

    Router::new()
        // unauthorized routes
        .route("/authorize-user", get(get_validate_user))
        .route("/auth/authorize-user", post(post_authorize_user))
        // redirect authorized users to tasks
        .layer(middleware::from_fn(redirect_authorized_users))
        .merge(unauthen_routes)
        .merge(authed_routes)
        .with_state(AuthServices {
            user_service,
            auth_service,
            task_service,
        })
}

#[derive(Debug, Clone, Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    is_authed: bool,
    globals: Globals,
}
async fn get_register() -> impl IntoResponse {
    let template = RegisterTemplate {
        is_authed: false,
        globals: Globals::default(),
    };
    HtmlTemplate(template)
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
//   4. auth_service│                     │                      │
//                  │  4. Yield PubKey    │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ──▶│                      │
//                  │                     │                      │
//                  │                     │     5. Send Reg      │
//                  │                     │        Opts          │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │          PubKey
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
    State(AuthServices { auth_service, .. }): State<AuthServices>,
    Json(payload): Json<RegistrationParams>,
) -> Result<Json<CreationChallengeResponse>, ApiError> {
    let username = payload.username;

    let (user, challenge) = auth_service
        .create_registration(&username)
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
    session: Session,
    session_auth: SessionAuthState,
    State(AuthServices { auth_service, .. }): State<AuthServices>,
    Json(cred): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, ApiError> {
    auth_service
        .validate_registration(session_auth.user_id(), &cred)
        .await
        .map_err(|err| {
            info!("Error registering user: {err:?}");
            ApiError::BadRequest {
                message: "Failed to validate credentials".to_string(),
            }
        })?;

    alert_success("Success. Login to continue.", &session)
        .await
        .map_err(|err| {
            info!("Err alerting: {err:?}");
            ApiError::InternalServerError
        })?;

    Ok((
        [(
            HeaderName::from_static("hx-redirect"),
            HeaderValue::from_static("/login"),
        )],
        "",
    ))
}

#[derive(Debug, Clone, Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    is_authed: bool,
    globals: Globals,
}
async fn get_login(session: Session) -> impl IntoResponse {
    let template = LoginTemplate {
        is_authed: false,
        globals: Globals::fetch(&session).await,
    };
    HtmlTemplate(template)
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
//            │     │  4. Yield Sig       │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │     5. Send Auth     │
//                  │                     │        Opts          │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │     │    Sig
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
        ..
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

async fn post_validate_authenticate(
    session: Session,
    session_auth: SessionAuthState,
    State(AuthServices { auth_service, .. }): State<AuthServices>,
    Json(pkc): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, ApiError> {
    auth_service
        .validate_login(session_auth.user_id(), &pkc)
        .await
        .map_err(|err| {
            info!("Error validating login: {:?}", err);
            ApiError::BadRequest {
                message: "Failed to validate login".to_string(),
            }
        })?;

    let auth_state = auth_service
        .get_authorization(session_auth.username())
        .await
        .map_err(|err| {
            info!("Error validating authorization: {:?}", err);
            ApiError::InternalServerError
        })?;

    session_auth
        .login(auth_state.clone())
        .update_session(&session)
        .await
        .or(Err(ApiError::InternalServerError))?;

    alert_success("Welcome Back!", &session)
        .await
        .or(Err(ApiError::InternalServerError))?;

    let redirect_path = match auth_state {
        UserAuthorizedState::Authorized(_) => HeaderValue::from_static("/task"),
        UserAuthorizedState::Not => HeaderValue::from_static("/authorize-user"),
    };

    Ok((
        [(HeaderName::from_static("hx-redirect"), redirect_path)],
        "",
    ))
}

#[derive(Deserialize, Debug)]
struct UsernameValidationParams {
    username: String,
    is_free: bool,
}

#[derive(Debug, Template)]
#[template(path = "partials/form-error.html")]
struct FormError {
    id: String,
    error: Option<String>,
}

async fn username_validation(
    State(AuthServices { user_service, .. }): State<AuthServices>,
    Form(UsernameValidationParams { username, is_free }): Form<UsernameValidationParams>,
) -> impl IntoResponse {
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

    let is_available = user_service.is_username_available(&username).await;
    let form_error = FormError {
        id: id.clone(),
        error: match (is_available, is_free) {
            (true, false) => Some("No user found for username".to_string()),
            (false, true) => Some("Username isn't available".to_string()),
            _ => None,
        },
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
    globals: Globals,
}
async fn get_validate_user(
    session: Session,
    session_auth: SessionAuthState,
    State(AuthServices { auth_service, .. }): State<AuthServices>,
) -> Result<impl IntoResponse, AppError> {
    let username = session_auth.username();

    let token = auth_service
        .get_authorization_token(username)
        .await
        .map_err(|err| {
            info!("auth err: {err:?}");
            AppError::InternalServerError
        })?;

    let templ = ValidateUser {
        token,
        is_authed: true,
        globals: Globals::fetch(&session).await,
    };

    Ok(HtmlTemplate(templ))
}

async fn post_authorize_user(
    session: Session,
    session_auth: SessionAuthState,
    State(AuthServices {
        task_service,
        auth_service,
        ..
    }): State<AuthServices>,
) -> Result<impl IntoResponse, Response> {
    let username = session_auth.username();
    let task = task_service
        .get_authorize_task()
        .await
        .map_err(map_err_to_alert)?;

    auth_service
        .authorize_user(username, task.get_uuid(), task.get_description())
        .await
        .map_err(map_err_to_alert)?;

    session_auth
        .authorize()
        .map_err(|err| {
            {
                info!("Error authorizing session for user: {err:?}");
                ApiError::InternalServerError
            }
            .into_response()
        })?
        .update_session(&session)
        .await
        .map_err(|err| {
            {
                info!("Error updating session for user: {err:?}");
                ApiError::InternalServerError
            }
            .into_response()
        })?;

    Ok((
        [(
            HeaderName::from_static("hx-redirect"),
            HeaderValue::from_static("/task"),
        )],
        "",
    )
        .into_response())
}

async fn get_logout(session: Session, session_auth: Option<SessionAuthState>) -> impl IntoResponse {
    if let Some(session_auth) = session_auth {
        let _ = session_auth.logout(&session).await.inspect_err(|err| {
            info!("Error flushing state: {err:?}");
        });
    }
    Redirect::temporary("/")
}

#[derive(Debug, Clone, Template)]
#[template(path = "add-passkey.html")]
struct AddPasskeyTemplate {
    is_authed: bool,
    globals: Globals,
}
async fn get_add_passkey(session: Session) -> impl IntoResponse {
    let template = AddPasskeyTemplate {
        is_authed: true,
        globals: Globals::fetch(&session).await,
    };
    HtmlTemplate(template)
}

//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │  1. Start Add Key    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 2. Fetch existing passkeys
//                  │                     │                      │     │ Start new registration
//                  │                     │                      │◀─ ─ ┘
//                  │                     │  3. Challenge w/     │
//                  │                     │     Ex. Credentials  │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  4. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//   5.Verify │     │                     │                      │
//            │     │  5. Yield PubKey    │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─  ─▶│                      │
//                  │                     │                      │
//                  │                     │  6. Send PK Cred     │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 6. Verify
//                  │                     │                      │       PubKey
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │─ ─ ─
//                  │                     │                      │     │ 7. Append
//                  │                     │                      │       Credential
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
async fn post_register_sec_passkey(
    session_auth: SessionAuthState,
    State(AuthServices { auth_service, .. }): State<AuthServices>,
) -> Result<Json<CreationChallengeResponse>, ApiError> {
    let user_id = session_auth.user_id();
    let username = session_auth.username();

    let challenge = auth_service
        .start_sec_passkey_registration(user_id, username)
        .await
        .map_err(|err| {
            info!("Error creating registration: {:?}", err);
            ApiError::BadRequest {
                message: "Failed to create registration".to_string(),
            }
        })?;

    Ok(Json(challenge))
}

async fn post_validate_sec_passkey(
    session: Session,
    session_auth: SessionAuthState,
    State(AuthServices { auth_service, .. }): State<AuthServices>,
    Json(cred): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, Response> {
    let user_id = session_auth.user_id();

    auth_service
        .validate_sec_passkey(user_id, &cred)
        .await
        .map_err(|err| {
            info!("Err validating sec passkey: {err:?}");
            ApiError::BadRequest {
                message: "Failed to validate credentials".to_string(),
            }
            .into_response()
        })?;

    alert_success("Successfully added second passkeys", &session)
        .await
        .map_err(map_err_to_alert)?;

    Ok((
        [(
            HeaderName::from_static("hx-redirect"),
            HeaderValue::from_static("/task"),
        )],
        "",
    ))
}
