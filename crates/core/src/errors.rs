use thiserror::Error;

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
