//! Telemetry and observability configuration for Azure Functions.
//!
//! This module provides console-based structured logging for the Merge Warden Azure Function.
//! The implementation is designed to be extensible, allowing for easy addition of different
//! telemetry backends (such as Application Insights, OpenTelemetry, etc.) in the future.

use crate::errors::AzureFunctionsError;
use tracing_subscriber::{prelude::*, EnvFilter};

#[cfg(test)]
#[path = "telemetry_tests.rs"]
mod tests;

/// Initializes console-based structured logging for the Azure Function.
///
/// This function sets up structured logging that outputs to the console using the `tracing`
/// framework. The logging output is formatted for readability and includes structured fields
/// for better observability.
///
/// # Features
///
/// - Environment-based log level filtering via `RUST_LOG` environment variable
/// - Structured JSON-compatible logging output
/// - Timestamp and level information for each log entry
/// - Support for spans and structured fields
///
/// # Environment Variables
///
/// The logging level can be controlled using the `RUST_LOG` environment variable:
/// - `RUST_LOG=debug` - Debug level and above
/// - `RUST_LOG=info` - Info level and above (default)
/// - `RUST_LOG=warn` - Warning level and above
/// - `RUST_LOG=error` - Error level only
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an error if logging cannot be configured.
///
/// # Errors
///
/// Returns `AzureFunctionsError::ConfigError` if the logging subscriber cannot be initialized.
///
/// # Example
///
/// ```rust
/// use crate::telemetry;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     telemetry::init_console_logging()?;
///
///     tracing::info!("Application started");
///     tracing::debug!(user_id = 123, "Processing user request");
///
///     Ok(())
/// }
/// ```
pub fn init_console_logging() -> Result<(), AzureFunctionsError> {
    // Create a formatting layer for console output
    // Use compact format for better readability in Azure Function logs
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Disable ANSI colors for Azure Function logs
        .with_target(true) // Include the target (module path) in logs
        .with_thread_ids(false) // Don't include thread IDs to reduce noise
        .with_file(false) // Don't include file names to reduce noise
        .with_line_number(false) // Don't include line numbers to reduce noise
        .compact(); // Use compact format for cleaner output

    // Set up environment-based filtering
    // Defaults to "info" level if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Initialize the global subscriber
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .try_init()
        .map_err(|e| {
            AzureFunctionsError::ConfigError(format!("Failed to initialize console logging: {}", e))
        })?;

    tracing::info!("Console logging initialized successfully");

    Ok(())
}
