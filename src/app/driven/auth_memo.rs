use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;
use webauthn_rs::{
    prelude::{
        AuthenticationResult, CreationChallengeResponse, Passkey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse
    },
    Webauthn,
};

use crate::core::models::User;

#[derive(Debug, Clone)]
pub struct UserAuth {
    registration: Option<PasskeyRegistration>,
    authentication: Option<PasskeyAuthentication>,
    passkeys: Vec<Passkey>,
    user_id: Uuid,
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

#[derive(Debug)]
pub struct AuthStore {
    auths: std::collections::HashMap<Uuid, UserAuth>,
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn add(&self, stored_challenge: UserAuth) -> Result<UserAuth, String>;
    async fn get_registration(&self, user_id: &Uuid) -> Result<PasskeyRegistration, String>;

    async fn update_passkey(&self, user_id: &Uuid, pk: Passkey) -> Result<(), String>;
    async fn get_passkeys(&self, user_id: &Uuid) -> Result<Vec<Passkey>, String>;

    async fn update_authen(&self, user_id: &Uuid, pka: PasskeyAuthentication)
        -> Result<(), String>;
    async fn get_authentication(&self, user_id: &Uuid) -> Result<PasskeyAuthentication, String>;
    async fn update_credentials(&self, user_id: &Uuid, credentials: AuthenticationResult) -> Result<(), String>;
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

    async fn get_passkeys(&self, user_id: &Uuid) -> Result<Vec<Passkey>, String> {
        let store = self.store.lock().await;
        let user_auth = store
            .auths
            .get(user_id)
            .ok_or("No auth found for user".to_string())?;
        Ok(user_auth.passkeys.clone())
    }

    async fn update_passkey(&self, user_id: &Uuid, pk: Passkey) -> Result<(), String> {
        let mut store = self.store.lock().await;

        let Some(user_auth) = store.auths.get(user_id) else {
            return Err("No auth found for user".to_string());
        };
        let mut user_auth = user_auth.clone();

        user_auth.passkeys.push(pk);
        user_auth.registration = None;

        store.auths.insert(user_id.clone(), user_auth);
        Ok(())
    }

    async fn update_authen(
        &self,
        user_id: &Uuid,
        pka: PasskeyAuthentication,
    ) -> Result<(), String> {
        let mut store = self.store.lock().await;
        let mut user_auth = store.auths.get(user_id).ok_or("No auth found for user".to_string())?.clone();

        user_auth.authentication = Some(pka);
        user_auth.registration = None;

        store.auths.insert(user_id.clone(), user_auth);

        Ok(())
    }

    async fn get_authentication(&self, user_id: &Uuid) -> Result<PasskeyAuthentication, String> {
        let mut store = self.store.lock().await;
        let mut user_auth = store
            .auths
            .get(user_id)
            .ok_or("Could not find user auth".to_string())?
            .clone();

        let pka = user_auth
            .authentication()
            .ok_or("User has no authentication in progress".to_string())?;

        user_auth.authentication = None;
        user_auth.registration = None;

        store.auths.insert(user_id.clone(), user_auth);

        Ok(pka)
    }

    async fn update_credentials(&self, user_id: &Uuid, credentials: AuthenticationResult) -> Result<(), String> {
        let mut store = self.store.lock().await;
        let mut auth_state = store.auths.get(user_id).ok_or("Could not find user auth".to_string())?.clone();

        auth_state.passkeys.iter_mut().for_each(|pk| {
            // This will update the credential if it's the matching
            // one. Otherwise it's ignored. That is why it is safe to
            // iterate this over the full list.
            pk.update_credential(&credentials);
        });

        store.auths.insert(user_id.clone(), auth_state);
        Ok(())
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

    pub async fn validate_registration(
        &self,
        user_id: &Uuid,
        cred: &RegisterPublicKeyCredential,
    ) -> Result<(), String> {
        let reg = self.repo.get_registration(user_id).await?;
        match self.webauthn.finish_passkey_registration(cred, &reg) {
            Ok(pk) => {
                self.repo.update_passkey(user_id, pk).await?;
                return Ok(());
            }
            Err(e) => {
                info!("validating registration {:?}", e);
                return Err("Error validating registration".to_string());
            }
        }
    }

    pub async fn login(&self, user_id: &Uuid) -> Result<RequestChallengeResponse, String> {
        let passkeys = self.repo.get_passkeys(user_id).await?;
        let (rcr, pka) = self
            .webauthn
            .start_passkey_authentication(&passkeys)
            .or_else(|err| {
                return Err(err.to_string());
            })?;

        self.repo.update_authen(user_id, pka).await?;

        Ok(rcr)
    }

    pub async fn validate_login(&self, user_id: &Uuid, pkc: &PublicKeyCredential) -> Result<(), String> {
        let pka = self.repo.get_authentication(user_id).await?;

        let credentials = self.webauthn.finish_passkey_authentication(pkc, &pka).or_else(|err| { Err(err.to_string()) })?;
        self.repo.update_credentials(user_id, credentials).await?;
        Ok(())
    }
}

pub fn create_auth_service(webauthn: Arc<Webauthn>) -> AuthService {
    let store = Arc::new(Mutex::new(AuthStore {
        auths: HashMap::new(),
    }));
    let repo = Arc::new(AuthMemRepo { store });

    AuthService::new(repo, webauthn)
}
