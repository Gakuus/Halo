use chrono::{DateTime, Utc};

use super::{
    error::DomainError,
    value_objects::{Email, PasswordHash, UserId, Username},
};

const MAX_FAILED_ATTEMPTS: u32 = 20;
const LOCKOUT_DURATION_HOURS: i64 = 24;

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    username: Username,
    email: Email,
    password_hash: PasswordHash,
    identity_public_key: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    failed_login_attempts: u32,
    locked_until: Option<DateTime<Utc>>,
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
            failed_login_attempts: 0,
            locked_until: None,
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

    pub fn failed_login_attempts(&self) -> u32 {
        self.failed_login_attempts
    }

    pub fn locked_until(&self) -> Option<DateTime<Utc>> {
        self.locked_until
    }

    pub fn is_locked(&self) -> bool {
        if let Some(until) = self.locked_until {
            Utc::now() < until
        } else {
            false
        }
    }

    pub fn record_failed_login(&mut self) {
        self.failed_login_attempts += 1;
        if self.failed_login_attempts >= MAX_FAILED_ATTEMPTS {
            self.locked_until = Some(Utc::now() + chrono::Duration::hours(LOCKOUT_DURATION_HOURS));
        }
        self.updated_at = Utc::now();
    }

    pub fn reset_failed_logins(&mut self) {
        self.failed_login_attempts = 0;
        self.locked_until = None;
        self.updated_at = Utc::now();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_db(
        id: UserId,
        username: Username,
        email: Email,
        password_hash: PasswordHash,
        identity_public_key: Vec<u8>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        failed_login_attempts: u32,
        locked_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            identity_public_key,
            created_at,
            updated_at,
            failed_login_attempts,
            locked_until,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user() -> User {
        User::new(
            Username::new("test_user").unwrap(),
            Email::new("test@example.com").unwrap(),
            PasswordHash::new("$argon2id$v=19$..."),
            vec![0u8; 32],
        )
        .unwrap()
    }

    #[test]
    fn test_new_user_success() {
        let user = make_user();
        assert_eq!(user.failed_login_attempts(), 0);
        assert!(user.locked_until().is_none());
        assert!(!user.is_locked());
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

    #[test]
    fn test_record_failed_login() {
        let mut user = make_user();
        assert_eq!(user.failed_login_attempts(), 0);

        for _ in 0..19 {
            user.record_failed_login();
        }
        assert_eq!(user.failed_login_attempts(), 19);
        assert!(!user.is_locked());

        user.record_failed_login();
        assert_eq!(user.failed_login_attempts(), 20);
        assert!(user.is_locked());
        assert!(user.locked_until().unwrap() > Utc::now());
    }

    #[test]
    fn test_reset_failed_logins() {
        let mut user = make_user();
        for _ in 0..20 {
            user.record_failed_login();
        }
        assert!(user.is_locked());

        user.reset_failed_logins();
        assert_eq!(user.failed_login_attempts(), 0);
        assert!(!user.is_locked());
        assert!(user.locked_until().is_none());
    }

    #[test]
    fn test_is_locked_expired() {
        let user = User::from_db(
            UserId::new(),
            Username::new("test").unwrap(),
            Email::new("test@test.com").unwrap(),
            PasswordHash::new("hash"),
            vec![0u8; 32],
            Utc::now(),
            Utc::now(),
            20,
            Some(Utc::now() - chrono::Duration::hours(1)),
        );
        assert!(!user.is_locked());
    }

    #[test]
    fn test_from_db_restores_lockout_state() {
        let locked_until = Some(Utc::now() + chrono::Duration::hours(1));
        let user = User::from_db(
            UserId::new(),
            Username::new("test").unwrap(),
            Email::new("test@test.com").unwrap(),
            PasswordHash::new("hash"),
            vec![0u8; 32],
            Utc::now(),
            Utc::now(),
            5,
            locked_until,
        );
        assert_eq!(user.failed_login_attempts(), 5);
        assert!(user.is_locked());
    }
}
