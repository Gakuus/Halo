use std::sync::Arc;

use sqlx::PgPool;

use crate::adapters::auth::jwt::JwtAuthAdapter;
use crate::adapters::db::{
    conversation::PostgresConversationRepository,
    group::PostgresGroupRepository,
    session::PostgresSessionRepository,
    user::PostgresUserRepository,
};
use crate::adapters::api::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub user_repo: PostgresUserRepository,
    pub session_repo: PostgresSessionRepository,
    pub conversation_repo: PostgresConversationRepository,
    pub group_repo: PostgresGroupRepository,
    pub auth_port: Arc<JwtAuthAdapter>,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db_pool: PgPool,
        user_repo: PostgresUserRepository,
        session_repo: PostgresSessionRepository,
        conversation_repo: PostgresConversationRepository,
        group_repo: PostgresGroupRepository,
        auth_port: JwtAuthAdapter,
        rate_limiter: RateLimiter,
    ) -> Self {
        Self {
            db_pool,
            user_repo,
            session_repo,
            conversation_repo,
            group_repo,
            auth_port: Arc::new(auth_port),
            rate_limiter,
        }
    }
}
