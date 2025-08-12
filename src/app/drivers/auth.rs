use axum::extract::State;
use axum::{
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::core::models;
use crate::infra::axum::route;
use crate::services::user::UserService;

pub fn auth_routes<S>(user_service: UserService) -> axum::Router<S> {
    route("/auth/register", post(post_registration))
        .route("/auth/is_username_available", get(is_username_available))
        .with_state(user_service)
}

#[derive(Deserialize)]
struct RegistrationParams {
    username: String,
}

#[derive(serde::Serialize)]
struct RegistrationOptions {
    user: models::User,
}

#[derive(serde::Serialize)]
struct RegistrationFail {
    message: String,
}

async fn post_registration(
    State(user_service): State<UserService>,
    Json(payload): Json<RegistrationParams>,
) -> Result<Json<RegistrationOptions>, Json<RegistrationFail>> {
    let username = payload.username;
    let Ok(user) = user_service.register_user(username).await else {
        return Err(Json(RegistrationFail {
            message: "Fail".to_string(),
        }));
    };
    Ok(Json(RegistrationOptions { user }))
}

#[derive(Serialize)]
struct IsUsernameAvailRes {
    is_available: bool,
}

async fn is_username_available(
    State(user_service): State<UserService>,
    Json(payload): Json<RegistrationParams>,
) -> Json<IsUsernameAvailRes> {
    let username = payload.username;
    let is_available = user_service.is_username_available(username).await;
    Json(IsUsernameAvailRes { is_available })
}
