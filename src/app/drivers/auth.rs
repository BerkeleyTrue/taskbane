use axum::extract::State;
use axum::{routing::post, Json};
use serde::Deserialize;

use crate::core::models;
use crate::infra::axum::route;
use crate::services::user::UserService;

pub fn auth_routes<S>(user_service: UserService) -> axum::Router<S> {
    route("/auth/register", post(post_registration)).with_state(user_service)
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
