//! Sign-in tokens (JWT), the "who is calling" extractors, and a login rate limiter.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::{header, request::Parts};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::users::User;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // user id (as text, like the old backend)
    role: String,
    ver: i64,    // token_version: changes when the password/role/status changes
    exp: i64,
}

/// Returns the token and its lifetime in seconds.
pub fn create_token(user: &User, secret: &str, minutes: i64) -> Result<(String, i64), ApiError> {
    let exp = (chrono::Utc::now() + chrono::Duration::minutes(minutes)).timestamp();
    let claims = Claims { sub: user.id.to_string(), role: user.role.clone(), ver: user.token_version, exp };
    let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| {
            tracing::error!("token: {e}");
            ApiError::internal()
        })?;
    Ok((token, minutes * 60))
}

/// The signed-in user. Rejects missing/invalid/expired tokens, and users who were disabled,
/// deleted, or whose password or role changed since the token was issued.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")))
            .ok_or_else(ApiError::unauthorized)?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = ["exp", "sub"].iter().map(|s| s.to_string()).collect();
        let data = decode::<Claims>(token, &DecodingKey::from_secret(state.settings.jwt_secret.as_bytes()), &validation)
            .map_err(|_| ApiError::unauthorized())?;
        let id: i64 = data.claims.sub.parse().map_err(|_| ApiError::unauthorized())?;
        let user = state.users.get(id)?.ok_or_else(ApiError::unauthorized)?;
        if !user.active || user.token_version != data.claims.ver {
            return Err(ApiError::unauthorized());
        }
        Ok(CurrentUser(user))
    }
}

/// A signed-in administrator.
pub struct AdminUser(pub User);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
        if !user.is_admin() {
            return Err(ApiError::forbidden("Administrator role required"));
        }
        Ok(AdminUser(user))
    }
}

/// The caller's IP address ("?" when unknown, e.g. in tests).
pub struct ClientIp(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientIp(
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|c| c.0.ip().to_string())
                .unwrap_or_else(|| "?".into()),
        ))
    }
}

/// Blocks an IP + username after too many failed logins within a time window.
pub struct LoginRateLimiter {
    max_attempts: usize,
    window: Duration,
    failures: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl LoginRateLimiter {
    pub fn new(max_attempts: usize, window_seconds: u64) -> Self {
        Self { max_attempts, window: Duration::from_secs(window_seconds), failures: Mutex::new(HashMap::new()) }
    }

    fn prune(q: &mut VecDeque<Instant>, window: Duration) {
        while q.front().is_some_and(|t| t.elapsed() > window) {
            q.pop_front();
        }
    }

    /// Seconds to wait, or 0 if the login may be attempted.
    pub fn retry_after(&self, key: &str) -> u64 {
        let mut map = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        let q = map.entry(key.to_string()).or_default();
        Self::prune(q, self.window);
        if q.len() >= self.max_attempts {
            let waited = q.front().map(|t| t.elapsed()).unwrap_or_default();
            return self.window.saturating_sub(waited).as_secs().max(1);
        }
        0
    }

    pub fn failed(&self, key: &str) {
        let mut map = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        let q = map.entry(key.to_string()).or_default();
        Self::prune(q, self.window);
        q.push_back(Instant::now());
    }

    pub fn reset(&self, key: &str) {
        self.failures.lock().unwrap_or_else(|e| e.into_inner()).remove(key);
    }
}
