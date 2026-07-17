use sqlx::PgPool;

use crate::domain::{
    conversation::Conversation,
    error::DomainError,
    value_objects::{ConversationId, Timestamp, UserId},
};
use crate::ports::repositories::ConversationRepository;

#[derive(Debug, Clone)]
pub struct PostgresConversationRepository {
    pool: PgPool,
}

impl PostgresConversationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_conversation(row: sqlx::postgres::PgRow) -> sqlx::Result<Conversation> {
    use sqlx::Row;
    let id: uuid::Uuid = row.get("id");
    let a: uuid::Uuid = row.get("participant_a");
    let b: uuid::Uuid = row.get("participant_b");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let last_message_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_message_at");
    let is_active: bool = row.get("is_active");

    Ok(Conversation::from_db(
        ConversationId::from_uuid(id),
        UserId::from_uuid(a),
        UserId::from_uuid(b),
        Timestamp::from_millis(created_at.timestamp_millis()),
        last_message_at.map(|t| Timestamp::from_millis(t.timestamp_millis())),
        is_active,
    ))
}

impl ConversationRepository for PostgresConversationRepository {
    async fn find_by_id(&self, id: &ConversationId) -> Result<Conversation, DomainError> {
        sqlx::query(
            "SELECT id, participant_a, participant_b, created_at, last_message_at, is_active FROM conversations WHERE id = $1",
        )
        .bind(id.as_uuid())
        .try_map(row_to_conversation)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::ConversationNotFound)
    }

    async fn find_by_participants(
        &self,
        a: &UserId,
        b: &UserId,
    ) -> Result<Option<Conversation>, DomainError> {
        let (p1, p2) = if a.as_uuid() < b.as_uuid() {
            (a, b)
        } else {
            (b, a)
        };

        let result = sqlx::query(
            "SELECT id, participant_a, participant_b, created_at, last_message_at, is_active FROM conversations WHERE participant_a = $1 AND participant_b = $2",
        )
        .bind(p1.as_uuid())
        .bind(p2.as_uuid())
        .try_map(row_to_conversation)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("find conversation failed: {e}")))?;

        Ok(result)
    }

    async fn save(&self, conversation: &Conversation) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO conversations (id, participant_a, participant_b, created_at, is_active) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(conversation.id().as_uuid())
        .bind(conversation.participant_a().as_uuid())
        .bind(conversation.participant_b().as_uuid())
        .bind(
            chrono::DateTime::from_timestamp_millis(conversation.created_at().as_millis())
                .unwrap_or_default(),
        )
        .bind(conversation.is_active())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("failed to save conversation: {e}")))?;

        Ok(())
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<Conversation>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, participant_a, participant_b, created_at, last_message_at, is_active FROM conversations WHERE participant_a = $1 OR participant_b = $1 ORDER BY last_message_at DESC NULLS LAST",
        )
        .bind(user_id.as_uuid())
        .try_map(row_to_conversation)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("find conversations failed: {e}")))?;

        Ok(rows)
    }
}
