use std::sync::Arc;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::domain::{
    error::DomainError,
    value_objects::{JwtToken, UserId},
};
use crate::ports::auth::{AuthPort, JwtClaims, TokenPair};

#[derive(Debug, Clone)]
pub struct JwtAuthAdapter {
    secret: String,
    access_exp: i64,
    refresh_exp: i64,
    issuer: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    jti: String,
    exp: usize,
    iat: usize,
    iss: String,
    username: Option<String>,
    #[serde(rename = "type")]
    token_type: Option<String>,
}

impl JwtAuthAdapter {
    pub fn new(secret: String, access_exp: i64, refresh_exp: i64, issuer: String) -> Self {
        Self {
            secret,
            access_exp,
            refresh_exp,
            issuer,
        }
    }

    fn encode_token(
        &self,
        user_id: &UserId,
        session_jti: uuid::Uuid,
        exp_secs: i64,
        token_type: Option<&str>,
    ) -> Result<JwtToken, DomainError> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.as_uuid().to_string(),
            jti: session_jti.to_string(),
            exp: now + exp_secs as usize,
            iat: now,
            iss: self.issuer.clone(),
            username: None,
            token_type: token_type.map(String::from),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| DomainError::Internal(format!("JWT encode failed: {e}")))?;

        Ok(JwtToken::new(token))
    }

    fn decode_token(&self, token: &str) -> Result<JwtClaims, DomainError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        validation.validate_exp = true;

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => DomainError::SessionExpired,
            _ => DomainError::InvalidToken,
        })?;

        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&data.claims.sub)
                .map_err(|_| DomainError::InvalidToken)?,
        );
        let session_jti = uuid::Uuid::parse_str(&data.claims.jti)
            .map_err(|_| DomainError::InvalidToken)?;

        Ok(JwtClaims {
            user_id,
            session_jti,
            exp: data.claims.exp,
        })
    }
}

impl AuthPort for JwtAuthAdapter {
    #[instrument(skip(self))]
    async fn generate_token_pair(
        &self,
        user_id: &UserId,
        session_jti: uuid::Uuid,
    ) -> Result<TokenPair, DomainError> {
        let access_token =
            self.encode_token(user_id, session_jti, self.access_exp, Some("access"))?;
        let refresh_token =
            self.encode_token(user_id, session_jti, self.refresh_exp, Some("refresh"))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    #[instrument(skip(self))]
    async fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<JwtClaims, DomainError> {
        self.decode_token(token)
    }

    #[instrument(skip(self))]
    async fn validate_refresh_token(
        &self,
        token: &str,
    ) -> Result<JwtClaims, DomainError> {
        self.decode_token(token)
    }
}

impl AuthPort for Arc<JwtAuthAdapter> {
    async fn generate_token_pair(
        &self,
        user_id: &UserId,
        session_jti: uuid::Uuid,
    ) -> Result<TokenPair, DomainError> {
        self.as_ref().generate_token_pair(user_id, session_jti).await
    }

    async fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<JwtClaims, DomainError> {
        self.as_ref().validate_access_token(token).await
    }

    async fn validate_refresh_token(
        &self,
        token: &str,
    ) -> Result<JwtClaims, DomainError> {
        self.as_ref().validate_refresh_token(token).await
    }
}
