use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::BatchConfigBuilder;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::Resource;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::errors::AzureFunctionsError;

pub async fn init_telemetry(
    app_insights_connection_string: &str,
) -> Result<(), AzureFunctionsError> {
    // Set up Azure Monitor exporter
    let azure_monitor_exporter =
        opentelemetry_application_insights::Exporter::new_from_connection_string(
            app_insights_connection_string,
            reqwest::Client::new(),
        )
        .expect("valid connection string");

    // Create a BatchSpanProcessor for each exporter
    let azure_monitor_processor = BatchSpanProcessor::builder(azure_monitor_exporter)
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

    let tracer = provider.tracer("thing");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    tracing_subscriber::registry()
        .with(telemetry)
        .with(EnvFilter::from_default_env())
        .init();

    Ok(())
}
