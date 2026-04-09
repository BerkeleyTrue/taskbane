use std::sync::Arc;

use anyhow::{anyhow, Result};
use derive_more::Constructor;
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
    models::{
        user::User,
        user_auth::{UserAuth, UserAuthorizedState},
    },
    ports::auth::AuthRepository,
    services::UserService,
};

#[derive(Clone, Constructor)]
pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
    webauthn: Arc<Webauthn>,
    user_service: UserService,
}

impl AuthService {
    pub async fn create_registration(
        &self,
        username: &str,
    ) -> Result<(User, CreationChallengeResponse)> {
        let user = self
            .user_service
            .register_user(username)
            .await
            .map_err(|err| {
                info!("Error registering user: {:?}", err);
                anyhow!("Username already exists")
            })?;

        let (ccr, registration) = self.webauthn.start_passkey_registration(
            user.id(),
            user.username(),
            user.username(),
            None,
        )?;

        let user_auth = UserAuth::new(user.id(), registration);

        self.repo.add(user_auth).await?;
        Ok((user, ccr))
    }

    pub async fn validate_registration(
        &self,
        user_id: Uuid,
        cred: &RegisterPublicKeyCredential,
    ) -> Result<()> {
        let reg = self.repo.get_registration(user_id).await?;

        let pk = self.webauthn.finish_passkey_registration(cred, &reg)?;

        self.repo.update_passkey(user_id, pk).await?;

        Ok(())
    }

    pub async fn login(&self, user_id: Uuid) -> Result<RequestChallengeResponse> {
        let passkeys = self.repo.get_passkeys(user_id).await?;

        let (rcr, pka) = self.webauthn.start_passkey_authentication(&passkeys)?;

        self.repo.update_authen(user_id, pka).await?;

        Ok(rcr)
    }

    pub async fn validate_login(&self, user_id: Uuid, pkc: &PublicKeyCredential) -> Result<()> {
        let pka = self.repo.get_authentication(user_id).await?;

        let credentials = self.webauthn.finish_passkey_authentication(pkc, &pka)?;

        self.repo.update_credentials(user_id, credentials).await?;

        Ok(())
    }

    pub async fn start_sec_passkey_registration(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<CreationChallengeResponse> {
        let passkeys = self
            .repo
            .get_passkeys(user_id)
            .await
            .map_err(|err| {
                info!("get passkeys err: {err:?}");
                anyhow!("No existing passkeys found for user")
            })?
            .into_iter()
            .map(|pk| pk.cred_id().to_owned())
            .collect::<Vec<_>>();

        let (ccr, registration) = self.webauthn.start_passkey_registration(
            user_id,
            username,
            username,
            Some(passkeys),
        )?;

        self.repo.update_registration(user_id, registration).await?;

        Ok(ccr)
    }

    pub async fn validate_sec_passkey(
        &self,
        user_id: Uuid,
        cred: &RegisterPublicKeyCredential,
    ) -> Result<()> {
        let reg = self.repo.get_registration(user_id).await?;

        let pk = self.webauthn.finish_passkey_registration(cred, &reg)?;

        self.repo.update_passkey(user_id, pk).await?;

        Ok(())
    }

    // # Authorization logic

    pub async fn get_authorization_token(&self, username: &str) -> Result<Uuid> {
        let user = self.user_service.get_user(username).await?;
        let authorize_token = match self.repo.get_authorization_token(user.id()).await? {
            Some(uuid) => uuid,
            None => {
                let new_token = Uuid::new_v4();
                self.repo
                    .update_authorization_token(user.id(), new_token)
                    .await?;
                new_token
            }
        };

        Ok(authorize_token)
    }

    pub async fn authorize_user(
        &self,
        username: &str,
        task_id: Uuid,
        task_description: &str,
    ) -> Result<()> {
        info!("auth token: {task_id}, '{task_description}");
        if !task_description.starts_with("taskbane:") {
            return Err(anyhow!("Task is not an authorizing task"));
        }

        let uploaded_token = task_description
            .split("taskbane:")
            .nth(1)
            .inspect(|str| info!("uuid: {str}"))
            .and_then(|str| Uuid::parse_str(str).ok())
            .ok_or(anyhow!("No token found in authorizing task"))?;

        let user = self.user_service.get_user(username).await?;

        let authorize_token = self
            .repo
            .get_authorization_token(user.id())
            .await
            .and_then(|maybe_token| {
                maybe_token.ok_or(anyhow!("User currently has no authorizing token."))
            })?;

        if authorize_token != uploaded_token {
            return Err(anyhow!(
                "User authorizing token did not match given task token"
            ));
        }
        self.repo
            .update_authorization_token(user.id(), uploaded_token)
            .await?;

        Ok(())
    }

    pub async fn get_authorization(&self, username: &str) -> Result<UserAuthorizedState> {
        let user = self.user_service.get_user(username).await?;
        self.repo.get_authorization(user.id()).await
    }
}
