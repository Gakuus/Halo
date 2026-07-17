use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::adapters::api::error::{ApiError, ErrorCode};
use crate::adapters::api::middleware::auth::AuthContext;
use crate::adapters::api::state::AppState;
use crate::domain::value_objects::UserId;
use crate::ports::repositories::UserRepository;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub users: Vec<UserSummary>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub user_id: String,
    pub username: String,
    pub identity_public_key: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PublicKeyResponse {
    pub user_id: String,
    pub identity_public_key: String,
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[instrument(skip(state))]
pub async fn search_users(
    State(state): State<AppState>,
    _auth: AuthContext,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, ApiError)> {
    if query.q.trim().is_empty() || query.q.len() < 2 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "Search query must be at least 2 characters",
        ));
    }

    let users = state
        .user_repo
        .search(&query.q)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    let limit = query.limit.unwrap_or(20).min(100) as usize;

    let filtered: Vec<_> = users
        .into_iter()
        .filter(|u| {
            query.cursor.as_deref().is_none_or(|c| {
                u.id().as_uuid().to_string().as_str() > c
            })
        })
        .collect();

    let has_more = filtered.len() > limit;
    let displayed = if has_more {
        &filtered[..limit]
    } else {
        &filtered[..]
    };

    let next_cursor = displayed.last().map(|u| u.id().as_uuid().to_string());

    let users: Vec<UserSummary> = displayed
        .iter()
        .map(|u| UserSummary {
            user_id: u.id().as_uuid().to_string(),
            username: u.username().to_string(),
        })
        .collect();

    Ok(Json(SearchResponse {
        users,
        next_cursor,
        has_more,
    }))
}

#[instrument(skip(state))]
pub async fn get_user(
    State(state): State<AppState>,
    _auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<UserProfile>, (StatusCode, ApiError)> {
    let user_id = UserId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|_| {
            ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, "Invalid user ID")
        })?,
    );

    let user = state.user_repo.find_by_id(&user_id).await.map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok(Json(UserProfile {
        user_id: user.id().as_uuid().to_string(),
        username: user.username().to_string(),
        identity_public_key: encode_hex(user.identity_public_key()),
        created_at: user.created_at().to_rfc3339(),
    }))
}

#[instrument(skip(state))]
pub async fn get_public_key(
    State(state): State<AppState>,
    _auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<PublicKeyResponse>, (StatusCode, ApiError)> {
    let user_id = UserId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|_| {
            ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, "Invalid user ID")
        })?,
    );

    let user = state.user_repo.find_by_id(&user_id).await.map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok(Json(PublicKeyResponse {
        user_id: user.id().as_uuid().to_string(),
        identity_public_key: encode_hex(user.identity_public_key()),
    }))
}
