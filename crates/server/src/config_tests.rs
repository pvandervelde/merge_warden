use std::sync::Mutex;

use super::*;
use crate::errors::ServerError;

// Serialise all env-var tests to prevent parallel threads from clobbering each
// other's environment state.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Helper: sets env vars before a test and removes them when dropped.
// ---------------------------------------------------------------------------

struct EnvGuard(Vec<String>);

impl EnvGuard {
    fn prepare(set: &[(&str, &str)], clear: &[&str]) -> Self {
        for &var in clear {
            std::env::remove_var(var);
        }
        for &(k, v) in set {
            std::env::set_var(k, v);
        }
        let mut all: Vec<String> = clear.iter().map(|s| (*s).to_string()).collect();
        for &(k, _) in set {
            if !all.contains(&k.to_string()) {
                all.push(k.to_string());
            }
        }
        Self(all)
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for var in &self.0 {
            std::env::remove_var(var);
        }
    }
}

// ---------------------------------------------------------------------------
// SecretString
// ---------------------------------------------------------------------------

#[test]
fn secret_string_debug_shows_redacted() {
    let s = SecretString::new("my-sensitive-value".to_string());
    assert_eq!(format!("{:?}", s), "[REDACTED]");
}

#[test]
fn secret_string_display_shows_redacted() {
    let s = SecretString::new("my-sensitive-value".to_string());
    assert_eq!(format!("{}", s), "[REDACTED]");
}

#[test]
fn secret_string_expose_returns_inner_value() {
    let s = SecretString::new("the-actual-secret".to_string());
    assert_eq!(s.expose(), "the-actual-secret");
}

// ---------------------------------------------------------------------------
// load_secrets — happy path
// ---------------------------------------------------------------------------

#[test]
fn load_secrets_succeeds_when_all_vars_present() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_GITHUB_APP_ID", "42"),
            ("MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY", "pem-content"),
            ("GITHUB_WEBHOOK_SECRET", "hook-secret"),
        ],
        &[
            "MERGE_WARDEN_GITHUB_APP_ID",
            "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY",
            "GITHUB_APP_ID",
            "GITHUB_APP_PRIVATE_KEY",
            "GITHUB_WEBHOOK_SECRET",
        ],
    );

    let result = load_secrets();
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

    let s = result.unwrap();
    assert_eq!(s.github_app_id, 42);
    assert_eq!(s.github_app_private_key.expose(), "pem-content");
    assert_eq!(
        s.github_webhook_secret.as_ref().unwrap().expose(),
        "hook-secret"
    );
}

// ---------------------------------------------------------------------------
// load_secrets — missing variable errors
// ---------------------------------------------------------------------------

#[test]
fn load_secrets_errors_when_app_id_missing() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY", "k"),
            ("GITHUB_WEBHOOK_SECRET", "s"),
        ],
        &[
            "MERGE_WARDEN_GITHUB_APP_ID",
            "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY",
            "GITHUB_APP_ID",
            "GITHUB_APP_PRIVATE_KEY",
            "GITHUB_WEBHOOK_SECRET",
        ],
    );

    let r = load_secrets();
    assert!(
        matches!(&r, Err(ServerError::MissingEnvVar(n)) if n == "MERGE_WARDEN_GITHUB_APP_ID"),
        "Expected MissingEnvVar(MERGE_WARDEN_GITHUB_APP_ID), got: {:?}",
        r
    );
}

#[test]
fn load_secrets_errors_when_app_id_is_not_numeric() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_GITHUB_APP_ID", "not-a-number"),
            ("MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY", "k"),
            ("GITHUB_WEBHOOK_SECRET", "s"),
        ],
        &[
            "MERGE_WARDEN_GITHUB_APP_ID",
            "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY",
            "GITHUB_APP_ID",
            "GITHUB_APP_PRIVATE_KEY",
            "GITHUB_WEBHOOK_SECRET",
        ],
    );

    let r = load_secrets();
    assert!(
        matches!(&r, Err(ServerError::InvalidEnvVar { name, .. }) if name == "MERGE_WARDEN_GITHUB_APP_ID"),
        "Expected InvalidEnvVar(MERGE_WARDEN_GITHUB_APP_ID), got: {:?}",
        r
    );
}

#[test]
fn load_secrets_errors_when_private_key_missing() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_GITHUB_APP_ID", "1"),
            ("GITHUB_WEBHOOK_SECRET", "s"),
        ],
        &[
            "MERGE_WARDEN_GITHUB_APP_ID",
            "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY",
            "GITHUB_APP_ID",
            "GITHUB_APP_PRIVATE_KEY",
            "GITHUB_WEBHOOK_SECRET",
        ],
    );

    let r = load_secrets();
    assert!(
        matches!(&r, Err(ServerError::MissingEnvVar(n)) if n == "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY"),
        "Expected MissingEnvVar(MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY), got: {:?}",
        r
    );
}

#[test]
fn load_secrets_returns_none_webhook_secret_when_var_absent() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_GITHUB_APP_ID", "1"),
            ("MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY", "k"),
        ],
        &[
            "MERGE_WARDEN_GITHUB_APP_ID",
            "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY",
            "GITHUB_APP_ID",
            "GITHUB_APP_PRIVATE_KEY",
            "GITHUB_WEBHOOK_SECRET",
        ],
    );

    let r = load_secrets();
    assert!(r.is_ok(), "Expected Ok, got: {:?}", r);
    let s = r.unwrap();
    assert!(
        s.github_webhook_secret.is_none(),
        "Expected github_webhook_secret to be None when GITHUB_WEBHOOK_SECRET is absent"
    );
}

// ---------------------------------------------------------------------------
// load_config — port
// ---------------------------------------------------------------------------

#[test]
fn load_config_uses_default_port_3000_when_var_absent() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
            "MERGE_WARDEN_QUEUE_PROVIDER",
        ],
    );

    let r = load_config();
    assert!(r.is_ok(), "{:?}", r);
    let c = r.unwrap();
    assert_eq!(c.port, 3000);
    assert_eq!(c.receiver_mode, ReceiverMode::Webhook);
    assert!(c.queue.is_none());
}

#[test]
fn load_config_reads_custom_port() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_PORT", "9090")],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    assert!(r.is_ok(), "{:?}", r);
    assert_eq!(r.unwrap().port, 9090);
}

#[test]
fn load_config_errors_on_non_numeric_port() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_PORT", "ninety-nine")],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    assert!(
        matches!(&r, Err(ServerError::InvalidEnvVar { name, .. }) if name == "MERGE_WARDEN_PORT"),
        "Expected InvalidEnvVar(MERGE_WARDEN_PORT), got: {:?}",
        r
    );
}

// ---------------------------------------------------------------------------
// load_config — receiver mode
// ---------------------------------------------------------------------------

#[test]
fn load_config_accepts_webhook_mode_case_insensitively() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_RECEIVER_MODE", "WEBHOOK")],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    assert!(r.is_ok(), "{:?}", r);
    assert_eq!(r.unwrap().receiver_mode, ReceiverMode::Webhook);
}

#[test]
fn load_config_errors_on_unknown_receiver_mode() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_RECEIVER_MODE", "grpc")],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    assert!(
        matches!(
            &r,
            Err(ServerError::InvalidEnvVar { name, .. }) if name == "MERGE_WARDEN_RECEIVER_MODE"
        ),
        "Expected InvalidEnvVar(MERGE_WARDEN_RECEIVER_MODE), got: {:?}",
        r
    );
}

// ---------------------------------------------------------------------------
// load_config — queue mode
// ---------------------------------------------------------------------------

#[test]
fn load_config_queue_mode_requires_provider_var() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_RECEIVER_MODE", "queue")],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
            "MERGE_WARDEN_QUEUE_PROVIDER",
        ],
    );

    let r = load_config();
    assert!(
        matches!(&r, Err(ServerError::MissingEnvVar(n)) if n == "MERGE_WARDEN_QUEUE_PROVIDER"),
        "Expected MissingEnvVar(MERGE_WARDEN_QUEUE_PROVIDER), got: {:?}",
        r
    );
}

#[test]
fn load_config_queue_mode_populates_queue_config() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_RECEIVER_MODE", "queue"),
            ("MERGE_WARDEN_QUEUE_PROVIDER", "azure"),
            ("MERGE_WARDEN_QUEUE_NAME", "my-events"),
            ("MERGE_WARDEN_QUEUE_CONCURRENCY", "8"),
            ("AZURE_SERVICEBUS_NAMESPACE", "myns"),
        ],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
            "MERGE_WARDEN_QUEUE_PROVIDER",
            "MERGE_WARDEN_QUEUE_NAME",
            "MERGE_WARDEN_QUEUE_CONCURRENCY",
            "AZURE_SERVICEBUS_NAMESPACE",
        ],
    );

    let r = load_config();
    assert!(r.is_ok(), "{:?}", r);
    let c = r.unwrap();
    assert_eq!(c.receiver_mode, ReceiverMode::Queue);
    let q = c.queue.expect("queue config should be Some in queue mode");
    assert_eq!(q.provider, "azure");
    assert_eq!(q.queue_name, "my-events");
    assert_eq!(q.concurrency, 8);
    assert_eq!(q.namespace.as_deref(), Some("myns"));
}

#[test]
fn load_config_queue_mode_errors_when_concurrency_is_zero() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[
            ("MERGE_WARDEN_RECEIVER_MODE", "queue"),
            ("MERGE_WARDEN_QUEUE_PROVIDER", "memory"),
            ("MERGE_WARDEN_QUEUE_CONCURRENCY", "0"),
        ],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
            "MERGE_WARDEN_QUEUE_PROVIDER",
            "MERGE_WARDEN_QUEUE_CONCURRENCY",
        ],
    );

    let r = load_config();
    assert!(
        matches!(
            &r,
            Err(ServerError::InvalidEnvVar { name, .. }) if name == "MERGE_WARDEN_QUEUE_CONCURRENCY"
        ),
        "Expected InvalidEnvVar(MERGE_WARDEN_QUEUE_CONCURRENCY), got: {:?}",
        r
    );
}

// ---------------------------------------------------------------------------
// load_config — TOML config file
// ---------------------------------------------------------------------------

#[test]
fn load_config_uses_defaults_when_config_file_var_is_absent() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _env = EnvGuard::prepare(
        &[],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    assert!(r.is_ok(), "{:?}", r);
    let defaults = merge_warden_core::config::ApplicationDefaults::default();
    assert_eq!(
        r.unwrap().application_defaults.enable_title_validation,
        defaults.enable_title_validation
    );
}

#[test]
fn load_config_reads_application_defaults_from_toml_file() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // Write a minimal TOML file with one overridden field.
    let mut path = std::env::temp_dir();
    path.push("merge_warden_server_load_config_test.toml");
    std::fs::write(&path, "[policies]\nenforceTitleValidation = true\n").unwrap();

    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_CONFIG_FILE", path.to_str().unwrap())],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    let _ = std::fs::remove_file(&path);

    assert!(r.is_ok(), "{:?}", r);
    assert!(
        r.unwrap().application_defaults.enable_title_validation,
        "Expected enforceTitleValidation = true from TOML"
    );
}

#[test]
fn load_config_returns_config_error_for_malformed_toml() {
    let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let mut path = std::env::temp_dir();
    path.push("merge_warden_server_bad_config_test.toml");
    std::fs::write(&path, "NOT { valid TOML !!@#$%^&*()").unwrap();

    let _env = EnvGuard::prepare(
        &[("MERGE_WARDEN_CONFIG_FILE", path.to_str().unwrap())],
        &[
            "MERGE_WARDEN_PORT",
            "MERGE_WARDEN_RECEIVER_MODE",
            "MERGE_WARDEN_CONFIG_FILE",
        ],
    );

    let r = load_config();
    let _ = std::fs::remove_file(&path);

    assert!(
        matches!(r, Err(ServerError::ConfigError(_))),
        "Expected ConfigError, got: {:?}",
        r
    );
}
