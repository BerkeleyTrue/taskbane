use std::sync::Arc;

use tracing::info;
use uuid::Uuid;
use webauthn_rs::{
    prelude::{
        CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
        RequestChallengeResponse,
    },
    Webauthn,
};

use crate::core::{
    models::{User, UserAuth},
    ports::AuthRepository,
};

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

    pub async fn validate_login(
        &self,
        user_id: &Uuid,
        pkc: &PublicKeyCredential,
    ) -> Result<(), String> {
        let pka = self.repo.get_authentication(user_id).await?;

        let credentials = self
            .webauthn
            .finish_passkey_authentication(pkc, &pka)
            .or_else(|err| Err(err.to_string()))?;
        self.repo.update_credentials(user_id, credentials).await?;
        Ok(())
    }
}
