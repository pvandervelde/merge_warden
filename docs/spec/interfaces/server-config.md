# Interface Spec: server — Configuration and Startup

**Source**: `crates/server/src/config.rs`, `errors.rs`, `telemetry.rs`
**Spec**: `docs/spec/design/containerisation.md`
**Task**: 2.0

---

## Environment Variables

All configuration is injected via environment variables (or a TOML file for policy
defaults). The binary fails fast with code 1 if any **required** variable is absent.

### Required at startup

| Variable | Type | Used by |
|---|---|---|
| `MERGE_WARDEN_GITHUB_APP_ID` | `u64` | `ServerSecrets.github_app_id` |
| `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY` | PEM string | `ServerSecrets.github_app_private_key` |

### Optional

| Variable | Default | Used by |
|---|---|---|
| `GITHUB_WEBHOOK_SECRET` | none | `ServerSecrets.github_webhook_secret` — required in `webhook` mode for HMAC signature validation; absent in `queue` mode |
| `MERGE_WARDEN_PORT` | `3000` | `ServerConfig.port` |
| `MERGE_WARDEN_RECEIVER_MODE` | `webhook` | `ServerConfig.receiver_mode` |
| `MERGE_WARDEN_CONFIG_FILE` | none | loads policy TOML; not stored on `ServerConfig` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | none | `TelemetryConfig.otlp_endpoint` |
| `OTEL_SERVICE_NAME` | `merge-warden` | `TelemetryConfig.service_name` |
| `OTEL_SERVICE_VERSION` | from `CARGO_PKG_VERSION` | `TelemetryConfig.service_version` |
| `RUST_LOG` | `info` | `tracing_subscriber` filter |

### Required only when `MERGE_WARDEN_RECEIVER_MODE=queue`

| Variable | Default | Used by |
|---|---|---|
| `MERGE_WARDEN_QUEUE_PROVIDER` | none | `QueueServerConfig.provider` |
| `MERGE_WARDEN_QUEUE_NAME` | `merge-warden-events` | `QueueServerConfig.queue_name` |
| `MERGE_WARDEN_QUEUE_CONCURRENCY` | `4` | `QueueServerConfig.concurrency` |
| `AZURE_SERVICEBUS_NAMESPACE` | none | `QueueServerConfig.namespace` (Azure only) |

---

## `SecretString`

A newtype over `String` that prevents the contained value from appearing in `Debug`,
`Display`, or `tracing` structured fields.

```rust
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: String) -> Self;
    /// Returns the inner string slice for use with APIs that require it.
    /// The returned reference does NOT implement Display or Debug.
    pub fn expose(&self) -> &str;
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
```

**Security rule**: `SecretString` must never implement `Serialize`. The `Deref` trait
must not be implemented (caller must explicitly call `.expose()`).

---

## `ServerSecrets`

```rust
/// GitHub App credentials and webhook signing secret loaded from environment variables.
///
/// Loaded once at startup by `load_secrets()`. The struct is passed into `AppState`
/// and into the `GitHubClient` builder (task 1.0).
///
/// # Environment Variables
/// - `MERGE_WARDEN_GITHUB_APP_ID` → `github_app_id`
/// - `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY` → `github_app_private_key`
/// - `GITHUB_WEBHOOK_SECRET` → `github_webhook_secret`
pub struct ServerSecrets {
    pub github_app_id: u64,
    pub github_app_private_key: SecretString,
    pub github_webhook_secret: Option<SecretString>,
}
```

---

## `ReceiverMode`

```rust
/// Controls how the server receives GitHub events.
///
/// Selected once at startup from `MERGE_WARDEN_RECEIVER_MODE`. Cannot change
/// without a process restart.
///
/// - `Webhook`: Axum POST handler processes events via an in-process channel.
/// - `Queue`: Axum POST handler enqueues events; a separate Tokio task processes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiverMode {
    Webhook,
    Queue,
}
```

Parsing rule: the env var value is case-insensitive. Any value other than `"webhook"`
or `"queue"` produces `ServerError::InvalidEnvVar`.

---

## `QueueServerConfig`

```rust
/// Queue provider settings, populated only when `MERGE_WARDEN_RECEIVER_MODE=queue`.
///
/// # Environment Variables
/// - `MERGE_WARDEN_QUEUE_PROVIDER` → `provider` (required in queue mode)
/// - `MERGE_WARDEN_QUEUE_NAME`     → `queue_name` (default: `"merge-warden-events"`)
/// - `MERGE_WARDEN_QUEUE_CONCURRENCY` → `concurrency` (default: `4`)
/// - `AZURE_SERVICEBUS_NAMESPACE`  → `namespace` (required when provider = "azure")
#[derive(Debug, Clone)]
pub struct QueueServerConfig {
    pub provider: String,
    pub queue_name: String,
    pub concurrency: usize,
    pub namespace: Option<String>,
}
```

---

## `ServerConfig`

```rust
/// Full server configuration assembled from environment variables and optional TOML file.
///
/// Loading priority for `application_defaults`:
/// 1. TOML file at `MERGE_WARDEN_CONFIG_FILE` path (if set and present)
/// 2. Individual `MERGE_WARDEN_*` env var overrides
/// 3. `ApplicationDefaults::default()`
///
/// `queue` is `Some(...)` only when `receiver_mode == ReceiverMode::Queue`.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub receiver_mode: ReceiverMode,
    pub application_defaults: merge_warden_core::config::ApplicationDefaults,
    pub queue: Option<QueueServerConfig>,
}
```

---

## `load_secrets()`

```rust
/// Reads the three required GitHub secrets from environment variables.
///
/// # Errors
/// - `ServerError::MissingEnvVar("MERGE_WARDEN_GITHUB_APP_ID")` if the variable is absent.
/// - `ServerError::InvalidEnvVar { name: "MERGE_WARDEN_GITHUB_APP_ID", .. }` if the value
///   cannot be parsed as `u64`.
/// - `ServerError::MissingEnvVar("MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY")` if absent.
///
/// # Guarantees
/// This function performs no network I/O.
pub fn load_secrets() -> Result<ServerSecrets, ServerError>;
```

---

## `load_config()`

```rust
/// Builds `ServerConfig` from environment variables and optional TOML file.
///
/// # Errors
/// - `ServerError::InvalidEnvVar` for `MERGE_WARDEN_PORT` if not parseable as `u16`.
/// - `ServerError::InvalidEnvVar` for `MERGE_WARDEN_RECEIVER_MODE` if not
///   `"webhook"` or `"queue"` (case-insensitive).
/// - `ServerError::MissingEnvVar("MERGE_WARDEN_QUEUE_PROVIDER")` when
///   `receiver_mode == Queue` and the variable is absent.
/// - `ServerError::ConfigError` if the TOML file exists but cannot be parsed.
///
/// # Guarantees
/// - Absent TOML config file is NOT an error; `ApplicationDefaults::default()` is used.
/// - This function performs no network I/O.
pub fn load_config() -> Result<ServerConfig, ServerError>;
```

---

## `ServerError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// A required environment variable was absent.
    #[error("Missing required environment variable '{0}'")]
    MissingEnvVar(String),

    /// An environment variable was present but its value was invalid.
    #[error("Invalid environment variable '{name}': {message}")]
    InvalidEnvVar { name: String, message: String },

    /// TOML configuration file was found but could not be parsed.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// GitHub authentication or client initialisation failed.
    #[error("GitHub authentication error: {0}")]
    AuthError(String),

    /// Telemetry subscriber initialisation failed.
    #[error("Telemetry initialization failed: {0}")]
    TelemetryInitFailed(String),

    /// Propagated from the ingress layer.
    #[error("Ingress error: {source}")]
    IngressError {
        #[from]
        source: crate::ingress::IngressError,
    },
}
```

---

## `TelemetryConfig`

```rust
/// Parameters for the tracing subscriber initialisation.
///
/// Built from environment variables by `TelemetryConfig::from_env()`.
/// An OTLP export layer is added only when `otlp_endpoint` is `Some`.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Value of `OTEL_EXPORTER_OTLP_ENDPOINT`. `None` → console output only.
    pub otlp_endpoint: Option<String>,
    /// Value of `OTEL_SERVICE_NAME`. Default: `"merge-warden"`.
    pub service_name: String,
    /// Value of `OTEL_SERVICE_VERSION`. Default: `env!("CARGO_PKG_VERSION")`.
    pub service_version: String,
}

impl TelemetryConfig {
    /// Reads OTLP and service metadata from standard environment variables.
    /// Never fails — absent variables produce default values.
    pub fn from_env() -> Self;
}
```

---

## `init_telemetry()`

```rust
/// Initialises the global `tracing` subscriber.
///
/// Always installs a console (fmt) layer with `RUST_LOG`-based filtering.
/// When `config.otlp_endpoint` is `Some`, also installs an OTLP gRPC export
/// layer via `opentelemetry-otlp` + `tracing-opentelemetry`.
///
/// Must be called once, before any `tracing::info!()` / `tracing::debug!()`
/// calls. Calling it more than once returns an error (subscriber already set).
///
/// # Errors
/// - `ServerError::TelemetryInitFailed` if the subscriber cannot be installed
///   (e.g., a global subscriber is already set).
/// - `ServerError::TelemetryInitFailed` if the OTLP exporter cannot connect to
///   the configured endpoint (only when `otlp_endpoint` is `Some`).
pub fn init_telemetry(config: &TelemetryConfig) -> Result<(), ServerError>;
```
