use std::{env, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use webauthn_rs::{prelude::Url, Webauthn, WebauthnBuilder};

#[derive(Error, Debug)]
pub enum WebauthnError {
    #[error("unknown webauthn error")]
    Unknown,
    #[error("User Not Found")]
    UserNotFound,
    #[error("User Has No Credentials")]
    UserHasNoCredentials,
}

impl IntoResponse for WebauthnError {
    fn into_response(self) -> Response {
        let body = match self {
            WebauthnError::UserNotFound => "User Not Found",
            WebauthnError::Unknown => "Unknown Error",
            WebauthnError::UserHasNoCredentials => "User Has No Credentials",
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

pub fn create_authn() -> Arc<Webauthn> {
    let rp_id = &env::var("RP_ID").unwrap_or("localhost".to_string());
    let rp_name = env::var("RP_NAME").unwrap_or("taskbane".to_string());
    let rp_origin = &env::var("ORIGIN").expect("No Origin Defined");
    let rp_origin = Url::parse(rp_origin).expect("Invalid Url");

    let builder = WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid WebAuthn config")
        .rp_name(&rp_name);

    Arc::new(builder.build().expect("Invalid WebAuthn config"))
}
