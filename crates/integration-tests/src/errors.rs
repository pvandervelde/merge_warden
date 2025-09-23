//! Error types and result definitions for integration testing.
//!
//! This module provides comprehensive error handling for all integration test operations,
//! including GitHub API failures, test environment setup issues, and Azure service
//! simulation errors.

use thiserror::Error;

/// Result type for integration test operations.
///
/// This is a type alias for `Result<T, TestError>` to provide consistent error handling
/// across all integration test functions.
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{TestResult, TestError};
///
/// async fn create_test_repository(name: &str) -> TestResult<String> {
///     if name.is_empty() {
///         return Err(TestError::InvalidConfiguration("Repository name cannot be empty".to_string()));
///     }
///     Ok(format!("test-repo-{}", name))
/// }
/// ```
pub type TestResult<T> = Result<T, TestError>;

/// Comprehensive error type for integration test operations.
///
/// This enum covers all possible error conditions that can occur during integration
/// testing, from environment setup failures to GitHub API errors and mock service
/// issues.
///
/// # Error Categories
///
/// - **Configuration Errors**: Invalid test configuration or missing environment variables
/// - **GitHub API Errors**: Repository creation, webhook setup, or API communication failures
/// - **Environment Errors**: Test environment setup, cleanup, or resource management issues
/// - **Mock Service Errors**: Azure service simulation failures or configuration issues
/// - **Timeout Errors**: Operations that exceed configured timeout limits
/// - **Validation Errors**: Test assertion failures or unexpected bot behavior
#[derive(Error, Debug, Clone)]
pub enum TestError {
    /// Configuration error indicating invalid or missing test configuration.
    ///
    /// This error occurs when:
    /// - Required environment variables are missing or invalid
    /// - Test configuration files contain invalid values
    /// - GitHub App credentials are malformed
    ///
    /// # Parameters
    /// - `message`: Detailed description of the configuration issue
    #[error("Test configuration error: {0}")]
    InvalidConfiguration(String),

    /// CI/CD configuration or execution error.
    ///
    /// This error occurs during CI/CD pipeline execution or configuration:
    /// - Invalid CI environment setup
    /// - Missing CI environment variables or secrets
    /// - CI execution environment validation failures
    /// - Test execution coordination issues
    ///
    /// # Parameters
    /// - `message`: Detailed description of the CI configuration issue
    #[error("CI configuration error: {0}")]
    CiConfigurationError(String),

    /// GitHub API operation failed.
    ///
    /// This error indicates failures in GitHub API operations such as:
    /// - Repository creation or deletion
    /// - Webhook configuration
    /// - Pull request or issue operations
    /// - Authentication or authorization failures
    ///
    /// # Parameters
    /// - `operation`: The GitHub operation that failed (e.g., "create_repository", "setup_webhook")
    /// - `message`: Detailed error message from the GitHub API or client
    #[error("GitHub API error during {operation}: {message}")]
    GitHubApiError {
        /// The specific GitHub operation that failed
        operation: String,
        /// Detailed error message
        message: String,
    },

    /// Test environment setup or management failed.
    ///
    /// This error occurs during:
    /// - Test environment initialization
    /// - Resource allocation or cleanup
    /// - Service coordination failures
    /// - Local development environment issues
    ///
    /// # Parameters
    /// - `component`: The environment component that failed (e.g., "repository_manager", "bot_instance")
    /// - `message`: Detailed description of the failure
    #[error("Environment error in {component}: {message}")]
    EnvironmentError {
        /// The environment component that failed
        component: String,
        /// Detailed error message
        message: String,
    },

    /// Azure service mock simulation failed.
    ///
    /// This error indicates issues with mock Azure services:
    /// - Mock App Config service failures
    /// - Mock Key Vault service issues
    /// - Service simulation configuration errors
    /// - Mock service state inconsistencies
    ///
    /// # Parameters
    /// - `service`: The Azure service being mocked (e.g., "app_config", "key_vault")
    /// - `message`: Description of the mock service failure
    #[error("Mock Azure service error in {service}: {message}")]
    MockServiceError {
        /// The Azure service that failed
        service: String,
        /// Detailed error message
        message: String,
    },

    /// Operation timed out waiting for completion.
    ///
    /// This error occurs when operations exceed configured timeout limits:
    /// - Webhook processing timeouts
    /// - GitHub API response delays
    /// - Bot processing timeouts
    /// - Test environment setup delays
    ///
    /// # Parameters
    /// - `operation`: The operation that timed out
    /// - `timeout_seconds`: The configured timeout limit in seconds
    #[error("Timeout waiting for {operation} (limit: {timeout_seconds}s)")]
    Timeout {
        /// The operation that timed out
        operation: String,
        /// The timeout limit in seconds
        timeout_seconds: u64,
    },

    /// Test validation failed due to unexpected bot behavior.
    ///
    /// This error indicates that the bot did not behave as expected:
    /// - Missing or incorrect labels on pull requests
    /// - Unexpected comment content or format
    /// - Incorrect validation status or check results
    /// - Missing webhook responses
    ///
    /// # Parameters
    /// - `expected`: Description of the expected behavior
    /// - `actual`: Description of the actual behavior observed
    #[error("Validation failed - expected: {expected}, actual: {actual}")]
    ValidationFailed {
        /// Expected behavior description
        expected: String,
        /// Actual behavior observed
        actual: String,
    },

    /// Resource cleanup failed after test completion.
    ///
    /// This error occurs when test resources cannot be properly cleaned up:
    /// - Repository deletion failures
    /// - Webhook removal issues
    /// - GitHub App installation cleanup problems
    /// - Temporary file or directory cleanup failures
    ///
    /// # Parameters
    /// - `resource`: The type of resource that failed to clean up
    /// - `message`: Detailed cleanup failure description
    #[error("Cleanup failed for {resource}: {message}")]
    CleanupFailed {
        /// The resource type that failed cleanup
        resource: String,
        /// Detailed error message
        message: String,
    },

    /// Network or connectivity error.
    ///
    /// This error indicates network-related failures:
    /// - HTTP client errors
    /// - Network connectivity issues
    /// - DNS resolution failures
    /// - SSL/TLS handshake problems
    ///
    /// # Parameters
    /// - `message`: Detailed network error description
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Authentication or authorization error.
    ///
    /// This error occurs when:
    /// - GitHub token has insufficient permissions
    /// - GitHub App authentication fails
    /// - Webhook signature validation fails
    /// - JWT token generation or validation issues
    ///
    /// # Parameters
    /// - `context`: Authentication context (e.g., "github_token", "webhook_signature")
    /// - `message`: Detailed authentication error
    #[error("Authentication error in {context}: {message}")]
    AuthenticationError {
        /// Authentication context
        context: String,
        /// Detailed error message
        message: String,
    },

    /// Internal error that should not occur in normal operation.
    ///
    /// This error indicates unexpected internal failures:
    /// - Programming errors or assertion failures
    /// - Unexpected state transitions
    /// - Resource allocation failures
    /// - System-level errors
    ///
    /// # Parameters
    /// - `message`: Description of the internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl TestError {
    /// Creates a new GitHub API error.
    ///
    /// # Parameters
    /// - `operation`: The GitHub operation that failed
    /// - `message`: Detailed error message
    ///
    /// # Returns
    /// A new `TestError::GitHubApiError` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::github_api_error("create_repository", "Repository already exists");
    /// ```
    pub fn github_api_error(operation: &str, message: &str) -> Self {
        Self::GitHubApiError {
            operation: operation.to_string(),
            message: message.to_string(),
        }
    }

    /// Creates a new environment error.
    ///
    /// # Parameters
    /// - `component`: The environment component that failed
    /// - `message`: Detailed error message
    ///
    /// # Returns
    /// A new `TestError::EnvironmentError` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::environment_error("bot_instance", "Failed to start local tunnel");
    /// ```
    pub fn environment_error(component: &str, message: &str) -> Self {
        Self::EnvironmentError {
            component: component.to_string(),
            message: message.to_string(),
        }
    }

    /// Creates a new mock service error.
    ///
    /// # Parameters
    /// - `service`: The Azure service being mocked
    /// - `message`: Error description
    ///
    /// # Returns
    /// A new `TestError::MockServiceError` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::mock_service_error("app_config", "Configuration key not found");
    /// ```
    pub fn mock_service_error(service: &str, message: &str) -> Self {
        Self::MockServiceError {
            service: service.to_string(),
            message: message.to_string(),
        }
    }

    /// Creates a new timeout error.
    ///
    /// # Parameters
    /// - `operation`: The operation that timed out
    /// - `timeout_seconds`: The timeout limit
    ///
    /// # Returns
    /// A new `TestError::Timeout` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::timeout("webhook_processing", 30);
    /// ```
    pub fn timeout(operation: &str, timeout_seconds: u64) -> Self {
        Self::Timeout {
            operation: operation.to_string(),
            timeout_seconds,
        }
    }

    /// Creates a new validation error.
    ///
    /// # Parameters
    /// - `expected`: Description of expected behavior
    /// - `actual`: Description of actual behavior
    ///
    /// # Returns
    /// A new `TestError::ValidationFailed` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::validation_failed(
    ///     "Pull request should have size label",
    ///     "No size label found"
    /// );
    /// ```
    pub fn validation_failed(expected: &str, actual: &str) -> Self {
        Self::ValidationFailed {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Creates a new cleanup error.
    ///
    /// # Parameters
    /// - `resource`: The resource type that failed cleanup
    /// - `message`: Detailed error message
    ///
    /// # Returns
    /// A new `TestError::CleanupFailed` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::cleanup_failed("test_repository", "Repository deletion failed");
    /// ```
    pub fn cleanup_failed(resource: &str, message: &str) -> Self {
        Self::CleanupFailed {
            resource: resource.to_string(),
            message: message.to_string(),
        }
    }

    /// Creates a new authentication error.
    ///
    /// # Parameters
    /// - `context`: Authentication context
    /// - `message`: Detailed error message
    ///
    /// # Returns
    /// A new `TestError::AuthenticationError` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let error = TestError::authentication_error("github_token", "Token has insufficient permissions");
    /// ```
    pub fn authentication_error(context: &str, message: &str) -> Self {
        Self::AuthenticationError {
            context: context.to_string(),
            message: message.to_string(),
        }
    }

    /// Checks if this error is related to authentication or authorization.
    ///
    /// # Returns
    /// `true` if the error is authentication-related, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let auth_error = TestError::authentication_error("github_token", "Invalid token");
    /// assert!(auth_error.is_authentication_error());
    ///
    /// let config_error = TestError::InvalidConfiguration("Missing env var".to_string());
    /// assert!(!config_error.is_authentication_error());
    /// ```
    pub fn is_authentication_error(&self) -> bool {
        matches!(self, TestError::AuthenticationError { .. })
    }

    /// Checks if this error is recoverable and the operation should be retried.
    ///
    /// # Returns
    /// `true` if the error is potentially recoverable, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestError;
    ///
    /// let network_error = TestError::NetworkError("Connection timeout".to_string());
    /// assert!(network_error.is_recoverable());
    ///
    /// let config_error = TestError::InvalidConfiguration("Missing env var".to_string());
    /// assert!(!config_error.is_recoverable());
    /// ```
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            TestError::NetworkError(_)
                | TestError::Timeout { .. }
                | TestError::GitHubApiError { .. }
                | TestError::MockServiceError { .. }
        )
    }
}

/// Converts from `std::io::Error` to `TestError`.
impl From<std::io::Error> for TestError {
    fn from(err: std::io::Error) -> Self {
        TestError::EnvironmentError {
            component: "filesystem".to_string(),
            message: err.to_string(),
        }
    }
}

/// Converts from `reqwest::Error` to `TestError`.
impl From<reqwest::Error> for TestError {
    fn from(err: reqwest::Error) -> Self {
        TestError::NetworkError(err.to_string())
    }
}

/// Converts from `serde_json::Error` to `TestError`.
impl From<serde_json::Error> for TestError {
    fn from(err: serde_json::Error) -> Self {
        TestError::InvalidConfiguration(format!("JSON parsing error: {}", err))
    }
}

/// Converts from `toml::de::Error` to `TestError`.
impl From<toml::de::Error> for TestError {
    fn from(err: toml::de::Error) -> Self {
        TestError::InvalidConfiguration(format!("TOML parsing error: {}", err))
    }
}

/// Converts from `octocrab::Error` to `TestError`.
impl From<octocrab::Error> for TestError {
    fn from(err: octocrab::Error) -> Self {
        TestError::GitHubApiError {
            operation: "github_client_operation".to_string(),
            message: err.to_string(),
        }
    }
}
