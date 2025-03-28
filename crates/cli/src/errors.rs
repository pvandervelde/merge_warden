use thiserror::Error;

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum CliError {
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

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Other(err.to_string())
    }
}

impl std::process::Termination for CliError {
    fn report(self) -> std::process::ExitCode {
        match self {
            CliError::ConfigError(_) => std::process::ExitCode::from(2),
            CliError::AuthError(_) => std::process::ExitCode::from(3),
            CliError::NetworkError(_) => std::process::ExitCode::from(4),
            CliError::InvalidArguments(_) => std::process::ExitCode::from(5),
            CliError::ValidationFailed(_) => std::process::ExitCode::from(1),
            CliError::Other(_) => std::process::ExitCode::FAILURE,
        }
    }
}
