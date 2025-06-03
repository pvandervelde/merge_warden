use crate::{AppConfig, AppState};

use super::{create_github_app, get_azure_config, handle_post_request, verify_github_signature};
use axum::{extract::State, http::HeaderMap};
use hmac::{Hmac, Mac};
use merge_warden_core::config::RulesConfig;
use merge_warden_developer_platforms::models::User;
use octocrab::Octocrab;
use sha2::Sha256;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_handle_webhook() {
    let state = Arc::new(AppState {
        octocrab: Octocrab::default(),
        user: User {
            id: 10,
            login: "a".to_string(),
        },
        rules: RulesConfig {
            require_work_items: true,
            enforce_title_convention: Some(true),
            min_approvals: None,
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
