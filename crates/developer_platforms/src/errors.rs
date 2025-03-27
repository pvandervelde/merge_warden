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
