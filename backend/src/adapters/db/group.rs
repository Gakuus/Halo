use sqlx::PgPool;

use crate::domain::{
    error::DomainError,
    group::{Group, GroupMember, GroupRole},
    value_objects::{GroupId, GroupName, Timestamp, UserId},
};
use crate::ports::repositories::GroupRepository;

#[derive(Debug, Clone)]
pub struct PostgresGroupRepository {
    pool: PgPool,
}

impl PostgresGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl GroupRepository for PostgresGroupRepository {
    async fn find_by_id(&self, id: &GroupId) -> Result<Group, DomainError> {
        let row = sqlx::query("SELECT id, name, owner_id, created_at FROM groups WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DomainError::GroupNotFound)?;

        use sqlx::Row;
        let group_id: uuid::Uuid = row.get("id");
        let name: String = row.get("name");
        let owner_id: uuid::Uuid = row.get("owner_id");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

        let group_name = GroupName::new(name).map_err(|e| DomainError::Internal(e.to_string()))?;

        let members = self.load_members(&GroupId::from_uuid(group_id)).await?;

        Ok(Group::from_db(
            GroupId::from_uuid(group_id),
            group_name,
            UserId::from_uuid(owner_id),
            members,
            Timestamp::from_millis(created_at.timestamp_millis()),
        ))
    }

    async fn find_by_member(&self, user_id: &UserId) -> Result<Vec<Group>, DomainError> {
        let rows = sqlx::query(
            "SELECT g.id, g.name, g.owner_id, g.created_at FROM groups g JOIN group_members gm ON g.id = gm.group_id WHERE gm.user_id = $1",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("find groups failed: {e}")))?;

        let mut groups = Vec::new();
        for row in rows {
            use sqlx::Row;
            let id: uuid::Uuid = row.get("id");
            let name: String = row.get("name");
            let owner_id: uuid::Uuid = row.get("owner_id");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let group_name =
                GroupName::new(name).map_err(|e| DomainError::Internal(e.to_string()))?;

            let members = self.load_members(&GroupId::from_uuid(id)).await?;

            groups.push(Group::from_db(
                GroupId::from_uuid(id),
                group_name,
                UserId::from_uuid(owner_id),
                members,
                Timestamp::from_millis(created_at.timestamp_millis()),
            ));
        }
        Ok(groups)
    }

    async fn save(&self, group: &Group) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO groups (id, name, owner_id, created_at) VALUES ($1, $2, $3, $4)")
            .bind(group.id().as_uuid())
            .bind(group.name().as_str())
            .bind(group.owner_id().as_uuid())
            .bind(
                chrono::DateTime::from_timestamp_millis(group.created_at().as_millis())
                    .unwrap_or_default(),
            )
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("failed to save group: {e}")))?;

        for member in group.members() {
            let role_str = match member.role() {
                GroupRole::Owner => "owner",
                GroupRole::Admin => "admin",
                GroupRole::Member => "member",
            };

            sqlx::query(
                "INSERT INTO group_members (group_id, user_id, role) VALUES ($1, $2, $3::group_role)",
            )
            .bind(group.id().as_uuid())
            .bind(member.user_id().as_uuid())
            .bind(role_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("failed to save group member: {e}")))?;
        }

        Ok(())
    }

    async fn add_member(
        &self,
        group_id: &GroupId,
        member: &GroupMember,
    ) -> Result<(), DomainError> {
        let role_str = match member.role() {
            GroupRole::Owner => "owner",
            GroupRole::Admin => "admin",
            GroupRole::Member => "member",
        };

        sqlx::query(
            "INSERT INTO group_members (group_id, user_id, role) VALUES ($1, $2, $3::group_role)",
        )
        .bind(group_id.as_uuid())
        .bind(member.user_id().as_uuid())
        .bind(role_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("failed to add member: {e}")))?;

        Ok(())
    }

    async fn remove_member(&self, group_id: &GroupId, user_id: &UserId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id.as_uuid())
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("failed to remove member: {e}")))?;

        Ok(())
    }

    async fn update(&self, group: &Group) -> Result<(), DomainError> {
        sqlx::query("UPDATE groups SET name = $1 WHERE id = $2")
            .bind(group.name().as_str())
            .bind(group.id().as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("failed to update group: {e}")))?;

        Ok(())
    }
}

impl PostgresGroupRepository {
    async fn load_members(&self, group_id: &GroupId) -> Result<Vec<GroupMember>, DomainError> {
        let rows =
            sqlx::query("SELECT user_id, role, joined_at FROM group_members WHERE group_id = $1")
                .bind(group_id.as_uuid())
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::Internal(format!("failed to load members: {e}")))?;

        let mut members = Vec::with_capacity(rows.len());
        for row in rows {
            use sqlx::Row;
            let user_id: uuid::Uuid = row.get("user_id");
            let role_str: String = row.get("role");
            let joined_at: chrono::DateTime<chrono::Utc> = row.get("joined_at");

            let role = match role_str.as_str() {
                "owner" => GroupRole::Owner,
                "admin" => GroupRole::Admin,
                _ => GroupRole::Member,
            };

            members.push(GroupMember::from_db(
                UserId::from_uuid(user_id),
                role,
                Timestamp::from_millis(joined_at.timestamp_millis()),
            ));
        }

        Ok(members)
    }
}
