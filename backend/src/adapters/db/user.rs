use sqlx::PgPool;

use crate::domain::{
    error::DomainError,
    user::User,
    value_objects::{Email, PasswordHash, UserId, Username},
};
use crate::ports::repositories::UserRepository;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_user(row: sqlx::postgres::PgRow) -> sqlx::Result<User> {
    use sqlx::Row;
    let id: uuid::Uuid = row.get("id");
    let username: String = row.get("username");
    let email: String = row.get("email");
    let password_hash: String = row.get("password_hash");
    let identity_key: Vec<u8> = row.get("identity_public_key");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
    let failed_login_attempts: i32 = row.get("failed_login_attempts");
    let locked_until: Option<chrono::DateTime<chrono::Utc>> = row.get("locked_until");

    Ok(User::from_db(
        UserId::from_uuid(id),
        Username::new(username).expect("invalid username in database"),
        Email::new(email).expect("invalid email in database"),
        PasswordHash::new(password_hash),
        identity_key,
        created_at,
        updated_at,
        failed_login_attempts as u32,
        locked_until,
    ))
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<User, DomainError> {
        sqlx::query("SELECT id, username, email, password_hash, identity_public_key, created_at, updated_at, failed_login_attempts, locked_until FROM users WHERE id = $1")
            .bind(id.as_uuid())
            .try_map(row_to_user)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DomainError::UserNotFound)
    }

    async fn find_by_username(&self, username: &Username) -> Result<User, DomainError> {
        sqlx::query("SELECT id, username, email, password_hash, identity_public_key, created_at, updated_at, failed_login_attempts, locked_until FROM users WHERE username = $1")
            .bind(username.as_str())
            .try_map(row_to_user)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DomainError::UserNotFound)
    }

    async fn find_by_email(&self, email: &Email) -> Result<User, DomainError> {
        sqlx::query("SELECT id, username, email, password_hash, identity_public_key, created_at, updated_at, failed_login_attempts, locked_until FROM users WHERE email = $1")
            .bind(email.as_str())
            .try_map(row_to_user)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DomainError::UserNotFound)
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, identity_public_key, created_at, updated_at, failed_login_attempts, locked_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO UPDATE SET username = $2, email = $3, password_hash = $4, identity_public_key = $5, updated_at = $7, failed_login_attempts = $8, locked_until = $9",
        )
        .bind(user.id().as_uuid())
        .bind(user.username().as_str())
        .bind(user.email().as_str())
        .bind(user.password_hash().as_str())
        .bind(user.identity_public_key())
        .bind(user.created_at())
        .bind(user.updated_at())
        .bind(user.failed_login_attempts() as i32)
        .bind(user.locked_until())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("failed to save user: {e}")))?;
        Ok(())
    }

    async fn search(&self, query: &str) -> Result<Vec<User>, DomainError> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query(
            "SELECT id, username, email, password_hash, identity_public_key, created_at, updated_at, failed_login_attempts, locked_until FROM users WHERE username ILIKE $1 OR email ILIKE $1 LIMIT 20",
        )
        .bind(&pattern)
        .try_map(row_to_user)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("search failed: {e}")))?;
        Ok(rows)
    }

    async fn username_exists(&self, username: &Username) -> Result<bool, DomainError> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(username.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DomainError::Internal(format!("check username failed: {e}")))?;
        Ok(exists)
    }

    async fn email_exists(&self, email: &Email) -> Result<bool, DomainError> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(email.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DomainError::Internal(format!("check email failed: {e}")))?;
        Ok(exists)
    }
}
