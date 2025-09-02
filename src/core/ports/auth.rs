use async_trait::async_trait;
use uuid::Uuid;
use webauthn_rs::prelude::{
    AuthenticationResult, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

use crate::core::models::UserAuth;

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn add(&self, stored_challenge: UserAuth) -> Result<UserAuth, String>;
    async fn get_registration(&self, user_id: &Uuid) -> Result<PasskeyRegistration, String>;

    async fn update_passkey(&self, user_id: &Uuid, pk: Passkey) -> Result<(), String>;
    async fn get_passkeys(&self, user_id: &Uuid) -> Result<Vec<Passkey>, String>;

    async fn update_authen(&self, user_id: &Uuid, pka: PasskeyAuthentication)
        -> Result<(), String>;
    async fn get_authentication(&self, user_id: &Uuid) -> Result<PasskeyAuthentication, String>;
    async fn update_credentials(
        &self,
        user_id: &Uuid,
        credentials: AuthenticationResult,
    ) -> Result<(), String>;
}
