use sqlx::PgPool;

use crate::domain::{
    error::DomainError,
    session::{Session, SessionStatus},
    value_objects::{IpAddress, JwtId, SessionId, Timestamp, UserAgent, UserId},
};
use crate::ports::repositories::SessionRepository;

#[derive(Debug, Clone)]
pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_session(row: sqlx::postgres::PgRow) -> sqlx::Result<Session> {
    use sqlx::Row;
    let id: uuid::Uuid = row.get("id");
    let user_id: uuid::Uuid = row.get("user_id");
    let jwt_id: uuid::Uuid = row.get("jwt_id");
    let status_str: String = row.get("status");
    let connected_at: chrono::DateTime<chrono::Utc> = row.get("connected_at");
    let last_heartbeat: chrono::DateTime<chrono::Utc> = row.get("last_heartbeat");
    let ip_address_str: String = row.get("ip_address");
    let user_agent: String = row.get("user_agent");

    let ip_address: std::net::IpAddr = ip_address_str
        .parse()
        .unwrap_or_else(|_| "0.0.0.0".parse().unwrap());

    let status = match status_str.as_str() {
        "active" => SessionStatus::Active,
        "expired" => SessionStatus::Expired,
        "revoked" => SessionStatus::Revoked,
        "reconnecting" => SessionStatus::Reconnecting,
        _ => SessionStatus::Active,
    };

    let session = Session::from_db(
        SessionId::from_uuid(id),
        UserId::from_uuid(user_id),
        JwtId::from_uuid(jwt_id),
        status,
        Timestamp::from_millis(connected_at.timestamp_millis()),
        Timestamp::from_millis(last_heartbeat.timestamp_millis()),
        IpAddress::new(ip_address),
        UserAgent::new(user_agent),
    );

    Ok(session)
}

impl SessionRepository for PostgresSessionRepository {
    async fn find_by_id(&self, id: &SessionId) -> Result<Session, DomainError> {
        sqlx::query(
            "SELECT id, user_id, jwt_id, status, connected_at, last_heartbeat, ip_address, user_agent FROM sessions WHERE id = $1",
        )
        .bind(id.as_uuid())
        .try_map(row_to_session)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::SessionNotFound)
    }

    async fn find_by_jwt_id(&self, jwt_id: &uuid::Uuid) -> Result<Session, DomainError> {
        sqlx::query(
            "SELECT id, user_id, jwt_id, status, connected_at, last_heartbeat, ip_address, user_agent FROM sessions WHERE jwt_id = $1",
        )
        .bind(jwt_id)
        .try_map(row_to_session)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::SessionNotFound)
    }

    async fn find_active_by_user(&self, user_id: &UserId) -> Result<Option<Session>, DomainError> {
        let result = sqlx::query(
            "SELECT id, user_id, jwt_id, status, connected_at, last_heartbeat, ip_address, user_agent FROM sessions WHERE user_id = $1 AND status = 'active' ORDER BY connected_at DESC LIMIT 1",
        )
        .bind(user_id.as_uuid())
        .try_map(row_to_session)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("find active session failed: {e}")))?;

        Ok(result)
    }

    async fn save(&self, session: &Session) -> Result<(), DomainError> {
        let status_str = match session.status() {
            SessionStatus::Active => "active",
            SessionStatus::Expired => "expired",
            SessionStatus::Revoked => "revoked",
            SessionStatus::Reconnecting => "reconnecting",
        };

        sqlx::query(
            "INSERT INTO sessions (id, user_id, jwt_id, status, connected_at, last_heartbeat, ip_address, user_agent) VALUES ($1, $2, $3, $4::session_status, $5, $6, $7, $8)",
        )
        .bind(session.id().as_uuid())
        .bind(session.user_id().as_uuid())
        .bind(session.jwt_id().as_uuid())
        .bind(status_str)
        .bind(chrono::DateTime::from_timestamp_millis(session.connected_at().as_millis()).unwrap_or_default())
        .bind(chrono::DateTime::from_timestamp_millis(session.last_heartbeat().as_millis()).unwrap_or_default())
        .bind(session.ip_address().as_ip_addr().to_string())
        .bind(session.user_agent().as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("failed to save session: {e}")))?;

        Ok(())
    }

    async fn update_status(
        &self,
        id: &SessionId,
        status: &SessionStatus,
    ) -> Result<(), DomainError> {
        let status_str = match status {
            SessionStatus::Active => "active",
            SessionStatus::Expired => "expired",
            SessionStatus::Revoked => "revoked",
            SessionStatus::Reconnecting => "reconnecting",
        };

        sqlx::query("UPDATE sessions SET status = $1::session_status WHERE id = $2")
            .bind(status_str)
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("failed to update session status: {e}")))?;

        Ok(())
    }
}
