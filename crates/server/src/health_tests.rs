use std::{sync::Arc, time::Duration};

use axum::{body::to_bytes, extract::State, http::StatusCode, response::IntoResponse};
use queue_runtime::{QueueClientFactory, QueueName};

use super::*;
use crate::test_support::{test_app_state, TestAppStateOptions};
use crate::webhook::AppState;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Builds an `AppState` whose GitHub client points at an unroutable address
/// (see [`TestAppStateOptions::github_base_url`]) — every health check test
/// in this file needs "GitHub API unreachable" behaviour, so that override is
/// baked in here rather than repeated at every call site.
fn make_app_state(
    full_checks_enabled: bool,
    queue: Option<(Arc<dyn queue_runtime::QueueClient>, QueueName)>,
) -> Arc<AppState> {
    test_app_state(TestAppStateOptions {
        github_base_url: "http://127.0.0.1:1",
        full_checks_enabled,
        queue,
        ..Default::default()
    })
}

async fn call_handler(state: Arc<AppState>) -> (StatusCode, serde_json::Value) {
    let response = health_check_handler(State(state)).await.into_response();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body must be readable");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("body must be valid JSON");
    (status, json)
}

/// Runs `fut`, panicking if it does not complete within `bound`. Used to
/// prove "basic mode" health checks never block on a real network call.
async fn with_bound<F: std::future::Future>(bound: Duration, fut: F) -> F::Output {
    tokio::time::timeout(bound, fut)
        .await
        .expect("operation must complete within the time bound (did it try a real network call?)")
}

// ---------------------------------------------------------------------------
// HealthState::http_status
// ---------------------------------------------------------------------------

#[test]
fn healthy_maps_to_200_ok() {
    assert_eq!(HealthState::Healthy.http_status(), StatusCode::OK);
}

#[test]
fn degraded_maps_to_207_multi_status() {
    assert_eq!(
        HealthState::Degraded.http_status(),
        StatusCode::from_u16(207).unwrap(),
        "Degraded must map to HTTP 207 Multi-Status, not 200"
    );
}

#[test]
fn unhealthy_maps_to_503_service_unavailable() {
    assert_eq!(
        HealthState::Unhealthy.http_status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Unhealthy must map to HTTP 503, not 200"
    );
}

#[test]
fn all_three_health_states_map_to_distinct_status_codes() {
    let healthy = HealthState::Healthy.http_status();
    let degraded = HealthState::Degraded.http_status();
    let unhealthy = HealthState::Unhealthy.http_status();

    assert_ne!(healthy, degraded);
    assert_ne!(degraded, unhealthy);
    assert_ne!(healthy, unhealthy);
}

// ---------------------------------------------------------------------------
// Serde shape
// ---------------------------------------------------------------------------

#[test]
fn health_state_serializes_to_lowercase_strings() {
    assert_eq!(
        serde_json::to_value(HealthState::Healthy).unwrap(),
        serde_json::json!("healthy")
    );
    assert_eq!(
        serde_json::to_value(HealthState::Degraded).unwrap(),
        serde_json::json!("degraded")
    );
    assert_eq!(
        serde_json::to_value(HealthState::Unhealthy).unwrap(),
        serde_json::json!("unhealthy")
    );
}

#[test]
fn component_health_omits_absent_optional_fields() {
    let component = ComponentHealth {
        status: HealthState::Healthy,
        latency_ms: None,
        depth: None,
        message: None,
    };

    let value = serde_json::to_value(&component).unwrap();
    let obj = value.as_object().unwrap();

    assert!(obj.contains_key("status"));
    assert!(
        !obj.contains_key("latency_ms"),
        "latency_ms must be omitted, not serialized as null, when absent"
    );
    assert!(!obj.contains_key("depth"));
    assert!(!obj.contains_key("message"));
}

#[test]
fn component_health_includes_present_optional_fields() {
    let component = ComponentHealth {
        status: HealthState::Healthy,
        latency_ms: Some(42),
        depth: Some(3),
        message: None,
    };

    let value = serde_json::to_value(&component).unwrap();

    assert_eq!(value["latency_ms"], serde_json::json!(42));
    assert_eq!(value["depth"], serde_json::json!(3));
}

#[test]
fn health_report_top_level_shape_matches_spec() {
    let mut checks = std::collections::BTreeMap::new();
    checks.insert(
        "github_api".to_string(),
        ComponentHealth {
            status: HealthState::Healthy,
            latency_ms: Some(42),
            depth: None,
            message: None,
        },
    );
    checks.insert(
        "config".to_string(),
        ComponentHealth {
            status: HealthState::Healthy,
            latency_ms: None,
            depth: None,
            message: None,
        },
    );

    let report = HealthReport {
        status: HealthState::Healthy,
        checks,
    };

    let value = serde_json::to_value(&report).unwrap();

    assert_eq!(value["status"], serde_json::json!("healthy"));
    assert_eq!(
        value["checks"]["github_api"]["status"],
        serde_json::json!("healthy")
    );
    assert_eq!(
        value["checks"]["github_api"]["latency_ms"],
        serde_json::json!(42)
    );
    assert_eq!(
        value["checks"]["config"]["status"],
        serde_json::json!("healthy")
    );
}

// ---------------------------------------------------------------------------
// HealthCheckConfig::from_env
// ---------------------------------------------------------------------------

static HEALTH_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn from_env_defaults_to_basic_when_var_absent() {
    let _lock = HEALTH_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("MERGE_WARDEN_HEALTH_CHECKS");

    let cfg = HealthCheckConfig::from_env();

    assert!(!cfg.full_checks_enabled);
}

#[test]
fn from_env_enables_full_checks_when_var_is_full() {
    let _lock = HEALTH_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("MERGE_WARDEN_HEALTH_CHECKS", "full");

    let cfg = HealthCheckConfig::from_env();

    std::env::remove_var("MERGE_WARDEN_HEALTH_CHECKS");
    assert!(cfg.full_checks_enabled);
}

#[test]
fn from_env_stays_basic_when_var_is_explicitly_basic() {
    let _lock = HEALTH_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("MERGE_WARDEN_HEALTH_CHECKS", "basic");

    let cfg = HealthCheckConfig::from_env();

    std::env::remove_var("MERGE_WARDEN_HEALTH_CHECKS");
    assert!(!cfg.full_checks_enabled);
}

#[test]
fn from_env_stays_basic_when_var_is_unrecognised_value() {
    let _lock = HEALTH_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("MERGE_WARDEN_HEALTH_CHECKS", "yes-please");

    let cfg = HealthCheckConfig::from_env();

    std::env::remove_var("MERGE_WARDEN_HEALTH_CHECKS");
    assert!(!cfg.full_checks_enabled);
}

#[test]
fn from_env_never_panics() {
    let _lock = HEALTH_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("MERGE_WARDEN_HEALTH_CHECKS");
    let _cfg = HealthCheckConfig::from_env();
}

// ---------------------------------------------------------------------------
// health_check_handler — basic mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn basic_mode_webhook_mode_reports_healthy_without_queue_key() {
    let state = make_app_state(false, None);

    let (status, body) = with_bound(Duration::from_secs(2), call_handler(state)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], serde_json::json!("healthy"));
    assert!(
        body["checks"].get("queue").is_none(),
        "webhook mode must not report a 'queue' check: {body}"
    );
}

#[tokio::test]
async fn basic_mode_includes_config_check() {
    let state = make_app_state(false, None);

    let (_, body) = with_bound(Duration::from_secs(2), call_handler(state)).await;

    assert!(
        body["checks"].get("config").is_some(),
        "expected a 'config' check in the response: {body}"
    );
}

#[tokio::test]
async fn basic_mode_does_not_block_on_unreachable_github_api() {
    // Even though `github_client` points at an unroutable address, basic mode
    // must not attempt to contact it — it must return near-instantly.
    let state = make_app_state(false, None);

    let (status, _) = with_bound(Duration::from_millis(500), call_handler(state)).await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn basic_mode_queue_mode_reports_queue_key_present() {
    let client: Arc<dyn queue_runtime::QueueClient> =
        Arc::from(QueueClientFactory::create_test_client());
    let name = QueueName::new("test-queue".to_string()).unwrap();
    let state = make_app_state(false, Some((client, name)));

    let (_, body) = with_bound(Duration::from_secs(2), call_handler(state)).await;

    assert!(
        body["checks"].get("queue").is_some(),
        "queue mode must report a 'queue' check: {body}"
    );
}

// ---------------------------------------------------------------------------
// health_check_handler — full mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_mode_reports_non_healthy_when_github_api_is_unreachable() {
    let state = make_app_state(true, None);

    // Bounded generously: a real implementation should use a short
    // request-level timeout for the GitHub reachability probe, but we don't
    // want this adversarial test to hang indefinitely against an
    // implementation with a long default HTTP client timeout.
    let (status, body) = with_bound(Duration::from_secs(15), call_handler(state)).await;

    assert_ne!(
        status,
        StatusCode::OK,
        "full mode must not report healthy (200) when GitHub API is unreachable: {body}"
    );
    assert_ne!(
        body["status"],
        serde_json::json!("healthy"),
        "full mode's overall status must reflect GitHub unreachability: {body}"
    );
}

#[tokio::test]
async fn full_mode_includes_github_api_check() {
    let state = make_app_state(true, None);

    let (_, body) = with_bound(Duration::from_secs(15), call_handler(state)).await;

    assert!(
        body["checks"].get("github_api").is_some(),
        "full mode must report a 'github_api' check: {body}"
    );
}

#[tokio::test]
async fn full_mode_queue_mode_includes_queue_key() {
    let client: Arc<dyn queue_runtime::QueueClient> =
        Arc::from(QueueClientFactory::create_test_client());
    let name = QueueName::new("test-queue".to_string()).unwrap();
    let state = make_app_state(true, Some((client, name)));

    let (_, body) = with_bound(Duration::from_secs(15), call_handler(state)).await;

    assert!(
        body["checks"].get("queue").is_some(),
        "full mode queue-mode must report a 'queue' check: {body}"
    );
}

#[tokio::test]
async fn webhook_mode_never_includes_queue_key_even_in_full_mode() {
    let state = make_app_state(true, None);

    let (_, body) = with_bound(Duration::from_secs(15), call_handler(state)).await;

    assert!(
        body["checks"].get("queue").is_none(),
        "webhook mode (queue_client=None) must never report a 'queue' check, in any mode: {body}"
    );
}

// ---------------------------------------------------------------------------
// Security: no leakage of internal state / secrets in health responses
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_mode_error_message_does_not_leak_private_key_material() {
    let state = make_app_state(true, None);

    let (_, body) = with_bound(Duration::from_secs(15), call_handler(state)).await;
    let body_text = body.to_string();

    assert!(
        !body_text.contains("BEGIN RSA PRIVATE KEY") && !body_text.contains("BEGIN PRIVATE KEY"),
        "health response must never contain PEM private key material: {body_text}"
    );
}

#[tokio::test]
async fn full_mode_error_message_does_not_contain_rust_backtrace_markers() {
    let state = make_app_state(true, None);

    let (_, body) = with_bound(Duration::from_secs(15), call_handler(state)).await;
    let body_text = body.to_string();

    for marker in ["RUST_BACKTRACE", "panicked at", "src/main.rs", "unwrap()"] {
        assert!(
            !body_text.contains(marker),
            "health response must not leak internal diagnostics (found '{marker}'): {body_text}"
        );
    }
}
