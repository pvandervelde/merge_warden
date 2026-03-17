// See docs/spec/interfaces/server-config.md for the full contract.

use std::{fmt, path::PathBuf};

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

use merge_warden_core::config::ApplicationDefaults;

use crate::errors::ServerError;

// ---------------------------------------------------------------------------
// SecretString
// ---------------------------------------------------------------------------

/// Opaque wrapper around a `String` that prevents the value appearing in logs.
///
/// Only the `expose()` method allows access to the inner string.
///
/// See docs/spec/interfaces/server-config.md — `SecretString`
pub struct SecretString(String);

impl SecretString {
    /// Wraps `value` in a `SecretString`.
    pub fn new(value: String) -> Self {
        SecretString(value)
    }

    /// Returns the contained string slice for use with APIs that require it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// ---------------------------------------------------------------------------
// ServerSecrets
// ---------------------------------------------------------------------------

/// GitHub App credentials and webhook signing secret.
///
/// Loaded once at startup by [`load_secrets`].
///
/// See docs/spec/interfaces/server-config.md — `ServerSecrets`
#[derive(Debug)]
pub struct ServerSecrets {
    /// Numeric GitHub App ID from `GITHUB_APP_ID`.
    pub github_app_id: u64,
    /// PEM-encoded private key from `GITHUB_APP_PRIVATE_KEY`.
    pub github_app_private_key: SecretString,
    /// Webhook signing secret from `GITHUB_WEBHOOK_SECRET`.
    pub github_webhook_secret: SecretString,
}

// ---------------------------------------------------------------------------
// ReceiverMode
// ---------------------------------------------------------------------------

/// Controls how the server receives GitHub events.
///
/// Selected once at startup from `MERGE_WARDEN_RECEIVER_MODE`. Values are
/// case-insensitive (`"webhook"` or `"queue"`).
///
/// See docs/spec/interfaces/server-config.md — `ReceiverMode`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiverMode {
    /// Axum POST handler processes events via an in-process channel.
    Webhook,
    /// Axum POST handler enqueues events; a separate Tokio task processes them.
    Queue,
}

// ---------------------------------------------------------------------------
// QueueServerConfig
// ---------------------------------------------------------------------------

/// Queue provider settings, present only when `receiver_mode == ReceiverMode::Queue`.
///
/// See docs/spec/interfaces/server-config.md — `QueueServerConfig`
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QueueServerConfig {
    /// Queue provider identifier (e.g. `"azure"`). From `MERGE_WARDEN_QUEUE_PROVIDER`.
    pub provider: String,
    /// Queue name. From `MERGE_WARDEN_QUEUE_NAME`. Default: `"merge-warden-events"`.
    pub queue_name: String,
    /// Max in-flight messages. From `MERGE_WARDEN_QUEUE_CONCURRENCY`. Default: `4`.
    pub concurrency: usize,
    /// Provider-specific namespace (e.g. Azure Service Bus namespace). From
    /// `AZURE_SERVICEBUS_NAMESPACE`.
    pub namespace: Option<String>,
}

// ---------------------------------------------------------------------------
// ServerConfig
// ---------------------------------------------------------------------------

/// Full server configuration assembled from environment variables and an
/// optional TOML file.
///
/// See docs/spec/interfaces/server-config.md — `ServerConfig`
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// TCP port the Axum server listens on. From `MERGE_WARDEN_PORT`. Default: `3000`.
    pub port: u16,
    /// Ingress mode. From `MERGE_WARDEN_RECEIVER_MODE`. Default: `Webhook`.
    pub receiver_mode: ReceiverMode,
    /// Optional path to a TOML policy configuration file. From `MERGE_WARDEN_CONFIG_FILE`.
    #[allow(dead_code)]
    pub config_file_path: Option<PathBuf>,
    /// Merge-warden application policy defaults (from TOML file or `ApplicationDefaults::default()`).
    pub application_defaults: ApplicationDefaults,
    /// Queue-mode settings. `Some(...)` only when `receiver_mode == ReceiverMode::Queue`.
    #[allow(dead_code)]
    pub queue: Option<QueueServerConfig>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Reads the three required GitHub secrets from environment variables.
///
/// See docs/spec/interfaces/server-config.md — `load_secrets()`
///
/// # Errors
/// - [`ServerError::MissingEnvVar`] when any required variable is absent.
/// - [`ServerError::InvalidEnvVar`] when `GITHUB_APP_ID` is not a valid `u64`.
pub fn load_secrets() -> Result<ServerSecrets, ServerError> {
    let app_id_str = std::env::var("GITHUB_APP_ID")
        .map_err(|_| ServerError::MissingEnvVar("GITHUB_APP_ID".to_string()))?;

    let github_app_id: u64 = app_id_str.parse().map_err(|e| ServerError::InvalidEnvVar {
        name: "GITHUB_APP_ID".to_string(),
        message: format!("Expected an unsigned integer: {}", e),
    })?;

    let github_app_private_key = std::env::var("GITHUB_APP_PRIVATE_KEY")
        .map(SecretString::new)
        .map_err(|_| ServerError::MissingEnvVar("GITHUB_APP_PRIVATE_KEY".to_string()))?;

    let github_webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .map(SecretString::new)
        .map_err(|_| ServerError::MissingEnvVar("GITHUB_WEBHOOK_SECRET".to_string()))?;

    Ok(ServerSecrets {
        github_app_id,
        github_app_private_key,
        github_webhook_secret,
    })
}

/// Builds [`ServerConfig`] from environment variables and an optional TOML file.
///
/// See docs/spec/interfaces/server-config.md — `load_config()`
///
/// # Errors
/// - [`ServerError::InvalidEnvVar`] for malformed port or receiver mode values.
/// - [`ServerError::MissingEnvVar`] for `MERGE_WARDEN_QUEUE_PROVIDER` in queue mode.
/// - [`ServerError::ConfigError`] if the TOML file cannot be parsed.
pub fn load_config() -> Result<ServerConfig, ServerError> {
    use tracing::info;

    // --- Port ---
    let port = match std::env::var("MERGE_WARDEN_PORT") {
        Ok(val) => val.parse::<u16>().map_err(|e| ServerError::InvalidEnvVar {
            name: "MERGE_WARDEN_PORT".to_string(),
            message: format!("Expected a TCP port number (1–65535): {}", e),
        })?,
        Err(_) => 3000,
    };

    // --- Receiver mode ---
    let receiver_mode_val = std::env::var("MERGE_WARDEN_RECEIVER_MODE")
        .unwrap_or_else(|_| "webhook".to_string())
        .to_lowercase();

    let receiver_mode = match receiver_mode_val.as_str() {
        "webhook" => ReceiverMode::Webhook,
        "queue" => ReceiverMode::Queue,
        other => {
            return Err(ServerError::InvalidEnvVar {
                name: "MERGE_WARDEN_RECEIVER_MODE".to_string(),
                message: format!("Expected 'webhook' or 'queue', got '{}'", other),
            })
        }
    };

    // --- Config file path ---
    let config_file_path = std::env::var("MERGE_WARDEN_CONFIG_FILE")
        .ok()
        .map(PathBuf::from);

    // --- Application defaults: TOML file → env overrides → compiled defaults ---
    let application_defaults = if let Some(ref path) = config_file_path {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ServerError::ConfigError(format!("Failed to read '{}': {}", path.display(), e))
        })?;

        // The TOML file uses a `[policies]` section to hold `ApplicationDefaults`.
        #[derive(serde::Deserialize, Default)]
        struct ServerTomlConfig {
            #[serde(default)]
            policies: ApplicationDefaults,
        }

        let parsed: ServerTomlConfig = toml::from_str(&content).map_err(|e| {
            ServerError::ConfigError(format!("Failed to parse '{}': {}", path.display(), e))
        })?;

        info!(path = %path.display(), "Loaded application defaults from TOML config file");
        parsed.policies
    } else {
        info!("No MERGE_WARDEN_CONFIG_FILE set; using compiled-in application defaults");
        ApplicationDefaults::default()
    };

    // --- Queue config (only in queue mode) ---
    let queue = if receiver_mode == ReceiverMode::Queue {
        let provider = std::env::var("MERGE_WARDEN_QUEUE_PROVIDER")
            .map_err(|_| ServerError::MissingEnvVar("MERGE_WARDEN_QUEUE_PROVIDER".to_string()))?;

        let queue_name = std::env::var("MERGE_WARDEN_QUEUE_NAME")
            .unwrap_or_else(|_| "merge-warden-events".to_string());

        let concurrency = match std::env::var("MERGE_WARDEN_QUEUE_CONCURRENCY") {
            Ok(v) => v.parse::<usize>().map_err(|e| ServerError::InvalidEnvVar {
                name: "MERGE_WARDEN_QUEUE_CONCURRENCY".to_string(),
                message: format!("Expected a positive integer: {}", e),
            })?,
            Err(_) => 4,
        };

        let namespace = std::env::var("AZURE_SERVICEBUS_NAMESPACE").ok();

        Some(QueueServerConfig {
            provider,
            queue_name,
            concurrency,
            namespace,
        })
    } else {
        None
    };

    Ok(ServerConfig {
        port,
        receiver_mode,
        config_file_path,
        application_defaults,
        queue,
    })
}
