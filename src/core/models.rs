use derive_more::{Constructor, Eq, PartialEq};
use serde::Serialize;
use sqlx::prelude::FromRow;
use taskchampion::{Status, Task};
use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Clone, Serialize, FromRow, PartialEq, Eq, Constructor)]
pub struct User {
    pub id: Uuid,
    #[eq(skip)]
    pub username: String,
}

impl User {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn with_username(&mut self, username: String) -> &mut Self {
        self.username = username;
        self
    }
}

#[derive(Debug, Clone)]
pub struct UserAuth {
    pub user_id: Uuid,
    pub passkeys: Vec<Passkey>,
    pub registration: Option<PasskeyRegistration>,
    pub authentication: Option<PasskeyAuthentication>,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkeys: Vec::new(),
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
}

#[derive(Debug, Clone)]
pub struct TaskDto {
    pub id: usize,
    pub status: Status,
    pub description: String,
    pub priority: String,
}

impl TaskDto {
    pub fn from(id: usize, value: Task) -> Self {
        Self {
            id,
            status: value.get_status(),
            description: value.get_description().to_owned(),
            priority: value.get_priority().to_owned(),
        }
    }
}
