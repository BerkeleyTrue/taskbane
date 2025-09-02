use std::{sync::Arc};

use async_trait::async_trait;
use sqlx::{self, SqlitePool};
use uuid::Uuid;
use webauthn_rs::{
    prelude::{
        AuthenticationResult, Passkey, PasskeyAuthentication,
        PasskeyRegistration,
    },
};

use crate::core::{
    models::UserAuth,
    ports::AuthRepository,
};

#[derive(Debug)]
struct AuthStateDb {
    user_id: Uuid,
    passkeys: String,
    registration: Option<String>,
    authentication: Option<String>,
}

impl From<AuthStateDb> for UserAuth {
    fn from(value: AuthStateDb) -> Self {
        let registration = value.registration.and_then(|r| {
            serde_json::from_str::<PasskeyRegistration>(&r).ok()
        });
        let authentication = value.authentication.and_then(|a| {
            serde_json::from_str::<PasskeyAuthentication>(&a).ok()
        });
        let passkeys = serde_json::from_str::<Vec<Passkey>>(&value.passkeys).unwrap_or(Vec::new());
        Self {
            user_id: value.user_id,
            passkeys,
            registration,
            authentication,
        }
    }
}

pub struct AuthSqlRepo {
    pool: SqlitePool,
}

#[async_trait]
impl AuthRepository for AuthSqlRepo {
    async fn add(&self, auth: UserAuth) -> Result<UserAuth, String> {
        let user_id = auth.user_id().clone();
        let existing_auth = sqlx::query!(
            r#"
                SELECT registration FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .is_some();

        if existing_auth {
            return Err("Existing Challenge for user".to_string());
        }

        let user_id = auth.user_id();
        let registration = auth.registration().and_then(|r| serde_json::to_string(&r).ok());
        sqlx::query!(
            r#"
                INSERT INTO auth (user_id, registration, authentication, passkeys)
                VALUES (?, ?, ?, ?)
                returning user_id as `user_id:uuid::Uuid`
            "#,
            user_id,
            registration,
            None::<String>,
            "[]",
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|err| err.to_string())?;

        Ok(auth)
    }

    async fn get_registration(&self, user_id: &Uuid) -> Result<PasskeyRegistration, String> {
        let user_id_clone = user_id.clone();

        let maybe_registration = sqlx::query!(
            r#"
                SELECT registration FROM auth
                WHERE user_id = ?
            "#,
            user_id_clone,
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .and_then(|r| r.registration)
            .and_then(|r| serde_json::from_str::<PasskeyRegistration>(&r).ok());

        let Some(registration) = maybe_registration else {
            return Err(format!("No registration found for {:?}", user_id.to_string()));
        };

        Ok(registration)
    }

    async fn get_passkeys(&self, user_id: &Uuid) -> Result<Vec<Passkey>, String> {
        sqlx::query!(
            r#"
                SELECT passkeys FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .map(|r| r.passkeys)
            .and_then(|pks| serde_json::from_str::<Vec<Passkey>>(&pks).ok())
            .ok_or("No passkeys found for user".to_string())
    }

    async fn update_passkey(&self, user_id: &Uuid, pk: Passkey) -> Result<(), String> {
        let mut passkeys = sqlx::query!(
            r#"
                SELECT passkeys FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .map(|record| record.passkeys)
            .ok_or("No auth found for user".to_string())
            .and_then(|psk_str| serde_json::from_str::<Vec<Passkey>>(&psk_str).map_err(|err| err.to_string()))?;

        passkeys.push(pk);

        let passkeys_json = serde_json::to_string(&passkeys).map_err(|err| err.to_string())?;

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
            .await
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    async fn update_authen(
        &self,
        user_id: &Uuid,
        pka: PasskeyAuthentication,
    ) -> Result<(), String> {

        // make sure user has existing auth
        sqlx::query!(
            r#"
                SELECT authentication FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("No auth found for user".to_string())?;

        let pka_json = serde_json::to_string(&pka).map_err(|err| err.to_string())?;

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
            .map_err(|err| err.to_string())
            .map(|_| ())
    }

    async fn get_authentication(&self, user_id: &Uuid) -> Result<PasskeyAuthentication, String> {
        sqlx::query!(
            r#"
                SELECT authentication FROM auth
                WHERE user_id = ?
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .ok_or("No auth found for user".to_string())
            .and_then(|r| r.authentication.ok_or("No auth found for user".to_string()))
            .and_then(|a| serde_json::from_str::<PasskeyAuthentication>(&a).map_err(|err| err.to_string()))
    }

    async fn update_credentials(
        &self,
        user_id: &Uuid,
        credentials: AuthenticationResult,
    ) -> Result<(), String> {
        let mut passkeys = sqlx::query!(
            r#"
                SELECT passkeys FROM auth
                WHERE user_id = ?
            "#,
            user_id,
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?
            .map(|record| record.passkeys)
            .ok_or("No auth found for user".to_string())
            .and_then(|psk_str| serde_json::from_str::<Vec<Passkey>>(&psk_str).map_err(|err| err.to_string()))?;

        // This will update the credential if it's the matching
        // one. Otherwise it's ignored. That is why it is safe to
        // iterate this over the full list.
        passkeys.iter_mut().for_each(|pk| {pk.update_credential(&credentials);});

        let passkeys_json = serde_json::to_string(&passkeys).map_err(|err| err.to_string())?;

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
            .await
            .map_err(|err| err.to_string())?;

        Ok(())

    }
}

pub fn create_auth_repo(pool: &SqlitePool) -> Arc<AuthSqlRepo> {
    Arc::new(AuthSqlRepo { pool: pool.clone() })
}
