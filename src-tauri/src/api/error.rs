//! Unified API error type for all HTTP handler endpoints.
//!
//! All handlers should return `Result<impl IntoResponse, ApiError>`.
//! This ensures consistent JSON error responses and avoids mixing
//! `StatusCode` direct returns, `(StatusCode, Json)` tuples, and `anyhow` errors.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde_json::json;
use std::fmt;

/// Unified error type for the API layer.
#[derive(Debug)]
pub enum ApiError {
    /// 400 Bad Request
    BadRequest(String),
    /// 401 Unauthorized
    Unauthorized,
    /// 403 Forbidden
    Forbidden(String),
    /// 404 Not Found
    NotFound(String),
    /// 429 Too Many Requests
    RateLimited,
    /// 500 Internal Server Error
    Internal(String),
    /// 503 Service Unavailable
    Unavailable(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::Unavailable(msg) => write!(f, "Service unavailable: {msg}"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Invalid or missing API key".into()),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".into()),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Self::Unavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
        };

        tracing::warn!(module = "ApiError", error = %self, status = %status);

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self::Unavailable(msg.into())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::BadRequest(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    #[test]
    fn bad_request_returns_400() {
        let err = ApiError::bad_request("missing field");
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn unauthorized_returns_401() {
        let resp = ApiError::Unauthorized.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_returns_403() {
        let resp = ApiError::Forbidden("access denied".into()).into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn not_found_returns_404() {
        let resp = ApiError::NotFound("resource missing".into()).into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn rate_limited_returns_429() {
        let resp = ApiError::RateLimited.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn internal_returns_500() {
        let resp = ApiError::Internal("db error".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn unavailable_returns_503() {
        let resp = ApiError::Unavailable("engine not ready".into()).into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn display_formatting() {
        assert!(format!("{}", ApiError::Unauthorized).contains("Unauthorized"));
        assert!(format!("{}", ApiError::RateLimited).contains("Rate limited"));
    }

    #[test]
    fn from_anyhow_converts_to_internal() {
        let err = anyhow::anyhow!("test error");
        let api_err = ApiError::from(err);
        assert!(matches!(api_err, ApiError::Internal(_)));
    }

    #[test]
    fn from_serde_json_converts_to_bad_request() {
        let err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let api_err = ApiError::from(err);
        assert!(matches!(api_err, ApiError::BadRequest(_)));
    }

    #[test]
    fn builder_methods() {
        let err = ApiError::not_found("user 42");
        assert!(matches!(err, ApiError::NotFound(_)));
        assert!(format!("{}", err).contains("user 42"));
    }

    #[test]
    fn all_responses_contain_error_json() {
        let cases: Vec<ApiError> = vec![
            ApiError::bad_request("test"),
            ApiError::Unauthorized,
            ApiError::not_found("thing"),
            ApiError::RateLimited,
            ApiError::internal("boom"),
        ];
        for err in cases {
            let resp = err.into_response();
            let headers = resp.headers().get("content-type").cloned();
            assert!(headers.is_some(), "Response should have content-type header");
        }
    }
}
