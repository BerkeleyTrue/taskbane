use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;
use webauthn_rs::{
    prelude::{
        CreationChallengeResponse, Passkey, PasskeyAuthentication, PasskeyRegistration,
        RegisterPublicKeyCredential,
    },
    Webauthn,
};

use crate::core::models::User;

#[derive(Debug, Clone)]
pub struct UserAuth {
    registration: Option<PasskeyRegistration>,
    authentication: Option<PasskeyAuthentication>,
    passkey: Option<Passkey>,
    user_id: Uuid,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkey: None,
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
    pub fn passkey(&self) -> Option<Passkey> {
        self.passkey.clone()
    }
}

#[derive(Debug)]
pub struct AuthStore {
    auths: std::collections::HashMap<Uuid, UserAuth>,
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn add(&self, stored_challenge: UserAuth) -> Result<UserAuth, String>;
    async fn get_registration(&self, user_id: &Uuid) -> Result<PasskeyRegistration, String>;
}

struct AuthMemRepo {
    store: Arc<Mutex<AuthStore>>,
}

#[async_trait]
impl AuthRepository for AuthMemRepo {
    async fn add(&self, auth: UserAuth) -> Result<UserAuth, String> {
        let mut store = self.store.lock().await;
        let user_id = auth.user_id().clone();
        let existing_auth = store
            .auths
            .iter()
            .any(move |(uuid, _)| uuid.clone() == user_id);

        if existing_auth {
            return Err("Existing Challenge for user".to_string());
        }

        let auth_clone = auth.clone();
        store.auths.insert(auth_clone.user_id(), auth_clone);
        Ok(auth.clone())
    }

    async fn get_registration(&self, user_id: &Uuid) -> Result<PasskeyRegistration, String> {
        let store = self.store.lock().await;
        let user_id_clone = user_id.clone();

        let Some((_, user_auth)) = store
            .auths
            .iter()
            .find(|(&u_id, _)| user_id_clone == u_id.clone())
        else {
            return Err(format!("No auth found for {:?}", user_id.to_string()));
        };

        let Some(reg) = user_auth.registration() else {
            return Err("No auth found for user".to_string());
        };

        Ok(reg)
    }
}

#[derive(Clone)]
pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
    webauthn: Arc<Webauthn>,
}

impl AuthService {
    pub fn new(repo: Arc<dyn AuthRepository>, webauthn: Arc<Webauthn>) -> Self {
        Self { repo, webauthn }
    }

    pub async fn create_registration(
        &self,
        user: User,
    ) -> Result<CreationChallengeResponse, String> {
        match self.webauthn.start_passkey_registration(
            user.id(),
            user.username(),
            user.username(),
            None,
        ) {
            Ok((ccr, registration)) => {
                let user_auth = UserAuth::new(user.id().clone(), registration);

                self.repo.add(user_auth).await?;
                Ok(ccr)
            }

            Err(e) => {
                info!("register error {:?}", e);
                Err("Failed to register user".to_string())
            }
        }
    }

    pub async fn validate_registration(&self, user_id: &Uuid, cred: &RegisterPublicKeyCredential) -> Result<(), String> {
        let reg = self.repo.get_registration(user_id).await?;
        match self.webauthn.finish_passkey_registration(cred, &reg) {
            Ok(pk) => {
                // let mut store = self.repo;
                return Ok(());
            },
            Err(e) => {
                info!("validating registration {:?}", e);
                return Err("Error validating registration".to_string())
            }
        }
    }
}

pub fn create_auth_service(webauthn: Arc<Webauthn>) -> AuthService {
    let store = Arc::new(Mutex::new(AuthStore {
        auths: HashMap::new(),
    }));
    let repo = Arc::new(AuthMemRepo { store });

    AuthService::new(repo, webauthn)
}
