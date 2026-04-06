use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum CliError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

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
            CliError::InvalidArguments(_) => std::process::ExitCode::from(5),
            CliError::Other(_) => std::process::ExitCode::FAILURE,
        }
    }
}
