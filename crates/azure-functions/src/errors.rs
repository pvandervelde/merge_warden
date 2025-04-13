use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum AzureFunctionsError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}
