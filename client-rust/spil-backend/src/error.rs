//! Every error becomes `{"detail": "..."}` with a proper HTTP status (same shape the React app expects).

use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub detail: String,
    retry_after: Option<u64>,
}

impl ApiError {
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self { status, detail: detail.into(), retry_after: None }
    }
    pub fn bad_request(d: impl Into<String>) -> Self { Self::new(StatusCode::BAD_REQUEST, d) }
    pub fn unauthorized() -> Self { Self::new(StatusCode::UNAUTHORIZED, "Not signed in or session expired") }
    pub fn forbidden(d: impl Into<String>) -> Self { Self::new(StatusCode::FORBIDDEN, d) }
    pub fn not_found(d: impl Into<String>) -> Self { Self::new(StatusCode::NOT_FOUND, d) }
    pub fn conflict(d: impl Into<String>) -> Self { Self::new(StatusCode::CONFLICT, d) }
    pub fn invalid(d: impl Into<String>) -> Self { Self::new(StatusCode::UNPROCESSABLE_ENTITY, d) }
    pub fn internal() -> Self { Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error") }
    pub fn too_many(seconds: u64) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            detail: format!("Too many failed attempts. Try again in {seconds} seconds."),
            retry_after: Some(seconds),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut res = (self.status, Json(json!({ "detail": self.detail }))).into_response();
        if self.status == StatusCode::UNAUTHORIZED {
            res.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        if let Some(s) = self.retry_after {
            if let Ok(v) = HeaderValue::from_str(&s.to_string()) {
                res.headers_mut().insert(header::RETRY_AFTER, v);
            }
        }
        res
    }
}

impl From<JsonRejection> for ApiError {
    fn from(r: JsonRejection) -> Self {
        Self::invalid(format!("Invalid request body: {}", r.body_text()))
    }
}
impl From<PathRejection> for ApiError {
    fn from(r: PathRejection) -> Self {
        Self::invalid(format!("Invalid address: {}", r.body_text()))
    }
}
impl From<QueryRejection> for ApiError {
    fn from(r: QueryRejection) -> Self {
        Self::invalid(format!("Invalid query: {}", r.body_text()))
    }
}

impl From<crate::users::StoreError> for ApiError {
    fn from(e: crate::users::StoreError) -> Self {
        match e {
            crate::users::StoreError::Invalid(m) => Self::conflict(m),
            other => {
                tracing::error!("user store: {other}");
                Self::internal()
            }
        }
    }
}

/// `Json<T>` whose errors are returned as `{"detail": ...}` with status 422.
#[derive(axum::extract::FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

/// `Path<T>` with the same error shape.
#[derive(axum::extract::FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

/// `Query<T>` with the same error shape.
#[derive(axum::extract::FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct ApiQuery<T>(pub T);
