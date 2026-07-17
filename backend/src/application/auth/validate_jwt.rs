use tracing::instrument;

use crate::domain::{error::DomainError, value_objects::UserId};
use crate::ports::auth::AuthPort;
use crate::ports::repositories::SessionRepository;

pub struct ValidateJwtUseCase<A, S>
where
    A: AuthPort,
    S: SessionRepository,
{
    auth_port: A,
    session_repo: S,
}

impl<A, S> ValidateJwtUseCase<A, S>
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
    pub async fn execute(
        &self,
        token: &str,
    ) -> Result<(UserId, crate::domain::value_objects::SessionId), DomainError> {
        let claims = self.auth_port.validate_access_token(token).await?;

        let session = self
            .session_repo
            .find_by_jwt_id(&claims.session_jti)
            .await?;

        if !session.is_active() {
            return Err(DomainError::SessionExpired);
        }

        Ok((claims.user_id, *session.id()))
    }
}
