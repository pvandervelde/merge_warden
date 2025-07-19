use thiserror::Error;

/// Errors that can occur when loading the merge-warden configuration
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// Configuration file was not found at the specified path
    #[error("Configuration file not found at {0}")]
    NotFound(String),

    /// Failed to parse the configuration file due to syntax or format errors
    #[error("Failed to parse configuration file: {0}")]
    ParseError(String),

    /// The configuration file uses an unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(u32),

    /// File system I/O error occurred while reading the configuration
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error occurred while reading the configuration
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Main error type for Merge Warden operations.
///
/// This enum represents all possible errors that can occur during
/// pull request validation, configuration management, and Git provider
/// interactions within the Merge Warden system.
///
/// # Examples
///
/// ```rust
/// use merge_warden_core::errors::MergeWardenError;
///
/// // Configuration error
/// let config_error = MergeWardenError::ConfigError("Invalid regex pattern".to_string());
/// println!("{}", config_error);
///
/// // PR title validation error
/// let title_error = MergeWardenError::InvalidPrTitleFormat;
/// assert_eq!(title_error.to_string(), "Invalid PR title format");
/// ```
#[derive(Error, Debug)]
pub enum MergeWardenError {
    /// Configuration-related error with details
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Failed to update pull request through the Git provider API
    #[error("Failed to update pull request. Issue was: '{0}'.")]
    FailedToUpdatePullRequest(String),

    /// Error from the underlying Git provider (GitHub, GitLab, etc.)
    #[error("Git provider error: {0}")]
    GitProviderError(String),

    /// Pull request title does not follow the required format
    #[error("Invalid PR title format")]
    InvalidPrTitleFormat,

    /// Pull request description is missing a required work item reference
    #[error("Missing work item reference")]
    MissingWorkItemReference,

    /// Regular expression compilation or matching error
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Generic error for unspecified issues
    #[error("Unknown error: {0}")]
    Unknown(String),
}
