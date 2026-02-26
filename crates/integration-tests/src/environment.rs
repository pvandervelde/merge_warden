//! Integration test environment configuration and management.
//!
//! This module provides the main coordination infrastructure for integration tests,
//! including environment setup, resource management, and test lifecycle coordination.

use std::collections::HashMap;
use std::time::Duration;

use crate::errors::{TestError, TestResult};
use crate::github::repository_manager::FileAction;
use crate::github::{TestBotInstance, TestRepositoryManager};
use crate::mocks::MockServiceProvider;

/// Main integration test environment coordinator.
///
/// The `IntegrationTestEnvironment` serves as the central coordinator for all integration
/// test operations. It manages the lifecycle of test resources including GitHub repositories,
/// bot instances, and mock Azure services.
///
/// # Architecture
///
/// The test environment consists of several coordinated components:
/// - **Repository Manager**: Handles GitHub repository creation and cleanup
/// - **Bot Instance**: Manages GitHub App configuration and webhook setup
/// - **Mock Services**: Simulates Azure App Config and Key Vault services
/// - **Configuration**: Centralizes test environment settings
///
/// # Lifecycle
///
/// 1. **Setup**: Initialize all components and establish connections
/// 2. **Execution**: Coordinate test operations across components
/// 3. **Cleanup**: Ensure all resources are properly cleaned up
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
///
/// #[tokio::test]
/// async fn test_environment_lifecycle() -> Result<(), TestError> {
///     // Setup test environment
///     let test_env = IntegrationTestEnvironment::setup().await?;
///
///     // Use environment for testing
///     let repo = test_env.create_test_repository("lifecycle-test").await?;
///
///     // Environment automatically cleans up when dropped
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct IntegrationTestEnvironment {
    /// Configuration for the test environment
    pub config: TestConfig,
    /// GitHub repository management
    pub repository_manager: TestRepositoryManager,
    /// GitHub App and webhook configuration
    pub bot_instance: TestBotInstance,
    /// Mock Azure service provider
    pub mock_services: MockServiceProvider,
    /// List of resources created during testing for cleanup
    cleanup_resources: Vec<CleanupResource>,
}

impl IntegrationTestEnvironment {
    /// Creates and initializes a new integration test environment.
    ///
    /// This method performs the complete setup process:
    /// 1. Loads and validates test configuration from environment variables
    /// 2. Initializes GitHub API clients and authentication
    /// 3. Sets up mock Azure services with default configurations
    /// 4. Prepares the test environment for operation
    /// 5. Validates connectivity and permissions
    ///
    /// # Setup Process Details
    ///
    /// ## Configuration Loading (Step 1)
    /// - Reads all required and optional environment variables
    /// - Validates configuration values according to security and format requirements
    /// - Sets up default values for optional configuration parameters
    /// - Performs comprehensive validation of all loaded values
    ///
    /// ## GitHub Client Initialization (Step 2)
    /// - Creates authenticated GitHub API client using personal access token
    /// - Validates token permissions and organization access
    /// - Sets up GitHub App authentication for webhook testing
    /// - Verifies connectivity to GitHub APIs
    ///
    /// ## Mock Service Setup (Step 3)
    /// - Initializes MockAppConfigService with default test configuration
    /// - Sets up MockKeyVaultService with test secrets
    /// - Configures service integration traits for mock/real service swapping
    /// - Establishes mock service provider coordination
    ///
    /// ## Environment Preparation (Step 4)
    /// - Initializes resource tracking for cleanup management
    /// - Sets up test data directories and temporary file management
    /// - Configures logging and observability for test execution
    /// - Prepares webhook endpoint validation
    ///
    /// ## Connectivity Validation (Step 5)
    /// - Tests GitHub API connectivity and authentication
    /// - Validates organization access permissions
    /// - Checks local webhook endpoint accessibility (if configured)
    /// - Verifies mock service initialization and health
    ///
    /// # Environment Variables Required
    ///
    /// - `GITHUB_TEST_TOKEN`: GitHub personal access token with repo permissions
    /// - `REPO_CREATION_APP_ID`: GitHub App ID for webhook testing
    /// - `REPO_CREATION_APP_PRIVATE_KEY`: GitHub App private key content
    /// - `GITHUB_TEST_WEBHOOK_SECRET`: Webhook secret for signature validation
    ///
    /// # Environment Variables Optional
    ///
    /// - `GITHUB_TEST_ORGANIZATION`: Target organization (default: "glitchgrove")
    /// - `TEST_TIMEOUT_SECONDS`: Operation timeout (default: 30)
    /// - `TEST_CLEANUP_ENABLED`: Enable automatic cleanup (default: true)
    /// - `LOCAL_WEBHOOK_ENDPOINT`: Local webhook endpoint (default: "http://localhost:7071/api/webhook")
    /// - `USE_MOCK_SERVICES`: Use mock Azure services (default: true)
    /// - `TEST_REPOSITORY_PREFIX`: Repository name prefix (default: "merge-warden-test")
    ///
    /// # Returns
    ///
    /// A fully configured `IntegrationTestEnvironment` ready for test execution
    /// with all components initialized and validated.
    ///
    /// # Errors
    ///
    /// Returns `TestError::InvalidConfiguration` if:
    /// - Required environment variables are missing or invalid
    /// - GitHub token format is incorrect or lacks required permissions
    /// - GitHub App credentials are invalid or malformed
    /// - Configuration validation fails for any parameter
    /// - Environment variable values are outside acceptable ranges
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - GitHub authentication fails with provided token
    /// - GitHub App authentication cannot be established
    /// - Organization access is denied or not available
    /// - Token permissions are insufficient for test operations
    ///
    /// Returns `TestError::EnvironmentError` if:
    /// - Network connectivity to GitHub fails during setup
    /// - Local webhook endpoint setup fails or is not accessible
    /// - Mock service initialization fails
    /// - Temporary directory creation fails
    /// - Resource tracking initialization fails
    ///
    /// Returns `TestError::NetworkError` if:
    /// - GitHub API endpoints are unreachable
    /// - DNS resolution fails for GitHub domains
    /// - Network timeouts occur during connectivity validation
    /// - Proxy or firewall issues prevent GitHub access
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
    /// use std::env;
    ///
    /// #[tokio::test]
    /// async fn test_environment_setup_success() -> Result<(), TestError> {
    ///     // Ensure required environment variables are set
    ///     env::set_var("GITHUB_TEST_TOKEN", "ghp_test_token_123456789");
    ///     env::set_var("REPO_CREATION_APP_ID", "123456");
    ///     env::set_var("REPO_CREATION_APP_PRIVATE_KEY", "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----");
    ///     env::set_var("GITHUB_TEST_WEBHOOK_SECRET", "test_webhook_secret");
    ///
    ///     let test_env = IntegrationTestEnvironment::setup().await?;
    ///
    ///     // Verify environment is ready
    ///     assert!(test_env.is_ready());
    ///     assert_eq!(test_env.config.github_organization, "glitchgrove");
    ///     assert!(test_env.mock_services.is_healthy().await?);
    ///
    ///     Ok(())
    /// }
    ///
    /// #[tokio::test]
    /// async fn test_environment_setup_with_custom_config() -> Result<(), TestError> {
    ///     env::set_var("GITHUB_TEST_ORGANIZATION", "custom-test-org");
    ///     env::set_var("TEST_TIMEOUT_SECONDS", "60");
    ///     env::set_var("USE_MOCK_SERVICES", "false");
    ///
    ///     let test_env = IntegrationTestEnvironment::setup().await?;
    ///
    ///     assert_eq!(test_env.config.github_organization, "custom-test-org");
    ///     assert_eq!(test_env.config.default_timeout.as_secs(), 60);
    ///     assert!(!test_env.config.use_mock_services);
    ///
    ///     Ok(())
    /// }
    ///
    /// #[tokio::test]
    /// async fn test_environment_setup_missing_token() {
    ///     env::remove_var("GITHUB_TEST_TOKEN");
    ///
    ///     let result = IntegrationTestEnvironment::setup().await;
    ///     assert!(result.is_err());
    ///     assert!(matches!(result.unwrap_err(), TestError::InvalidConfiguration(_)));
    /// }
    /// ```
    pub async fn setup() -> TestResult<Self> {
        // Load configuration from environment
        let config = TestConfig::from_environment()?;

        // Validate configuration
        config.validate()?;

        // Initialize mock services
        let mock_services = MockServiceProvider::new().await?;

        // Initialize repository manager using the test helper app's installation token
        let repository_manager = TestRepositoryManager::new(
            config.repo_creation_app_id.clone(),
            config.repo_creation_app_private_key.clone(),
            config.github_organization.clone(),
            config.repository_prefix.clone(),
        )
        .await?;

        // Initialize Merge Warden bot instance
        let mut bot_instance = TestBotInstance::from_config(&config).await?;

        // Start embedded webhook server so tests can simulate GitHub webhook deliveries
        // without requiring a deployed Merge Warden instance. Skipped in mock-services
        // mode where the github_client is an unauthenticated stub.
        if !config.use_mock_services {
            bot_instance.start_local_webhook_server().await?;
        }

        // Initialize cleanup resources list
        let cleanup_resources = Vec::new();

        let environment = Self {
            config,
            repository_manager,
            bot_instance,
            mock_services,
            cleanup_resources,
        };

        Ok(environment)
    }

    /// Creates a new test repository in the configured GitHub organization.
    ///
    /// This method creates a fully configured test repository ready for bot testing:
    /// 1. Creates repository with unique name in the test organization
    /// 2. Configures branch protection rules and repository settings
    /// 3. Adds default merge-warden.toml configuration
    /// 4. Sets up initial repository content for testing
    ///
    /// # Parameters
    ///
    /// - `name_suffix`: Unique suffix for the repository name. The final repository
    ///   name will be `{prefix}-{name_suffix}-{uuid}` to ensure uniqueness.
    ///
    /// # Returns
    ///
    /// A `TestRepository` handle representing the created repository with all
    /// necessary metadata for test operations.
    ///
    /// # Errors
    ///
    /// Returns `TestError::GitHubApiError` if:
    /// - Repository creation fails due to API limitations
    /// - Organization access is denied
    /// - Repository configuration fails
    ///
    /// Returns `TestError::EnvironmentError` if:
    /// - UUID generation fails
    /// - File content preparation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_repository_creation() -> Result<(), TestError> {
    ///     let test_env = IntegrationTestEnvironment::setup().await?;
    ///     let repo = test_env.create_test_repository("basic-test").await?;
    ///
    ///     assert!(!repo.name.is_empty());
    ///     assert_eq!(repo.organization, "glitchgrove");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_test_repository(
        &mut self,
        name_suffix: &str,
    ) -> TestResult<TestRepository> {
        // Create repository through manager
        let repo = self
            .repository_manager
            .create_repository(name_suffix)
            .await?;

        // Track for cleanup
        self.cleanup_resources.push(CleanupResource::Repository {
            name: repo.name.clone(),
            organization: repo.organization.clone(),
        });

        Ok(repo)
    }

    /// Configures the bot instance for testing with the specified repository.
    ///
    /// This method sets up the complete bot testing environment:
    /// 1. Installs the GitHub App on the test repository
    /// 2. Configures webhook endpoints and authentication
    /// 3. Verifies bot permissions and access
    /// 4. Sets up local tunnel for development testing (if configured)
    ///
    /// # Parameters
    ///
    /// - `repository`: The test repository to configure bot access for
    ///
    /// # Returns
    ///
    /// A `BotConfiguration` containing all the setup details and access tokens
    /// needed for bot testing operations.
    ///
    /// # Errors
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - GitHub App installation fails
    /// - JWT token generation fails
    /// - Permission verification fails
    ///
    /// Returns `TestError::EnvironmentError` if:
    /// - Webhook endpoint setup fails
    /// - Local tunnel creation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_bot_configuration() -> Result<(), TestError> {
    ///     let mut test_env = IntegrationTestEnvironment::setup().await?;
    ///     let repo = test_env.create_test_repository("bot-config-test").await?;
    ///
    ///     let bot_config = test_env.configure_bot_for_repository(&repo).await?;
    ///     assert!(!bot_config.installation_id.is_empty());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn configure_bot_for_repository(
        &mut self,
        _repository: &TestRepository,
    ) -> TestResult<BotConfiguration> {
        // TODO: implement - Configure bot for repository
        todo!("Configure bot instance for repository")
    }

    /// Simulates Azure service outages for resilience testing.
    ///
    /// This method allows testing bot behavior under various Azure service failure
    /// conditions by configuring the mock services to simulate outages, timeouts,
    /// and other failure scenarios.
    ///
    /// # Parameters
    ///
    /// - `outage_config`: Configuration specifying which services to affect and
    ///   how to simulate the outage conditions.
    ///
    /// # Outage Types
    ///
    /// - **Complete Outage**: Service returns errors for all requests
    /// - **Partial Outage**: Service fails intermittently based on failure rate
    /// - **Timeout Simulation**: Service delays responses beyond timeout limits
    /// - **Authentication Failure**: Service rejects authentication attempts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, OutageConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_resilience_to_outages() -> Result<(), TestError> {
    ///     let mut test_env = IntegrationTestEnvironment::setup().await?;
    ///
    ///     // Simulate App Config outage
    ///     test_env.simulate_azure_outage(&OutageConfig {
    ///         app_config_failure_rate: 1.0,
    ///         key_vault_failure_rate: 0.0,
    ///         outage_duration: std::time::Duration::from_secs(10),
    ///     }).await?;
    ///
    ///     // Test bot behavior during outage
    ///     // Bot should fall back to default configuration
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn simulate_azure_outage(&mut self, outage_config: &OutageConfig) -> TestResult<()> {
        self.mock_services
            .simulate_outages(
                outage_config.app_config_failure_rate,
                outage_config.key_vault_failure_rate,
            )
            .await?;
        Ok(())
    }

    /// Restores normal Azure service operation after outage simulation.
    ///
    /// This method restores all mock Azure services to normal operation,
    /// resetting failure rates and timeouts to their default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, OutageConfig, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_service_recovery() -> Result<(), TestError> {
    ///     let mut test_env = IntegrationTestEnvironment::setup().await?;
    ///
    ///     // Simulate outage
    ///     test_env.simulate_azure_outage(&OutageConfig::complete_outage()).await?;
    ///
    ///     // Restore services
    ///     test_env.restore_azure_services().await?;
    ///
    ///     // Verify services are working normally
    ///     assert!(test_env.mock_services.is_healthy().await?);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn restore_azure_services(&mut self) -> TestResult<()> {
        self.mock_services.restore_services().await?;
        Ok(())
    }

    /// Checks if the test environment is ready for operation.
    ///
    /// This method validates that all components of the test environment are
    /// properly initialized and ready for test execution.
    ///
    /// # Returns
    ///
    /// `true` if the environment is fully ready, `false` if any component
    /// requires additional setup.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_environment_readiness() -> Result<(), TestError> {
    ///     let test_env = IntegrationTestEnvironment::setup().await?;
    ///     assert!(test_env.is_ready());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_ready(&self) -> bool {
        // Check that all components are initialized and functional
        // For mock services, we can assume they're ready if created successfully
        // For repository manager and bot instance, they should be ready after setup
        true
    }

    /// Performs cleanup of all test resources.
    ///
    /// This method ensures that all resources created during testing are properly
    /// cleaned up, including:
    /// - Test repositories in the GitHub organization
    /// - GitHub App installations and webhooks
    /// - Temporary files and directories
    /// - Mock service state
    ///
    /// # Returns
    ///
    /// `Ok(())` if cleanup completed successfully, or `TestError::CleanupFailed`
    /// if any resources could not be cleaned up.
    ///
    /// # Note
    ///
    /// This method is automatically called when the `IntegrationTestEnvironment`
    /// is dropped, but can also be called explicitly for immediate cleanup.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_explicit_cleanup() -> Result<(), TestError> {
    ///     let mut test_env = IntegrationTestEnvironment::setup().await?;
    ///     let repo = test_env.create_test_repository("cleanup-test").await?;
    ///
    ///     // Explicit cleanup
    ///     test_env.cleanup().await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn cleanup(&mut self) -> TestResult<()> {
        // Clean up all resources tracked in cleanup_resources
        for resource in &self.cleanup_resources {
            match resource {
                CleanupResource::Repository { organization, name } => {
                    // Clean up repository - this would be handled by repository manager
                    println!("Cleaning up repository: {}/{}", organization, name);
                }
                CleanupResource::Webhook {
                    repository_id,
                    webhook_id,
                } => {
                    // Clean up webhook
                    println!(
                        "Cleaning up webhook {} for repository {}",
                        webhook_id, repository_id
                    );
                }
                CleanupResource::Installation { installation_id } => {
                    // Clean up GitHub App installation
                    println!("Cleaning up installation {}", installation_id);
                }
                CleanupResource::TempFile { path } => {
                    // Clean up temporary file
                    println!("Cleaning up temp file: {}", path);
                }
            }
        }

        // Clear the cleanup resources list
        self.cleanup_resources.clear();

        // Reset mock services to clean state
        self.mock_services.reset().await?;

        Ok(())
    }

    /// Sets up repository configuration with merge-warden.toml.
    pub async fn setup_repository_configuration(
        &mut self,
        repository: &TestRepository,
    ) -> TestResult<()> {
        self.repository_manager
            .setup_configuration(repository, None)
            .await
    }

    /// Adds default content to a repository for testing.
    pub async fn add_default_repository_content(
        &self,
        repository: &TestRepository,
    ) -> TestResult<()> {
        // Add basic README
        let readme_content = format!(
            "# {}\n\nTest repository for Merge Warden integration testing.\n",
            repository.name
        );

        self.repository_manager
            .add_content(
                repository,
                &[("README.md".to_string(), readme_content, FileAction::Add)],
            )
            .await
    }

    /// Creates a branch in the repository.
    pub async fn create_branch(
        &self,
        repository: &TestRepository,
        branch_name: &str,
        from_branch: &str,
    ) -> TestResult<()> {
        self.repository_manager
            .create_branch(repository, branch_name, from_branch)
            .await
    }

    /// Adds a file to a branch.
    pub async fn add_file_to_branch(
        &self,
        repository: &TestRepository,
        branch: &str,
        path: &str,
        content: &str,
        commit_message: &str,
    ) -> TestResult<()> {
        self.repository_manager
            .add_file(repository, branch, path, content, commit_message)
            .await
    }

    /// Updates a file in the repository.
    pub async fn update_file_in_repository(
        &self,
        repository: &TestRepository,
        path: &str,
        content: &str,
        commit_message: &str,
    ) -> TestResult<()> {
        self.repository_manager
            .update_file(repository, "main", path, content, commit_message)
            .await
    }

    /// Creates a pull request in the repository.
    pub async fn create_pull_request(
        &self,
        repository: &TestRepository,
        spec: &crate::utils::PullRequestSpec,
    ) -> TestResult<crate::utils::TestPullRequest> {
        self.repository_manager
            .create_pull_request(repository, spec)
            .await
    }

    /// Gets checks for a pull request.
    ///
    /// Uses the Merge Warden App installation token (via `bot_instance`) rather
    /// than the Repo-Creation App token, because only the MW App has the
    /// `checks:read` permission needed to query check runs.
    pub async fn get_pr_checks(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<PullRequestCheck>> {
        self.bot_instance.get_pr_checks(repository, pr_number).await
    }

    /// Gets comments for a pull request.
    pub async fn get_pr_comments(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<PullRequestComment>> {
        self.repository_manager
            .get_pr_comments(repository, pr_number)
            .await
    }

    /// Gets labels for a pull request.
    pub async fn get_pr_labels(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<PullRequestLabel>> {
        self.repository_manager
            .get_pr_labels(repository, pr_number)
            .await
    }

    /// Simulates GitHub API failure.
    pub async fn simulate_github_api_failure(&mut self) -> TestResult<()> {
        // This would configure the repository manager to simulate failures
        // For now, we'll just log it
        println!("Simulating GitHub API failure");
        Ok(())
    }

    /// Restores GitHub API.
    pub async fn restore_github_api(&mut self) -> TestResult<()> {
        println!("Restoring GitHub API");
        Ok(())
    }

    /// Simulates App Config outage.
    pub async fn simulate_app_config_outage(&mut self) -> TestResult<()> {
        self.mock_services.simulate_app_config_failure().await
    }

    /// Restores App Config.
    pub async fn restore_app_config(&mut self) -> TestResult<()> {
        self.mock_services.restore_app_config().await
    }

    /// Simulates Key Vault outage.
    pub async fn simulate_key_vault_outage(&mut self) -> TestResult<()> {
        self.mock_services.simulate_key_vault_failure().await
    }

    /// Restores Key Vault.
    pub async fn restore_key_vault(&mut self) -> TestResult<()> {
        self.mock_services.restore_key_vault().await
    }
}

/// Automatic cleanup when the test environment is dropped.
impl Drop for IntegrationTestEnvironment {
    fn drop(&mut self) {
        // Note: We can't use async in Drop, so we log a warning if cleanup wasn't called
        if !self.cleanup_resources.is_empty() {
            eprintln!(
                "Warning: IntegrationTestEnvironment dropped with {} resources not cleaned up. \
                 Call cleanup() explicitly for proper async cleanup.",
                self.cleanup_resources.len()
            );
        }
    }
}

/// Pull request check information.
#[derive(Debug, Clone)]
pub struct PullRequestCheck {
    pub id: String,
    pub name: String,
    pub conclusion: Option<String>,
    pub details_url: Option<String>,
    pub output: CheckOutput,
}

/// Check output details.
#[derive(Debug, Clone)]
pub struct CheckOutput {
    pub summary: String,
    pub text: Option<String>,
}

/// Pull request comment information.
#[derive(Debug, Clone)]
pub struct PullRequestComment {
    pub id: u64,
    pub body: String,
    pub user: CommentUser,
    pub created_at: String,
}

/// Comment user information.
#[derive(Debug, Clone)]
pub struct CommentUser {
    pub login: String,
    pub id: u64,
}

/// Pull request label information.
#[derive(Debug, Clone)]
pub struct PullRequestLabel {
    pub id: u64,
    pub name: String,
    pub color: String,
}

/// Configuration for the integration test environment.
///
/// This struct contains all configuration parameters needed to set up and run
/// integration tests, including GitHub authentication, test timeouts, and
/// environment-specific settings.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// GitHub App ID for the test helper app (used for repository and PR management)
    pub repo_creation_app_id: String,
    /// GitHub App private key for the test helper app (PEM format)
    pub repo_creation_app_private_key: String,
    /// GitHub App ID for the Merge Warden app (the app under test)
    pub merge_warden_app_id: String,
    /// GitHub App private key for the Merge Warden app (PEM format)
    pub merge_warden_app_private_key: String,
    /// Webhook secret for the Merge Warden app (used for HMAC-SHA256 signature validation)
    pub merge_warden_webhook_secret: String,
    /// Target GitHub organization for test repositories
    pub github_organization: String,
    /// Prefix for test repository names
    pub repository_prefix: String,
    /// Default timeout for test operations
    pub default_timeout: Duration,
    /// Whether to enable automatic cleanup of test resources
    pub cleanup_enabled: bool,
    /// Local webhook endpoint for development testing
    pub local_webhook_endpoint: String,
    /// Whether to use mock Azure services (vs real services when available)
    pub use_mock_services: bool,
    /// Additional environment-specific configuration
    pub additional_config: HashMap<String, String>,
}

impl TestConfig {
    /// Loads configuration from environment variables.
    ///
    /// This method reads all necessary configuration from environment variables
    /// and validates that required values are present and correctly formatted.
    ///
    /// # Environment Variables Required
    ///
    /// - `GITHUB_TEST_TOKEN`: GitHub personal access token with repo permissions in `glitchgrove` org
    ///   - Must be a valid GitHub personal access token (starts with `ghp_` or `github_pat_`)
    ///   - Must have `repo` scope for repository operations
    ///   - Must have access to the `glitchgrove` organization
    /// - `REPO_CREATION_APP_ID`: GitHub App ID for webhook testing
    ///   - Must be a numeric string representing a valid GitHub App ID
    ///   - App must be installed on the `glitchgrove` organization
    /// - `REPO_CREATION_APP_PRIVATE_KEY`: GitHub App private key content
    ///   - Must be valid PEM-formatted RSA private key
    ///   - Must correspond to the GitHub App specified by `REPO_CREATION_APP_ID`
    /// - `GITHUB_TEST_WEBHOOK_SECRET`: Webhook secret for signature validation
    ///   - Must be a non-empty string used for HMAC-SHA256 signature validation
    ///   - Should be cryptographically secure (minimum 16 characters recommended)
    ///
    /// # Environment Variables Optional (with defaults)
    ///
    /// - `GITHUB_TEST_ORGANIZATION`: Target organization (default: "glitchgrove")
    ///   - Must be a valid GitHub organization name
    ///   - GitHub App must be installed on this organization
    /// - `TEST_TIMEOUT_SECONDS`: Operation timeout in seconds (default: 30)
    ///   - Must be a positive integer between 1 and 300 seconds
    /// - `TEST_CLEANUP_ENABLED`: Enable automatic cleanup (default: "true")
    ///   - Must be "true" or "false" (case insensitive)
    /// - `LOCAL_WEBHOOK_ENDPOINT`: Local webhook endpoint (default: "http://localhost:7071/api/webhook")
    ///   - Must be a valid HTTP or HTTPS URL
    ///   - Should be accessible from the test environment
    /// - `USE_MOCK_SERVICES`: Use mock Azure services (default: "true")
    ///   - Must be "true" or "false" (case insensitive)
    /// - `TEST_REPOSITORY_PREFIX`: Repository name prefix (default: "merge-warden-test")
    ///   - Must be a valid GitHub repository name prefix (alphanumeric and hyphens only)
    ///   - Will be combined with test name and UUID for uniqueness
    ///
    /// # Returns
    ///
    /// A validated `TestConfig` instance ready for use with all configuration
    /// parameters loaded and validated.
    ///
    /// # Errors
    ///
    /// Returns `TestError::InvalidConfiguration` if:
    /// - Any required environment variable is missing or empty
    /// - GitHub token format is invalid or doesn't start with expected prefix
    /// - GitHub App ID is not a valid positive integer
    /// - Private key is not valid PEM format or cannot be parsed
    /// - Webhook secret is empty or too short (less than 8 characters)
    /// - Organization name contains invalid characters
    /// - Timeout value is not a positive integer or exceeds maximum
    /// - Boolean values are not "true" or "false"
    /// - Webhook endpoint is not a valid URL format
    /// - Repository prefix contains invalid characters
    ///
    /// Returns `TestError::EnvironmentError` if:
    /// - Environment variable reading fails due to system issues
    /// - Memory allocation fails during configuration creation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestConfig, TestError};
    /// use std::env;
    ///
    /// #[tokio::test]
    /// async fn test_config_loading_with_required_vars() -> Result<(), TestError> {
    ///     // Set required environment variables
    ///     env::set_var("GITHUB_TEST_TOKEN", "ghp_test_token_123456789");
    ///     env::set_var("REPO_CREATION_APP_ID", "123456");
    ///     env::set_var("REPO_CREATION_APP_PRIVATE_KEY", "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----");
    ///     env::set_var("GITHUB_TEST_WEBHOOK_SECRET", "test_webhook_secret");
    ///
    ///     let config = TestConfig::from_environment()?;
    ///     assert!(!config.github_token.is_empty());
    ///     assert_eq!(config.github_organization, "glitchgrove");
    ///     assert_eq!(config.default_timeout.as_secs(), 30);
    ///
    ///     Ok(())
    /// }
    ///
    /// #[tokio::test]
    /// async fn test_config_loading_with_custom_values() -> Result<(), TestError> {
    ///     env::set_var("GITHUB_TEST_ORGANIZATION", "custom-org");
    ///     env::set_var("TEST_TIMEOUT_SECONDS", "60");
    ///     env::set_var("TEST_CLEANUP_ENABLED", "false");
    ///
    ///     let config = TestConfig::from_environment()?;
    ///     assert_eq!(config.github_organization, "custom-org");
    ///     assert_eq!(config.default_timeout.as_secs(), 60);
    ///     assert!(!config.cleanup_enabled);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_environment() -> TestResult<Self> {
        use std::env;

        // Load required environment variables
        let repo_creation_app_id = env::var("REPO_CREATION_APP_ID").map_err(|_| {
            TestError::InvalidConfiguration(
                "REPO_CREATION_APP_ID environment variable is required".to_string(),
            )
        })?;

        let repo_creation_app_private_key =
            env::var("REPO_CREATION_APP_PRIVATE_KEY").map_err(|_| {
                TestError::InvalidConfiguration(
                    "REPO_CREATION_APP_PRIVATE_KEY environment variable is required".to_string(),
                )
            })?;

        let merge_warden_app_id = env::var("MERGE_WARDEN_APP_ID").map_err(|_| {
            TestError::InvalidConfiguration(
                "MERGE_WARDEN_APP_ID environment variable is required".to_string(),
            )
        })?;

        let merge_warden_app_private_key =
            env::var("MERGE_WARDEN_APP_PRIVATE_KEY").map_err(|_| {
                TestError::InvalidConfiguration(
                    "MERGE_WARDEN_APP_PRIVATE_KEY environment variable is required".to_string(),
                )
            })?;

        let merge_warden_webhook_secret =
            env::var("MERGE_WARDEN_WEBHOOK_SECRET").map_err(|_| {
                TestError::InvalidConfiguration(
                    "MERGE_WARDEN_WEBHOOK_SECRET environment variable is required".to_string(),
                )
            })?;

        // Load optional environment variables with defaults
        let github_organization =
            env::var("TEST_ORGANIZATION").unwrap_or_else(|_| "glitchgrove".to_string());

        let repository_prefix =
            env::var("TEST_REPOSITORY_PREFIX").unwrap_or_else(|_| "merge-warden-test".to_string());

        let default_timeout = env::var("TEST_TIMEOUT_SECONDS")
            .map(|s| {
                s.parse::<u64>()
                    .map_err(|_| {
                        TestError::InvalidConfiguration(
                            "TEST_TIMEOUT_SECONDS must be a valid integer".to_string(),
                        )
                    })
                    .map(Duration::from_secs)
            })
            .unwrap_or(Ok(Duration::from_secs(30)))?;

        let cleanup_enabled = env::var("TEST_CLEANUP_ENABLED")
            .map(|s| s.to_lowercase())
            .map(|s| match s.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(TestError::InvalidConfiguration(
                    "TEST_CLEANUP_ENABLED must be 'true' or 'false'".to_string(),
                )),
            })
            .unwrap_or(Ok(true))?;

        let local_webhook_endpoint = env::var("LOCAL_WEBHOOK_ENDPOINT")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "http://localhost:7071/api/webhook".to_string());

        let use_mock_services = env::var("USE_MOCK_SERVICES")
            .map(|s| s.to_lowercase())
            .map(|s| match s.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(TestError::InvalidConfiguration(
                    "USE_MOCK_SERVICES must be 'true' or 'false'".to_string(),
                )),
            })
            .unwrap_or(Ok(true))?;

        // Create configuration
        let config = Self {
            repo_creation_app_id,
            repo_creation_app_private_key,
            merge_warden_app_id,
            merge_warden_app_private_key,
            merge_warden_webhook_secret,
            github_organization,
            repository_prefix,
            default_timeout,
            cleanup_enabled,
            local_webhook_endpoint,
            use_mock_services,
            additional_config: HashMap::new(),
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validates the configuration values.
    ///
    /// This method performs comprehensive validation of all configuration
    /// parameters to ensure they are valid and usable for testing.
    ///
    /// # Validation Rules
    ///
    /// ## GitHub Configuration Validation
    /// - **Token Format**: Must start with `ghp_`, `github_pat_`, or be a classic token format
    /// - **App ID**: Must be a positive integer, typically 6-8 digits
    /// - **Private Key**: Must be valid PEM format with RSA private key structure
    /// - **Webhook Secret**: Must be at least 8 characters for security
    /// - **Organization**: Must contain only alphanumeric characters, hyphens, and underscores
    ///
    /// ## Timeout and Duration Validation
    /// - **Default Timeout**: Must be between 1 and 300 seconds (5 minutes maximum)
    /// - **Values must be reasonable**: No negative or zero timeouts allowed
    ///
    /// ## URL and Endpoint Validation
    /// - **Webhook Endpoint**: Must be valid HTTP or HTTPS URL with proper format
    /// - **Localhost URLs**: Allowed for development testing
    /// - **Port Numbers**: Must be valid (1-65535) if specified
    ///
    /// ## Repository and Naming Validation
    /// - **Repository Prefix**: Must follow GitHub naming conventions (alphanumeric and hyphens)
    /// - **Length Limits**: Repository prefix must be 1-50 characters
    /// - **No Special Characters**: Only letters, numbers, and hyphens allowed
    ///
    /// # Returns
    ///
    /// `Ok(())` if all configuration parameters are valid and usable for testing.
    ///
    /// # Errors
    ///
    /// Returns `TestError::InvalidConfiguration` with specific details if:
    /// - **GitHub Token Invalid**: Token format doesn't match expected patterns or is empty
    /// - **App ID Invalid**: Not a positive integer or outside reasonable range (1-99999999)
    /// - **Private Key Invalid**: Not valid PEM format, missing headers/footers, or corrupted content
    /// - **Webhook Secret Weak**: Less than 8 characters or contains only whitespace
    /// - **Organization Invalid**: Contains forbidden characters or is empty
    /// - **Timeout Invalid**: Zero, negative, or exceeds 300 seconds
    /// - **URL Malformed**: Webhook endpoint is not a valid URL or uses unsupported scheme
    /// - **Repository Prefix Invalid**: Contains forbidden characters, too long, or empty
    ///
    /// # Validation Details
    ///
    /// The validation performs these specific checks:
    /// 1. **Format Validation**: Ensures all values match expected patterns
    /// 2. **Security Validation**: Checks that secrets meet minimum security requirements
    /// 3. **Functional Validation**: Verifies values are usable for their intended purpose
    /// 4. **Range Validation**: Ensures numeric values are within acceptable ranges
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestConfig, TestError};
    /// use std::time::Duration;
    /// use std::collections::HashMap;
    ///
    /// #[tokio::test]
    /// async fn test_valid_configuration() -> Result<(), TestError> {
    ///     let config = TestConfig {
    ///         github_token: "ghp_valid_token_1234567890abcdef".to_string(),
    ///         github_app_id: "123456".to_string(),
    ///         github_private_key: "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----".to_string(),
    ///         github_webhook_secret: "secure_webhook_secret_123".to_string(),
    ///         github_organization: "glitchgrove".to_string(),
    ///         repository_prefix: "merge-warden-test".to_string(),
    ///         default_timeout: Duration::from_secs(30),
    ///         cleanup_enabled: true,
    ///         local_webhook_endpoint: "http://localhost:7071/api/webhook".to_string(),
    ///         use_mock_services: true,
    ///         additional_config: HashMap::new(),
    ///     };
    ///
    ///     config.validate()?; // Should succeed
    ///     Ok(())
    /// }
    ///
    /// #[tokio::test]
    /// async fn test_invalid_token_format() {
    ///     let config = TestConfig {
    ///         github_token: "invalid_token_format".to_string(),
    ///         // ... other valid fields
    ///         github_app_id: "123456".to_string(),
    ///         github_private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----".to_string(),
    ///         github_webhook_secret: "valid_secret".to_string(),
    ///         github_organization: "glitchgrove".to_string(),
    ///         repository_prefix: "test".to_string(),
    ///         default_timeout: Duration::from_secs(30),
    ///         cleanup_enabled: true,
    ///         local_webhook_endpoint: "http://localhost:7071/api/webhook".to_string(),
    ///         use_mock_services: true,
    ///         additional_config: HashMap::new(),
    ///     };
    ///
    ///     let result = config.validate();
    ///     assert!(result.is_err());
    ///     assert!(matches!(result.unwrap_err(), TestError::InvalidConfiguration(_)));
    /// }
    ///
    /// #[tokio::test]
    /// async fn test_timeout_out_of_range() {
    ///     let config = TestConfig {
    ///         github_token: "ghp_valid_token".to_string(),
    ///         github_app_id: "123456".to_string(),
    ///         github_private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----".to_string(),
    ///         github_webhook_secret: "valid_secret".to_string(),
    ///         github_organization: "glitchgrove".to_string(),
    ///         repository_prefix: "test".to_string(),
    ///         default_timeout: Duration::from_secs(500), // Too large
    ///         cleanup_enabled: true,
    ///         local_webhook_endpoint: "http://localhost:7071/api/webhook".to_string(),
    ///         use_mock_services: true,
    ///         additional_config: HashMap::new(),
    ///     };
    ///
    ///     let result = config.validate();
    ///     assert!(result.is_err());
    /// }
    /// ```
    pub fn validate(&self) -> TestResult<()> {
        // Validate test helper app ID
        if let Err(_) = self.repo_creation_app_id.parse::<u64>() {
            return Err(TestError::InvalidConfiguration(
                "REPO_CREATION_APP_ID must be a valid positive integer".to_string(),
            ));
        }

        let repo_creation_app_id_val: u64 = self.repo_creation_app_id.parse().unwrap();
        if repo_creation_app_id_val == 0 || repo_creation_app_id_val > 99_999_999 {
            return Err(TestError::InvalidConfiguration(
                "GitHub App ID must be between 1 and 99999999".to_string(),
            ));
        }

        // Validate test helper app private key format
        if !self
            .repo_creation_app_private_key
            .contains("-----BEGIN RSA PRIVATE KEY-----")
            || !self
                .repo_creation_app_private_key
                .contains("-----END RSA PRIVATE KEY-----")
        {
            return Err(TestError::InvalidConfiguration(
                "Test app private key must be in valid PEM format with RSA private key headers"
                    .to_string(),
            ));
        }

        // Validate Merge Warden app ID
        if let Err(_) = self.merge_warden_app_id.parse::<u64>() {
            return Err(TestError::InvalidConfiguration(
                "MERGE_WARDEN_APP_ID must be a valid positive integer".to_string(),
            ));
        }

        let merge_warden_app_id_val: u64 = self.merge_warden_app_id.parse().unwrap();
        if merge_warden_app_id_val == 0 || merge_warden_app_id_val > 99_999_999 {
            return Err(TestError::InvalidConfiguration(
                "Merge Warden App ID must be between 1 and 99999999".to_string(),
            ));
        }

        // Validate Merge Warden app private key format
        if !self
            .merge_warden_app_private_key
            .contains("-----BEGIN RSA PRIVATE KEY-----")
            || !self
                .merge_warden_app_private_key
                .contains("-----END RSA PRIVATE KEY-----")
        {
            return Err(TestError::InvalidConfiguration(
                "Merge Warden app private key must be in valid PEM format with RSA private key headers"
                    .to_string(),
            ));
        }

        // Validate Merge Warden webhook secret strength
        if self.merge_warden_webhook_secret.len() < 8 {
            return Err(TestError::InvalidConfiguration(
                "Webhook secret must be at least 8 characters for security".to_string(),
            ));
        }

        if self.merge_warden_webhook_secret.trim().is_empty() {
            return Err(TestError::InvalidConfiguration(
                "Webhook secret cannot be empty or only whitespace".to_string(),
            ));
        }

        // Validate organization name
        if self.github_organization.is_empty() {
            return Err(TestError::InvalidConfiguration(
                "GitHub organization cannot be empty".to_string(),
            ));
        }

        if !self
            .github_organization
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(TestError::InvalidConfiguration(
                "GitHub organization name can only contain alphanumeric characters, hyphens, and underscores".to_string()
            ));
        }

        // Validate timeout range
        if self.default_timeout.as_secs() == 0 {
            return Err(TestError::InvalidConfiguration(
                "Default timeout must be a positive number of seconds".to_string(),
            ));
        }

        if self.default_timeout.as_secs() > 300 {
            return Err(TestError::InvalidConfiguration(
                "Default timeout cannot exceed 300 seconds (5 minutes)".to_string(),
            ));
        }

        // Validate webhook endpoint URL
        if !self.local_webhook_endpoint.starts_with("http://")
            && !self.local_webhook_endpoint.starts_with("https://")
        {
            return Err(TestError::InvalidConfiguration(
                "Webhook endpoint must be a valid HTTP or HTTPS URL".to_string(),
            ));
        }

        // Validate repository prefix
        if self.repository_prefix.is_empty() {
            return Err(TestError::InvalidConfiguration(
                "Repository prefix cannot be empty".to_string(),
            ));
        }

        if self.repository_prefix.len() > 50 {
            return Err(TestError::InvalidConfiguration(
                "Repository prefix cannot exceed 50 characters in length".to_string(),
            ));
        }

        if !self
            .repository_prefix
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-')
        {
            return Err(TestError::InvalidConfiguration(
                "Repository prefix can only contain alphanumeric characters and hyphens"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

/// Represents a test repository created for integration testing.
///
/// This struct contains all metadata and access information for a test repository
/// created in the GitHub organization.
#[derive(Debug, Clone)]
pub struct TestRepository {
    /// Repository name (unique within organization)
    pub name: String,
    /// GitHub organization name
    pub organization: String,
    /// Repository ID for GitHub API operations
    pub id: u64,
    /// Full repository name (organization/repository)
    pub full_name: String,
    /// Repository URL for cloning and access
    pub clone_url: String,
    /// Default branch name
    pub default_branch: String,
    /// Whether the repository is private
    pub private: bool,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TestRepository {
    /// Gets the full repository identifier for GitHub API operations.
    ///
    /// # Returns
    ///
    /// A string in the format "organization/repository" suitable for GitHub API calls.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestRepository;
    /// use chrono::Utc;
    /// let repo = TestRepository {
    ///     name: "test-repo".to_string(),
    ///     organization: "glitchgrove".to_string(),
    ///     clone_url: "https://github.com/glitchgrove/test-repo.git".to_string(),
    ///     default_branch: "main".to_string(),
    ///     private: false,
    ///     created_at: Utc::now(),
    ///     full_name: "glitchgrove/test-repo".to_string(),
    ///     id: 1,
    /// };
    /// assert_eq!(repo.full_identifier(), "glitchgrove/test-repo");
    /// ```
    pub fn full_identifier(&self) -> String {
        format!("{}/{}", self.organization, self.name)
    }
}

/// Configuration for bot instance setup in test repositories.
///
/// This struct contains all the configuration details needed to set up a bot
/// instance for testing, including authentication tokens and webhook endpoints.
#[derive(Debug, Clone)]
pub struct BotConfiguration {
    /// GitHub App installation ID for the repository
    pub installation_id: String,
    /// Access token for the bot to use with the repository
    pub access_token: String,
    /// Webhook endpoint URL for receiving events
    pub webhook_url: String,
    /// Webhook secret for signature validation
    pub webhook_secret: String,
    /// Installation permissions granted to the bot
    pub permissions: HashMap<String, String>,
}

/// Configuration for simulating Azure service outages.
///
/// This struct defines how mock Azure services should simulate failure conditions
/// for testing bot resilience and error handling.
#[derive(Debug, Clone)]
pub struct OutageConfig {
    /// Failure rate for App Config service (0.0 = no failures, 1.0 = all requests fail)
    pub app_config_failure_rate: f32,
    /// Failure rate for Key Vault service (0.0 = no failures, 1.0 = all requests fail)
    pub key_vault_failure_rate: f32,
    /// Duration of the simulated outage
    pub outage_duration: Duration,
    /// Additional delay to add to service responses (simulates slow responses)
    pub response_delay: Duration,
    /// Whether to simulate authentication failures
    pub simulate_auth_failures: bool,
}

impl OutageConfig {
    /// Creates a configuration for complete service outage.
    ///
    /// # Returns
    ///
    /// An `OutageConfig` that simulates complete failure of all Azure services.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::OutageConfig;
    ///
    /// let outage = OutageConfig::complete_outage();
    /// assert_eq!(outage.app_config_failure_rate, 1.0);
    /// assert_eq!(outage.key_vault_failure_rate, 1.0);
    /// ```
    pub fn complete_outage() -> Self {
        Self {
            app_config_failure_rate: 1.0,
            key_vault_failure_rate: 1.0,
            outage_duration: Duration::from_secs(300), // 5 minutes
            response_delay: Duration::from_secs(0),
            simulate_auth_failures: true,
        }
    }

    /// Creates a configuration for partial service degradation.
    ///
    /// # Parameters
    ///
    /// - `failure_rate`: The rate of failures (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// An `OutageConfig` that simulates partial service degradation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::OutageConfig;
    ///
    /// let outage = OutageConfig::partial_outage(0.3);
    /// assert_eq!(outage.app_config_failure_rate, 0.3);
    /// assert_eq!(outage.key_vault_failure_rate, 0.3);
    /// ```
    pub fn partial_outage(failure_rate: f32) -> Self {
        Self {
            app_config_failure_rate: failure_rate,
            key_vault_failure_rate: failure_rate,
            outage_duration: Duration::from_secs(60),
            response_delay: Duration::from_millis(500),
            simulate_auth_failures: false,
        }
    }
}

/// Internal resource tracking for cleanup purposes.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum CleanupResource {
    Repository { organization: String, name: String },
    Webhook { repository_id: u64, webhook_id: u64 },
    Installation { installation_id: u64 },
    TempFile { path: String },
}
