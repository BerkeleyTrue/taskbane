use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Clone)]
pub struct UserAuth {
    pub user_id: Uuid,
    pub passkeys: Vec<Passkey>,
    pub registration: Option<PasskeyRegistration>,
    pub authentication: Option<PasskeyAuthentication>,
    pub authorize_token: Option<Uuid>,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkeys: Vec::new(),
            authorize_token: None,
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    pub fn registration(&self) -> Option<PasskeyRegistration> {
        self.registration.clone()
    }
    pub fn authentication(&self) -> Option<PasskeyAuthentication> {
        self.authentication.clone()
    }
    pub fn passkey(&self) -> Vec<Passkey> {
        self.passkeys.clone()
    }

    pub fn authorize_token(&self) -> Option<Uuid> {
        self.authorize_token
    }
}
