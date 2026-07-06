CREATE TABLE IF NOT EXISTS conversations (
    id              UUID PRIMARY KEY,
    participant_a   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    participant_b   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_message_at TIMESTAMPTZ,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_conversations_participant_a ON conversations (participant_a);
CREATE INDEX idx_conversations_participant_b ON conversations (participant_b);
CREATE UNIQUE INDEX idx_conversations_pair ON conversations (
    LEAST(participant_a, participant_b),
    GREATEST(participant_a, participant_b)
);
