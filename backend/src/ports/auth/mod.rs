use std::future::Future;

use crate::domain::{
    error::DomainError,
    value_objects::{JwtToken, UserId},
};

pub struct TokenPair {
    pub access_token: JwtToken,
    pub refresh_token: JwtToken,
}

pub struct JwtClaims {
    pub user_id: UserId,
    pub session_jti: uuid::Uuid,
    pub exp: usize,
}

pub trait AuthPort: Send + Sync {
    fn generate_token_pair(
        &self,
        user_id: &UserId,
        session_jti: uuid::Uuid,
    ) -> impl Future<Output = Result<TokenPair, DomainError>> + Send;

    fn validate_access_token(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<JwtClaims, DomainError>> + Send;

    fn validate_refresh_token(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<JwtClaims, DomainError>> + Send;
}
