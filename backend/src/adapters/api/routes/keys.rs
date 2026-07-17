use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use crate::adapters::api::error::{ApiError, ErrorCode};
use crate::adapters::api::extract::ValidatedBody;
use crate::adapters::api::middleware::auth::AuthContext;
use crate::adapters::api::state::AppState;
use crate::domain::pre_key::{OneTimePreKey, SignedPreKey};
use crate::domain::value_objects::{KeyId, Signature, X25519PublicKey};
use crate::ports::repositories::PreKeyRepository;

#[derive(Debug, Deserialize)]
pub struct UploadPreKeysBody {
    pub signed_pre_key: SignedPreKeyDto,
    pub one_time_pre_keys: Vec<OneTimePreKeyDto>,
}

#[derive(Debug, Deserialize)]
pub struct SignedPreKeyDto {
    pub key_id: u32,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct OneTimePreKeyDto {
    pub key_id: u32,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct PreKeyBundleResponse {
    pub identity_key: String,
    pub signed_pre_key: SignedPreKeyResponse,
    pub one_time_pre_keys: Vec<OneTimePreKeyResponse>,
}

#[derive(Debug, Serialize)]
pub struct SignedPreKeyResponse {
    pub key_id: u32,
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct OneTimePreKeyResponse {
    pub key_id: u32,
    pub public_key: String,
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[instrument(skip(state))]
pub async fn get_pre_key_bundle(
    State(state): State<AppState>,
    _auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<PreKeyBundleResponse>, (StatusCode, ApiError)> {
    let user_id = crate::domain::value_objects::UserId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                "Invalid user ID",
            )
        })?,
    );

    let bundle = state
        .pre_key_repo
        .find_bundle(&user_id)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    let one_time: Vec<OneTimePreKeyResponse> = bundle
        .one_time_pre_keys
        .iter()
        .map(|k| OneTimePreKeyResponse {
            key_id: k.id().as_u32(),
            public_key: encode_hex(k.public_key().as_bytes()),
        })
        .collect();

    Ok(Json(PreKeyBundleResponse {
        identity_key: encode_hex(&bundle.identity_key),
        signed_pre_key: SignedPreKeyResponse {
            key_id: bundle.signed_pre_key.id().as_u32(),
            public_key: encode_hex(bundle.signed_pre_key.public_key().as_bytes()),
            signature: encode_hex(bundle.signed_pre_key.signature().as_bytes()),
        },
        one_time_pre_keys: one_time,
    }))
}

#[instrument(skip(state))]
pub async fn upload_pre_keys(
    State(state): State<AppState>,
    auth: AuthContext,
    ValidatedBody(body): ValidatedBody<UploadPreKeysBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, ApiError)> {
    let user_id = auth.claims.user_id;

    let spk_pk: [u8; 32] = body.signed_pre_key.public_key.try_into().map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "signed_pre_key.public_key must be 32 bytes",
        )
    })?;

    let spk_sig: [u8; 64] = body.signed_pre_key.signature.try_into().map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationError,
            "signed_pre_key.signature must be 64 bytes",
        )
    })?;

    let signed_pre_key = SignedPreKey::new(
        KeyId::new(body.signed_pre_key.key_id),
        user_id,
        X25519PublicKey::new(spk_pk),
        Signature::new(spk_sig),
    );

    state
        .pre_key_repo
        .save_signed_pre_key(&signed_pre_key)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    let mut one_time_keys = Vec::with_capacity(body.one_time_pre_keys.len());
    for otk in &body.one_time_pre_keys {
        let pk: [u8; 32] = otk.public_key.clone().try_into().map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                "one_time_pre_key.public_key must be 32 bytes",
            )
        })?;

        one_time_keys.push(OneTimePreKey::new(
            KeyId::new(otk.key_id),
            user_id,
            X25519PublicKey::new(pk),
        ));
    }

    state
        .pre_key_repo
        .save_one_time_pre_keys(&one_time_keys)
        .await
        .map_err(|e| -> (StatusCode, ApiError) { e.into() })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "signed_pre_key_id": body.signed_pre_key.key_id,
            "one_time_keys_uploaded": one_time_keys.len(),
        })),
    ))
}
