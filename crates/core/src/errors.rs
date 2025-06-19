use thiserror::Error;

/// Errors that can occur when loading the merge-warden configuration
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("Configuration file not found at {0}")]
    NotFound(String),

    #[error("Failed to parse configuration file: {0}")]
    ParseError(String),

    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(u32),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Error, Debug)]
pub enum MergeWardenError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to update pull request. Issue was: '{0}'.")]
    FailedToUpdatePullRequest(String),

    #[error("Git provider error: {0}")]
    GitProviderError(String),

    #[error("Invalid PR title format")]
    InvalidPrTitleFormat,

    #[error("Missing work item reference")]
    MissingWorkItemReference,

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
