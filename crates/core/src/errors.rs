use thiserror::Error;

#[derive(Error, Debug)]
pub enum MergeWardenError {
    #[error("Git provider error: {0}")]
    GitProviderError(String),

    #[error("Invalid PR title format")]
    InvalidPrTitleFormat,

    #[error("Missing work item reference")]
    MissingWorkItemReference,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
