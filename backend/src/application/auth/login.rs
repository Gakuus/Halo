use tracing::instrument;

use crate::domain::{
    error::DomainError,
    session::SessionStatus,
    value_objects::{JwtId, PasswordHash, Username},
};
use crate::ports::auth::AuthPort;
use crate::ports::repositories::{SessionRepository, UserRepository};

use super::dto::{AuthResponse, LoginCommand, TokenResponse};

pub struct LoginUseCase<U, A, S>
where
    U: UserRepository,
    A: AuthPort,
    S: SessionRepository,
{
    user_repo: U,
    auth_port: A,
    session_repo: S,
}

impl<U, A, S> LoginUseCase<U, A, S>
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
    pub async fn execute(&self, cmd: LoginCommand) -> Result<AuthResponse, DomainError> {
        let user = self.find_user(&cmd.login).await?;
        let user_id = *user.id();
        let username = user.username().to_string();

        if user.is_locked() {
            return Err(DomainError::AccountLocked);
        }

        if !verify_password(&cmd.password, user.password_hash()) {
            let mut user = user;
            user.record_failed_login();
            self.user_repo.save(&user).await?;
            return Err(DomainError::InvalidCredentials);
        }

        {
            let mut user = user;
            user.reset_failed_logins();
            self.user_repo.save(&user).await?;
        }

        if let Some(existing_session) = self.session_repo.find_active_by_user(&user_id).await? {
            self.session_repo
                .update_status(existing_session.id(), &SessionStatus::Revoked)
                .await?;
        }

        let jwt_id = JwtId::new();
        let token_pair = self
            .auth_port
            .generate_token_pair(&user_id, *jwt_id.as_uuid())
            .await?;

        let session = crate::domain::session::Session::new(
            user_id,
            jwt_id,
            crate::domain::value_objects::IpAddress::new("127.0.0.1".parse().unwrap()),
            crate::domain::value_objects::UserAgent::new("halo-client"),
        );
        self.session_repo.save(&session).await?;

        Ok(AuthResponse {
            user_id,
            username,
            token: TokenResponse {
                access_token: token_pair.access_token.to_string(),
                refresh_token: token_pair.refresh_token.to_string(),
                expires_in: 900,
            },
        })
    }

    async fn find_user(&self, login: &str) -> Result<crate::domain::user::User, DomainError> {
        if let Ok(username) = Username::new(login)
            && let Ok(user) = self.user_repo.find_by_username(&username).await
        {
            return Ok(user);
        }

        if let Ok(email) = crate::domain::value_objects::Email::new(login)
            && let Ok(user) = self.user_repo.find_by_email(&email).await
        {
            return Ok(user);
        }

        Err(DomainError::InvalidCredentials)
    }
}

fn verify_password(password: &str, hash: &PasswordHash) -> bool {
    use argon2::{
        Argon2,
        password_hash::{PasswordHash as Ph, PasswordVerifier},
    };

    if let Ok(parsed_hash) = Ph::new(hash.as_str()) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}
