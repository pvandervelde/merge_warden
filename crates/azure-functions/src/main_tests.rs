use crate::AppState;

use super::{handle_post_request, verify_github_signature};
use axum::{extract::State, http::HeaderMap};
use hmac::{Hmac, Mac};
use merge_warden_core::config::{ApplicationDefaults, BypassRules};
use merge_warden_developer_platforms::models::User;
use octocrab::Octocrab;
use sha2::Sha256;
use std::sync::Arc;

#[tokio::test]
async fn test_handle_webhook() {
    let state = Arc::new(AppState {
        octocrab: Octocrab::default(),
        user: User {
            id: 10,
            login: "a".to_string(),
        },
        policies: ApplicationDefaults {
            enable_title_validation: false,
            default_title_pattern: "ab".to_string(),
            default_invalid_title_label: None,
            enable_work_item_validation: false,
            default_work_item_pattern: "cd".to_string(),
            default_missing_work_item_label: None,
            bypass_rules: BypassRules::default(),
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
async fn test_get_azure_config_missing_env_vars() {
    std::env::remove_var("KEY_VAULT_NAME");
    let result = super::get_azure_config().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_github_app_invalid_key() {
    let config = super::AppConfig {
        app_id: 123,
        app_private_key: "invalid".to_string(),
        webhook_secret: "secret".to_string(),
        port_number: 3000,
        enforce_title_convention: false,
        default_title_pattern: Some("ab".to_string()),
        default_invalid_title_label: None,
        require_work_items: false,
        default_work_item_pattern: Some("cd".to_string()),
        default_missing_work_item_label: None,
    };
    let result = super::create_github_app(&config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_post_request_invalid_signature() {
    use super::AppState;
    use axum::http::HeaderMap;
    use std::sync::Arc;
    let state = Arc::new(AppState {
        octocrab: octocrab::Octocrab::default(),
        user: merge_warden_developer_platforms::models::User::default(),
        policies: merge_warden_core::config::ApplicationDefaults::default(),
        webhook_secret: "secret".to_string(),
    });
    let headers = HeaderMap::new();
    let body = "{}".to_string();
    let result = super::handle_post_request(axum::extract::State(state), headers, body).await;
    assert!(result.is_err());
}
