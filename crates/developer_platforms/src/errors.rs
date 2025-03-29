#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API request failed")]
    ApiError(),

    #[error("Approval attempted - blocked by policy")]
    ApprovalProhibited,

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Failed to create an app access token for repository: {0}/{1}. For app with ID: {2}")]
    FailedToCreateAccessToken(String, String, u64),

    #[error("Failed to find installation for repository: {0}/{1} with ID: {2}")]
    FailedToFindAppInstallation(String, String, u64),

    #[error("Failed to update the PR.")]
    FailedToUpdatePullRequest(String),

    #[error("Invalid response format")]
    InvalidResponse,

    #[error("Invalid review state transition attempted")]
    InvalidStateTransition,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Review operation conflict: {0}")]
    ReviewConflict(String),
}
