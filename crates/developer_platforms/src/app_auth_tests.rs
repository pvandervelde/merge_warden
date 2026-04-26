//! Tests for `AppAuthProvider::resolve_installation_id` using WireMock.
//!
//! Each test spins up a WireMock server, creates an `AppAuthProvider` pointing
//! at it, and verifies the correct behaviour for the happy path and error cases.

use github_bot_sdk::error::AuthError;
use serde_json::json;
use wiremock::{
    matchers::{header_exists, method, path},
    Mock, MockServer, ResponseTemplate,
};

use super::AppAuthProvider;

// ---------------------------------------------------------------------------
// Test RSA private key (2048-bit, PKCS#1, for tests only)
// ---------------------------------------------------------------------------
//
// The key is stored in testdata/ rather than inline to avoid GitGuardian
// false-positive alerts on every scan. The testdata path is allowlisted in
// .gitguardian.yml. This key is not used in production; it is the same test
// key shipped with the github-bot-sdk crate (jwt_tests.rs).
const TEST_PRIVATE_KEY_PEM: &str = include_str!("../testdata/test-rsa-key.pem");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates an `AppAuthProvider` pointing at the given WireMock server URI.
fn make_auth_provider(server_uri: &str) -> AppAuthProvider {
    AppAuthProvider::new(12345, TEST_PRIVATE_KEY_PEM, server_uri)
        .expect("Failed to create AppAuthProvider with test key")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_resolve_installation_id_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/some-owner/some-repo/installation"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": 99_u64 })))
        .mount(&server)
        .await;

    let provider = make_auth_provider(&server.uri());
    let result = provider
        .resolve_installation_id("some-owner", "some-repo")
        .await;

    let id = result.expect("Expected Ok(InstallationId)");
    assert_eq!(id.as_u64(), 99);
}

#[tokio::test]
async fn test_resolve_installation_id_404_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/some-owner/some-repo/installation"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let provider = make_auth_provider(&server.uri());
    let result = provider
        .resolve_installation_id("some-owner", "some-repo")
        .await;

    assert!(
        matches!(result, Err(AuthError::TokenExchangeFailed { .. })),
        "Expected TokenExchangeFailed, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_resolve_installation_id_missing_id_field_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/some-owner/some-repo/installation"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "other_field": 42_u64 })))
        .mount(&server)
        .await;

    let provider = make_auth_provider(&server.uri());
    let result = provider
        .resolve_installation_id("some-owner", "some-repo")
        .await;

    assert!(
        matches!(result, Err(AuthError::TokenExchangeFailed { .. })),
        "Expected TokenExchangeFailed, got {:?}",
        result
    );
}
