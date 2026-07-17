use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::domain::error::DomainError;

#[derive(Debug, Serialize)]
pub enum ErrorCode {
    ValidationError,
    InvalidCredentials,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    AccountLocked,
    RateLimited,
    InternalError,
    SessionExpired,
    TokenInvalid,
}

impl ErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationError => "VALIDATION_ERROR",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::AccountLocked => "ACCOUNT_LOCKED",
            Self::RateLimited => "RATE_LIMIT_EXCEEDED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::TokenInvalid => "TOKEN_INVALID",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ApiErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> (StatusCode, Self) {
        (
            status,
            Self {
                error: ApiErrorBody {
                    code: code.as_str().to_string(),
                    message: message.into(),
                },
            },
        )
    }

}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = status_code_for_error_code(&self.error.code);
        (status, Json(self)).into_response()
    }
}

fn status_code_for_error_code(code: &str) -> StatusCode {
    match code {
        "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
        "INVALID_CREDENTIALS" => StatusCode::UNAUTHORIZED,
        "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
        "FORBIDDEN" => StatusCode::FORBIDDEN,
        "NOT_FOUND" => StatusCode::NOT_FOUND,
        "CONFLICT" => StatusCode::CONFLICT,
        "ACCOUNT_LOCKED" => StatusCode::LOCKED,
        "RATE_LIMIT_EXCEEDED" => StatusCode::TOO_MANY_REQUESTS,
        "SESSION_EXPIRED" => StatusCode::UNAUTHORIZED,
        "TOKEN_INVALID" => StatusCode::UNAUTHORIZED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<DomainError> for (StatusCode, ApiError) {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::InvalidCredentials => {
                ApiError::new(StatusCode::UNAUTHORIZED, ErrorCode::InvalidCredentials, "Invalid credentials")
            }
            DomainError::UserNotFound | DomainError::ConversationNotFound | DomainError::GroupNotFound | DomainError::NotFound(_) => {
                ApiError::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, err.to_string())
            }
            DomainError::SessionNotFound | DomainError::SessionExpired => {
                ApiError::new(StatusCode::UNAUTHORIZED, ErrorCode::SessionExpired, err.to_string())
            }
            DomainError::SessionRevoked => {
                ApiError::new(StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized, "Session has been revoked")
            }
            DomainError::InvalidToken => {
                ApiError::new(StatusCode::UNAUTHORIZED, ErrorCode::TokenInvalid, "Invalid token")
            }
            DomainError::InvalidUsername(_) | DomainError::InvalidEmail(_) | DomainError::WeakPassword | DomainError::MessageTooLarge => {
                ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, err.to_string())
            }
            DomainError::DuplicateUsername | DomainError::DuplicateEmail => {
                ApiError::new(StatusCode::CONFLICT, ErrorCode::Conflict, err.to_string())
            }
            DomainError::AccountLocked => {
                ApiError::new(StatusCode::LOCKED, ErrorCode::AccountLocked, "Account is locked due to too many failed login attempts")
            }
            DomainError::NotParticipant | DomainError::NotMember | DomainError::NotOwner | DomainError::InsufficientPermissions => {
                ApiError::new(StatusCode::FORBIDDEN, ErrorCode::Forbidden, err.to_string())
            }
            DomainError::MemberAlreadyExists => {
                ApiError::new(StatusCode::CONFLICT, ErrorCode::Conflict, err.to_string())
            }
            DomainError::GroupFull => {
                ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, err.to_string())
            }
            DomainError::UserOffline => {
                ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, err.to_string())
            }
            DomainError::InvalidSignature => {
                ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, err.to_string())
            }
            DomainError::Internal(_) => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError, "An internal error occurred")
            }
            DomainError::Unauthorized => {
                ApiError::new(StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized, "Unauthorized")
            }
        }
    }
}
