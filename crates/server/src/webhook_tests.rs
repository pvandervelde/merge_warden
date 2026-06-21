use axum::{http::StatusCode, response::IntoResponse};
use chrono::Utc;
use github_bot_sdk::{
    client::{ClientConfig, GitHubClient, OwnerType, Repository, RepositoryOwner},
    events::{EventEnvelope, EventPayload},
    webhook::WebhookHandler,
};
use merge_warden_core::config::ApplicationDefaults;
use merge_warden_developer_platforms::app_auth::AppAuthProvider;
use serde_json::json;

use super::health_check;
use super::MergeWardenWebhookHandler;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// RSA private key used only in tests.  Generated offline; never used in
/// production.  Must be a valid PEM-encoded PKCS#8 or traditional RSA key so
/// `AppAuthProvider` can parse it.
const TEST_PEM: &str = include_str!("../../developer_platforms/testdata/test-rsa-key.pem");

fn make_test_handler() -> MergeWardenWebhookHandler {
    let auth = AppAuthProvider::new(12345, TEST_PEM, "https://api.github.com")
        .expect("test RSA key must be valid");
    let github_client = GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .expect("GitHub client must build");
    MergeWardenWebhookHandler::new(github_client, ApplicationDefaults::default())
}

fn make_status_envelope(context: &str) -> EventEnvelope {
    let repo = Repository {
        id: 1,
        name: "test-repo".to_string(),
        full_name: "owner/test-repo".to_string(),
        owner: RepositoryOwner {
            login: "owner".to_string(),
            id: 1,
            avatar_url: "https://example.com/avatar.png".to_string(),
            owner_type: OwnerType::User,
        },
        private: false,
        description: None,
        default_branch: "main".to_string(),
        html_url: "https://github.com/owner/test-repo".to_string(),
        clone_url: "https://github.com/owner/test-repo.git".to_string(),
        ssh_url: "git@github.com:owner/test-repo.git".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    EventEnvelope::new(
        "status".to_string(),
        repo,
        EventPayload::new(json!({
            "context": context,
            "sha": "deadbeef",
            "state": "pending",
            "installation": { "id": 99 }
        })),
    )
}

// ---------------------------------------------------------------------------
// health_check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_check_returns_200_ok() {
    let response = health_check().await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// handle_status_event
// ---------------------------------------------------------------------------

/// A status event whose context is NOT `renovate/stability-days` must be
/// silently ignored — no API calls, no errors.
#[tokio::test]
async fn handle_status_event_ignores_non_renovate_context() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let result = handler.handle_status_event(&envelope).await;

    assert!(
        result.is_ok(),
        "non-renovate status event should be ignored: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// handle_event routing
// ---------------------------------------------------------------------------

/// A `status` event must be dispatched to `handle_status_event`.
/// Using a non-renovate context means no GitHub API calls are made, so the
/// handler returns Ok(()) without a live network connection.
#[tokio::test]
async fn handle_event_routes_status_events() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "status event should be routed and return Ok(()): {:?}",
        result
    );
}
