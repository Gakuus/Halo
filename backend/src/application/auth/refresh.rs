use tracing::instrument;

use crate::domain::{error::DomainError, value_objects::JwtId};
use crate::ports::auth::AuthPort;
use crate::ports::repositories::SessionRepository;

use super::dto::{RefreshTokenCommand, TokenResponse};

pub struct RefreshTokenUseCase<A, S>
where
    A: AuthPort,
    S: SessionRepository,
{
    auth_port: A,
    session_repo: S,
}

impl<A, S> RefreshTokenUseCase<A, S>
where
    A: AuthPort,
    S: SessionRepository,
{
    pub fn new(auth_port: A, session_repo: S) -> Self {
        Self {
            auth_port,
            session_repo,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, cmd: RefreshTokenCommand) -> Result<TokenResponse, DomainError> {
        let claims = self
            .auth_port
            .validate_refresh_token(&cmd.refresh_token)
            .await?;

        let session = self
            .session_repo
            .find_by_jwt_id(&claims.session_jti)
            .await?;

        self.session_repo
            .update_status(
                session.id(),
                &crate::domain::session::SessionStatus::Revoked,
            )
            .await?;

        let new_jwt_id = JwtId::new();
        let token_pair = self
            .auth_port
            .generate_token_pair(&claims.user_id, *new_jwt_id.as_uuid())
            .await?;

        let new_session = crate::domain::session::Session::new(
            claims.user_id,
            new_jwt_id,
            crate::domain::value_objects::IpAddress::new("127.0.0.1".parse().unwrap()),
            crate::domain::value_objects::UserAgent::new("halo-client"),
        );
        self.session_repo.save(&new_session).await?;

        Ok(TokenResponse {
            access_token: token_pair.access_token.to_string(),
            refresh_token: token_pair.refresh_token.to_string(),
            expires_in: 900,
        })
    }
}
