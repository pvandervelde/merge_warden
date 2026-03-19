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
    ///
    /// Required in webhook mode; `None` in queue mode (the receiving service
    /// owns signature validation, not merge-warden).
    pub github_webhook_secret: Option<SecretString>,
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

impl QueueServerConfig {
    /// Converts this configuration into a [`queue_runtime::QueueConfig`] suitable
    /// for passing to [`queue_runtime::QueueClientFactory::create_client`].
    ///
    /// # Errors
    /// Returns [`ServerError::ConfigError`] when `provider` is unrecognised or
    /// a required provider-specific variable (e.g. `AZURE_SERVICEBUS_NAMESPACE`)
    /// is absent.
    pub fn to_queue_config(&self) -> Result<queue_runtime::QueueConfig, crate::errors::ServerError> {
        use queue_runtime::{
            AzureAuthMethod, AzureServiceBusConfig, AwsSqsConfig, InMemoryConfig, ProviderConfig,
            QueueConfig,
        };

        let provider = match self.provider.to_lowercase().as_str() {
            "azure" => {
                let namespace =
                    self.namespace.clone().ok_or_else(|| crate::errors::ServerError::MissingEnvVar(
                        "AZURE_SERVICEBUS_NAMESPACE".to_string(),
                    ))?;
                ProviderConfig::AzureServiceBus(AzureServiceBusConfig {
                    connection_string: None,
                    namespace: Some(namespace),
                    auth_method: AzureAuthMethod::DefaultCredential,
                    use_sessions: true,
                    session_timeout: chrono::Duration::minutes(5),
                })
            }
            "aws" => ProviderConfig::AwsSqs(AwsSqsConfig {
                region: std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                access_key_id: std::env::var("AWS_ACCESS_KEY_ID").ok(),
                secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
                use_fifo_queues: true,
            }),
            "memory" | "" => ProviderConfig::InMemory(InMemoryConfig::default()),
            other => {
                return Err(crate::errors::ServerError::ConfigError(format!(
                    "Unknown queue provider '{}'. Expected 'azure', 'aws', or 'memory'.",
                    other
                )))
            }
        };

        Ok(QueueConfig {
            provider,
            ..QueueConfig::default()
        })
    }
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
    /// Populated at startup and available for hot-reload or diagnostic introspection.
    #[allow(dead_code)]
    pub config_file_path: Option<PathBuf>,
    /// Merge-warden application policy defaults (from TOML file or `ApplicationDefaults::default()`).
    pub application_defaults: ApplicationDefaults,
    /// Queue-mode settings. `Some(...)` only when `receiver_mode == ReceiverMode::Queue`.
    pub queue: Option<QueueServerConfig>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Reads GitHub App credentials from environment variables.
/// `GITHUB_WEBHOOK_SECRET` is only required in webhook mode — in queue mode
/// the separate receiving service owns signature validation.
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
        .ok();

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
