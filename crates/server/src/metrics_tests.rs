use std::{
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{
        data::{AggregatedMetrics, Metric, MetricData, ResourceMetrics},
        reader::MetricReader,
        InstrumentKind, ManualReader, SdkMeterProvider, Temporality,
    },
};

use super::*;

// ---------------------------------------------------------------------------
// Test infrastructure: an in-process, synchronously-collectible metric reader.
//
// `ManualReader` is consumed by value when passed to `.with_reader(...)`, so
// we wrap it in `Arc` and keep a `Clone`-able handle (`SharedReader`) that
// still implements `MetricReader` by delegating — the exact pattern used by
// opentelemetry_sdk's own benches (see `opentelemetry_sdk-0.32.1/benches/metric.rs`).
// This lets tests call `.collect()` on the *same* reader instance the
// provider was built with, without needing an OTLP collector, a Prometheus
// registry, or any I/O.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SharedReader(Arc<ManualReader>);

impl MetricReader for SharedReader {
    fn register_pipeline(&self, pipeline: Weak<opentelemetry_sdk::metrics::Pipeline>) {
        self.0.register_pipeline(pipeline)
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult {
        self.0.collect(rm)
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.0.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}

/// Builds a fresh, isolated `Metrics` instance backed by an in-process
/// `ManualReader`. Every test gets its own `SdkMeterProvider`, so there is no
/// cross-test interference (unlike a shared global meter provider).
/// Returns `(reader, provider, metrics)`. The caller MUST keep `provider`
/// alive (even if unused) for as long as `reader`/`metrics` are used —
/// dropping the last `SdkMeterProvider` handle shuts down its pipeline,
/// which makes subsequent `reader.collect()` calls fail with
/// `InternalFailure("reader is shut down or not registered")`.
fn new_test_metrics() -> (SharedReader, SdkMeterProvider, Metrics) {
    let reader = SharedReader(Arc::new(ManualReader::builder().build()));
    let provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let meter = provider.meter("merge-warden-test");
    let metrics = Metrics::new(&meter);
    (reader, provider, metrics)
}

fn collect(reader: &SharedReader) -> ResourceMetrics {
    let mut rm = ResourceMetrics::default();
    reader.collect(&mut rm).expect("collect must not fail");
    rm
}

fn find_metric<'a>(rm: &'a ResourceMetrics, name: &str) -> Option<&'a Metric> {
    rm.scope_metrics()
        .flat_map(|sm| sm.metrics())
        .find(|m| m.name() == name)
}

fn attrs_match<'a>(dp_attrs: impl Iterator<Item = &'a KeyValue>, want: &[(&str, &str)]) -> bool {
    let dp_attrs: Vec<&KeyValue> = dp_attrs.collect();
    if dp_attrs.len() != want.len() {
        return false;
    }
    want.iter().all(|(k, v)| {
        dp_attrs
            .iter()
            .any(|kv| kv.key.as_str() == *k && kv.value.as_str() == *v)
    })
}

/// Returns the `u64` value of the sum data point matching `want_attrs`, and
/// whether the sum is monotonic (true for counters).
fn u64_sum_point(
    rm: &ResourceMetrics,
    name: &str,
    want_attrs: &[(&str, &str)],
) -> Option<(u64, bool)> {
    let metric = find_metric(rm, name)?;
    match metric.data() {
        AggregatedMetrics::U64(MetricData::Sum(sum)) => {
            let is_monotonic = sum.is_monotonic();
            sum.data_points()
                .find(|dp| attrs_match(dp.attributes(), want_attrs))
                .map(|dp| (dp.value(), is_monotonic))
        }
        _ => None,
    }
}

fn u64_gauge_point(rm: &ResourceMetrics, name: &str, want_attrs: &[(&str, &str)]) -> Option<u64> {
    let metric = find_metric(rm, name)?;
    match metric.data() {
        AggregatedMetrics::U64(MetricData::Gauge(gauge)) => gauge
            .data_points()
            .find(|dp| attrs_match(dp.attributes(), want_attrs))
            .map(|dp| dp.value()),
        _ => None,
    }
}

fn f64_gauge_point(rm: &ResourceMetrics, name: &str, want_attrs: &[(&str, &str)]) -> Option<f64> {
    let metric = find_metric(rm, name)?;
    match metric.data() {
        AggregatedMetrics::F64(MetricData::Gauge(gauge)) => gauge
            .data_points()
            .find(|dp| attrs_match(dp.attributes(), want_attrs))
            .map(|dp| dp.value()),
        _ => None,
    }
}

/// Returns `(count, sum)` of the histogram data point matching `want_attrs`.
fn f64_histogram_point(
    rm: &ResourceMetrics,
    name: &str,
    want_attrs: &[(&str, &str)],
) -> Option<(u64, f64)> {
    let metric = find_metric(rm, name)?;
    match metric.data() {
        AggregatedMetrics::F64(MetricData::Histogram(hist)) => hist
            .data_points()
            .find(|dp| attrs_match(dp.attributes(), want_attrs))
            .map(|dp| (dp.count(), dp.sum())),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// MetricsConfig::from_env
// ---------------------------------------------------------------------------

static METRICS_ENV_MUTEX: Mutex<()> = Mutex::new(());

fn clear_metrics_env() {
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::remove_var("OTEL_SERVICE_NAME");
    std::env::remove_var("OTEL_SERVICE_VERSION");
    std::env::remove_var("MERGE_WARDEN_METRICS_ENDPOINT");
}

#[test]
fn from_env_otlp_endpoint_is_none_when_var_absent() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();

    let cfg = MetricsConfig::from_env();

    assert!(cfg.otlp_endpoint.is_none());
}

#[test]
fn from_env_reads_otlp_endpoint_when_set() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318");

    let cfg = MetricsConfig::from_env();

    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    assert_eq!(cfg.otlp_endpoint.as_deref(), Some("http://localhost:4318"));
}

#[test]
fn from_env_uses_default_service_name_when_absent() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();

    let cfg = MetricsConfig::from_env();

    assert_eq!(cfg.service_name, "merge-warden");
}

#[test]
fn from_env_reads_custom_service_name() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();
    std::env::set_var("OTEL_SERVICE_NAME", "custom-svc");

    let cfg = MetricsConfig::from_env();

    std::env::remove_var("OTEL_SERVICE_NAME");
    assert_eq!(cfg.service_name, "custom-svc");
}

#[test]
fn from_env_prometheus_disabled_when_var_absent() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();

    let cfg = MetricsConfig::from_env();

    assert!(
        !cfg.prometheus_enabled,
        "prometheus_enabled must default to false when MERGE_WARDEN_METRICS_ENDPOINT is unset"
    );
}

#[test]
fn from_env_prometheus_enabled_when_var_is_prometheus() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();
    std::env::set_var("MERGE_WARDEN_METRICS_ENDPOINT", "prometheus");

    let cfg = MetricsConfig::from_env();

    std::env::remove_var("MERGE_WARDEN_METRICS_ENDPOINT");
    assert!(
        cfg.prometheus_enabled,
        "MERGE_WARDEN_METRICS_ENDPOINT=prometheus must enable the Prometheus endpoint"
    );
}

#[test]
fn from_env_prometheus_disabled_when_var_is_unrecognised_value() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();
    std::env::set_var("MERGE_WARDEN_METRICS_ENDPOINT", "otlp");

    let cfg = MetricsConfig::from_env();

    std::env::remove_var("MERGE_WARDEN_METRICS_ENDPOINT");
    assert!(
        !cfg.prometheus_enabled,
        "an unrecognised MERGE_WARDEN_METRICS_ENDPOINT value must not enable the Prometheus endpoint"
    );
}

#[test]
fn from_env_prometheus_disabled_when_var_is_empty_string() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();
    std::env::set_var("MERGE_WARDEN_METRICS_ENDPOINT", "");

    let cfg = MetricsConfig::from_env();

    std::env::remove_var("MERGE_WARDEN_METRICS_ENDPOINT");
    assert!(!cfg.prometheus_enabled);
}

#[test]
fn from_env_never_panics() {
    let _lock = METRICS_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    clear_metrics_env();
    let _cfg = MetricsConfig::from_env();
}

// ---------------------------------------------------------------------------
// Metrics — instrument identity: every documented metric name must exist
// under exactly the name in docs/spec/operations/monitoring.md / the task's
// Interface Contract metric table. This single test would fail against any
// typo'd, renamed, or missing instrument.
// ---------------------------------------------------------------------------

#[test]
fn all_eleven_documented_metric_names_are_emitted() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_webhook_request("pull_request", "accepted");
    metrics.record_webhook_processing_duration_ms("pull_request", 12.0);
    metrics.record_queue_enqueue_duration_ms(5.0);
    metrics.record_queue_processing_duration_ms(20.0);
    metrics.set_queue_depth(1);
    metrics.set_queue_age_oldest_message_secs(1.0);
    metrics.record_dlq_message();
    metrics.record_worker_error();
    metrics.set_processing_success_rate(1.0);
    metrics.record_pr_validation_duration_ms(3.0);
    metrics.record_bypass_activation("admin_override");

    let rm = collect(&reader);
    let names: Vec<&str> = rm
        .scope_metrics()
        .flat_map(|sm| sm.metrics())
        .map(|m| m.name())
        .collect();

    let expected = [
        "ingress.webhook.requests_total",
        "ingress.webhook.processing_duration_ms",
        "ingress.queue.enqueue_duration_ms",
        "ingress.queue.processing_duration_ms",
        "ingress.queue.depth",
        "ingress.queue.age_oldest_message_secs",
        "ingress.queue.dlq_count",
        "ingress.queue.worker_errors_total",
        "processing.success_rate",
        "pr.validation.duration_ms",
        "pr.bypass.activations_total",
    ];

    for name in expected {
        assert!(
            names.contains(&name),
            "expected metric '{name}' to be emitted, got names: {names:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// record_webhook_request — Counter<u64>, labels: event_type, result
// ---------------------------------------------------------------------------

#[test]
fn record_webhook_request_increments_counter_with_labels() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_webhook_request("pull_request", "accepted");

    let rm = collect(&reader);
    let (value, is_monotonic) = u64_sum_point(
        &rm,
        "ingress.webhook.requests_total",
        &[("event_type", "pull_request"), ("result", "accepted")],
    )
    .expect("expected a data point for event_type=pull_request,result=accepted");

    assert_eq!(value, 1);
    assert!(is_monotonic, "requests_total must be a monotonic counter");
}

#[test]
fn record_webhook_request_distinguishes_accepted_from_rejected() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_webhook_request("pull_request", "accepted");
    metrics.record_webhook_request("pull_request", "rejected");
    metrics.record_webhook_request("pull_request", "rejected");

    let rm = collect(&reader);
    let accepted = u64_sum_point(
        &rm,
        "ingress.webhook.requests_total",
        &[("event_type", "pull_request"), ("result", "accepted")],
    )
    .expect("accepted series must exist");
    let rejected = u64_sum_point(
        &rm,
        "ingress.webhook.requests_total",
        &[("event_type", "pull_request"), ("result", "rejected")],
    )
    .expect("rejected series must exist");

    assert_eq!(
        accepted.0, 1,
        "accepted count must not be conflated with rejected"
    );
    assert_eq!(
        rejected.0, 2,
        "rejected count must accumulate independently"
    );
}

#[test]
fn record_webhook_request_distinguishes_event_types() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_webhook_request("pull_request", "accepted");
    metrics.record_webhook_request("status", "accepted");

    let rm = collect(&reader);
    let pr = u64_sum_point(
        &rm,
        "ingress.webhook.requests_total",
        &[("event_type", "pull_request"), ("result", "accepted")],
    )
    .expect("pull_request series must exist");
    let status = u64_sum_point(
        &rm,
        "ingress.webhook.requests_total",
        &[("event_type", "status"), ("result", "accepted")],
    )
    .expect("status series must exist");

    assert_eq!(pr.0, 1);
    assert_eq!(status.0, 1);
}

// ---------------------------------------------------------------------------
// record_webhook_processing_duration_ms — Histogram<f64>, label: event_type
// ---------------------------------------------------------------------------

#[test]
fn record_webhook_processing_duration_ms_records_value_with_event_type_label() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_webhook_processing_duration_ms("pull_request", 42.0);

    let rm = collect(&reader);
    let (count, sum) = f64_histogram_point(
        &rm,
        "ingress.webhook.processing_duration_ms",
        &[("event_type", "pull_request")],
    )
    .expect("expected a histogram data point for event_type=pull_request");

    assert_eq!(count, 1);
    assert!((sum - 42.0).abs() < f64::EPSILON);
}

#[test]
fn record_webhook_processing_duration_ms_accumulates_multiple_observations() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_webhook_processing_duration_ms("pull_request", 10.0);
    metrics.record_webhook_processing_duration_ms("pull_request", 20.0);

    let rm = collect(&reader);
    let (count, sum) = f64_histogram_point(
        &rm,
        "ingress.webhook.processing_duration_ms",
        &[("event_type", "pull_request")],
    )
    .expect("expected a histogram data point");

    assert_eq!(count, 2);
    assert!((sum - 30.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// record_queue_enqueue_duration_ms — Histogram<f64>, no labels
// ---------------------------------------------------------------------------

#[test]
fn record_queue_enqueue_duration_ms_records_unlabelled_value() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_queue_enqueue_duration_ms(15.5);

    let rm = collect(&reader);
    let (count, sum) = f64_histogram_point(&rm, "ingress.queue.enqueue_duration_ms", &[])
        .expect("expected an unlabelled histogram data point");

    assert_eq!(count, 1);
    assert!((sum - 15.5).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// record_queue_processing_duration_ms — Histogram<f64>, no labels
// ---------------------------------------------------------------------------

#[test]
fn record_queue_processing_duration_ms_records_unlabelled_value() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_queue_processing_duration_ms(1234.0);

    let rm = collect(&reader);
    let (count, sum) = f64_histogram_point(&rm, "ingress.queue.processing_duration_ms", &[])
        .expect("expected an unlabelled histogram data point");

    assert_eq!(count, 1);
    assert!((sum - 1234.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// set_queue_depth — Gauge<u64>, no labels, last-write-wins semantics
// ---------------------------------------------------------------------------

#[test]
fn set_queue_depth_reports_last_value() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.set_queue_depth(10);
    metrics.set_queue_depth(3);

    let rm = collect(&reader);
    let value = u64_gauge_point(&rm, "ingress.queue.depth", &[])
        .expect("expected a queue depth gauge data point");

    assert_eq!(
        value, 3,
        "a gauge must report the last-set value, not an accumulated sum (10+3=13 would indicate \
         a Counter was used instead of a Gauge)"
    );
}

#[test]
fn set_queue_depth_zero_is_reported_not_omitted() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.set_queue_depth(0);

    let rm = collect(&reader);
    let value = u64_gauge_point(&rm, "ingress.queue.depth", &[])
        .expect("a depth of 0 must still be recorded");

    assert_eq!(value, 0);
}

// ---------------------------------------------------------------------------
// set_queue_age_oldest_message_secs — Gauge<f64>, no labels
// ---------------------------------------------------------------------------

#[test]
fn set_queue_age_oldest_message_secs_reports_last_value() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.set_queue_age_oldest_message_secs(120.0);
    metrics.set_queue_age_oldest_message_secs(300.0);

    let rm = collect(&reader);
    let value = f64_gauge_point(&rm, "ingress.queue.age_oldest_message_secs", &[])
        .expect("expected an age gauge data point");

    assert!((value - 300.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// record_dlq_message — Counter<u64>, no labels
// ---------------------------------------------------------------------------

#[test]
fn record_dlq_message_increments_by_one_per_call() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_dlq_message();
    metrics.record_dlq_message();
    metrics.record_dlq_message();

    let rm = collect(&reader);
    let (value, is_monotonic) = u64_sum_point(&rm, "ingress.queue.dlq_count", &[])
        .expect("expected a dlq_count data point");

    assert_eq!(value, 3);
    assert!(is_monotonic);
}

// ---------------------------------------------------------------------------
// record_worker_error — Counter<u64>, no labels
// ---------------------------------------------------------------------------

#[test]
fn record_worker_error_increments_by_one_per_call() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_worker_error();

    let rm = collect(&reader);
    let (value, _) = u64_sum_point(&rm, "ingress.queue.worker_errors_total", &[])
        .expect("expected a worker_errors_total data point");

    assert_eq!(value, 1);
}

#[test]
fn record_dlq_message_and_record_worker_error_are_independent_counters() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_dlq_message();
    metrics.record_dlq_message();
    metrics.record_worker_error();

    let rm = collect(&reader);
    let dlq = u64_sum_point(&rm, "ingress.queue.dlq_count", &[])
        .expect("dlq_count series must exist")
        .0;
    let worker_errors = u64_sum_point(&rm, "ingress.queue.worker_errors_total", &[])
        .expect("worker_errors_total series must exist")
        .0;

    assert_eq!(
        dlq, 2,
        "dlq_count must not be incremented by record_worker_error calls"
    );
    assert_eq!(
        worker_errors, 1,
        "worker_errors_total must not be incremented by record_dlq_message calls"
    );
}

// ---------------------------------------------------------------------------
// set_processing_success_rate — Gauge<f64>, no labels, range 0.0..=1.0
// ---------------------------------------------------------------------------

#[test]
fn set_processing_success_rate_records_fractional_value() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.set_processing_success_rate(0.987);

    let rm = collect(&reader);
    let value = f64_gauge_point(&rm, "processing.success_rate", &[])
        .expect("expected a processing.success_rate data point");

    assert!((value - 0.987).abs() < 1e-9);
}

#[test]
fn set_processing_success_rate_boundary_zero_and_one() {
    let (reader, _provider, metrics) = new_test_metrics();
    metrics.set_processing_success_rate(0.0);
    let rm = collect(&reader);
    let value = f64_gauge_point(&rm, "processing.success_rate", &[]).expect("0.0 must be recorded");
    assert!((value - 0.0).abs() < f64::EPSILON);

    let (reader2, _provider2, metrics2) = new_test_metrics();
    metrics2.set_processing_success_rate(1.0);
    let rm2 = collect(&reader2);
    let value2 =
        f64_gauge_point(&rm2, "processing.success_rate", &[]).expect("1.0 must be recorded");
    assert!((value2 - 1.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// record_pr_validation_duration_ms — Histogram<f64>, no labels
// ---------------------------------------------------------------------------

#[test]
fn record_pr_validation_duration_ms_records_unlabelled_value() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_pr_validation_duration_ms(87.0);

    let rm = collect(&reader);
    let (count, sum) = f64_histogram_point(&rm, "pr.validation.duration_ms", &[])
        .expect("expected a pr.validation.duration_ms data point");

    assert_eq!(count, 1);
    assert!((sum - 87.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// record_bypass_activation — Counter<u64>, label: bypass_type
// ---------------------------------------------------------------------------

#[test]
fn record_bypass_activation_increments_counter_with_bypass_type_label() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_bypass_activation("admin_override");

    let rm = collect(&reader);
    let (value, is_monotonic) = u64_sum_point(
        &rm,
        "pr.bypass.activations_total",
        &[("bypass_type", "admin_override")],
    )
    .expect("expected a data point for bypass_type=admin_override");

    assert_eq!(value, 1);
    assert!(is_monotonic);
}

#[test]
fn record_bypass_activation_distinguishes_bypass_types() {
    let (reader, _provider, metrics) = new_test_metrics();

    metrics.record_bypass_activation("admin_override");
    metrics.record_bypass_activation("stability_days_waived");
    metrics.record_bypass_activation("admin_override");

    let rm = collect(&reader);
    let admin = u64_sum_point(
        &rm,
        "pr.bypass.activations_total",
        &[("bypass_type", "admin_override")],
    )
    .expect("admin_override series must exist")
    .0;
    let waived = u64_sum_point(
        &rm,
        "pr.bypass.activations_total",
        &[("bypass_type", "stability_days_waived")],
    )
    .expect("stability_days_waived series must exist")
    .0;

    assert_eq!(admin, 2);
    assert_eq!(waived, 1);
}

// ---------------------------------------------------------------------------
// Metrics — Clone semantics: a clone must share the same underlying
// instruments (recording through a clone must be observable through the
// original reader), not create independent, disconnected instrument state.
// ---------------------------------------------------------------------------

#[test]
fn cloned_metrics_share_the_same_underlying_instruments() {
    let (reader, _provider, metrics) = new_test_metrics();
    let cloned = metrics.clone();

    cloned.record_dlq_message();

    let rm = collect(&reader);
    let value = u64_sum_point(&rm, "ingress.queue.dlq_count", &[])
        .expect("recording through a clone must be observable via the original reader")
        .0;

    assert_eq!(value, 1);
}

// ---------------------------------------------------------------------------
// init_metrics
// ---------------------------------------------------------------------------

#[test]
fn init_metrics_succeeds_when_otlp_and_prometheus_both_disabled() {
    let config = MetricsConfig {
        otlp_endpoint: None,
        service_name: "test-service".to_string(),
        service_version: "0.0.1".to_string(),
        prometheus_enabled: false,
    };

    let result = init_metrics(&config);

    assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
}

#[test]
fn init_metrics_succeeds_when_otlp_endpoint_set() {
    // Uses a syntactically valid but unreachable endpoint. Building an OTLP
    // HTTP exporter must not perform actual network I/O (mirrors
    // `telemetry::init_telemetry`'s trace exporter construction), so this
    // must complete instantly without requiring a live collector.
    let config = MetricsConfig {
        otlp_endpoint: Some("http://127.0.0.1:1".to_string()),
        service_name: "test-service".to_string(),
        service_version: "0.0.1".to_string(),
        prometheus_enabled: false,
    };

    let result = init_metrics(&config);

    assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
}

#[test]
fn init_metrics_succeeds_when_prometheus_enabled() {
    let config = MetricsConfig {
        otlp_endpoint: None,
        service_name: "test-service".to_string(),
        service_version: "0.0.1".to_string(),
        prometheus_enabled: true,
    };

    let result = init_metrics(&config);

    assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
}

#[test]
fn init_metrics_returned_provider_can_build_a_working_meter() {
    let config = MetricsConfig {
        otlp_endpoint: None,
        service_name: "test-service".to_string(),
        service_version: "0.0.1".to_string(),
        prometheus_enabled: false,
    };

    let provider = init_metrics(&config).expect("init_metrics must succeed");
    let meter = provider.meter("merge-warden");
    // Constructing `Metrics` from the returned provider's meter must not panic.
    let _metrics = Metrics::new(&meter);
}
