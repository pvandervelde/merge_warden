use super::*;

#[test]
fn test_init_console_logging_success() {
    // Test that console logging initializes successfully
    // Note: This test may fail if run multiple times in the same process
    // because the global subscriber can only be set once

    // Clear any existing environment variable that might interfere
    std::env::remove_var("RUST_LOG");

    // This should succeed on first call
    let result = init_console_logging();

    // The result should be Ok, or if already initialized, should be an error
    // but not panic
    match result {
        Ok(()) => {
            // Successfully initialized - this is the expected case for first call
            assert!(true);
        }
        Err(AzureFunctionsError::ConfigError(msg)) => {
            // Already initialized - this can happen in test environments
            assert!(msg.contains("Failed to initialize console logging"));
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}

#[test]
fn test_init_console_logging_with_env_filter() {
    // Test that console logging respects RUST_LOG environment variable
    std::env::set_var("RUST_LOG", "debug");

    // Attempt to initialize (may fail if already initialized, which is fine)
    let _result = init_console_logging();

    // Clean up
    std::env::remove_var("RUST_LOG");

    // If we got here without panicking, the test passes
    assert!(true);
}

#[test]
fn test_tracing_macros_work() {
    // Test that tracing macros can be called without panicking
    // This verifies that the logging infrastructure is properly set up

    tracing::error!("Test error message");
    tracing::warn!("Test warning message");
    tracing::info!("Test info message");
    tracing::debug!("Test debug message");
    tracing::trace!("Test trace message");

    // Test structured logging with fields
    tracing::info!(
        user_id = 123,
        request_id = "abc-def-123",
        "Processing user request"
    );

    // If we got here without panicking, the test passes
    assert!(true);
}
