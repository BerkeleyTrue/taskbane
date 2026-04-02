use std::sync::Arc;

use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use sqlx::{self, SqlitePool};
use uuid::Uuid;
use webauthn_rs::prelude::{
    AuthenticationResult, Passkey, PasskeyAuthentication, PasskeyRegistration,
};

use crate::core::{
    models::user_auth::{UserAuth, UserAuthorizedState},
    ports::AuthRepository,
};

pub struct AuthSqlRepo {
    pool: SqlitePool,
}

#[async_trait]
impl AuthRepository for AuthSqlRepo {
    async fn add(&self, auth: UserAuth) -> Result<UserAuth> {
        let user_id = auth.user_id();
        let existing_auth = sqlx::query!(
            r#"
                SELECT registration FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .is_some();

        if existing_auth {
            return Err(anyhow!("Existing Challenge for user"));
        }

        let user_id = auth.user_id();
        let registration = auth
            .registration()
            .and_then(|r| serde_json::to_string(&r).ok());

        let auth_state = auth.auth_state();
        sqlx::query!(
            r#"
                INSERT INTO auth (user_id, registration, authentication, passkeys, auth_state)
                VALUES (?, ?, ?, ?, ?)
                returning user_id as `user_id:uuid::Uuid`
            "#,
            user_id,
            registration,
            None::<String>,
            "[]",
            auth_state,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(auth)
    }

    async fn get_registration(&self, user_id: Uuid) -> Result<PasskeyRegistration> {
        let maybe_registration = sqlx::query!(
            r#"
                SELECT registration FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .and_then(|r| r.registration)
        .and_then(|r| serde_json::from_str::<PasskeyRegistration>(&r).ok());

        let Some(registration) = maybe_registration else {
            return Err(anyhow!(
                "No registration found for {:?}",
                user_id.to_string()
            ));
        };

        Ok(registration)
    }

    async fn get_passkeys(&self, user_id: Uuid) -> Result<Vec<Passkey>> {
        sqlx::query!(
            r#"
                SELECT passkeys FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| r.passkeys)
        .and_then(|pks| serde_json::from_str::<Vec<Passkey>>(&pks).ok())
        .ok_or(anyhow!("No passkeys found for user"))
    }

    async fn update_passkey(&self, user_id: Uuid, pk: Passkey) -> Result<()> {
        let mut passkeys = sqlx::query!(
            r#"
                SELECT passkeys FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|record| record.passkeys)
        .ok_or(anyhow!("No auth found for user"))
        .and_then(|psk_str| serde_json::from_str::<Vec<Passkey>>(&psk_str).map_err(Error::from))?;

        passkeys.push(pk);

        let passkeys_json = serde_json::to_string(&passkeys)?;

        sqlx::query!(
            r#"
                UPDATE auth
                SET passkeys = ?, registration = NULL
                WHERE user_id = ?
            "#,
            passkeys_json,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_authen(&self, user_id: Uuid, pka: PasskeyAuthentication) -> Result<()> {
        // make sure user has existing auth
        sqlx::query!(
            r#"
                SELECT authentication FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow!("No auth found for user"))?;

        let pka_json = serde_json::to_string(&pka)?;

        sqlx::query!(
            r#"
                UPDATE auth
                SET authentication = ?, registration = NULL
                WHERE user_id = ?
                RETURNING user_id as `user_id:uuid::Uuid`
            "#,
            pka_json,
            user_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::from)
        .map(|_| ())
    }

    async fn get_authentication(&self, user_id: Uuid) -> Result<PasskeyAuthentication> {
        sqlx::query!(
            r#"
                SELECT authentication FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow!("No auth found for user"))
        .and_then(|r| r.authentication.ok_or(anyhow!("No auth found for user")))
        .and_then(|a| serde_json::from_str::<PasskeyAuthentication>(&a).map_err(Error::from))
    }

    async fn update_credentials(
        &self,
        user_id: Uuid,
        credentials: AuthenticationResult,
    ) -> Result<()> {
        let mut passkeys = sqlx::query!(
            r#"
                SELECT passkeys FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|record| record.passkeys)
        .ok_or(anyhow!("No auth found for user"))
        .and_then(|psk_str| serde_json::from_str::<Vec<Passkey>>(&psk_str).map_err(Error::from))?;

        // This will update the credential if it's the matching
        // one. Otherwise it's ignored. That is why it is safe to
        // iterate this over the full list.
        passkeys.iter_mut().for_each(|pk| {
            pk.update_credential(&credentials);
        });

        let passkeys_json = serde_json::to_string(&passkeys)?;

        sqlx::query!(
            r#"
                UPDATE auth
                SET passkeys = ?
                WHERE user_id = ?
            "#,
            passkeys_json,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_authorization_token(&self, user_id: Uuid) -> Result<Option<Uuid>> {
        sqlx::query!(
            r#"
                SELECT authorize_token as `authorize_token:uuid::Uuid` FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow!("No authorize token found for user"))
        .map(|r| r.authorize_token)
    }

    async fn update_authorization_token(&self, user_id: Uuid, token: Uuid) -> Result<()> {
        // make sure user has existing auth
        sqlx::query!(
            r#"
                SELECT authorize_token FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow!("No authorization token found for user"))?;

        sqlx::query!(
            r#"
                UPDATE auth
                SET authorize_token = ?, auth_state = ?
                WHERE user_id = ?
            "#,
            token,
            UserAuthorizedState::Authorized,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(())
    }
    async fn get_authorization(&self, user_id: Uuid) -> Result<UserAuthorizedState> {
        sqlx::query!(
            r#"
                SELECT auth_state as "auth_state:crate::core::models::user_auth::UserAuthorizedState" FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow!("No auth state found for user"))
        .map(|r| r.auth_state)
    }
}

pub fn create_auth_repo(pool: &SqlitePool) -> Arc<AuthSqlRepo> {
    Arc::new(AuthSqlRepo { pool: pool.clone() })
}
