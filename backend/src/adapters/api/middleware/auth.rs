use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::instrument;

use crate::ports::auth::{AuthPort, JwtClaims};

use crate::adapters::api::state::AppState;

use super::super::error::{ApiError, ErrorCode};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub claims: JwtClaims,
    pub raw_token: String,
}

impl AuthContext {
    fn from_claims(claims: JwtClaims, raw_token: String) -> Self {
        Self { claims, raw_token }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = (StatusCode, ApiError);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    ErrorCode::Unauthorized,
                    "Missing or invalid authentication",
                )
            })
    }
}

#[instrument(skip_all)]
pub async fn auth_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let token = match extract_bearer_token(&req) {
        Ok(t) => t,
        Err(_) => {
            let (_, err) = ApiError::new(
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Missing or invalid Authorization header",
            );
            return err.into_response();
        }
    };

    match state.auth_port.validate_access_token(&token).await {
        Ok(claims) => {
            let ctx = AuthContext::from_claims(claims, token);
            let mut req = req;
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        Err(e) => {
            let (status, api_err): (StatusCode, ApiError) = e.into();
            (status, api_err).into_response()
        }
    }
}

fn extract_bearer_token(req: &Request) -> Result<String, ()> {
    let header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(())?;

    if let Some(token) = header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(())
    }
}
