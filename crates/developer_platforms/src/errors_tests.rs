use super::*;
use std::error::Error as StdError;

#[test]
fn test_api_error() {
    let error = Error::ApiError();

    // Test error message
    assert_eq!(error.to_string(), "API request failed");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_approval_prohibited_error() {
    let error = Error::ApprovalProhibited;

    // Test error message
    assert_eq!(error.to_string(), "Approval attempted - blocked by policy");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_auth_error() {
    let error = Error::AuthError("Invalid credentials".to_string());

    // Test error message
    assert_eq!(
        error.to_string(),
        "Authentication failed: Invalid credentials"
    );

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_failed_to_update_pull_request_error() {
    let error = Error::FailedToUpdatePullRequest("Network error".to_string());

    // Test error message
    assert_eq!(error.to_string(), "Failed to update the PR.");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_invalid_response_error() {
    let error = Error::InvalidResponse;

    // Test error message
    assert_eq!(error.to_string(), "Invalid response format");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_invalid_state_transition_error() {
    let error = Error::InvalidStateTransition;

    // Test error message
    assert_eq!(
        error.to_string(),
        "Invalid review state transition attempted"
    );

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_rate_limit_exceeded_error() {
    let error = Error::RateLimitExceeded;

    // Test error message
    assert_eq!(error.to_string(), "Rate limit exceeded");

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_review_conflict_error() {
    let error = Error::ReviewConflict("Review already exists".to_string());

    // Test error message
    assert_eq!(
        error.to_string(),
        "Review operation conflict: Review already exists"
    );

    // Test error source
    assert!(error.source().is_none());
}

#[test]
fn test_error_is_send_sync() {
    // This test verifies that Error implements Send and Sync traits
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
}
