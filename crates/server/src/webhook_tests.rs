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

// ---------------------------------------------------------------------------
// handle_event — unknown event types
// ---------------------------------------------------------------------------

/// Events that are not `pull_request`, `pull_request_review`, or `status` must
/// be silently dropped (no API calls, no error).
#[tokio::test]
async fn handle_event_ignores_push_events() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build"); // re-use helper; event_type irrelevant

    // Manufacture an envelope with event_type = "push"
    let push_envelope = EventEnvelope::new(
        "push".to_string(),
        envelope.repository.clone(),
        envelope.payload.clone(),
    );

    let result = handler.handle_event(&push_envelope).await;
    assert!(result.is_ok(), "push events must be silently ignored: {:?}", result);
}

#[tokio::test]
async fn handle_event_ignores_issue_comment_events() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let ic_envelope = EventEnvelope::new(
        "issue_comment".to_string(),
        envelope.repository.clone(),
        envelope.payload.clone(),
    );

    let result = handler.handle_event(&ic_envelope).await;
    assert!(result.is_ok(), "issue_comment events must be silently ignored: {:?}", result);
}

// ---------------------------------------------------------------------------
// handle_pull_request — action filtering
// ---------------------------------------------------------------------------

/// Actions that are not in the allow-list must be silently dropped (Ok(())).
#[tokio::test]
async fn handle_pull_request_ignores_labeled_action() {
    let handler = make_test_handler();

    let envelope = make_status_envelope("ci/build");
    let pr_envelope = EventEnvelope::new(
        "pull_request".to_string(),
        envelope.repository.clone(),
        EventPayload::new(json!({"action": "labeled"})),
    );

    let result = handler.handle_pull_request(&pr_envelope).await;
    assert!(result.is_ok(), "labeled action must return Ok(()): {:?}", result);
}

#[tokio::test]
async fn handle_pull_request_ignores_assigned_action() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let pr_envelope = EventEnvelope::new(
        "pull_request".to_string(),
        envelope.repository.clone(),
        EventPayload::new(json!({"action": "assigned"})),
    );

    let result = handler.handle_pull_request(&pr_envelope).await;
    assert!(result.is_ok(), "assigned action must return Ok(()): {:?}", result);
}

/// `pull_request_review` events bypass the action filter and go straight to
/// `handle_pull_request`. With a missing PR number the function returns an error
/// rather than panicking.
#[tokio::test]
async fn handle_pull_request_returns_error_when_pr_number_missing() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let pr_envelope = EventEnvelope::new(
        "pull_request".to_string(),
        envelope.repository.clone(),
        // action is "opened" (in allow-list) but no pull_request.number field
        EventPayload::new(json!({"action": "opened"})),
    );

    let result = handler.handle_pull_request(&pr_envelope).await;
    assert!(result.is_err(), "missing PR number must yield an error");
}

#[tokio::test]
async fn handle_pull_request_returns_error_when_installation_id_missing() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let pr_envelope = EventEnvelope::new(
        "pull_request".to_string(),
        envelope.repository.clone(),
        // PR number present, but no installation.id
        EventPayload::new(json!({
            "action": "opened",
            "pull_request": { "number": 42 }
        })),
    );

    let result = handler.handle_pull_request(&pr_envelope).await;
    assert!(result.is_err(), "missing installation ID must yield an error");
}

// ---------------------------------------------------------------------------
// handle_status_event — error paths
// ---------------------------------------------------------------------------

/// A status event for the renovate context but without an `installation.id`
/// field must return a `ProcessingError`.
#[tokio::test]
async fn handle_status_event_returns_error_when_installation_id_missing() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let status_envelope = EventEnvelope::new(
        "status".to_string(),
        envelope.repository.clone(),
        EventPayload::new(json!({
            "context": merge_warden_core::config::RENOVATE_STABILITY_CHECK_CONTEXT,
            "sha": "deadbeef"
            // "installation" is deliberately absent
        })),
    );

    let result = handler.handle_status_event(&status_envelope).await;
    assert!(result.is_err(), "missing installation ID in status event must return error");
}

/// A status event for the renovate context but without a `sha` field must
/// return a `ProcessingError`.
#[tokio::test]
async fn handle_status_event_returns_error_when_sha_missing() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let status_envelope = EventEnvelope::new(
        "status".to_string(),
        envelope.repository.clone(),
        EventPayload::new(json!({
            "context": merge_warden_core::config::RENOVATE_STABILITY_CHECK_CONTEXT
            // sha is absent
        })),
    );

    let result = handler.handle_status_event(&status_envelope).await;
    assert!(result.is_err(), "missing sha in status event must return error");
}

// ---------------------------------------------------------------------------
// ChannelForwardingHandler
// ---------------------------------------------------------------------------

/// The forwarding handler must deliver the envelope into the channel.
#[tokio::test]
async fn channel_forwarding_handler_sends_event_to_channel() {
    use std::sync::Arc;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<crate::ingress::EventEnvelope>(8);
    let handler = Arc::new(super::ChannelForwardingHandler { sender: tx });

    let envelope = make_status_envelope("ci/build");
    handler.handle_event(&envelope).await.expect("should send without error");

    let received = rx.recv().await.expect("should have received one event");
    assert_eq!(received.event_type, envelope.event_type);
}

// ---------------------------------------------------------------------------
// build_router / build_queue_router
// ---------------------------------------------------------------------------

/// `build_router` and `build_queue_router` must construct without panicking.
/// We verify the Router can be built and wrapped in Arc<AppState>.
#[tokio::test]
async fn build_router_constructs_without_panic() {
    use std::sync::Arc;

    let auth = AppAuthProvider::new(12345, TEST_PEM, "https://api.github.com")
        .expect("test RSA key must be valid");
    let github_client = GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .expect("GitHub client must build");

    let state = Arc::new(super::AppState {
        receiver: None,
        github_client,
        policies: merge_warden_core::config::ApplicationDefaults::default(),
    });

    // Neither call should panic.
    let _router = super::build_router(Arc::clone(&state));
    let _queue_router = super::build_queue_router(state);
}

// ---------------------------------------------------------------------------
// WebhookSecretProvider — SecretProvider implementation
// ---------------------------------------------------------------------------

/// `get_webhook_secret` must return the stored secret.
#[tokio::test]
async fn webhook_secret_provider_returns_secret() {
    use github_bot_sdk::auth::SecretProvider;

    let provider = super::WebhookSecretProvider {
        webhook_secret: "my-secret".to_string(),
    };

    let result = provider.get_webhook_secret().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my-secret");
}

/// `get_private_key` must return `NotFound` — it is not used in this path.
#[tokio::test]
async fn webhook_secret_provider_private_key_returns_not_found() {
    use github_bot_sdk::auth::SecretProvider;

    let provider = super::WebhookSecretProvider {
        webhook_secret: "x".to_string(),
    };

    let result = provider.get_private_key().await;
    assert!(result.is_err());
}

/// `get_app_id` must return `NotFound` — it is not used in this path.
#[tokio::test]
async fn webhook_secret_provider_app_id_returns_not_found() {
    use github_bot_sdk::auth::SecretProvider;

    let provider = super::WebhookSecretProvider {
        webhook_secret: "x".to_string(),
    };

    let result = provider.get_app_id().await;
    assert!(result.is_err());
}
