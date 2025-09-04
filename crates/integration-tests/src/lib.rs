//! # Merge Warden Integration Tests
//!
//! Comprehensive integration testing infrastructure for the Merge Warden bot system.
//! This crate provides end-to-end validation of bot functionality including GitHub
//! integration, webhook processing, and Azure service interactions.
//!
//! ## Architecture
//!
//! The integration test framework consists of several key components:
//!
//! - **TestEnvironment**: Main coordinator for test execution and resource management
//! - **GitHub Integration**: Repository management and webhook testing using the `glitchgrove` organization
//! - **Mock Services**: Simulation of Azure App Config and Key Vault for reliable testing
//! - **Test Scenarios**: Comprehensive test cases covering core bot functionality
//!
//! ## Usage
//!
//! ```rust
//! use merge_warden_integration_tests::{IntegrationTestEnvironment, TestError};
//!
//! #[tokio::test]
//! async fn test_basic_pull_request_workflow() -> Result<(), TestError> {
//!     let test_env = IntegrationTestEnvironment::setup().await?;
//!     let repo = test_env.create_test_repository("basic-pr-test").await?;
//!
//!     // Test implementation...
//!
//!     test_env.cleanup().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Environment Requirements
//!
//! The following environment variables must be configured for integration tests:
//!
//! - `GITHUB_TEST_TOKEN`: GitHub personal access token with repo permissions in `glitchgrove` org
//! - `GITHUB_TEST_APP_ID`: Test GitHub App ID for webhook testing
//! - `GITHUB_TEST_PRIVATE_KEY`: Test GitHub App private key content
//! - `GITHUB_TEST_WEBHOOK_SECRET`: Webhook secret for signature validation
//! - `GITHUB_TEST_ORGANIZATION`: Target organization for test repositories (default: "glitchgrove")

pub mod ci_config;
pub mod environment;
pub mod errors;
pub mod github;
pub mod mocks;
pub mod utils;

#[cfg(test)]
mod ci_config_tests;
#[cfg(test)]
mod environment_tests;

// Re-export main types for convenient access
pub use ci_config::{
    CiTestConfig, CiTestExecutor, CleanupConfig, EnvironmentIsolation, GitHubRateLimit,
    RetryConfig, TestExecutionResults, TestStatus, TestTimeouts,
};
pub use environment::{
    BotConfiguration, IntegrationTestEnvironment, OutageConfig, TestConfig, TestRepository,
};
pub use errors::{TestError, TestResult};
pub use github::{
    FileAction, FileChange, RepositorySpec, TestBotInstance, TestRepositoryManager, WebhookResponse,
};
pub use mocks::{MockAppConfigService, MockKeyVaultService, MockServiceProvider};
pub use utils::{CommentSpec, PullRequestSpec, ReviewSpec, TestDataManager};

// Re-export utility functions that are used in doc tests
pub use utils::{
    create_webhook_payload, generate_unique_id, retry_operation, validate_webhook_signature,
    wait_for_condition, wait_for_webhook_processing,
};

/// Default timeout for test operations in seconds.
pub const DEFAULT_TEST_TIMEOUT_SECONDS: u64 = 30;

/// Default prefix for test repository names.
pub const DEFAULT_REPOSITORY_PREFIX: &str = "merge-warden-test";

/// Default GitHub organization for integration testing.
pub const DEFAULT_TEST_ORGANIZATION: &str = "glitchgrove";
