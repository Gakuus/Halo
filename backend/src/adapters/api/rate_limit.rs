use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::sync::Mutex;
use tracing::instrument;

use crate::adapters::api::state::AppState;

use super::error::{ApiError, ErrorCode};

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_refill: tokio::time::Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_second: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate: refill_per_second,
            last_refill: tokio::time::Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = tokio::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill = now;
    }

    fn try_consume(&mut self, cost: f64) -> bool {
        self.refill();
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    capacity: u32,
    refill_rate: f64,
}

impl RateLimiter {
    pub fn new(capacity: u32, refill_per_second: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            refill_rate: refill_per_second,
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.capacity, self.refill_rate));
        bucket.try_consume(1.0)
    }
}

#[instrument(skip_all)]
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let key = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .or_else(|| req.headers().get("X-Real-IP").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown")
        .to_string();

    if !state.rate_limiter.check(&key).await {
        let (_, err) = ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::RateLimited,
            "Too many requests. Please try again later.",
        );
        return err.into_response();
    }

    next.run(req).await
}
