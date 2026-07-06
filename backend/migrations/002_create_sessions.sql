CREATE TYPE session_status AS ENUM ('active', 'expired', 'revoked', 'reconnecting');

CREATE TABLE IF NOT EXISTS sessions (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    jwt_id          UUID NOT NULL UNIQUE,
    status          session_status NOT NULL DEFAULT 'active',
    connected_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address      INET NOT NULL,
    user_agent      TEXT NOT NULL DEFAULT ''
);

CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_jwt_id ON sessions (jwt_id);
CREATE INDEX idx_sessions_status ON sessions (status);
