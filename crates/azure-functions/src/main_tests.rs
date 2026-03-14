use crate::AppState;

use super::handle_post_request;
use async_trait::async_trait;
use axum::{extract::State, http::HeaderMap};
use chrono::Utc;
use github_bot_sdk::{
    auth::{
        AuthenticationProvider, GitHubAppId, Installation, InstallationId, InstallationPermissions,
        InstallationToken, JsonWebToken, Repository, SecretProvider,
    },
    client::{ClientConfig, GitHubClient},
    error::{AuthError, SecretError},
    events::{EventProcessor, ProcessorConfig},
    webhook::WebhookReceiver,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

/// Minimal mock authentication provider for unit tests.
///
/// Returns pre-configured tokens without making any network calls.
struct MockAuth;

#[async_trait]
impl AuthenticationProvider for MockAuth {
    async fn app_token(&self) -> Result<JsonWebToken, AuthError> {
        let app_id = GitHubAppId::new(1);
        let expires_at = Utc::now() + chrono::Duration::minutes(10);
        Ok(JsonWebToken::new(
            "test.jwt.token".to_string(),
            app_id,
            expires_at,
        ))
    }

    async fn installation_token(
        &self,
        _installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        let id = InstallationId::new(12345);
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        Ok(InstallationToken::new(
            "test_token".to_string(),
            id,
            expires_at,
            InstallationPermissions::default(),
            vec![],
        ))
    }

    async fn refresh_installation_token(
        &self,
        installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        self.installation_token(installation_id).await
    }

    async fn list_installations(&self) -> Result<Vec<Installation>, AuthError> {
        Ok(vec![])
    }

    async fn get_installation_repositories(
        &self,
        _installation_id: InstallationId,
    ) -> Result<Vec<Repository>, AuthError> {
        Ok(vec![])
    }
}

/// `SecretProvider` backed by a known in-memory secret for unit tests.
struct TestSecretProvider {
    webhook_secret: String,
}

#[async_trait]
impl SecretProvider for TestSecretProvider {
    async fn get_private_key(&self) -> Result<github_bot_sdk::auth::PrivateKey, SecretError> {
        Err(SecretError::NotFound {
            key: "private_key".to_string(),
        })
    }

    async fn get_app_id(&self) -> Result<GitHubAppId, SecretError> {
        Err(SecretError::NotFound {
            key: "app_id".to_string(),
        })
    }

    async fn get_webhook_secret(&self) -> Result<String, SecretError> {
        Ok(self.webhook_secret.clone())
    }

    fn cache_duration(&self) -> chrono::Duration {
        chrono::Duration::hours(1)
    }
}

/// Creates a `GitHubClient` backed by the mock auth provider for use in tests.
fn make_test_github_client() -> GitHubClient {
    GitHubClient::builder(MockAuth)
        .config(ClientConfig::default())
        .build()
        .expect("Failed to build test GitHubClient")
}

/// Builds a test `AppState` backed by the given webhook secret and no-op handler.
async fn make_test_app_state(webhook_secret: &str) -> Arc<AppState> {
    let secret_provider = Arc::new(TestSecretProvider {
        webhook_secret: webhook_secret.to_string(),
    });
    let processor = EventProcessor::new(ProcessorConfig::default());
    let receiver = WebhookReceiver::new(secret_provider, processor);
    // No handlers registered — tests only exercise validation/routing.
    Arc::new(AppState { receiver })
}

/// Computes the expected `X-Hub-Signature-256` header value for a given secret and body.
fn make_signature(secret: &str, body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

#[tokio::test]
async fn test_handle_webhook_missing_signature_returns_error() {
    let state = make_test_app_state("test_secret").await;
    let headers = HeaderMap::new();
    let body = "{}".to_string();

    let result = handle_post_request(State(state), headers, body).await;
    assert!(
        result.is_err(),
        "Webhook handling should fail when X-Hub-Signature-256 is absent"
    );
}

#[tokio::test]
async fn test_handle_webhook_invalid_signature_returns_error() {
    let state = make_test_app_state("secret").await;

    let mut headers = HeaderMap::new();
    headers.insert("X-Hub-Signature-256", "sha256=badhash".parse().unwrap());
    headers.insert("X-GitHub-Event", "pull_request".parse().unwrap());

    let body = "{}".to_string();
    let result = handle_post_request(State(state), headers, body).await;
    assert!(
        result.is_err(),
        "Webhook handling should fail with an invalid signature"
    );
}

#[tokio::test]
async fn test_handle_webhook_valid_signature_non_pr_event_returns_ok() {
    let secret = "test_secret";
    // EventProcessor requires a full repository object to parse the payload.
    let body = r#"{
        "ref": "refs/heads/main",
        "repository": {
            "id": 123,
            "name": "test-repo",
            "full_name": "test-owner/test-repo",
            "owner": {
                "login": "test-owner",
                "id": 456,
                "avatar_url": "https://avatars.githubusercontent.com/u/456",
                "type": "User"
            },
            "description": null,
            "private": false,
            "default_branch": "main",
            "html_url": "https://github.com/test-owner/test-repo",
            "clone_url": "https://github.com/test-owner/test-repo.git",
            "ssh_url": "git@github.com:test-owner/test-repo.git",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    }"#;
    let sig = make_signature(secret, body.as_bytes());

    let state = make_test_app_state(secret).await;
    let mut headers = HeaderMap::new();
    headers.insert("X-Hub-Signature-256", sig.parse().unwrap());
    headers.insert("X-GitHub-Event", "push".parse().unwrap());
    headers.insert("X-GitHub-Delivery", "abc123".parse().unwrap());

    let result = handle_post_request(State(state), headers, body.to_string()).await;
    // push events are not pull_request; receiver accepts them (200) and the
    // handler (none registered) silently no-ops.
    assert!(
        result.is_ok(),
        "Non-PR events with valid signature should return 200"
    );
}

#[tokio::test]
async fn test_get_azure_secrets_missing_env_vars() {
    std::env::remove_var("KEY_VAULT_NAME");
    let result = super::get_azure_secrets().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_github_app_invalid_key() {
    let secrets = super::AppSecrets {
        app_id: 123,
        app_private_key: "invalid".to_string(),
        webhook_secret: "secret".to_string(),
    };
    let result = super::create_github_app(&secrets).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_post_request_missing_signature_returns_unauthorized() {
    let state = make_test_app_state("secret").await;
    let headers = HeaderMap::new();
    let body = "{}".to_string();
    let result = super::handle_post_request(axum::extract::State(state), headers, body).await;
    assert!(result.is_err());
}
