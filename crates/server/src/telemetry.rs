// See docs/spec/interfaces/server-config.md — TelemetryConfig / init_telemetry()

use crate::errors::ServerError;

/// Parameters for the `tracing` subscriber initialisation.
///
/// Built from environment variables by [`TelemetryConfig::from_env`]. An OTLP
/// export layer is installed only when `otlp_endpoint` is `Some`.
///
/// See docs/spec/interfaces/server-config.md — `TelemetryConfig`
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Value of `OTEL_EXPORTER_OTLP_ENDPOINT`. `None` → console output only.
    pub otlp_endpoint: Option<String>,
    /// Value of `OTEL_SERVICE_NAME`. Default: `"merge-warden"`.
    pub service_name: String,
    /// Value of `OTEL_SERVICE_VERSION`. Default: binary crate version.
    pub service_version: String,
}

impl TelemetryConfig {
    /// Reads OTLP and service-metadata settings from standard environment variables.
    ///
    /// Never fails — absent variables produce default values.
    ///
    /// See docs/spec/interfaces/server-config.md — `TelemetryConfig::from_env()`
    pub fn from_env() -> Self {
        todo!("See docs/spec/interfaces/server-config.md — TelemetryConfig::from_env()")
    }
}

/// Initialises the global `tracing` subscriber.
///
/// Always installs a `fmt` (console) layer filtered by `RUST_LOG`. When
/// `config.otlp_endpoint` is `Some`, also installs an OTLP gRPC export layer
/// via `opentelemetry-otlp` and `tracing-opentelemetry`.
///
/// Must be called exactly once before any `tracing::*` macro invocations.
///
/// See docs/spec/interfaces/server-config.md — `init_telemetry()`
///
/// # Errors
/// - [`ServerError::TelemetryInitFailed`] if a global subscriber is already set.
/// - [`ServerError::TelemetryInitFailed`] if the OTLP exporter cannot be built.
pub fn init_telemetry(config: &TelemetryConfig) -> Result<(), ServerError> {
    todo!("See docs/spec/interfaces/server-config.md — init_telemetry()")
}
