use crate::AppState;

use super::{handle_post_request, verify_github_signature};
use async_trait::async_trait;
use axum::{extract::State, http::HeaderMap};
use chrono::Utc;
use github_bot_sdk::{
    auth::{
        AuthenticationProvider, GitHubAppId, Installation, InstallationId, InstallationPermissions,
        InstallationToken, JsonWebToken, Repository,
    },
    client::{ClientConfig, GitHubClient},
    error::AuthError,
};
use hmac::{Hmac, Mac};
use merge_warden_core::config::{ApplicationDefaults, BypassRules, ChangeTypeLabelConfig};
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

/// Creates a `GitHubClient` backed by the mock auth provider for use in tests.
fn make_test_github_client() -> GitHubClient {
    GitHubClient::builder(MockAuth)
        .config(ClientConfig::default())
        .build()
        .expect("Failed to build test GitHubClient")
}

#[tokio::test]
async fn test_handle_webhook() {
    let state = Arc::new(AppState {
        github_client: make_test_github_client(),
        policies: ApplicationDefaults {
            enable_title_validation: false,
            default_title_pattern: "ab".to_string(),
            default_invalid_title_label: None,
            enable_work_item_validation: false,
            default_work_item_pattern: "cd".to_string(),
            default_missing_work_item_label: None,
            bypass_rules: BypassRules::default(),
            pr_size_check: Default::default(),
            change_type_labels: ChangeTypeLabelConfig::default(),
        },
        webhook_secret: "test_secret".to_string(),
    });

    let headers = HeaderMap::new();
    let body = "{}".to_string();

    let result = handle_post_request(State(state), headers, body).await;
    assert!(
        result.is_err(),
        "Webhook handling should fail with invalid data"
    );
}

#[test]
fn test_verify_github_signature() {
    let secret = "test_secret";
    let headers = HeaderMap::new();
    let body = "test_body";

    let result = verify_github_signature(secret, &headers, body);
    assert!(
        !result,
        "Signature verification should fail with missing headers"
    );
}

#[test]
fn test_verify_github_signature_valid_signature() {
    let secret = "test_secret";
    let mut headers = HeaderMap::new();
    let body = "test_body";

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    let result = mac.finalize();
    let computed_signature = format!("sha256={}", hex::encode(result.into_bytes()));

    headers.insert("X-Hub-Signature-256", computed_signature.parse().unwrap());

    let result = verify_github_signature(secret, &headers, body);
    assert!(
        result,
        "Signature verification should pass with a valid signature"
    );
}

#[test]
fn test_verify_github_signature_invalid_signature() {
    let secret = "test_secret";
    let mut headers = HeaderMap::new();
    let body = "test_body";

    headers.insert(
        "X-Hub-Signature-256",
        "sha256=invalid_signature".parse().unwrap(),
    );

    let result = verify_github_signature(secret, &headers, body);
    assert!(
        !result,
        "Signature verification should fail with an invalid signature"
    );
}

#[test]
fn test_verify_github_signature_missing_header() {
    let secret = "test_secret";
    let headers = HeaderMap::new();
    let body = "test_body";

    let result = verify_github_signature(secret, &headers, body);
    assert!(
        !result,
        "Signature verification should fail when the header is missing"
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
async fn test_handle_post_request_invalid_signature() {
    let state = Arc::new(AppState {
        github_client: make_test_github_client(),
        policies: merge_warden_core::config::ApplicationDefaults::default(),
        webhook_secret: "secret".to_string(),
    });
    let headers = HeaderMap::new();
    let body = "{}".to_string();
    let result = super::handle_post_request(axum::extract::State(state), headers, body).await;
    assert!(result.is_err());
}
