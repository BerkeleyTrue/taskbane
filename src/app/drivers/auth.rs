use axum::{routing::post, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::infra::axum::route;

pub fn auth_routes() -> axum::Router {
    route("/auth/register", post(post_registration))
}

#[derive(Deserialize)]
struct RegistrationParams {
    username: String,
}

#[derive(serde::Serialize)]
struct RegistrationOptions {
    id: Uuid,
    username: String,
}

async fn post_registration(Json(payload): Json<RegistrationParams>) -> Json<RegistrationOptions> {
    let username = payload.username;
    let id = Uuid::new_v4();
    Json(RegistrationOptions { id, username })
}
