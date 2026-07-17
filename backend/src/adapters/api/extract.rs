use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
};

use super::error::{ApiError, ErrorCode};

pub struct ValidatedBody<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedBody<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = (StatusCode, ApiError);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let value = Json::<T>::from_request(req, state).await.map_err(|err| {
            let message = if let axum::extract::rejection::JsonRejection::JsonDataError(e) = &err {
                format!("Invalid request body: {e}")
            } else {
                "Invalid request body".to_string()
            };
            ApiError::new(StatusCode::BAD_REQUEST, ErrorCode::ValidationError, message)
        })?;
        Ok(Self(value.0))
    }
}
