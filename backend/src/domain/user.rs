use chrono::{DateTime, Utc};

use super::{
    error::DomainError,
    value_objects::{Email, PasswordHash, UserId, Username},
};

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    username: Username,
    email: Email,
    password_hash: PasswordHash,
    identity_public_key: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        username: Username,
        email: Email,
        password_hash: PasswordHash,
        identity_public_key: Vec<u8>,
    ) -> Result<Self, DomainError> {
        if identity_public_key.len() != 32 {
            return Err(DomainError::Internal(
                "identity public key must be 32 bytes".into(),
            ));
        }

        Ok(Self {
            id: UserId::new(),
            username,
            email,
            password_hash,
            identity_public_key,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn identity_public_key(&self) -> &[u8] {
        &self.identity_public_key
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_success() {
        let username = Username::new("test_user").unwrap();
        let email = Email::new("test@example.com").unwrap();
        let hash = PasswordHash::new("$argon2id$v=19$...");
        let key = vec![0u8; 32];

        let user = User::new(username, email, hash, key);
        assert!(user.is_ok());
    }

    #[test]
    fn test_new_user_invalid_key_length() {
        let username = Username::new("test_user").unwrap();
        let email = Email::new("test@example.com").unwrap();
        let hash = PasswordHash::new("$argon2id$v=19$...");
        let key = vec![0u8; 16];

        let user = User::new(username, email, hash, key);
        assert!(user.is_err());
    }
}
