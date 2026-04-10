use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Clone)]
pub enum UserAuthorizedState {
    Not,
    Authorized(Uuid),
}

#[derive(Debug, Clone)]
pub struct UserAuth {
    user_id: Uuid,
    passkeys: Vec<Passkey>,
    registration: Option<PasskeyRegistration>,
    authentication: Option<PasskeyAuthentication>,
    authorize_token: Uuid,
    authorized_state: UserAuthorizedState,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkeys: Vec::new(),
            authorize_token: Uuid::new_v4(),
            authorized_state: UserAuthorizedState::Not,
        }
    }

    #[must_use]
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    #[must_use]
    pub fn registration(&self) -> Option<PasskeyRegistration> {
        self.registration.clone()
    }
    #[must_use]
    pub fn authentication(&self) -> Option<PasskeyAuthentication> {
        self.authentication.clone()
    }
    pub fn set_authentication(&mut self, authentication: Option<PasskeyAuthentication>) {
        self.authentication = authentication;
    }

    #[must_use]
    pub fn passkeys(&self) -> Vec<Passkey> {
        self.passkeys.clone()
    }
    pub fn set_passkey(&mut self, passkeys: Vec<Passkey>) {
        self.passkeys = passkeys;
    }

    #[must_use]
    pub fn authorize_token(&self) -> Uuid {
        self.authorize_token
    }

    pub fn is_authorized(&self) -> bool {
        matches!(self.authorized_state, UserAuthorizedState::Authorized(_))
    }

    #[must_use]
    pub fn authorized_state(&self) -> UserAuthorizedState {
        self.authorized_state.clone()
    }
}
