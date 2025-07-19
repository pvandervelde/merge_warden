#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// Error types for developer platform operations.
///
/// This enum represents all possible errors that can occur when interacting
/// with developer platforms like GitHub, GitLab, etc. Each variant provides
/// specific context about the type of failure encountered.
///
/// The errors are designed to be informative for both debugging and user-facing
/// error messages, with appropriate error codes and descriptions.
///
/// # Examples
///
/// ```rust
/// use merge_warden_developer_platforms::errors::Error;
///
/// // Authentication error
/// let auth_error = Error::AuthError("Invalid token".to_string());
/// println!("{}", auth_error);
///
/// // Rate limit error
/// let rate_limit = Error::RateLimitExceeded;
/// assert_eq!(rate_limit.to_string(), "Rate limit exceeded");
/// ```
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Generic API request failure.
    ///
    /// This error indicates that an API call to the developer platform failed
    /// for an unspecified reason. This is typically used as a fallback when
    /// more specific error information is not available.
    #[error("API request failed")]
    ApiError(),

    /// Approval attempted but blocked by policy.
    ///
    /// This error occurs when attempting to approve a pull request that is
    /// blocked by repository policies or merge requirements. For example,
    /// when required status checks are failing or when the PR doesn't meet
    /// the configured approval requirements.
    #[error("Approval attempted - blocked by policy")]
    ApprovalProhibited,

    /// Authentication failed with the platform.
    ///
    /// This error indicates that the provided credentials (token, app credentials, etc.)
    /// are invalid, expired, or insufficient for the requested operation.
    /// The string parameter contains additional details about the authentication failure.
    ///
    /// # Examples
    ///
    /// - Invalid personal access token
    /// - Expired GitHub App installation token
    /// - Insufficient permissions for the operation
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Failed to create app access token for repository.
    ///
    /// This error occurs when attempting to create an installation access token
    /// for a GitHub App fails. This typically happens when:
    /// - The app is not installed on the repository
    /// - The app installation has insufficient permissions
    /// - The app credentials are invalid
    ///
    /// Parameters: repository owner, repository name, app ID
    #[error("Failed to create an app access token for repository: {0}/{1}. For app with ID: {2}")]
    FailedToCreateAccessToken(String, String, u64),

    /// Failed to find app installation for repository.
    ///
    /// This error occurs when a GitHub App installation cannot be found for
    /// the specified repository. This typically means the app is not installed
    /// on the repository or organization, or the installation has been suspended.
    ///
    /// Parameters: repository owner, repository name, installation ID
    #[error("Failed to find installation for repository: {0}/{1} with ID: {2}")]
    FailedToFindAppInstallation(String, String, u64),

    /// Failed to update pull request.
    ///
    /// This error occurs when an operation to modify a pull request fails.
    /// This could include failures to:
    /// - Add or remove labels
    /// - Post comments
    /// - Update the PR status
    /// - Modify PR metadata
    ///
    /// The string parameter contains specific details about what operation failed.
    #[error("Failed to update the PR: {0}")]
    FailedToUpdatePullRequest(String),

    /// Invalid response format from platform API.
    ///
    /// This error indicates that the response received from the developer platform
    /// API was not in the expected format. This could happen due to:
    /// - API version changes
    /// - Malformed JSON responses
    /// - Missing required fields in the response
    /// - Unexpected response structure
    #[error("Invalid response format")]
    InvalidResponse,

    /// Invalid state transition attempted.
    ///
    /// This error occurs when attempting to transition a pull request or review
    /// to an invalid state. For example:
    /// - Trying to approve an already merged PR
    /// - Attempting to request changes on a closed PR
    /// - Invalid review state transitions
    #[error("Invalid review state transition attempted")]
    InvalidStateTransition,

    /// Platform rate limit exceeded.
    ///
    /// This error indicates that the API rate limit for the developer platform
    /// has been exceeded. The application should implement exponential backoff
    /// and retry the operation after the rate limit window resets.
    ///
    /// Different platforms have different rate limiting strategies:
    /// - GitHub: Typically 5000 requests per hour for authenticated requests
    /// - GitLab: Various rate limits depending on the plan and endpoint
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Review operation conflict.
    ///
    /// This error occurs when a review operation conflicts with the current
    /// state of the pull request or with other concurrent operations.
    /// Examples include:
    /// - Attempting to submit a review that conflicts with recent changes
    /// - Race conditions when multiple reviewers act simultaneously
    /// - Review submission conflicts with PR state changes
    ///
    /// The string parameter contains specific details about the conflict.
    #[error("Review operation conflict: {0}")]
    ReviewConflict(String),
}
