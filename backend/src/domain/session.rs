use super::value_objects::{IpAddress, JwtId, SessionId, Timestamp, UserAgent, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Expired,
    Revoked,
    Reconnecting,
}

#[derive(Debug, Clone)]
pub struct Session {
    id: SessionId,
    user_id: UserId,
    jwt_id: JwtId,
    status: SessionStatus,
    connected_at: Timestamp,
    last_heartbeat: Timestamp,
    ip_address: IpAddress,
    user_agent: UserAgent,
}

const MAX_HEARTBEAT_INTERVAL_MS: i64 = 60_000;
const RECONNECT_TIMEOUT_MS: i64 = 30_000;

impl Session {
    pub fn new(
        user_id: UserId,
        jwt_id: JwtId,
        ip_address: IpAddress,
        user_agent: UserAgent,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: SessionId::new(),
            user_id,
            jwt_id,
            status: SessionStatus::Active,
            connected_at: now,
            last_heartbeat: now,
            ip_address,
            user_agent,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_db(
        id: SessionId,
        user_id: UserId,
        jwt_id: JwtId,
        status: SessionStatus,
        connected_at: Timestamp,
        last_heartbeat: Timestamp,
        ip_address: IpAddress,
        user_agent: UserAgent,
    ) -> Self {
        Self {
            id,
            user_id,
            jwt_id,
            status,
            connected_at,
            last_heartbeat,
            ip_address,
            user_agent,
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn jwt_id(&self) -> &JwtId {
        &self.jwt_id
    }

    pub fn status(&self) -> SessionStatus {
        self.status
    }

    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }

    pub fn is_expired(&self, now: Timestamp) -> bool {
        (now.as_millis() - self.last_heartbeat.as_millis()) > MAX_HEARTBEAT_INTERVAL_MS
    }

    pub fn is_reconnect_expired(&self, now: Timestamp) -> bool {
        (now.as_millis() - self.last_heartbeat.as_millis()) > RECONNECT_TIMEOUT_MS
    }

    pub fn revoke(&mut self) {
        self.status = SessionStatus::Revoked;
    }

    pub fn mark_reconnecting(&mut self) {
        self.status = SessionStatus::Reconnecting;
    }

    pub fn mark_expired(&mut self) {
        self.status = SessionStatus::Expired;
    }

    pub fn reactivate(&mut self) {
        self.status = SessionStatus::Active;
        self.last_heartbeat = Timestamp::now();
    }

    pub fn refresh_heartbeat(&mut self) {
        self.last_heartbeat = Timestamp::now();
    }

    pub fn last_heartbeat(&self) -> &Timestamp {
        &self.last_heartbeat
    }

    pub fn connected_at(&self) -> &Timestamp {
        &self.connected_at
    }

    pub fn ip_address(&self) -> &IpAddress {
        &self.ip_address
    }

    pub fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    fn make_session() -> Session {
        Session::new(
            UserId::new(),
            JwtId::new(),
            IpAddress::new("127.0.0.1".parse::<IpAddr>().unwrap()),
            UserAgent::new("halo-tauri/0.1"),
        )
    }

    #[test]
    fn test_session_starts_active() {
        let session = make_session();
        assert_eq!(session.status(), SessionStatus::Active);
        assert!(session.is_active());
    }

    #[test]
    fn test_revoke_session() {
        let mut session = make_session();
        session.revoke();
        assert_eq!(session.status(), SessionStatus::Revoked);
        assert!(!session.is_active());
    }

    #[test]
    fn test_mark_reconnecting() {
        let mut session = make_session();
        session.mark_reconnecting();
        assert_eq!(session.status(), SessionStatus::Reconnecting);
    }

    #[test]
    fn test_reactivate() {
        let mut session = make_session();
        session.mark_reconnecting();
        session.reactivate();
        assert_eq!(session.status(), SessionStatus::Active);
    }

    #[test]
    fn test_is_expired() {
        let session = make_session();
        let now = Timestamp::now();
        assert!(!session.is_expired(now));
    }
}
