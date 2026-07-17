use tracing::instrument;

use crate::domain::session::SessionStatus;
use crate::domain::{error::DomainError, value_objects::SessionId};
use crate::ports::repositories::SessionRepository;

pub struct LogoutUseCase<S>
where
    S: SessionRepository,
{
    session_repo: S,
}

impl<S> LogoutUseCase<S>
where
    S: SessionRepository,
{
    pub fn new(session_repo: S) -> Self {
        Self { session_repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, session_id: &SessionId) -> Result<(), DomainError> {
        self.session_repo
            .update_status(session_id, &SessionStatus::Revoked)
            .await
    }
}
