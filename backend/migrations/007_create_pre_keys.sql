CREATE TABLE signed_pre_keys (
    id          INTEGER NOT NULL,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key  BYTEA NOT NULL CHECK (octet_length(public_key) = 32),
    signature   BYTEA NOT NULL CHECK (octet_length(signature) = 64),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, id)
);

CREATE TABLE one_time_pre_keys (
    id          INTEGER NOT NULL,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key  BYTEA NOT NULL CHECK (octet_length(public_key) = 32),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, id)
);

CREATE INDEX idx_one_time_pre_keys_user ON one_time_pre_keys(user_id);
