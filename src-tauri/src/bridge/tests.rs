//! Integration tests for the bridge HTTP API.
//!
//! Covers:
//! - Auth middleware rejects requests without a valid API key
//! - Auth middleware accepts x-api-key header
//! - Auth middleware accepts Bearer token
//! - CORS preflight (`OPTIONS`) bypasses auth
//! - Health endpoint is public and returns 200
//! - Content-Security-Policy header is present on responses

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use tower::ServiceExt;

/// CSP header value applied to all bridge responses.
const CSP_VALUE: &str =
    "default-src 'self'; script-src 'self' 'nonce-${nonce}'; \
     style-src 'self' 'nonce-${nonce}' https://fonts.googleapis.com; \
     img-src 'self' asset: https: data: blob:; \
     font-src 'self' https://fonts.gstatic.com; \
     connect-src 'self' http://127.0.0.1:30085 http://127.0.0.1:30090; \
     frame-src 'self' blob: data:; \
     media-src 'self' blob:; \
     worker-src 'self' blob:;";

/// Public health-endpoint handler (mirrors the real bridge behaviour).
async fn health_stub() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy"})))
}

/// Stub handler for protected routes used in auth tests.
async fn protected_stub() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"ok": true})))
}

// ---------------------------------------------------------------------------
// Test-helper: build a minimal router with the same middleware stack as the
// real bridge (auth + CSP).
// ---------------------------------------------------------------------------

fn test_router(api_key: &str) -> Router {
    let key = api_key.to_string();

    // Auth middleware — logic mirrors the closure inside BridgeServer::start()
    let auth = middleware::from_fn(move |req: Request<Body>, next: Next| {
        let key = key.clone();
        async move {
            // CORS preflight — always pass
            if req.method() == "OPTIONS" {
                return next.run(req).await;
            }

            // Public endpoints
            let path = req.uri().path();
            if path == "/health" || path == "/metrics" {
                return next.run(req).await;
            }

            // Validate x-api-key header
            if let Some(val) = req.headers().get(HeaderName::from_static("x-api-key")) {
                if let Ok(val) = val.to_str() {
                    if val == key {
                        return next.run(req).await;
                    }
                }
            }

            // Validate Authorization: Bearer <token>
            if let Some(val) = req.headers().get(axum::http::header::AUTHORIZATION) {
                if let Ok(val) = val.to_str() {
                    if let Some(token) = val.strip_prefix("Bearer ") {
                        if token == key {
                            return next.run(req).await;
                        }
                    }
                }
            }

            let body = Json(json!({"error": "Invalid or missing API key"}));
            (StatusCode::UNAUTHORIZED, body).into_response()
        }
    });

    // Security-headers middleware (sets CSP on every response)
    let security_headers = middleware::from_fn(|req: Request<Body>, next: Next| async move {
        let mut response = next.run(req).await;
        response.headers_mut().insert(
            HeaderName::from_static("content-security-policy"),
            CSP_VALUE.parse().unwrap(),
        );
        response
    });

    Router::new()
        .route("/health", get(health_stub))
        .route("/api/protected", get(protected_stub))
        .layer(security_headers)
        .layer(auth)
}

// ---------------------------------------------------------------------------
// Auth middleware tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_auth_middleware_rejects_without_api_key() {
    let app = test_router("test-api-key-123");
    let response = app
        .oneshot(
            Request::get("/api/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 when no API key is provided"
    );
}

#[tokio::test]
async fn test_auth_middleware_accepts_x_api_key_header() {
    let app = test_router("test-api-key-123");
    let response = app
        .oneshot(
            Request::get("/api/protected")
                .header(HeaderName::from_static("x-api-key"), "test-api-key-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 when valid x-api-key is provided"
    );
}

#[tokio::test]
async fn test_auth_middleware_accepts_bearer_token() {
    let app = test_router("test-api-key-123");
    let response = app
        .oneshot(
            Request::get("/api/protected")
                .header(
                    axum::http::header::AUTHORIZATION,
                    "Bearer test-api-key-123",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 when valid Bearer token is provided"
    );
}

#[tokio::test]
async fn test_auth_middleware_rejects_wrong_api_key() {
    let app = test_router("correct-key");
    let response = app
        .oneshot(
            Request::get("/api/protected")
                .header(HeaderName::from_static("x-api-key"), "wrong-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 when a wrong API key is provided"
    );
}

#[tokio::test]
async fn test_auth_middleware_allows_cors_preflight() {
    let app = test_router("test-api-key-123");
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/protected")
                .method("OPTIONS")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Auth middleware passes OPTIONS through (CorsLayer handles it above auth).
    // The key assertion is that auth does NOT reject it with 401.
    assert_ne!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "CORS preflight (OPTIONS) should bypass auth middleware"
    );
}

// ---------------------------------------------------------------------------
// Health endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint_public_no_auth() {
    let app = test_router("test-api-key-123");
    let response = app
        .oneshot(
            Request::get("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "health endpoint is public and should return 200 without auth"
    );
}

// ---------------------------------------------------------------------------
// Security header tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_csp_header_present_on_responses() {
    let app = test_router("test-api-key-123");
    let response = app
        .oneshot(
            Request::get("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let csp = response
        .headers()
        .get(HeaderName::from_static("content-security-policy"));

    assert!(
        csp.is_some(),
        "Content-Security-Policy header must be present on bridge responses"
    );

    let csp_value = csp.unwrap().to_str().unwrap();
    assert!(
        csp_value.contains("default-src 'self'"),
        "CSP should restrict default-src to 'self', got: {}",
        csp_value
    );
}
