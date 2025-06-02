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

fn init_logs(exporter: Exporter<Client>) -> Result<(), AzureFunctionsError> {
    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .build();
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).map_err(|e| {
        AzureFunctionsError::ConfigError("Failed to set the log provider.".to_string())
    })?;
    log::set_max_level(Level::Trace.to_level_filter());

    Ok(())
}

fn init_metrics(exporter: Exporter<Client>) -> Result<(), AzureFunctionsError> {
    let reader = PeriodicReader::builder(exporter).build();
    let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
    global::set_meter_provider(meter_provider.clone());

    Ok(())
}

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
        .map_err(|e| AzureFunctionsError::ConfigError("Failed to set log tracer".to_string()))?;

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    tracing_subscriber::registry()
        .with(telemetry)
        .with(EnvFilter::from_default_env())
        .init();

    Ok(())
}

pub async fn init_telemetry(
    app_insights_connection_string: &str,
) -> Result<(), AzureFunctionsError> {
    // Set up Azure Monitor exporter
    let azure_monitor_exporter =
        opentelemetry_application_insights::Exporter::new_from_connection_string(
            app_insights_connection_string,
            reqwest::Client::new(),
        )
        .map_err(|e| AzureFunctionsError::ConfigError("Invalid connection string".to_string()))?;

    //init_logs(azure_monitor_exporter.clone())?;
    init_metrics(azure_monitor_exporter.clone())?;
    init_tracing(azure_monitor_exporter)?;

    Ok(())
}
