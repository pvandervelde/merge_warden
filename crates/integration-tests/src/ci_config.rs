//! CI/CD configuration and test execution coordination.
//!
//! This module provides the infrastructure for running integration tests in CI/CD
//! environments with proper isolation, environment setup, and result reporting.

use std::collections::HashMap;
use std::time::Duration;

use crate::errors::{TestError, TestResult};

/// Configuration for integration test execution in CI/CD environments.
///
/// The `CiTestConfig` manages all aspects of running integration tests in automated
/// environments including GitHub Actions, with proper environment isolation,
/// resource management, and failure handling.
///
/// # Features
///
/// - **Environment Isolation**: Ensures tests don't interfere with each other
/// - **Resource Management**: Handles GitHub API rate limits and cleanup
/// - **Parallel Execution**: Configures safe parallel test execution strategies
/// - **Failure Recovery**: Provides mechanisms for handling transient failures
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{CiTestConfig, TestError};
///
/// #[tokio::test]
/// async fn test_ci_configuration() -> Result<(), TestError> {
///     let config = CiTestConfig::for_github_actions().await?;
///
///     assert!(config.is_ci_environment());
///     assert!(config.parallel_test_limit() > 0);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CiTestConfig {
    /// Whether running in CI environment (vs local development)
    pub is_ci: bool,
    /// Maximum number of parallel test threads
    pub parallel_limit: usize,
    /// GitHub API rate limit configuration
    pub github_rate_limit: GitHubRateLimit,
    /// Test timeout configuration
    pub test_timeouts: TestTimeouts,
    /// Resource cleanup configuration
    pub cleanup_config: CleanupConfig,
    /// Environment variable management
    pub environment_isolation: EnvironmentIsolation,
    /// Retry configuration for flaky tests
    pub retry_config: RetryConfig,
}

impl CiTestConfig {
    /// Creates CI configuration optimized for GitHub Actions environment.
    ///
    /// This method detects GitHub Actions environment and configures integration
    /// tests with appropriate resource limits, parallelism, and timeouts for
    /// reliable execution in the CI environment.
    ///
    /// # Environment Variables Used
    ///
    /// - `GITHUB_ACTIONS`: Detects if running in GitHub Actions
    /// - `RUNNER_OS`: Operating system for platform-specific configuration
    /// - `GITHUB_TOKEN`: GitHub API authentication (if available)
    /// - `CI_PARALLEL_LIMIT`: Override default parallel execution limit
    ///
    /// # Returns
    ///
    /// A `CiTestConfig` optimized for GitHub Actions with appropriate resource
    /// limits and configuration for reliable test execution.
    ///
    /// # Errors
    ///
    /// Returns `TestError::CiConfigurationError` if:
    /// - Required CI environment variables are missing
    /// - GitHub API authentication cannot be configured
    /// - Resource limit configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{CiTestConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_github_actions_config() -> Result<(), TestError> {
    ///     let config = CiTestConfig::for_github_actions().await?;
    ///
    ///     if config.is_ci_environment() {
    ///         // In CI, should have conservative settings
    ///         assert!(config.parallel_test_limit() <= 4);
    ///         assert!(config.test_timeouts().default_timeout >= Duration::from_secs(60));
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn for_github_actions() -> TestResult<Self> {
        // TODO: implement - Detect and configure for GitHub Actions environment
        todo!("Detect GitHub Actions environment and configure test execution")
    }

    /// Creates CI configuration for local development environment.
    ///
    /// This method configures integration tests for local development with
    /// more aggressive parallelism and shorter timeouts, optimized for
    /// developer productivity during test development and debugging.
    ///
    /// # Environment Variables Used
    ///
    /// - `LOCAL_PARALLEL_LIMIT`: Override default parallel execution limit
    /// - `LOCAL_TEST_TIMEOUT`: Override default test timeouts
    /// - `GITHUB_TEST_TOKEN`: Local GitHub authentication token
    ///
    /// # Returns
    ///
    /// A `CiTestConfig` optimized for local development with faster execution
    /// and more flexible resource usage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{CiTestConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_local_development_config() -> Result<(), TestError> {
    ///     let config = CiTestConfig::for_local_development().await?;
    ///
    ///     assert!(!config.is_ci_environment());
    ///     // Local development can be more aggressive
    ///     assert!(config.parallel_test_limit() >= 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn for_local_development() -> TestResult<Self> {
        // TODO: implement - Configure for local development environment
        todo!("Configure test execution for local development")
    }

    /// Validates the CI configuration and environment setup.
    ///
    /// This method performs comprehensive validation of the CI configuration
    /// including GitHub API access, resource limits, and environment variable
    /// availability required for test execution.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid and environment is ready, or
    /// `TestError::CiConfigurationError` with specific details about what
    /// configuration is missing or invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{CiTestConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_config_validation() -> Result<(), TestError> {
    ///     let config = CiTestConfig::for_github_actions().await?;
    ///     config.validate().await?;
    ///
    ///     // If we reach here, configuration is valid
    ///     Ok(())
    /// }
    /// ```
    pub async fn validate(&self) -> TestResult<()> {
        // TODO: implement - Validate CI configuration and environment
        todo!("Validate CI configuration and environment setup")
    }

    /// Gets whether running in CI environment.
    ///
    /// # Returns
    ///
    /// `true` if running in a CI environment (GitHub Actions, etc.), `false`
    /// if running in local development environment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::CiTestConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = CiTestConfig::for_github_actions().await?;
    /// if config.is_ci_environment() {
    ///     println!("Running in CI - using conservative settings");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_ci_environment(&self) -> bool {
        self.is_ci
    }

    /// Gets the maximum number of parallel test threads.
    ///
    /// # Returns
    ///
    /// The maximum number of tests that should run in parallel, considering
    /// resource constraints and environment limitations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::CiTestConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = CiTestConfig::for_local_development().await?;
    /// println!("Running up to {} tests in parallel", config.parallel_test_limit());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parallel_test_limit(&self) -> usize {
        self.parallel_limit
    }

    /// Gets the test timeout configuration.
    ///
    /// # Returns
    ///
    /// Reference to the `TestTimeouts` configuration for various test types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::CiTestConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = CiTestConfig::for_github_actions().await?;
    /// let timeouts = config.test_timeouts();
    /// println!("Default timeout: {:?}", timeouts.default_timeout);
    /// # Ok(())
    /// # }
    /// ```
    pub fn test_timeouts(&self) -> &TestTimeouts {
        &self.test_timeouts
    }

    /// Gets the GitHub API rate limit configuration.
    ///
    /// # Returns
    ///
    /// Reference to the `GitHubRateLimit` configuration for API usage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::CiTestConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = CiTestConfig::for_github_actions().await?;
    /// let rate_limit = config.github_rate_limit();
    /// println!("API requests per hour: {}", rate_limit.requests_per_hour);
    /// # Ok(())
    /// # }
    /// ```
    pub fn github_rate_limit(&self) -> &GitHubRateLimit {
        &self.github_rate_limit
    }
}

/// GitHub API rate limiting configuration for CI environments.
///
/// This struct manages GitHub API usage to prevent rate limit violations
/// during integration test execution, especially important in CI environments
/// where multiple jobs may be running simultaneously.
#[derive(Debug, Clone)]
pub struct GitHubRateLimit {
    /// Maximum API requests per hour
    pub requests_per_hour: u32,
    /// Delay between API requests to avoid bursting
    pub request_delay: Duration,
    /// Maximum concurrent API requests
    pub concurrent_requests: u32,
    /// Whether to use authenticated requests (higher limits)
    pub use_authentication: bool,
}

impl Default for GitHubRateLimit {
    fn default() -> Self {
        Self {
            requests_per_hour: 1000, // Conservative limit for unauthenticated requests
            request_delay: Duration::from_millis(100),
            concurrent_requests: 5,
            use_authentication: true,
        }
    }
}

/// Timeout configuration for different types of integration tests.
///
/// This struct provides timeout values optimized for different test scenarios
/// and environments, balancing test reliability with execution speed.
#[derive(Debug, Clone)]
pub struct TestTimeouts {
    /// Default timeout for standard integration tests
    pub default_timeout: Duration,
    /// Timeout for GitHub API operations
    pub github_api_timeout: Duration,
    /// Timeout for webhook delivery tests
    pub webhook_timeout: Duration,
    /// Timeout for repository setup operations
    pub repository_setup_timeout: Duration,
    /// Timeout for service outage simulation tests
    pub outage_simulation_timeout: Duration,
}

impl Default for TestTimeouts {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            github_api_timeout: Duration::from_secs(15),
            webhook_timeout: Duration::from_secs(10),
            repository_setup_timeout: Duration::from_secs(60),
            outage_simulation_timeout: Duration::from_secs(45),
        }
    }
}

/// Resource cleanup configuration for integration tests.
///
/// This struct manages the cleanup of test resources including repositories,
/// webhooks, and temporary files created during test execution.
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    /// Whether to perform cleanup after test completion
    pub cleanup_enabled: bool,
    /// Timeout for cleanup operations
    pub cleanup_timeout: Duration,
    /// Whether to force cleanup even if tests fail
    pub force_cleanup_on_failure: bool,
    /// Maximum age of resources before forced cleanup
    pub max_resource_age: Duration,
    /// Cleanup strategy for different resource types
    pub cleanup_strategies: HashMap<String, CleanupStrategy>,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert("repositories".to_string(), CleanupStrategy::Immediate);
        strategies.insert("webhooks".to_string(), CleanupStrategy::Immediate);
        strategies.insert("temp_files".to_string(), CleanupStrategy::Immediate);

        Self {
            cleanup_enabled: true,
            cleanup_timeout: Duration::from_secs(30),
            force_cleanup_on_failure: true,
            max_resource_age: Duration::from_secs(3600), // 1 hour
            cleanup_strategies: strategies,
        }
    }
}

/// Strategy for cleaning up different types of test resources.
#[derive(Debug, Clone)]
pub enum CleanupStrategy {
    /// Clean up immediately after test completion
    Immediate,
    /// Clean up after a delay to allow for debugging
    Delayed(Duration),
    /// Only clean up resources older than specified age
    AgeBasedOnly,
    /// Skip cleanup (for debugging purposes)
    Skip,
}

/// Environment variable isolation configuration for test execution.
///
/// This struct manages environment variable isolation between tests to prevent
/// tests from interfering with each other through shared environment state.
#[derive(Debug, Clone)]
pub struct EnvironmentIsolation {
    /// Whether to isolate environment variables between tests
    pub isolation_enabled: bool,
    /// List of environment variables to preserve across tests
    pub preserve_variables: Vec<String>,
    /// List of environment variables to clear before each test
    pub clear_variables: Vec<String>,
    /// Prefix for test-specific environment variables
    pub test_variable_prefix: String,
}

impl Default for EnvironmentIsolation {
    fn default() -> Self {
        Self {
            isolation_enabled: true,
            preserve_variables: vec![
                "GITHUB_ACTIONS".to_string(),
                "RUNNER_OS".to_string(),
                "CI".to_string(),
            ],
            clear_variables: vec![
                "GITHUB_TEST_TOKEN".to_string(),
                "GITHUB_TEST_APP_ID".to_string(),
                "GITHUB_TEST_PRIVATE_KEY".to_string(),
                "GITHUB_TEST_WEBHOOK_SECRET".to_string(),
                "USE_MOCK_SERVICES".to_string(),
            ],
            test_variable_prefix: "TEST_".to_string(),
        }
    }
}

/// Retry configuration for handling flaky tests in CI environments.
///
/// This struct provides configuration for automatically retrying failed tests
/// that may fail due to transient network issues, API rate limits, or other
/// temporary problems common in CI environments.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Whether retry is enabled
    pub retry_enabled: bool,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay between retry attempts
    pub base_delay: Duration,
    /// Maximum delay between retry attempts (for exponential backoff)
    pub max_delay: Duration,
    /// Whether to use exponential backoff for retry delays
    pub exponential_backoff: bool,
    /// Types of errors that should trigger a retry
    pub retryable_errors: Vec<RetryableErrorType>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            retry_enabled: true,
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            exponential_backoff: true,
            retryable_errors: vec![
                RetryableErrorType::NetworkError,
                RetryableErrorType::GitHubApiRateLimit,
                RetryableErrorType::TemporaryServiceUnavailable,
            ],
        }
    }
}

/// Types of errors that can be automatically retried.
#[derive(Debug, Clone, PartialEq)]
pub enum RetryableErrorType {
    /// Network connectivity issues
    NetworkError,
    /// GitHub API rate limit exceeded
    GitHubApiRateLimit,
    /// Temporary service unavailability
    TemporaryServiceUnavailable,
    /// Webhook delivery timeout
    WebhookTimeout,
    /// Repository creation conflicts
    RepositoryConflict,
}

/// Test execution coordinator for CI environments.
///
/// The `CiTestExecutor` manages the execution of integration tests in CI
/// environments with proper resource management, parallel execution control,
/// and failure handling.
///
/// # Features
///
/// - **Parallel Execution**: Controls parallel test execution with resource limits
/// - **Resource Management**: Manages GitHub API usage and cleanup
/// - **Failure Handling**: Provides retry mechanisms and failure reporting
/// - **Environment Isolation**: Ensures tests don't interfere with each other
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{CiTestExecutor, CiTestConfig, TestError};
///
/// #[tokio::test]
/// async fn test_ci_execution() -> Result<(), TestError> {
///     let config = CiTestConfig::for_github_actions().await?;
///     let executor = CiTestExecutor::new(config).await?;
///
///     let results = executor.run_integration_tests().await?;
///     assert!(results.all_passed());
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct CiTestExecutor {
    /// CI configuration
    config: CiTestConfig,
    /// Test execution state
    execution_state: TestExecutionState,
    /// Resource usage tracking
    resource_usage: ResourceUsageTracker,
}

impl CiTestExecutor {
    /// Creates a new CI test executor with the given configuration.
    ///
    /// # Parameters
    ///
    /// - `config`: CI test configuration for execution parameters
    ///
    /// # Returns
    ///
    /// A configured `CiTestExecutor` ready to run integration tests.
    ///
    /// # Errors
    ///
    /// Returns `TestError::CiConfigurationError` if the configuration is invalid
    /// or the execution environment cannot be properly initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{CiTestExecutor, CiTestConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_executor_creation() -> Result<(), TestError> {
    ///     let config = CiTestConfig::for_local_development().await?;
    ///     let executor = CiTestExecutor::new(config).await?;
    ///
    ///     assert!(executor.is_ready());
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(config: CiTestConfig) -> TestResult<Self> {
        // TODO: implement - Initialize CI test executor
        todo!("Initialize CI test executor with configuration")
    }

    /// Runs all integration tests with proper resource management.
    ///
    /// This method executes the full integration test suite with parallel
    /// execution control, resource usage monitoring, and automatic retry
    /// of failed tests according to the configuration.
    ///
    /// # Returns
    ///
    /// A `TestExecutionResults` containing detailed results of all test runs
    /// including pass/fail status, execution times, and resource usage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{CiTestExecutor, CiTestConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_full_integration_suite() -> Result<(), TestError> {
    ///     let config = CiTestConfig::for_github_actions().await?;
    ///     let executor = CiTestExecutor::new(config).await?;
    ///
    ///     let results = executor.run_integration_tests().await?;
    ///
    ///     println!("Tests passed: {}", results.passed_count());
    ///     println!("Tests failed: {}", results.failed_count());
    ///     println!("Total execution time: {:?}", results.total_execution_time());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_integration_tests(&mut self) -> TestResult<TestExecutionResults> {
        // TODO: implement - Execute full integration test suite
        todo!("Execute integration tests with resource management")
    }

    /// Runs a specific subset of integration tests.
    ///
    /// # Parameters
    ///
    /// - `test_filter`: Pattern to match test names to execute
    ///
    /// # Returns
    ///
    /// A `TestExecutionResults` containing results for the filtered test set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{CiTestExecutor, CiTestConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_filtered_execution() -> Result<(), TestError> {
    ///     let config = CiTestConfig::for_local_development().await?;
    ///     let mut executor = CiTestExecutor::new(config).await?;
    ///
    ///     let results = executor.run_filtered_tests("environment_*").await?;
    ///     assert!(results.test_count() > 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_filtered_tests(
        &mut self,
        test_filter: &str,
    ) -> TestResult<TestExecutionResults> {
        // TODO: implement - Execute filtered integration tests
        todo!("Execute filtered integration tests")
    }

    /// Checks if the executor is ready to run tests.
    ///
    /// # Returns
    ///
    /// `true` if the executor is properly configured and ready to run tests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::{CiTestExecutor, CiTestConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = CiTestConfig::for_local_development().await?;
    /// let executor = CiTestExecutor::new(config).await?;
    /// assert!(executor.is_ready());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_ready(&self) -> bool {
        // TODO: implement - Check if executor is ready
        todo!("Check if executor is ready")
    }
}

/// State tracking for test execution progress.
#[derive(Debug)]
pub struct TestExecutionState {
    /// Number of tests currently running
    pub running_tests: usize,
    /// Number of tests completed
    pub completed_tests: usize,
    /// Number of tests failed
    pub failed_tests: usize,
    /// Start time of test execution
    pub start_time: std::time::Instant,
}

/// Resource usage tracking for CI test execution.
#[derive(Debug)]
pub struct ResourceUsageTracker {
    /// GitHub API requests made
    pub github_api_requests: u32,
    /// Repositories created during testing
    pub repositories_created: u32,
    /// Webhooks configured during testing
    pub webhooks_created: u32,
    /// Peak memory usage during testing
    pub peak_memory_usage: u64,
}

/// Results of integration test execution.
///
/// This struct contains comprehensive results from integration test execution
/// including individual test results, timing information, and resource usage.
#[derive(Debug)]
pub struct TestExecutionResults {
    /// Individual test results
    pub test_results: Vec<IndividualTestResult>,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Resource usage during execution
    pub resource_usage: ResourceUsageTracker,
    /// Configuration used for execution
    pub execution_config: CiTestConfig,
}

impl TestExecutionResults {
    /// Gets the number of tests that passed.
    ///
    /// # Returns
    ///
    /// The count of tests that completed successfully.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::{TestExecutionResults, IndividualTestResult};
    /// # use std::time::Duration;
    /// # fn example(results: &TestExecutionResults) {
    /// println!("Passed: {}/{}", results.passed_count(), results.test_count());
    /// # }
    /// ```
    pub fn passed_count(&self) -> usize {
        self.test_results
            .iter()
            .filter(|result| result.status == TestStatus::Passed)
            .count()
    }

    /// Gets the number of tests that failed.
    ///
    /// # Returns
    ///
    /// The count of tests that failed or errored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::{TestExecutionResults, IndividualTestResult};
    /// # fn example(results: &TestExecutionResults) {
    /// if results.failed_count() > 0 {
    ///     println!("Some tests failed - see details for debugging");
    /// }
    /// # }
    /// ```
    pub fn failed_count(&self) -> usize {
        self.test_results
            .iter()
            .filter(|result| {
                matches!(
                    result.status,
                    TestStatus::Failed | TestStatus::Error | TestStatus::TimedOut
                )
            })
            .count()
    }

    /// Gets the total number of tests executed.
    ///
    /// # Returns
    ///
    /// The total count of all tests that were run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::TestExecutionResults;
    /// # fn example(results: &TestExecutionResults) {
    /// println!("Executed {} tests total", results.test_count());
    /// # }
    /// ```
    pub fn test_count(&self) -> usize {
        self.test_results.len()
    }

    /// Gets whether all tests passed.
    ///
    /// # Returns
    ///
    /// `true` if all tests passed, `false` if any tests failed or errored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::TestExecutionResults;
    /// # fn example(results: &TestExecutionResults) {
    /// if results.all_passed() {
    ///     println!("All tests passed successfully!");
    /// } else {
    ///     std::process::exit(1);
    /// }
    /// # }
    /// ```
    pub fn all_passed(&self) -> bool {
        self.failed_count() == 0 && self.test_count() > 0
    }

    /// Gets the total execution time.
    ///
    /// # Returns
    ///
    /// The duration from start to completion of all tests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::TestExecutionResults;
    /// # fn example(results: &TestExecutionResults) {
    /// println!("Tests completed in {:?}", results.total_execution_time());
    /// # }
    /// ```
    pub fn total_execution_time(&self) -> Duration {
        self.total_execution_time
    }
}

/// Result of an individual integration test.
#[derive(Debug)]
pub struct IndividualTestResult {
    /// Name of the test
    pub test_name: String,
    /// Test execution status
    pub status: TestStatus,
    /// Time taken to execute the test
    pub execution_time: Duration,
    /// Error message if test failed
    pub error_message: Option<String>,
    /// Number of retry attempts made
    pub retry_attempts: u32,
    /// Resource usage for this specific test
    pub resource_usage: ResourceUsageTracker,
}

/// Status of an individual test execution.
#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    /// Test passed successfully
    Passed,
    /// Test failed with assertion error
    Failed,
    /// Test encountered an error during execution
    Error,
    /// Test was skipped
    Skipped,
    /// Test timed out
    TimedOut,
}
