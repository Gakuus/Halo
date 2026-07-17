use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tracing::instrument;

use crate::adapters::api::error::{ApiError, ErrorCode};
use crate::adapters::api::extract::ValidatedBody;
use crate::adapters::api::middleware::auth::AuthContext;
use crate::adapters::api::state::AppState;
use crate::application::auth::dto::{
    AuthResponse, LoginCommand, RefreshTokenCommand, RegisterCommand, TokenResponse,
};
use crate::application::auth::login::LoginUseCase;
use crate::application::auth::logout::LogoutUseCase;
use crate::application::auth::refresh::RefreshTokenUseCase;
use crate::application::auth::register::RegisterUseCase;
use crate::domain::value_objects::SessionId;

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    username: String,
    email: String,
    password: String,
    identity_public_key: Vec<u8>,
}

#[instrument(skip(state))]
pub async fn register(
    State(state): State<AppState>,
    ValidatedBody(body): ValidatedBody<RegisterBody>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, ApiError)> {
    let key_bytes: [u8; 32] = body.identity_public_key.try_into().map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "identity_public_key must be 32 bytes",
        )
    })?;

    let cmd = RegisterCommand {
        username: body.username,
        email: body.email,
        password: body.password,
        identity_public_key: key_bytes.to_vec(),
    };

    let use_case = RegisterUseCase::new(
        state.user_repo.clone(),
        state.auth_port.clone(),
        state.session_repo.clone(),
    );

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    login: String,
    password: String,
}

#[instrument(skip(state))]
pub async fn login(
    State(state): State<AppState>,
    ValidatedBody(body): ValidatedBody<LoginBody>,
) -> Result<Json<AuthResponse>, (StatusCode, ApiError)> {
    let cmd = LoginCommand {
        login: body.login,
        password: body.password,
    };

    let use_case = LoginUseCase::new(
        state.user_repo.clone(),
        state.auth_port.clone(),
        state.session_repo.clone(),
    );

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct RefreshBody {
    refresh_token: String,
}

#[instrument(skip(state))]
pub async fn refresh(
    State(state): State<AppState>,
    ValidatedBody(body): ValidatedBody<RefreshBody>,
) -> Result<Json<TokenResponse>, (StatusCode, ApiError)> {
    let cmd = RefreshTokenCommand {
        refresh_token: body.refresh_token,
    };

    let use_case = RefreshTokenUseCase::new(state.auth_port.clone(), state.session_repo.clone());

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn logout(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<serde_json::Value>, (StatusCode, ApiError)> {
    let use_case = LogoutUseCase::new(state.session_repo.clone());

    let session_id = SessionId::from_uuid(auth.claims.session_jti);
    use_case
        .execute(&session_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok(Json(serde_json::json!({ "message": "logged_out" })))
}
