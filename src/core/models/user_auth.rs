use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Clone)]
pub struct UserAuth {
    user_id: Uuid,
    passkeys: Vec<Passkey>,
    registration: Option<PasskeyRegistration>,
    authentication: Option<PasskeyAuthentication>,
    authorize_token: Option<Uuid>,
    is_authorized: bool,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkeys: Vec::new(),
            authorize_token: None,
            is_authorized: false,
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
    pub fn set_authentication(&mut self, authentication: Option<PasskeyAuthentication>) {
        self.authentication = authentication;
    }

    pub fn passkey(&self) -> Vec<Passkey> {
        self.passkeys.clone()
    }
    pub fn set_passkey(&mut self, passkeys: Vec<Passkey>) {
        self.passkeys = passkeys;
    }

    pub fn authorize_token(&self) -> Option<Uuid> {
        self.authorize_token
    }

    pub fn gen_authorize_token(&mut self) {
        self.authorize_token = Some(Uuid::new_v4())
    }

    pub fn is_authorized(&self) -> bool {
        self.is_authorized
    }
}
