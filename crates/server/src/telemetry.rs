// See docs/spec/interfaces/server-config.md — TelemetryConfig / init_telemetry()

use crate::errors::ServerError;

#[cfg(test)]
#[path = "telemetry_tests.rs"]
mod tests;

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
        TelemetryConfig {
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "merge-warden".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
        }
    }
}

/// Initialises the global `tracing` subscriber.
///
/// Always installs a `fmt` (console) layer filtered by `RUST_LOG`. When
/// `config.otlp_endpoint` is `Some`, also installs an OTLP **HTTP** export layer
/// via `opentelemetry-otlp` and `tracing-opentelemetry`.
///
/// The endpoint should be the full OTLP HTTP collector URL including the
/// `/v1/traces` path suffix if required by the collector (e.g.
/// `http://localhost:4318/v1/traces` for an OTEL Collector, or just
/// `http://localhost:4318` for collectors that auto-append the path).
/// gRPC transport is not used; the default port is 4318 (HTTP), not 4317 (gRPC).
///
/// Must be called exactly once before any `tracing::*` macro invocations.
///
/// See docs/spec/interfaces/server-config.md — `init_telemetry()`
///
/// # Errors
/// - [`ServerError::TelemetryInitFailed`] if a global subscriber is already set.
/// - [`ServerError::TelemetryInitFailed`] if the OTLP exporter cannot be built.
pub fn init_telemetry(config: &TelemetryConfig) -> Result<(), ServerError> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .compact();

    if let Some(endpoint) = &config.otlp_endpoint {
        use opentelemetry_otlp::WithExportConfig;

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| ServerError::TelemetryInitFailed(format!("OTLP exporter: {}", e)))?;

        let resource = opentelemetry_sdk::Resource::builder()
            .with_service_name(config.service_name.clone())
            .with_attributes(vec![opentelemetry::KeyValue::new(
                "service.version",
                config.service_version.clone(),
            )])
            .build();

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter)
            .build();

        opentelemetry::global::set_tracer_provider(provider.clone());

        let tracer = {
            use opentelemetry::trace::TracerProvider as _;
            provider.tracer(config.service_name.clone())
        };
        let otlp_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(env_filter)
            .with(otlp_layer)
            .try_init()
            .map_err(|e| ServerError::TelemetryInitFailed(e.to_string()))?;
    } else {
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(env_filter)
            .try_init()
            .map_err(|e| ServerError::TelemetryInitFailed(e.to_string()))?;
    }

    Ok(())
}
