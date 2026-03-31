use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum UserAuthorizedState {
    Not,
    Authorized,
}

#[derive(Debug, Clone)]
pub struct UserAuth {
    user_id: Uuid,
    passkeys: Vec<Passkey>,
    registration: Option<PasskeyRegistration>,
    authentication: Option<PasskeyAuthentication>,
    authorize_token: Uuid,
    auth_state: UserAuthorizedState,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkeys: Vec::new(),
            authorize_token: Uuid::new_v4(),
            auth_state: UserAuthorizedState::Not,
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

    pub fn authorize_token(&self) -> Uuid {
        self.authorize_token
    }

    pub fn is_authorized(&self) -> bool {
        matches!(self.auth_state, UserAuthorizedState::Authorized)
    }
}
