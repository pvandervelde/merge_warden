// See .llm/task.md — "OpenTelemetry Metrics (OTLP)" and "Prometheus Endpoint"
// See docs/spec/operations/monitoring.md — queue-mode metric names/thresholds
//
// STATUS: pre-implementation stub (TDD RED phase). Every function/method body
// below is intentionally either a no-op or a hardcoded/incomplete value. None
// of this file's logic should be treated as a reference implementation — see
// `metrics_tests.rs` for the behavioural contract the real implementation
// must satisfy.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};

use crate::errors::ServerError;
use crate::webhook::AppState;

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;

// ---------------------------------------------------------------------------
// MetricsConfig
// ---------------------------------------------------------------------------

/// Parameters controlling OTLP metrics export and the optional Prometheus
/// scrape endpoint.
///
/// Built from environment variables by [`MetricsConfig::from_env`].
///
/// See `.llm/task.md` — Configuration table.
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Value of `OTEL_EXPORTER_OTLP_ENDPOINT` (shared with [`crate::telemetry::TelemetryConfig`]).
    /// `None` disables OTLP metrics export.
    pub otlp_endpoint: Option<String>,
    /// Value of `OTEL_SERVICE_NAME`. Default: `"merge-warden"`.
    pub service_name: String,
    /// Value of `OTEL_SERVICE_VERSION`. Default: binary crate version.
    pub service_version: String,
    /// `true` when `MERGE_WARDEN_METRICS_ENDPOINT == "prometheus"`. Controls
    /// whether `GET /metrics` is registered.
    pub prometheus_enabled: bool,
}

impl MetricsConfig {
    /// Reads OTLP, service-metadata, and Prometheus-endpoint settings from
    /// standard environment variables.
    ///
    /// Never fails — absent variables produce default values.
    ///
    /// # Errors
    /// This function never returns an error.
    pub fn from_env() -> Self {
        MetricsConfig {
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "merge-warden".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
            // STUB: MERGE_WARDEN_METRICS_ENDPOINT is intentionally never read.
            // A correct implementation must set this to `true` iff the
            // variable's value is "prometheus" (case sensitivity TBD — see
            // Tester's report), and `false` for unset/any other value.
            prometheus_enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

/// Handles to every OTel instrument merge-warden emits.
///
/// Cheap to `Clone` — every field is an `Arc`-backed OTel instrument handle.
/// Constructed once at startup via [`Metrics::new`] and threaded through
/// [`AppState`].
///
/// See the metric table in `.llm/task.md` for exact names/types/labels; the
/// names here MUST match `docs/spec/operations/monitoring.md` exactly.
#[derive(Clone)]
pub struct Metrics {
    webhook_requests_total: Counter<u64>,
    webhook_processing_duration_ms: Histogram<f64>,
    queue_enqueue_duration_ms: Histogram<f64>,
    queue_processing_duration_ms: Histogram<f64>,
    queue_depth: Gauge<u64>,
    queue_age_oldest_message_secs: Gauge<f64>,
    queue_dlq_count: Counter<u64>,
    queue_worker_errors_total: Counter<u64>,
    processing_success_rate: Gauge<f64>,
    pr_validation_duration_ms: Histogram<f64>,
    pr_bypass_activations_total: Counter<u64>,
}

impl Metrics {
    /// Builds every instrument handle from `meter`.
    ///
    /// Instrument creation is idempotent per `(name, kind)` pair on the
    /// underlying SDK, so calling this more than once against meters from the
    /// same provider is safe but wasteful — call once at startup.
    pub fn new(meter: &Meter) -> Self {
        Metrics {
            webhook_requests_total: meter.u64_counter("ingress.webhook.requests_total").build(),
            webhook_processing_duration_ms: meter
                .f64_histogram("ingress.webhook.processing_duration_ms")
                .build(),
            queue_enqueue_duration_ms: meter
                .f64_histogram("ingress.queue.enqueue_duration_ms")
                .build(),
            queue_processing_duration_ms: meter
                .f64_histogram("ingress.queue.processing_duration_ms")
                .build(),
            queue_depth: meter.u64_gauge("ingress.queue.depth").build(),
            queue_age_oldest_message_secs: meter
                .f64_gauge("ingress.queue.age_oldest_message_secs")
                .build(),
            queue_dlq_count: meter.u64_counter("ingress.queue.dlq_count").build(),
            // NOTE: task Interface Contract names this `worker_errors_total`;
            // docs/spec/operations/monitoring.md's table uses
            // `ingress.queue.worker_errors` (no `_total` suffix). Following
            // the Interface Contract here — see Tester's report for the
            // discrepancy flagged for architect clarification.
            queue_worker_errors_total: meter
                .u64_counter("ingress.queue.worker_errors_total")
                .build(),
            processing_success_rate: meter.f64_gauge("processing.success_rate").build(),
            pr_validation_duration_ms: meter.f64_histogram("pr.validation.duration_ms").build(),
            pr_bypass_activations_total: meter.u64_counter("pr.bypass.activations_total").build(),
        }
    }

    /// Records one webhook request. `result` MUST be `"accepted"` or `"rejected"`.
    ///
    /// # Panics
    /// Never panics.
    pub fn record_webhook_request(&self, _event_type: &str, _result: &str) {
        // STUB: intentionally not recorded. A real implementation calls
        // `self.webhook_requests_total.add(1, &[KeyValue::new("event_type",
        // ..), KeyValue::new("result", ..)])`.
    }

    /// Records end-to-end webhook processing latency in milliseconds.
    pub fn record_webhook_processing_duration_ms(&self, _event_type: &str, _duration_ms: f64) {
        // STUB: intentionally not recorded.
    }

    /// Records webhook-receipt-to-enqueue latency in milliseconds (queue mode only).
    pub fn record_queue_enqueue_duration_ms(&self, _duration_ms: f64) {
        // STUB: intentionally not recorded.
    }

    /// Records end-to-end queue event processing latency in milliseconds.
    pub fn record_queue_processing_duration_ms(&self, _duration_ms: f64) {
        // STUB: intentionally not recorded.
    }

    /// Sets the current approximate queue depth.
    pub fn set_queue_depth(&self, _depth: u64) {
        // STUB: intentionally not recorded.
    }

    /// Sets the age, in seconds, of the oldest unprocessed message.
    pub fn set_queue_age_oldest_message_secs(&self, _secs: f64) {
        // STUB: intentionally not recorded.
    }

    /// Increments the dead-letter-queue counter by one.
    pub fn record_dlq_message(&self) {
        // STUB: intentionally not recorded.
    }

    /// Increments the worker-task-terminated-with-error counter by one.
    pub fn record_worker_error(&self) {
        // STUB: intentionally not recorded.
    }

    /// Sets the current processing success rate, in the range `0.0..=1.0`.
    pub fn set_processing_success_rate(&self, _rate: f64) {
        // STUB: intentionally not recorded.
    }

    /// Records the time taken to run all validation rules for a single PR.
    pub fn record_pr_validation_duration_ms(&self, _duration_ms: f64) {
        // STUB: intentionally not recorded.
    }

    /// Increments the bypass-activation counter for `bypass_type` by one.
    pub fn record_bypass_activation(&self, _bypass_type: &str) {
        // STUB: intentionally not recorded.
    }
}

// ---------------------------------------------------------------------------
// init_metrics
// ---------------------------------------------------------------------------

/// Initialises the OTel [`opentelemetry_sdk::metrics::SdkMeterProvider`].
///
/// When `config.otlp_endpoint` is `Some`, an OTLP HTTP push exporter should be
/// attached (mirroring [`crate::telemetry::init_telemetry`]'s trace pipeline).
/// When `config.prometheus_enabled` is `true`, a pull-based Prometheus reader
/// should be attached so [`metrics_handler`] can render collected data.
///
/// Must be called at most once; the returned provider should be stored and
/// used to build the [`Metrics`] struct's [`Meter`].
///
/// # Errors
/// - [`ServerError::MetricsInitFailed`] if the OTLP exporter cannot be built.
pub fn init_metrics(
    _config: &MetricsConfig,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, ServerError> {
    // STUB: always returns a provider with no readers attached, regardless of
    // `otlp_endpoint` / `prometheus_enabled`. No metrics will ever be
    // exported or observable via `metrics_handler` against this stub.
    Ok(opentelemetry_sdk::metrics::SdkMeterProvider::builder().build())
}

// ---------------------------------------------------------------------------
// metrics_handler
// ---------------------------------------------------------------------------

/// `GET /metrics` — Prometheus text exposition format.
///
/// Only registered on the router when `state.metrics_config.prometheus_enabled`
/// is `true` (see `crate::webhook::build_router` / `build_queue_router`).
///
/// # Responses
/// - `200 OK` with `Content-Type: text/plain` body containing `# HELP` / `# TYPE`
///   lines and one sample line per recorded label combination.
pub async fn metrics_handler(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // STUB: always returns an empty 200 body, completely ignoring any
    // metrics recorded through `AppState.metrics`.
    (StatusCode::OK, String::new())
}
