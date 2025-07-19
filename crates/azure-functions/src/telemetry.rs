use log::Level;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_application_insights::Exporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::BatchConfigBuilder;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::Resource;
use reqwest::Client;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::errors::AzureFunctionsError;

#[cfg(test)]
#[path = "telemetry_tests.rs"]
mod tests;

/// Initializes the OpenTelemetry logging provider with Azure Monitor integration.
///
/// This function sets up structured logging that exports logs to Azure Application Insights
/// through the provided exporter.
///
/// # Arguments
///
/// * `exporter` - The Azure Monitor exporter for log data
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an error if logging cannot be configured.
///
/// # Errors
///
/// Returns `AzureFunctionsError::ConfigError` if the log provider cannot be set.
fn init_logs(exporter: Exporter<Client>) -> Result<(), AzureFunctionsError> {
    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .build();
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).map_err(|_e| {
        AzureFunctionsError::ConfigError("Failed to set the log provider.".to_string())
    })?;
    log::set_max_level(Level::Trace.to_level_filter());

    Ok(())
}

/// Initializes the OpenTelemetry metrics provider with Azure Monitor integration.
///
/// This function sets up metrics collection that exports metrics to Azure Application Insights
/// through the provided exporter.
///
/// # Arguments
///
/// * `exporter` - The Azure Monitor exporter for metrics data
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization.
fn init_metrics(exporter: Exporter<Client>) -> Result<(), AzureFunctionsError> {
    let reader = PeriodicReader::builder(exporter).build();
    let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
    global::set_meter_provider(meter_provider.clone());

    Ok(())
}

/// Initializes the OpenTelemetry tracing provider with Azure Monitor integration.
///
/// This function sets up distributed tracing that exports trace data to Azure Application Insights
/// through the provided exporter. It configures batching and resource identification for optimal
/// performance and observability.
///
/// # Arguments
///
/// * `azure_monitor_exporter` - The Azure Monitor exporter for trace data
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an error if tracing cannot be configured.
///
/// # Errors
///
/// Returns `AzureFunctionsError::ConfigError` if the tracer provider cannot be set.
fn init_tracing(azure_monitor_exporter: Exporter<Client>) -> Result<(), AzureFunctionsError> {
    // Create a BatchSpanProcessor for each exporter
    let azure_monitor_processor = BatchSpanProcessor::builder(azure_monitor_exporter.clone())
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_queue_size(4096)
                .build(),
        )
        .build();

    // Build the tracer provider
    let resource = Resource::builder()
        .with_attribute(opentelemetry::KeyValue::new("service.name", "merge_warden"))
        .build();
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_span_processor(azure_monitor_processor)
        .build();
    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("merge_warden");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Bridge log crate events to tracing
    tracing_log::LogTracer::init()
        .map_err(|_e| AzureFunctionsError::ConfigError("Failed to set log tracer".to_string()))?;

    // Add a console logging layer so logs are streamed to the console
    let fmt_layer = tracing_subscriber::fmt::layer().with_ansi(true);
    let _ = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry)
        .with(EnvFilter::from_default_env())
        .try_init();

    Ok(())
}

/// Initializes all telemetry components (metrics and tracing) with Azure Application Insights.
///
/// This function sets up comprehensive observability by configuring OpenTelemetry with Azure Monitor
/// integration. It initializes metrics collection and distributed tracing to provide insights into
/// application performance and behavior.
///
/// # Arguments
///
/// * `app_insights_connection_string` - The Azure Application Insights connection string
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an error if telemetry cannot be configured.
///
/// # Errors
///
/// Returns `AzureFunctionsError::ConfigError` if:
/// - The Azure Monitor exporter cannot be created
/// - Metrics or tracing initialization fails
pub async fn init_telemetry(
    app_insights_connection_string: &str,
) -> Result<(), AzureFunctionsError> {
    // Set up Azure Monitor exporter
    let azure_monitor_exporter =
        opentelemetry_application_insights::Exporter::new_from_connection_string(
            app_insights_connection_string,
            reqwest::Client::new(),
        )
        .map_err(|e| AzureFunctionsError::ConfigError(e.to_string()))?;

    //init_logs(azure_monitor_exporter.clone())?;
    init_metrics(azure_monitor_exporter.clone())?;
    init_tracing(azure_monitor_exporter)?;

    Ok(())
}
