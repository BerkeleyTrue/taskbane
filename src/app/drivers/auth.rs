use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::{
    routing::{get, post},
    Json,
};
use axum::{Form, Router};
use serde::Deserialize;

use crate::app::driven::auth::{ChallengeService, StoredChallenge};
use crate::core::models;
use crate::services::user::UserService;

#[derive(Clone)]
struct AuthState {
    user_service: UserService,
    challenge_service: ChallengeService,
}

pub fn auth_routes<S>(
    user_service: UserService,
    challenge_service: ChallengeService,
) -> axum::Router<S> {
    Router::new()
        .route("/auth/register", post(post_registration))
        .route("/auth/username_validation", get(username_validation))
        .with_state(AuthState {
            user_service,
            challenge_service,
        })
}

#[derive(Deserialize)]
struct RegistrationParams {
    username: String,
}

#[derive(serde::Serialize)]
struct RegistrationOptions {
    user: models::User,
    challenge: StoredChallenge,
}

#[derive(serde::Serialize)]
struct RegistrationFail {
    message: String,
}

async fn post_registration(
    State(state): State<AuthState>,
    Json(payload): Json<RegistrationParams>,
) -> Result<Json<RegistrationOptions>, Json<RegistrationFail>> {
    let username = payload.username;
    let Ok(user) = state.user_service.register_user(username).await else {
        return Err(Json(RegistrationFail {
            message: "Fail".to_string(),
        }));
    };

    let Ok(challenge) = state
        .challenge_service
        .create_challenge(user.id().clone())
        .await
    else {
        return Err(Json(RegistrationFail {
            message: "Failed to generate challenge".to_string(),
        }));
    };
    Ok(Json(RegistrationOptions { user, challenge }))
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
    State(state): State<AuthState>,
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
        return Html(body);
    } else {
        return Html("<p class='text-ctp-red text-xs'>Error rendering form error</p>".to_string());
    }
}
