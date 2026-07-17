use tracing::instrument;

use crate::domain::{
    error::DomainError,
    user::User,
    value_objects::{Email, PasswordHash, Username},
};
use crate::ports::auth::AuthPort;
use crate::ports::repositories::{SessionRepository, UserRepository};

use super::dto::{AuthResponse, RegisterCommand, TokenResponse};

pub struct RegisterUseCase<U, A, S>
where
    U: UserRepository,
    A: AuthPort,
    S: SessionRepository,
{
    user_repo: U,
    auth_port: A,
    session_repo: S,
}

impl<U, A, S> RegisterUseCase<U, A, S>
where
    U: UserRepository,
    A: AuthPort,
    S: SessionRepository,
{
    pub fn new(user_repo: U, auth_port: A, session_repo: S) -> Self {
        Self {
            user_repo,
            auth_port,
            session_repo,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, cmd: RegisterCommand) -> Result<AuthResponse, DomainError> {
        let username = Username::new(&cmd.username)?;
        let email = Email::new(&cmd.email)?;

        if cmd.password.len() < 8 {
            return Err(DomainError::WeakPassword);
        }

        if self.user_repo.username_exists(&username).await? {
            return Err(DomainError::DuplicateUsername);
        }

        if self.user_repo.email_exists(&email).await? {
            return Err(DomainError::DuplicateEmail);
        }

        let password_hash = PasswordHash::new(hash_password(&cmd.password));

        let user = User::new(username, email, password_hash, cmd.identity_public_key)?;

        self.user_repo.save(&user).await?;

        let jwt_id = crate::domain::value_objects::JwtId::new();
        let token_pair = self
            .auth_port
            .generate_token_pair(user.id(), *jwt_id.as_uuid())
            .await?;

        let session = crate::domain::session::Session::new(
            *user.id(),
            jwt_id,
            crate::domain::value_objects::IpAddress::new("127.0.0.1".parse().unwrap()),
            crate::domain::value_objects::UserAgent::new("halo-client"),
        );
        self.session_repo.save(&session).await?;

        Ok(AuthResponse {
            user_id: *user.id(),
            username: user.username().to_string(),
            token: TokenResponse {
                access_token: token_pair.access_token.to_string(),
                refresh_token: token_pair.refresh_token.to_string(),
                expires_in: 900,
            },
        })
    }
}

fn hash_password(password: &str) -> String {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_else(|_| password.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{error::DomainError, user::User, value_objects::UserId},
        ports::{
            auth::{AuthPort, JwtClaims, TokenPair},
            repositories::{SessionRepository, UserRepository},
        },
    };

    use super::*;

    struct MockUserRepo {
        users: Vec<User>,
        username_exists: bool,
        email_exists: bool,
    }

    impl UserRepository for MockUserRepo {
        async fn find_by_id(&self, _id: &UserId) -> Result<User, DomainError> {
            self.users.first().cloned().ok_or(DomainError::UserNotFound)
        }
        async fn find_by_username(&self, _username: &Username) -> Result<User, DomainError> {
            self.users.first().cloned().ok_or(DomainError::UserNotFound)
        }
        async fn find_by_email(&self, _email: &Email) -> Result<User, DomainError> {
            self.users.first().cloned().ok_or(DomainError::UserNotFound)
        }
        async fn save(&self, _user: &User) -> Result<(), DomainError> {
            Ok(())
        }
        async fn search(&self, _query: &str) -> Result<Vec<User>, DomainError> {
            Ok(self.users.clone())
        }
        async fn username_exists(&self, _username: &Username) -> Result<bool, DomainError> {
            Ok(self.username_exists)
        }
        async fn email_exists(&self, _email: &Email) -> Result<bool, DomainError> {
            Ok(self.email_exists)
        }
    }

    struct MockAuth;

    impl AuthPort for MockAuth {
        async fn generate_token_pair(
            &self,
            _user_id: &UserId,
            _session_jti: uuid::Uuid,
        ) -> Result<TokenPair, DomainError> {
            Ok(TokenPair {
                access_token: crate::domain::value_objects::JwtToken::new("access"),
                refresh_token: crate::domain::value_objects::JwtToken::new("refresh"),
            })
        }
        async fn validate_access_token(&self, _token: &str) -> Result<JwtClaims, DomainError> {
            unimplemented!()
        }
        async fn validate_refresh_token(&self, _token: &str) -> Result<JwtClaims, DomainError> {
            unimplemented!()
        }
    }

    struct MockSessionRepo;

    impl SessionRepository for MockSessionRepo {
        async fn find_by_id(
            &self,
            _id: &crate::domain::value_objects::SessionId,
        ) -> Result<crate::domain::session::Session, DomainError> {
            unimplemented!()
        }
        async fn find_by_jwt_id(
            &self,
            _jwt_id: &uuid::Uuid,
        ) -> Result<crate::domain::session::Session, DomainError> {
            unimplemented!()
        }
        async fn find_active_by_user(
            &self,
            _user_id: &UserId,
        ) -> Result<Option<crate::domain::session::Session>, DomainError> {
            Ok(None)
        }
        async fn save(
            &self,
            _session: &crate::domain::session::Session,
        ) -> Result<(), DomainError> {
            Ok(())
        }
        async fn update_status(
            &self,
            _id: &crate::domain::value_objects::SessionId,
            _status: &crate::domain::session::SessionStatus,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_success() {
        let user_repo = MockUserRepo {
            users: vec![],
            username_exists: false,
            email_exists: false,
        };
        let auth = MockAuth;
        let session_repo = MockSessionRepo;
        let use_case = RegisterUseCase::new(user_repo, auth, session_repo);

        let cmd = RegisterCommand {
            username: "newuser".into(),
            email: "new@test.com".into(),
            password: "password123".into(),
            identity_public_key: vec![0u8; 32],
        };

        let result = use_case.execute(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let user_repo = MockUserRepo {
            users: vec![],
            username_exists: true,
            email_exists: false,
        };
        let auth = MockAuth;
        let session_repo = MockSessionRepo;
        let use_case = RegisterUseCase::new(user_repo, auth, session_repo);

        let cmd = RegisterCommand {
            username: "existing".into(),
            email: "new@test.com".into(),
            password: "password123".into(),
            identity_public_key: vec![0u8; 32],
        };

        let result = use_case.execute(cmd).await;
        assert!(matches!(result, Err(DomainError::DuplicateUsername)));
    }

    #[tokio::test]
    async fn test_register_weak_password() {
        let user_repo = MockUserRepo {
            users: vec![],
            username_exists: false,
            email_exists: false,
        };
        let auth = MockAuth;
        let session_repo = MockSessionRepo;
        let use_case = RegisterUseCase::new(user_repo, auth, session_repo);

        let cmd = RegisterCommand {
            username: "newuser".into(),
            email: "new@test.com".into(),
            password: "1234567".into(),
            identity_public_key: vec![0u8; 32],
        };

        let result = use_case.execute(cmd).await;
        assert!(matches!(result, Err(DomainError::WeakPassword)));
    }
}
