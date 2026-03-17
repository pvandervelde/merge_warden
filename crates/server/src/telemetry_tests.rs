use super::*;
use crate::errors::ServerError;
use std::sync::Mutex;

static TELEM_MUTEX: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// TelemetryConfig::from_env
// ---------------------------------------------------------------------------

#[test]
fn from_env_otlp_endpoint_is_none_when_var_absent() {
    let _lock = TELEM_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::remove_var("OTEL_SERVICE_NAME");
    std::env::remove_var("OTEL_SERVICE_VERSION");

    let cfg = TelemetryConfig::from_env();
    assert!(cfg.otlp_endpoint.is_none());
}

#[test]
fn from_env_reads_otlp_endpoint_when_set() {
    let _lock = TELEM_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
    std::env::remove_var("OTEL_SERVICE_NAME");
    std::env::remove_var("OTEL_SERVICE_VERSION");

    let cfg = TelemetryConfig::from_env();

    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");

    assert_eq!(cfg.otlp_endpoint.as_deref(), Some("http://localhost:4317"));
}

#[test]
fn from_env_uses_default_service_name_when_var_absent() {
    let _lock = TELEM_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::remove_var("OTEL_SERVICE_NAME");
    std::env::remove_var("OTEL_SERVICE_VERSION");

    let cfg = TelemetryConfig::from_env();
    assert_eq!(cfg.service_name, "merge-warden");
}

#[test]
fn from_env_reads_custom_service_name() {
    let _lock = TELEM_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::set_var("OTEL_SERVICE_NAME", "my-service");
    std::env::remove_var("OTEL_SERVICE_VERSION");

    let cfg = TelemetryConfig::from_env();

    std::env::remove_var("OTEL_SERVICE_NAME");

    assert_eq!(cfg.service_name, "my-service");
}

#[test]
fn from_env_never_fails() {
    // from_env must always succeed; test with no vars set.
    let _lock = TELEM_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::remove_var("OTEL_SERVICE_NAME");
    std::env::remove_var("OTEL_SERVICE_VERSION");

    // Calling this should never panic.
    let _cfg = TelemetryConfig::from_env();
}

// ---------------------------------------------------------------------------
// init_telemetry — console-only path
// ---------------------------------------------------------------------------

#[test]
fn init_telemetry_without_otlp_succeeds_or_reports_already_set() {
    let config = TelemetryConfig {
        otlp_endpoint: None,
        service_name: "test-service".to_string(),
        service_version: "0.0.1".to_string(),
    };

    let result = init_telemetry(&config);

    // Real tests run in the same process; the subscriber may already be set.
    match &result {
        Ok(()) => {}
        Err(ServerError::TelemetryInitFailed(_)) => {}
        Err(e) => panic!("Unexpected error variant: {:?}", e),
    }
}
