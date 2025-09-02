use std::sync::Arc;

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
        let (ccr, registration) = self
            .webauthn
            .start_passkey_registration(user.id(), user.username(), user.username(), None)
            .map_err(|err| err.to_string())?;

        let user_auth = UserAuth::new(user.id().clone(), registration);

        self.repo.add(user_auth).await?;
        Ok(ccr)
    }

    pub async fn validate_registration(
        &self,
        user_id: &Uuid,
        cred: &RegisterPublicKeyCredential,
    ) -> Result<(), String> {
        let reg = self.repo.get_registration(user_id).await?;

        let pk = self
            .webauthn
            .finish_passkey_registration(cred, &reg)
            .map_err(|err| err.to_string())?;

        self.repo.update_passkey(user_id, pk).await?;

        Ok(())
    }

    pub async fn login(&self, user_id: &Uuid) -> Result<RequestChallengeResponse, String> {
        let passkeys = self.repo.get_passkeys(user_id).await?;

        let (rcr, pka) = self
            .webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|err| err.to_string())?;

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
