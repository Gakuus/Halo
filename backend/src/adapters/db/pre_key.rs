use sqlx::PgPool;

use crate::domain::{
    error::DomainError,
    pre_key::{OneTimePreKey, PreKeyBundle, SignedPreKey},
    value_objects::{KeyId, Signature, Timestamp, UserId, X25519PublicKey},
};
use crate::ports::repositories::PreKeyRepository;

#[derive(Debug, Clone)]
pub struct PostgresPreKeyRepository {
    pool: PgPool,
}

impl PostgresPreKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PreKeyRepository for PostgresPreKeyRepository {
    async fn find_bundle(&self, user_id: &UserId) -> Result<PreKeyBundle, DomainError> {
        let identity_key: Vec<u8> = sqlx::query_scalar(
            "SELECT identity_public_key FROM users WHERE id = $1",
        )
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("db error: {e}")))?
        .ok_or(DomainError::UserNotFound)?;

        let signed_key = sqlx::query_as::<_, (i32, Vec<u8>, Vec<u8>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, public_key, signature, created_at FROM signed_pre_keys WHERE user_id = $1 ORDER BY id DESC LIMIT 1",
        )
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("db error: {e}")))?
        .map(|(id, key, sig, created)| {
            let mut pk = [0u8; 32];
            pk.copy_from_slice(&key);
            let mut sg = [0u8; 64];
            sg.copy_from_slice(&sig);
            SignedPreKey::from_db(
                KeyId::new(id as u32),
                *user_id,
                X25519PublicKey::new(pk),
                Signature::new(sg),
                Timestamp::from_millis(created.timestamp_millis()),
            )
        })
        .ok_or_else(|| DomainError::Internal("no signed pre-key found".into()))?;

        let rows = sqlx::query_as::<_, (i32, Vec<u8>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, public_key, created_at FROM one_time_pre_keys WHERE user_id = $1 ORDER BY id LIMIT 100",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("db error: {e}")))?;

        let one_time = rows
            .into_iter()
            .map(|(id, key, created)| {
                let mut pk = [0u8; 32];
                pk.copy_from_slice(&key);
                OneTimePreKey::from_db(
                    KeyId::new(id as u32),
                    *user_id,
                    X25519PublicKey::new(pk),
                    Timestamp::from_millis(created.timestamp_millis()),
                )
            })
            .collect();

        Ok(PreKeyBundle {
            identity_key,
            signed_pre_key: signed_key,
            one_time_pre_keys: one_time,
        })
    }

    async fn save_signed_pre_key(&self, pre_key: &SignedPreKey) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO signed_pre_keys (id, user_id, public_key, signature) VALUES ($1, $2, $3, $4)",
        )
        .bind(pre_key.id().as_u32() as i32)
        .bind(pre_key.user_id().as_uuid())
        .bind(pre_key.public_key().as_bytes())
        .bind(pre_key.signature().as_bytes())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("db error: {e}")))?;
        Ok(())
    }

    async fn save_one_time_pre_keys(&self, keys: &[OneTimePreKey]) -> Result<(), DomainError> {
        if keys.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(format!("tx error: {e}")))?;

        for key in keys {
            sqlx::query(
                "INSERT INTO one_time_pre_keys (id, user_id, public_key) VALUES ($1, $2, $3)",
            )
            .bind(key.id().as_u32() as i32)
            .bind(key.user_id().as_uuid())
            .bind(key.public_key().as_bytes())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(format!("db error: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(format!("tx error: {e}")))?;

        Ok(())
    }

    async fn consume_one_time_pre_key(
        &self,
        user_id: &UserId,
        key_id: KeyId,
    ) -> Result<X25519PublicKey, DomainError> {
        let row = sqlx::query_scalar::<_, Vec<u8>>(
            "DELETE FROM one_time_pre_keys WHERE user_id = $1 AND id = $2 RETURNING public_key",
        )
        .bind(user_id.as_uuid())
        .bind(key_id.as_u32() as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("db error: {e}")))?
        .ok_or(DomainError::NotFound("one-time pre-key not found".into()))?;

        let mut pk = [0u8; 32];
        pk.copy_from_slice(&row);
        Ok(X25519PublicKey::new(pk))
    }

    async fn count_one_time_pre_keys(&self, user_id: &UserId) -> Result<u32, DomainError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM one_time_pre_keys WHERE user_id = $1",
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("db error: {e}")))?;
        Ok(count as u32)
    }
}
