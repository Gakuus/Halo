CREATE TABLE IF NOT EXISTS groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(50) NOT NULL,
    owner_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_groups_owner ON groups (owner_id);
