// See .llm/task.md — "OpenTelemetry Metrics (OTLP)" and "Prometheus Endpoint"
// See docs/spec/operations/monitoring.md — queue-mode metric names/thresholds

use std::sync::{Arc, OnceLock};

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use opentelemetry::{
    metrics::{Counter, Gauge, Histogram, Meter, MeterProvider as _},
    KeyValue,
};
use prometheus::Encoder as _;

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
        let (otlp_endpoint, service_name, service_version) =
            crate::telemetry::otel_service_metadata_from_env();

        let raw_endpoint_var = std::env::var("MERGE_WARDEN_METRICS_ENDPOINT").ok();
        if let Some(value) = raw_endpoint_var.as_deref() {
            if value != "prometheus" {
                tracing::warn!(
                    value,
                    "Unrecognized MERGE_WARDEN_METRICS_ENDPOINT value; the only supported value \
                     is \"prometheus\" — GET /metrics will not be registered"
                );
            }
        }

        MetricsConfig {
            otlp_endpoint,
            service_name,
            service_version,
            prometheus_enabled: raw_endpoint_var.as_deref() == Some("prometheus"),
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
    // These three instruments have no call site in this binary today (see the
    // doc comments on their `record_*`/`set_*` methods below for why) but are
    // kept as documented public API for future/out-of-process callers.
    #[allow(dead_code)]
    queue_enqueue_duration_ms: Histogram<f64>,
    queue_processing_duration_ms: Histogram<f64>,
    #[allow(dead_code)]
    queue_depth: Gauge<u64>,
    #[allow(dead_code)]
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
            webhook_requests_total: meter
                .u64_counter("ingress.webhook.requests_total")
                .with_description(
                    "Total webhook requests received, labelled by event_type and result (accepted/rejected)",
                )
                .build(),
            webhook_processing_duration_ms: meter
                .f64_histogram("ingress.webhook.processing_duration_ms")
                .with_description("End-to-end webhook processing latency, in milliseconds")
                .build(),
            queue_enqueue_duration_ms: meter
                .f64_histogram("ingress.queue.enqueue_duration_ms")
                .with_description(
                    "Webhook receipt to message enqueued latency, in milliseconds (queue mode only)",
                )
                .build(),
            queue_processing_duration_ms: meter
                .f64_histogram("ingress.queue.processing_duration_ms")
                .with_description(
                    "End-to-end queue event processing latency, in milliseconds (queue mode only)",
                )
                .build(),
            queue_depth: meter
                .u64_gauge("ingress.queue.depth")
                .with_description("Approximate messages waiting in the queue (queue mode only)")
                .build(),
            queue_age_oldest_message_secs: meter
                .f64_gauge("ingress.queue.age_oldest_message_secs")
                .with_description(
                    "Age, in seconds, of the oldest unprocessed message (queue mode only)",
                )
                .build(),
            queue_dlq_count: meter
                .u64_counter("ingress.queue.dlq_count")
                .with_description("Messages dead-lettered (queue mode only)")
                .build(),
            // NOTE: monitoring.md's queue-mode metric table has been updated to
            // match this name exactly (`ingress.queue.worker_errors_total`) —
            // see docs/spec/operations/monitoring.md.
            queue_worker_errors_total: meter
                .u64_counter("ingress.queue.worker_errors_total")
                .with_description("Unrecoverable worker task terminations (queue mode only)")
                .build(),
            processing_success_rate: meter
                .f64_gauge("processing.success_rate")
                .with_description(
                    "Rolling fraction, over the most recent processed events, of events processed without rejection",
                )
                .build(),
            pr_validation_duration_ms: meter
                .f64_histogram("pr.validation.duration_ms")
                .with_description("Time taken to run all validation rules for a single PR, in milliseconds")
                .build(),
            pr_bypass_activations_total: meter
                .u64_counter("pr.bypass.activations_total")
                .with_description("Bypass rule activations, labelled by bypass_type")
                .build(),
        }
    }

    /// Records one webhook request. `result` MUST be `"accepted"` or `"rejected"`.
    ///
    /// # Panics
    /// Never panics.
    pub fn record_webhook_request(&self, event_type: &str, result: &str) {
        self.webhook_requests_total.add(
            1,
            &[
                KeyValue::new("event_type", event_type.to_string()),
                KeyValue::new("result", result.to_string()),
            ],
        );
    }

    /// Records end-to-end webhook processing latency in milliseconds.
    pub fn record_webhook_processing_duration_ms(&self, event_type: &str, duration_ms: f64) {
        self.webhook_processing_duration_ms.record(
            duration_ms,
            &[KeyValue::new("event_type", event_type.to_string())],
        );
    }

    /// Records webhook-receipt-to-enqueue latency in milliseconds (queue mode only).
    ///
    /// Not called anywhere in this binary: in queue mode, merge-warden is a
    /// pure queue *consumer* — a separate service (not part of this
    /// workspace) receives GitHub webhooks, validates signatures, and
    /// enqueues messages, so it alone observes "receipt-to-enqueue" latency.
    /// This method is kept as public API for that service (or a future
    /// in-repo enqueue path) to call.
    #[allow(dead_code)]
    pub fn record_queue_enqueue_duration_ms(&self, duration_ms: f64) {
        self.queue_enqueue_duration_ms.record(duration_ms, &[]);
    }

    /// Records end-to-end queue event processing latency in milliseconds.
    pub fn record_queue_processing_duration_ms(&self, duration_ms: f64) {
        self.queue_processing_duration_ms.record(duration_ms, &[]);
    }

    /// Sets the current approximate queue depth.
    ///
    /// Not called anywhere in this binary: `queue-runtime` 0.2.1's
    /// `QueueClient`/`SessionClient` traits expose no depth-query method, so
    /// there is currently no data source for this gauge in-process. Kept as
    /// public API for a future `queue-runtime` version (or a
    /// provider-specific side channel, e.g. polling the Azure Service Bus
    /// management API) to call.
    #[allow(dead_code)]
    pub fn set_queue_depth(&self, depth: u64) {
        self.queue_depth.record(depth, &[]);
    }

    /// Sets the age, in seconds, of the oldest unprocessed message.
    ///
    /// Not called anywhere in this binary, for the same reason as
    /// [`Self::set_queue_depth`] — no depth/age-query API is available on the
    /// current `queue-runtime` client traits.
    #[allow(dead_code)]
    pub fn set_queue_age_oldest_message_secs(&self, secs: f64) {
        self.queue_age_oldest_message_secs.record(secs, &[]);
    }

    /// Increments the dead-letter-queue counter by one.
    pub fn record_dlq_message(&self) {
        self.queue_dlq_count.add(1, &[]);
    }

    /// Increments the worker-task-terminated-with-error counter by one.
    pub fn record_worker_error(&self) {
        self.queue_worker_errors_total.add(1, &[]);
    }

    /// Sets the current processing success rate, in the range `0.0..=1.0`.
    pub fn set_processing_success_rate(&self, rate: f64) {
        self.processing_success_rate.record(rate, &[]);
    }

    /// Records the time taken to run all validation rules for a single PR.
    pub fn record_pr_validation_duration_ms(&self, duration_ms: f64) {
        self.pr_validation_duration_ms.record(duration_ms, &[]);
    }

    /// Increments the bypass-activation counter for `bypass_type` by one.
    pub fn record_bypass_activation(&self, bypass_type: &str) {
        self.pr_bypass_activations_total
            .add(1, &[KeyValue::new("bypass_type", bypass_type.to_string())]);
    }
}

// ---------------------------------------------------------------------------
// CoreMetricsRecorder — bridges merge_warden_core::MetricsRecorder to Metrics
// ---------------------------------------------------------------------------

/// Implements `merge_warden_core::MetricsRecorder` on top of [`Metrics`], so
/// `crates/core`'s `MergeWarden` can report `pr.validation.duration_ms` and
/// `pr.bypass.activations_total` without that crate depending on
/// OpenTelemetry directly (see `.llm/task.md` gap #7 — trait injection,
/// chosen over `tracing_opentelemetry::MetricsLayer` because the field-name
/// convention `MetricsLayer` expects could not be verified against the
/// pinned `tracing-opentelemetry = "0.33.0"` without a higher-risk,
/// harder-to-test integration).
#[derive(Clone)]
pub struct CoreMetricsRecorder {
    metrics: Metrics,
}

impl CoreMetricsRecorder {
    /// Wraps `metrics` for injection into `merge_warden_core::MergeWarden::with_metrics_recorder`.
    pub fn new(metrics: Metrics) -> Self {
        CoreMetricsRecorder { metrics }
    }
}

impl std::fmt::Debug for CoreMetricsRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreMetricsRecorder").finish()
    }
}

impl merge_warden_core::MetricsRecorder for CoreMetricsRecorder {
    fn record_validation_duration_ms(&self, duration_ms: f64) {
        self.metrics.record_pr_validation_duration_ms(duration_ms);
    }

    fn record_bypass_activation(&self, bypass_type: &str) {
        self.metrics.record_bypass_activation(bypass_type);
    }
}

impl Default for Metrics {
    /// Builds a `Metrics` instance backed by a private, reader-less
    /// `SdkMeterProvider`. All instrument handles are fully functional (they
    /// accept `record`/`add` calls without panicking) but nothing is ever
    /// exported or observable — a safe, inert sink.
    ///
    /// Used as the default for callers that construct their processing
    /// pipeline before a real `Metrics` instance (built from `init_metrics`'s
    /// provider) is available, e.g. [`crate::webhook::MergeWardenWebhookHandler::new`].
    fn default() -> Self {
        let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder().build();
        let meter = provider.meter("merge-warden-default");
        Metrics::new(&meter)
    }
}

// ---------------------------------------------------------------------------
// Prometheus registry (module-private)
// ---------------------------------------------------------------------------

/// Holds the `prometheus::Registry` the OTel Prometheus exporter writes into,
/// when `MetricsConfig::prometheus_enabled` is `true`. Populated once by
/// [`init_metrics`] and read by [`metrics_handler`].
///
/// A dedicated module-level registry (rather than `prometheus::default_registry()`)
/// is used so this module's metrics export is fully self-contained and does not
/// interact with any other crate that might also use the `prometheus` crate's
/// process-wide default registry (e.g. picking up an unwanted default process
/// collector).
static PROMETHEUS_REGISTRY: OnceLock<prometheus::Registry> = OnceLock::new();

/// Builds the pull-based Prometheus [`opentelemetry_sdk::metrics::reader::MetricReader`]
/// backing `registry`, used by both [`init_metrics`] and (to exercise the
/// exact same builder chain, including the `_total`-suffix fix below)
/// `metrics_tests.rs`'s Prometheus-rendering tests.
///
/// # Errors
/// [`ServerError::MetricsInitFailed`] if the exporter cannot be built.
fn build_prometheus_exporter(
    registry: prometheus::Registry,
) -> Result<impl opentelemetry_sdk::metrics::reader::MetricReader, ServerError> {
    opentelemetry_prometheus::exporter()
        .with_registry(registry)
        // `opentelemetry_prometheus` unconditionally appends a `_total`
        // suffix to every monotonic-sum (counter) instrument name unless
        // this is set — it does not check whether the name already ends
        // in `_total`. Four of our counters
        // (`ingress.webhook.requests_total`, `pr.bypass.activations_total`,
        // `ingress.queue.worker_errors_total`, and, after dot-sanitization,
        // any future counter named the same way) already end in `_total`
        // per the metric table in `.llm/task.md` /
        // `docs/spec/operations/monitoring.md`, so without this option the
        // Prometheus endpoint would expose them as `..._total_total`,
        // silently diverging from the documented metric name that
        // dashboards/alerts are written against. Disabling the built-in
        // suffix keeps the Prometheus name identical to the OTLP name
        // (dots replaced with underscores), matching what operators and
        // `docs/spec/operations/monitoring.md` expect.
        .without_counter_suffixes()
        .build()
        .map_err(|e| ServerError::MetricsInitFailed(format!("Prometheus exporter: {e}")))
}

// ---------------------------------------------------------------------------
// init_metrics
// ---------------------------------------------------------------------------

/// Initialises the OTel [`opentelemetry_sdk::metrics::SdkMeterProvider`].
///
/// When `config.otlp_endpoint` is `Some`, an OTLP HTTP push exporter is
/// attached (mirroring [`crate::telemetry::init_telemetry`]'s trace pipeline),
/// wrapped in a [`opentelemetry_sdk::metrics::PeriodicReader`] that exports on
/// the default interval (60s, or `OTEL_METRIC_EXPORT_INTERVAL` if set).
/// When `config.prometheus_enabled` is `true`, a pull-based Prometheus reader
/// is attached so [`metrics_handler`] can render collected data.
///
/// Must be called at most once; the returned provider should be stored and
/// used to build the [`Metrics`] struct's [`Meter`].
///
/// # Errors
/// - [`ServerError::MetricsInitFailed`] if the OTLP exporter cannot be built.
/// - [`ServerError::MetricsInitFailed`] if the Prometheus exporter cannot be built.
pub fn init_metrics(
    config: &MetricsConfig,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, ServerError> {
    let resource =
        crate::telemetry::build_otel_resource(&config.service_name, &config.service_version);

    let mut builder =
        opentelemetry_sdk::metrics::SdkMeterProvider::builder().with_resource(resource);

    if let Some(endpoint) = &config.otlp_endpoint {
        use opentelemetry_otlp::WithExportConfig;

        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| ServerError::MetricsInitFailed(format!("OTLP metric exporter: {e}")))?;

        let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter).build();
        builder = builder.with_reader(reader);
    }

    if config.prometheus_enabled {
        let registry = prometheus::Registry::new();
        let exporter = build_prometheus_exporter(registry.clone())?;

        // Best-effort: if `init_metrics` is somehow called more than once with
        // Prometheus enabled (never happens in `main()`, which calls it
        // exactly once), the first registry wins and later calls' exporters
        // are simply not observable via `metrics_handler`. This cannot
        // happen in production and is documented rather than treated as an error.
        let _ = PROMETHEUS_REGISTRY.set(registry);

        builder = builder.with_reader(exporter);
    }

    Ok(builder.build())
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
/// - `200 OK` with an empty body if no Prometheus registry was initialised
///   (i.e. `init_metrics` was never called with `prometheus_enabled: true`).
// `State<Arc<AppState>>` is required so this handler's signature matches what
// `axum::routing::get` expects on a router built `.with_state(state)` (see
// `crate::webhook::with_metrics_route`) — the handler itself reads all metric
// data from the process-wide `PROMETHEUS_REGISTRY` above, not from `AppState`,
// so the extracted value is intentionally unused.
pub async fn metrics_handler(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let metric_families = match PROMETHEUS_REGISTRY.get() {
        Some(registry) => registry.gather(),
        None => Vec::new(),
    };

    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!(error = %e, "Failed to encode Prometheus metrics");
        return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
    }

    let body = String::from_utf8_lossy(&buffer).into_owned();
    (StatusCode::OK, body)
}
