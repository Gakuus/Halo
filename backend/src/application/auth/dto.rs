use serde::{Deserialize, Serialize};

use crate::domain::value_objects::UserId;

#[derive(Debug, Deserialize)]
pub struct RegisterCommand {
    pub username: String,
    pub email: String,
    pub password: String,
    pub identity_public_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct LoginCommand {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenCommand {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: UserId,
    pub username: String,
    pub token: TokenResponse,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}
