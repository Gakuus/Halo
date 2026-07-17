use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use tracing::instrument;

use crate::adapters::api::error::{ApiError, ErrorCode};
use crate::adapters::api::extract::ValidatedBody;
use crate::adapters::api::middleware::auth::AuthContext;
use crate::adapters::api::state::AppState;
use crate::domain::error::DomainError;
use crate::domain::group::{Group, GroupMember, GroupRole};
use crate::domain::value_objects::{GroupId, GroupName, UserId};
use crate::ports::repositories::GroupRepository;

#[derive(Debug, Deserialize)]
pub struct CreateGroupBody {
    pub name: String,
    pub member_ids: Vec<String>,
}

fn encode_group(group: &Group) -> serde_json::Value {
    let members: Vec<serde_json::Value> = group
        .members()
        .iter()
        .map(|m| {
            serde_json::json!({
                "user_id": m.user_id().as_uuid().to_string(),
                "role": format!("{:?}", m.role()).to_lowercase(),
            })
        })
        .collect();

    serde_json::json!({
        "group_id": group.id().as_uuid().to_string(),
        "name": group.name().as_str(),
        "owner_id": group.owner_id().as_uuid().to_string(),
        "members": members,
        "member_count": group.member_count(),
        "created_at": group.created_at().as_millis(),
    })
}

#[instrument(skip(state))]
pub async fn create_group(
    State(state): State<AppState>,
    auth: AuthContext,
    ValidatedBody(body): ValidatedBody<CreateGroupBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, ApiError)> {
    let group_name = GroupName::new(&body.name).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Group name must be between 1 and 50 characters",
        )
    })?;

    let owner_id = auth.claims.user_id;

    let mut member_ids = Vec::new();
    for id_str in &body.member_ids {
        let uid = UserId::from_uuid(uuid::Uuid::parse_str(id_str).map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                format!("Invalid user ID: {id_str}"),
            )
        })?);
        member_ids.push(uid);
    }

    let group = Group::new(group_name, owner_id, member_ids)
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    state
        .group_repo
        .save(&group)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok((StatusCode::CREATED, Json(encode_group(&group))))
}

#[instrument(skip(state))]
pub async fn get_group(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, ApiError)> {
    let group_id = GroupId::from_uuid(uuid::Uuid::parse_str(&id).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Invalid group ID",
        )
    })?);

    let group = state
        .group_repo
        .find_by_id(&group_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    if !group.is_member(&auth.claims.user_id) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::Forbidden,
            "Not a member of this group",
        ));
    }

    Ok(Json(encode_group(&group)))
}

#[derive(Debug, Deserialize)]
pub struct AddMembersBody {
    pub user_ids: Vec<String>,
}

#[instrument(skip(state))]
pub async fn add_members(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    ValidatedBody(body): ValidatedBody<AddMembersBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, ApiError)> {
    let group_id = GroupId::from_uuid(uuid::Uuid::parse_str(&id).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Invalid group ID",
        )
    })?);

    let group = state
        .group_repo
        .find_by_id(&group_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    if !group.is_admin_or_owner(&auth.claims.user_id) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::Forbidden,
            "Only group admins or the owner can add members",
        ));
    }

    let mut added = 0u32;
    for user_id_str in &body.user_ids {
        let user_id = UserId::from_uuid(uuid::Uuid::parse_str(user_id_str).map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                format!("Invalid user ID: {user_id_str}"),
            )
        })?);

        let member = GroupMember::new(user_id, GroupRole::Member);
        match state.group_repo.add_member(&group_id, &member).await {
            Ok(()) => added += 1,
            Err(DomainError::MemberAlreadyExists) => {}
            Err(e) => return Err(e.into()),
        }
    }

    Ok(Json(serde_json::json!({
        "group_id": group_id.as_uuid().to_string(),
        "members_added": added,
    })))
}

#[instrument(skip(state))]
pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((group_id_str, user_id_str)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, ApiError)> {
    let group_id = GroupId::from_uuid(uuid::Uuid::parse_str(&group_id_str).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Invalid group ID",
        )
    })?);

    let group = state
        .group_repo
        .find_by_id(&group_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    if !group.is_admin_or_owner(&auth.claims.user_id) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::Forbidden,
            "Only group admins or the owner can remove members",
        ));
    }

    let target_user_id = UserId::from_uuid(uuid::Uuid::parse_str(&user_id_str).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Invalid user ID",
        )
    })?);

    state
        .group_repo
        .remove_member(&group_id, &target_user_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state))]
pub async fn delete_group(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, ApiError)> {
    let group_id = GroupId::from_uuid(uuid::Uuid::parse_str(&id).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Invalid group ID",
        )
    })?);

    let group = state
        .group_repo
        .find_by_id(&group_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    if !group.is_owner(&auth.claims.user_id) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::Forbidden,
            "Only the group owner can delete the group",
        ));
    }

    let members: Vec<UserId> = group.members().iter().map(|m| *m.user_id()).collect();
    for member_id in &members {
        let _ = state.group_repo.remove_member(&group_id, member_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
