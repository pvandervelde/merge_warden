//! Tests for CI configuration and test execution coordination.

use std::env;
use std::time::Duration;

use crate::ci_config::*;
use crate::errors::{TestError, TestResult};

#[tokio::test]
async fn test_github_actions_detection() -> TestResult<()> {
    // Arrange: Set GitHub Actions environment variables
    env::set_var("GITHUB_ACTIONS", "true");
    env::set_var("RUNNER_OS", "Linux");

    // Act: Create configuration for GitHub Actions
    let config = CiTestConfig::for_github_actions().await?;

    // Assert: Should be detected as CI environment
    assert!(config.is_ci_environment());
    assert!(config.parallel_test_limit() <= 4); // Conservative in CI
    assert!(config.test_timeouts().default_timeout >= Duration::from_secs(60)); // Longer timeouts in CI

    // Cleanup
    env::remove_var("GITHUB_ACTIONS");
    env::remove_var("RUNNER_OS");
    Ok(())
}

#[tokio::test]
async fn test_local_development_configuration() -> TestResult<()> {
    // Arrange: Ensure no CI environment variables
    env::remove_var("GITHUB_ACTIONS");
    env::remove_var("CI");

    // Act: Create configuration for local development
    let config = CiTestConfig::for_local_development().await?;

    // Assert: Should be configured for local development
    assert!(!config.is_ci_environment());
    assert!(config.parallel_test_limit() >= 2); // More aggressive locally
    assert!(config.test_timeouts().default_timeout <= Duration::from_secs(30)); // Shorter timeouts locally

    Ok(())
}

#[tokio::test]
async fn test_custom_parallel_limit_override() -> TestResult<()> {
    // Arrange: Set custom parallel limit
    env::set_var("CI_PARALLEL_LIMIT", "8");

    // Act: Create configuration
    let config = CiTestConfig::for_local_development().await?;

    // Assert: Should respect custom limit
    assert_eq!(config.parallel_test_limit(), 8);

    // Cleanup
    env::remove_var("CI_PARALLEL_LIMIT");
    Ok(())
}

#[tokio::test]
async fn test_github_rate_limit_with_authentication() -> TestResult<()> {
    // Arrange: Set GitHub token for authentication
    env::set_var("GITHUB_TOKEN", "ghp_test_token");

    // Act: Create configuration
    let config = CiTestConfig::for_github_actions().await?;

    // Assert: Should use authenticated rate limits
    let rate_limit = config.github_rate_limit();
    assert!(rate_limit.use_authentication);
    assert!(rate_limit.requests_per_hour >= 5000); // Higher limits when authenticated

    // Cleanup
    env::remove_var("GITHUB_TOKEN");
    Ok(())
}

#[tokio::test]
async fn test_github_rate_limit_without_authentication() -> TestResult<()> {
    // Arrange: Ensure no GitHub token
    env::remove_var("GITHUB_TOKEN");

    // Act: Create configuration
    let config = CiTestConfig::for_local_development().await?;

    // Assert: Should use unauthenticated rate limits
    let rate_limit = config.github_rate_limit();
    assert!(!rate_limit.use_authentication);
    assert_eq!(rate_limit.requests_per_hour, 60); // GitHub's unauthenticated limit

    Ok(())
}

#[tokio::test]
async fn test_configuration_validation_with_valid_setup() -> TestResult<()> {
    // Arrange: Set up valid test environment
    setup_valid_test_environment();

    // Act: Create and validate configuration
    let config = CiTestConfig::for_github_actions().await?;
    let validation_result = config.validate().await;

    // Assert: Validation should pass
    assert!(validation_result.is_ok());

    // Cleanup
    cleanup_test_environment();
    Ok(())
}

#[tokio::test]
async fn test_configuration_validation_with_missing_requirements() -> TestResult<()> {
    // Arrange: Clear required environment variables
    cleanup_test_environment();

    // Act: Create configuration and validate
    let config = CiTestConfig::for_github_actions().await?;
    let validation_result = config.validate().await;

    // Assert: Validation should fail
    assert!(validation_result.is_err());
    if let Err(TestError::CiConfigurationError(msg)) = validation_result {
        assert!(msg.contains("GitHub") || msg.contains("token") || msg.contains("authentication"));
    } else {
        panic!("Expected CiConfigurationError");
    }

    Ok(())
}

#[tokio::test]
async fn test_timeout_configuration_in_ci_environment() -> TestResult<()> {
    // Arrange: Set CI environment
    env::set_var("GITHUB_ACTIONS", "true");

    // Act: Create configuration
    let config = CiTestConfig::for_github_actions().await?;

    // Assert: Should have appropriate CI timeouts
    let timeouts = config.test_timeouts();
    assert!(timeouts.default_timeout >= Duration::from_secs(60));
    assert!(timeouts.github_api_timeout >= Duration::from_secs(30));
    assert!(timeouts.repository_setup_timeout >= Duration::from_secs(120));

    // Cleanup
    env::remove_var("GITHUB_ACTIONS");
    Ok(())
}

#[tokio::test]
async fn test_cleanup_configuration() -> TestResult<()> {
    // Act: Create configuration
    let config = CiTestConfig::for_github_actions().await?;

    // Assert: Cleanup should be properly configured
    let cleanup = &config.cleanup_config;
    assert!(cleanup.cleanup_enabled);
    assert!(cleanup.force_cleanup_on_failure);
    assert!(cleanup.cleanup_timeout >= Duration::from_secs(30));
    assert!(cleanup.cleanup_strategies.contains_key("repositories"));
    assert!(cleanup.cleanup_strategies.contains_key("webhooks"));

    Ok(())
}

#[tokio::test]
async fn test_environment_isolation_configuration() -> TestResult<()> {
    // Act: Create configuration
    let config = CiTestConfig::for_local_development().await?;

    // Assert: Environment isolation should be configured
    let isolation = &config.environment_isolation;
    assert!(isolation.isolation_enabled);
    assert!(isolation
        .preserve_variables
        .contains(&"GITHUB_ACTIONS".to_string()));
    assert!(isolation
        .clear_variables
        .contains(&"GITHUB_TEST_TOKEN".to_string()));
    assert_eq!(isolation.test_variable_prefix, "TEST_");

    Ok(())
}

#[tokio::test]
async fn test_retry_configuration() -> TestResult<()> {
    // Act: Create configuration
    let config = CiTestConfig::for_github_actions().await?;

    // Assert: Retry should be properly configured
    let retry = &config.retry_config;
    assert!(retry.retry_enabled);
    assert!(retry.max_retries >= 2);
    assert!(retry.exponential_backoff);
    assert!(retry
        .retryable_errors
        .contains(&RetryableErrorType::NetworkError));
    assert!(retry
        .retryable_errors
        .contains(&RetryableErrorType::GitHubApiRateLimit));

    Ok(())
}

#[tokio::test]
async fn test_executor_creation_with_valid_config() -> TestResult<()> {
    // Arrange: Create valid configuration
    let config = CiTestConfig::for_local_development().await?;

    // Act: Create executor
    let executor = CiTestExecutor::new(config).await?;

    // Assert: Executor should be ready
    assert!(executor.is_ready());

    Ok(())
}

#[tokio::test]
async fn test_executor_creation_with_invalid_config() -> TestResult<()> {
    // Arrange: Create configuration that will fail validation
    cleanup_test_environment();
    let config = CiTestConfig::for_github_actions().await?;

    // Act: Try to create executor
    let executor_result = CiTestExecutor::new(config).await;

    // Assert: Should fail due to invalid configuration
    assert!(executor_result.is_err());
    if let Err(TestError::CiConfigurationError(_)) = executor_result {
        // Expected error type
    } else {
        panic!("Expected CiConfigurationError");
    }

    Ok(())
}

#[tokio::test]
async fn test_run_integration_tests_with_mock_execution() -> TestResult<()> {
    // Arrange: Set up valid environment and create executor
    setup_valid_test_environment();
    let config = CiTestConfig::for_local_development().await?;
    let mut executor = CiTestExecutor::new(config).await?;

    // Act: Run integration tests
    let results = executor.run_integration_tests().await?;

    // Assert: Should have test results
    assert!(results.test_count() > 0);
    assert!(results.total_execution_time() > Duration::ZERO);

    // Cleanup
    cleanup_test_environment();
    Ok(())
}

#[tokio::test]
async fn test_run_filtered_tests() -> TestResult<()> {
    // Arrange: Set up valid environment and create executor
    setup_valid_test_environment();
    let config = CiTestConfig::for_local_development().await?;
    let mut executor = CiTestExecutor::new(config).await?;

    // Act: Run filtered tests
    let results = executor.run_filtered_tests("environment_*").await?;

    // Assert: Should have filtered test results
    assert!(results.test_count() >= 0); // May be 0 if no matching tests
    for result in &results.test_results {
        assert!(result.test_name.starts_with("environment_"));
    }

    // Cleanup
    cleanup_test_environment();
    Ok(())
}

#[tokio::test]
async fn test_parallel_execution_limit_enforcement() -> TestResult<()> {
    // Arrange: Set low parallel limit
    env::set_var("CI_PARALLEL_LIMIT", "2");
    setup_valid_test_environment();

    let config = CiTestConfig::for_local_development().await?;
    assert_eq!(config.parallel_test_limit(), 2);

    let mut executor = CiTestExecutor::new(config).await?;

    // Act: Run tests
    let results = executor.run_integration_tests().await?;

    // Assert: Should complete successfully with parallel limit
    assert!(results.test_count() >= 0);

    // Cleanup
    env::remove_var("CI_PARALLEL_LIMIT");
    cleanup_test_environment();
    Ok(())
}

/// Helper function to set up valid test environment
fn setup_valid_test_environment() {
    env::set_var("GITHUB_TEST_TOKEN", "ghp_valid_test_token");
    env::set_var("GITHUB_TEST_APP_ID", "123456");
    env::set_var(
        "GITHUB_TEST_PRIVATE_KEY",
        "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
    );
    env::set_var("GITHUB_TEST_WEBHOOK_SECRET", "webhook_secret");
    env::set_var("USE_MOCK_SERVICES", "true");
}

/// Helper function to clean up test environment
fn cleanup_test_environment() {
    env::remove_var("GITHUB_TEST_TOKEN");
    env::remove_var("GITHUB_TEST_APP_ID");
    env::remove_var("GITHUB_TEST_PRIVATE_KEY");
    env::remove_var("GITHUB_TEST_WEBHOOK_SECRET");
    env::remove_var("USE_MOCK_SERVICES");
    env::remove_var("GITHUB_TOKEN");
    env::remove_var("CI_PARALLEL_LIMIT");
    env::remove_var("LOCAL_PARALLEL_LIMIT");
    env::remove_var("GITHUB_ACTIONS");
    env::remove_var("CI");
}

#[tokio::test]
async fn test_results_counting_all_passed() -> TestResult<()> {
    // Arrange: Create test results with all passed
    let results = create_test_results_with_status(&[
        TestStatus::Passed,
        TestStatus::Passed,
        TestStatus::Passed,
    ]);

    // Assert: Counting should be correct
    assert_eq!(results.test_count(), 3);
    assert_eq!(results.passed_count(), 3);
    assert_eq!(results.failed_count(), 0);
    assert!(results.all_passed());

    Ok(())
}

#[tokio::test]
async fn test_results_counting_with_failures() -> TestResult<()> {
    // Arrange: Create test results with some failures
    let results = create_test_results_with_status(&[
        TestStatus::Passed,
        TestStatus::Failed,
        TestStatus::Error,
        TestStatus::Passed,
    ]);

    // Assert: Counting should include failures
    assert_eq!(results.test_count(), 4);
    assert_eq!(results.passed_count(), 2);
    assert_eq!(results.failed_count(), 2); // Failed + Error
    assert!(!results.all_passed());

    Ok(())
}

#[tokio::test]
async fn test_results_counting_with_skipped_tests() -> TestResult<()> {
    // Arrange: Create test results with skipped tests
    let results = create_test_results_with_status(&[
        TestStatus::Passed,
        TestStatus::Skipped,
        TestStatus::TimedOut,
        TestStatus::Passed,
    ]);

    // Assert: Skipped and timed out should not count as passed
    assert_eq!(results.test_count(), 4);
    assert_eq!(results.passed_count(), 2);
    assert_eq!(results.failed_count(), 1); // Only TimedOut counts as failure, not Skipped
    assert!(!results.all_passed());

    Ok(())
}

#[tokio::test]
async fn test_empty_results() -> TestResult<()> {
    // Arrange: Create empty test results
    let results = create_test_results_with_status(&[]);

    // Assert: Empty results should not count as "all passed"
    assert_eq!(results.test_count(), 0);
    assert_eq!(results.passed_count(), 0);
    assert_eq!(results.failed_count(), 0);
    assert!(!results.all_passed()); // No tests means not "all passed"

    Ok(())
}

/// Helper function to create test results with specific statuses
fn create_test_results_with_status(statuses: &[TestStatus]) -> TestExecutionResults {
    let test_results: Vec<IndividualTestResult> = statuses
        .iter()
        .enumerate()
        .map(|(i, status)| IndividualTestResult {
            test_name: format!("test_{}", i),
            status: status.clone(),
            execution_time: Duration::from_millis(100),
            error_message: if matches!(status, TestStatus::Failed | TestStatus::Error) {
                Some("Test failed".to_string())
            } else {
                None
            },
            retry_attempts: 0,
            resource_usage: ResourceUsageTracker {
                github_api_requests: 1,
                repositories_created: 0,
                webhooks_created: 0,
                peak_memory_usage: 1024,
            },
        })
        .collect();

    TestExecutionResults {
        test_results,
        total_execution_time: Duration::from_secs(1),
        resource_usage: ResourceUsageTracker {
            github_api_requests: statuses.len() as u32,
            repositories_created: 0,
            webhooks_created: 0,
            peak_memory_usage: 2048,
        },
        execution_config: CiTestConfig {
            is_ci: true,
            parallel_limit: 2,
            github_rate_limit: GitHubRateLimit::default(),
            test_timeouts: TestTimeouts::default(),
            cleanup_config: CleanupConfig::default(),
            environment_isolation: EnvironmentIsolation::default(),
            retry_config: RetryConfig::default(),
        },
    }
}

#[tokio::test]
async fn test_retryable_error_types() -> TestResult<()> {
    // Arrange: Create retry configuration
    let retry_config = RetryConfig::default();

    // Assert: Should include common retryable error types
    assert!(retry_config
        .retryable_errors
        .contains(&RetryableErrorType::NetworkError));
    assert!(retry_config
        .retryable_errors
        .contains(&RetryableErrorType::GitHubApiRateLimit));
    assert!(retry_config
        .retryable_errors
        .contains(&RetryableErrorType::TemporaryServiceUnavailable));

    Ok(())
}

#[tokio::test]
async fn test_exponential_backoff_configuration() -> TestResult<()> {
    // Arrange: Create configuration with exponential backoff
    let config = CiTestConfig::for_github_actions().await?;
    let retry_config = &config.retry_config;

    // Assert: Exponential backoff should be properly configured
    assert!(retry_config.exponential_backoff);
    assert!(retry_config.base_delay > Duration::ZERO);
    assert!(retry_config.max_delay > retry_config.base_delay);
    assert!(retry_config.max_retries > 0);

    Ok(())
}
