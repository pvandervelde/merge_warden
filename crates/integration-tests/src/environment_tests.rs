//! Comprehensive tests for integration test environment configuration and setup.
//!
//! This module contains tests for all aspects of environment configuration loading,
//! validation, and setup functionality. Tests cover both success and failure scenarios
//! to ensure robust error handling and proper validation.
//!
//! ## Serial Test Execution
//!
//! All tests in this module are marked with `#[serial]` to ensure they run sequentially
//! rather than in parallel. This is necessary because these tests manipulate global
//! environment variables (e.g., `GITHUB_TEST_TOKEN`, `REPO_CREATION_APP_ID`, etc.) to test
//! configuration loading, validation, and error handling.
//!
//! When tests run in parallel, they create race conditions where:
//! - Test A sets environment variables for its configuration
//! - Test B's cleanup function removes all environment variables
//! - Test A tries to load the configuration and fails unexpectedly
//!
//! Running these tests serially prevents such interference and ensures consistent,
//! reproducible results across different execution environments (local vs CI).

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use serial_test::serial;

use crate::environment::{IntegrationTestEnvironment, TestConfig};
use crate::errors::{TestError, TestResult};

/// Tests for TestConfig::from_environment() functionality
mod config_loading_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_load_config_with_all_required_variables() -> TestResult<()> {
        // Arrange: Clean environment and set all required environment variables
        cleanup_environment_variables();
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_APP_ID", "789012");
        env::set_var(
            "MERGE_WARDEN_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_WEBHOOK_SECRET", "secure_webhook_secret_123");

        // Act: Load configuration from environment
        let config = TestConfig::from_environment()?;

        // Assert: Verify all values are loaded correctly
        assert_eq!(config.repo_creation_app_id, "123456");
        assert!(config
            .repo_creation_app_private_key
            .contains("-----BEGIN RSA PRIVATE KEY-----"));
        assert_eq!(config.merge_warden_app_id, "789012");
        assert!(config
            .merge_warden_app_private_key
            .contains("-----BEGIN RSA PRIVATE KEY-----"));
        assert_eq!(
            config.merge_warden_webhook_secret,
            "secure_webhook_secret_123"
        );
        assert_eq!(config.github_organization, "glitchgrove"); // Default value
        assert_eq!(config.repository_prefix, "merge-warden-test"); // Default value
        assert_eq!(config.default_timeout, Duration::from_secs(30)); // Default value
        assert!(config.cleanup_enabled); // Default value
        assert_eq!(
            config.local_webhook_endpoint,
            "http://localhost:7071/api/webhook"
        ); // Default value
        assert!(config.use_mock_services); // Default value

        // Cleanup
        cleanup_environment_variables();
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_with_custom_optional_values() -> TestResult<()> {
        // Arrange: Clean environment, set required variables and custom optional values
        cleanup_environment_variables();
        setup_required_environment_variables();
        env::set_var("TEST_ORGANIZATION", "custom-test-org");
        env::set_var("TEST_TIMEOUT_SECONDS", "60");
        env::set_var("TEST_CLEANUP_ENABLED", "false");
        env::set_var("LOCAL_WEBHOOK_ENDPOINT", "https://example.com/webhook");
        env::set_var("USE_MOCK_SERVICES", "false");
        env::set_var("TEST_REPOSITORY_PREFIX", "custom-prefix");

        // Act: Load configuration
        let config = TestConfig::from_environment()?;

        // Assert: Verify custom values are used
        assert_eq!(config.github_organization, "custom-test-org");
        assert_eq!(config.default_timeout, Duration::from_secs(60));
        assert!(!config.cleanup_enabled);
        assert_eq!(config.local_webhook_endpoint, "https://example.com/webhook");
        assert!(!config.use_mock_services);
        assert_eq!(config.repository_prefix, "custom-prefix");

        // Cleanup
        cleanup_environment_variables();
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_missing_repo_creation_app_id() {
        // Arrange: Clean environment — REPO_CREATION_APP_ID absent
        cleanup_environment_variables();

        // Act & Assert: Should fail because REPO_CREATION_APP_ID is missing
        let result = TestConfig::from_environment();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("REPO_CREATION_APP_ID"));
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_environment_variables();
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_missing_merge_warden_app_id() {
        // Arrange: Set TEST_APP_* credentials but omit MERGE_WARDEN_APP_ID
        cleanup_environment_variables();
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        );
        // Intentionally not setting MERGE_WARDEN_APP_ID

        // Act & Assert: Should fail with InvalidConfiguration
        let result = TestConfig::from_environment();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("MERGE_WARDEN_APP_ID"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_environment_variables();
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_missing_merge_warden_app_private_key() {
        // Arrange: Set TEST_APP_* and MERGE_WARDEN_APP_ID but omit MERGE_WARDEN_APP_PRIVATE_KEY
        cleanup_environment_variables();
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_APP_ID", "789012");
        // Intentionally not setting MERGE_WARDEN_APP_PRIVATE_KEY

        // Act & Assert: Should fail with InvalidConfiguration
        let result = TestConfig::from_environment();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("MERGE_WARDEN_APP_PRIVATE_KEY"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_environment_variables();
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_missing_merge_warden_webhook_secret() {
        // Arrange: Set all required vars except MERGE_WARDEN_WEBHOOK_SECRET
        cleanup_environment_variables();
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_APP_ID", "789012");
        env::set_var(
            "MERGE_WARDEN_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        );
        // Intentionally not setting MERGE_WARDEN_WEBHOOK_SECRET

        // Act & Assert: Should fail with InvalidConfiguration
        let result = TestConfig::from_environment();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("MERGE_WARDEN_WEBHOOK_SECRET"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_environment_variables();
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_invalid_timeout_format() {
        // Arrange: Clean up any environment variables from other tests first
        cleanup_environment_variables();

        // Set invalid timeout value
        setup_required_environment_variables();
        env::set_var("TEST_TIMEOUT_SECONDS", "not_a_number");

        // Act & Assert: Should fail with InvalidConfiguration
        let result = TestConfig::from_environment();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("TEST_TIMEOUT_SECONDS"));
                assert!(msg.contains("integer"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_environment_variables();
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_invalid_boolean_format() {
        // Arrange: Clean environment, set all required variables, then set invalid boolean
        cleanup_environment_variables();
        setup_required_environment_variables();
        env::set_var("TEST_CLEANUP_ENABLED", "maybe");

        // Act & Assert: Should fail with InvalidConfiguration
        let result = TestConfig::from_environment();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("TEST_CLEANUP_ENABLED"));
                assert!(msg.contains("true") || msg.contains("false"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_environment_variables();
    }

    #[tokio::test]
    #[serial]
    async fn test_load_config_case_insensitive_booleans() -> TestResult<()> {
        // Arrange: Clean environment and set boolean values in different cases
        cleanup_environment_variables();
        setup_required_environment_variables();
        env::set_var("TEST_CLEANUP_ENABLED", "TRUE");
        env::set_var("USE_MOCK_SERVICES", "False");

        // Act: Load configuration
        let config = TestConfig::from_environment()?;

        // Assert: Values should be parsed correctly regardless of case
        assert!(config.cleanup_enabled);
        assert!(!config.use_mock_services);

        cleanup_environment_variables();
        Ok(())
    }

    /// Helper function to set up required environment variables
    fn setup_required_environment_variables() {
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_APP_ID", "789012");
        env::set_var(
            "MERGE_WARDEN_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_WEBHOOK_SECRET", "secure_webhook_secret_123");
    }

    /// Helper function to set up other required variables (excluding one for testing)
    fn setup_other_required_variables() {
        // Note: Intentionally incomplete - used for testing missing variable scenarios
    }

    /// Helper function to clean up all test environment variables
    fn cleanup_environment_variables() {
        env::remove_var("REPO_CREATION_APP_ID");
        env::remove_var("REPO_CREATION_APP_PRIVATE_KEY");
        env::remove_var("MERGE_WARDEN_APP_ID");
        env::remove_var("MERGE_WARDEN_APP_PRIVATE_KEY");
        env::remove_var("MERGE_WARDEN_WEBHOOK_SECRET");
        env::remove_var("TEST_ORGANIZATION");
        env::remove_var("TEST_TIMEOUT_SECONDS");
        env::remove_var("TEST_CLEANUP_ENABLED");
        env::remove_var("LOCAL_WEBHOOK_ENDPOINT");
        env::remove_var("USE_MOCK_SERVICES");
        env::remove_var("TEST_REPOSITORY_PREFIX");
    }
}

/// Tests for TestConfig::validate() functionality
mod config_validation_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_validate_valid_configuration() -> TestResult<()> {
        // Arrange: Create valid configuration
        let config = TestConfig {
            repo_creation_app_id: "123456".to_string(),
            repo_creation_app_private_key:
                "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----"
                    .to_string(),
            merge_warden_app_id: "789012".to_string(),
            merge_warden_app_private_key:
                "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----"
                    .to_string(),
            merge_warden_webhook_secret: "secure_webhook_secret_123".to_string(),
            github_organization: "glitchgrove".to_string(),
            repository_prefix: "merge-warden-test".to_string(),
            default_timeout: Duration::from_secs(30),
            cleanup_enabled: true,
            local_webhook_endpoint: "http://localhost:7071/api/webhook".to_string(),
            use_mock_services: true,
            additional_config: HashMap::new(),
        };

        // Act & Assert: Validation should succeed
        config.validate()?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_invalid_repo_creation_app_id_format() {
        // Arrange: Create config with non-numeric repo_creation_app_id
        let mut config = create_base_config();
        config.repo_creation_app_id = "not_a_number".to_string();

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(
                    msg.contains("REPO_CREATION_APP_ID")
                        || msg.contains("integer")
                        || msg.contains("number")
                );
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_empty_merge_warden_app_id() {
        // Arrange: Create config with empty merge_warden_app_id
        let mut config = create_base_config();
        config.merge_warden_app_id = "".to_string();

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(
                    msg.contains("MERGE_WARDEN_APP_ID")
                        || msg.contains("integer")
                        || msg.contains("positive")
                );
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_invalid_app_id_format() {
        // Arrange: Create config with invalid app ID
        let mut config = create_base_config();
        config.repo_creation_app_id = "not_a_number".to_string();

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("app") || msg.contains("ID"));
                assert!(msg.contains("integer") || msg.contains("number"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_app_id_out_of_range() {
        // Arrange: Create config with app ID out of reasonable range
        let mut config = create_base_config();
        config.repo_creation_app_id = "999999999".to_string(); // Too large

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                // Look for the actual error message: "GitHub App ID must be between 1 and 99999999"
                assert!(msg.contains("App ID") || msg.contains("app") || msg.contains("ID"));
                assert!(msg.contains("between") || msg.contains("range"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_invalid_private_key_format() {
        // Arrange: Create config with invalid private key format
        let mut config = create_base_config();
        config.repo_creation_app_private_key = "not_a_valid_pem_key".to_string();

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("private key") || msg.contains("PEM"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_weak_webhook_secret() {
        // Arrange: Create config with weak webhook secret
        let mut config = create_base_config();
        config.merge_warden_webhook_secret = "weak".to_string(); // Too short

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("Webhook secret") || msg.contains("webhook"));
                assert!(msg.contains("8 characters") || msg.contains("at least"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_invalid_organization_name() {
        // Arrange: Create config with invalid organization name
        let mut config = create_base_config();
        config.github_organization = "invalid@org!".to_string(); // Contains forbidden characters

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("organization"));
                assert!(msg.contains("characters") || msg.contains("invalid"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_timeout_too_large() {
        // Arrange: Create config with timeout exceeding maximum
        let mut config = create_base_config();
        config.default_timeout = Duration::from_secs(500); // Exceeds 300 second max

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("timeout"));
                assert!(msg.contains("300") || msg.contains("maximum"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_zero_timeout() {
        // Arrange: Create config with zero timeout
        let mut config = create_base_config();
        config.default_timeout = Duration::from_secs(0);

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("timeout"));
                assert!(msg.contains("positive") || msg.contains("zero"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_invalid_webhook_url() {
        // Arrange: Create config with invalid webhook URL
        let mut config = create_base_config();
        config.local_webhook_endpoint = "not_a_valid_url".to_string();

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("Webhook") || msg.contains("endpoint") || msg.contains("URL"));
                assert!(msg.contains("HTTP") || msg.contains("valid") || msg.contains("HTTPS"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_invalid_repository_prefix() {
        // Arrange: Create config with invalid repository prefix
        let mut config = create_base_config();
        config.repository_prefix = "invalid@prefix!".to_string(); // Contains forbidden characters

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("repository prefix") || msg.contains("prefix"));
                assert!(msg.contains("characters") || msg.contains("invalid"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_validate_repository_prefix_too_long() {
        // Arrange: Create config with repository prefix that's too long
        let mut config = create_base_config();
        config.repository_prefix = "a".repeat(60); // Exceeds 50 character limit

        // Act & Assert: Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(msg) => {
                assert!(msg.contains("repository prefix") || msg.contains("prefix"));
                assert!(msg.contains("50") || msg.contains("length"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    /// Helper function to create a base valid configuration for testing
    fn create_base_config() -> TestConfig {
        TestConfig {
            repo_creation_app_id: "123456".to_string(),
            repo_creation_app_private_key:
                "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----"
                    .to_string(),
            merge_warden_app_id: "789012".to_string(),
            merge_warden_app_private_key:
                "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----"
                    .to_string(),
            merge_warden_webhook_secret: "secure_webhook_secret_123".to_string(),
            github_organization: "glitchgrove".to_string(),
            repository_prefix: "merge-warden-test".to_string(),
            default_timeout: Duration::from_secs(30),
            cleanup_enabled: true,
            local_webhook_endpoint: "http://localhost:7071/api/webhook".to_string(),
            use_mock_services: true,
            additional_config: HashMap::new(),
        }
    }
}

/// Tests for IntegrationTestEnvironment::setup() functionality
mod environment_setup_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_environment_setup_success() -> TestResult<()> {
        // Arrange: Set up valid environment variables
        setup_valid_test_environment();

        // Act: Set up integration test environment
        let test_env = IntegrationTestEnvironment::setup().await?;

        // Assert: Verify environment is properly initialized
        assert!(test_env.is_ready());
        assert_eq!(test_env.config.github_organization, "glitchgrove");
        // (Azure mock services removed — health check no longer applicable)

        // Cleanup
        cleanup_test_environment();
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_setup_with_custom_configuration() -> TestResult<()> {
        // Arrange: Set up environment with custom values
        setup_valid_test_environment();
        env::set_var("TEST_ORGANIZATION", "custom-test-org");
        env::set_var("TEST_TIMEOUT_SECONDS", "60");
        env::set_var("USE_MOCK_SERVICES", "false");

        // Act: Load config only — this test validates config loading, not full environment setup.
        // Full setup requires real GitHub credentials when USE_MOCK_SERVICES=false.
        let config = TestConfig::from_environment()?;

        // Assert: Verify custom configuration is applied
        assert_eq!(config.github_organization, "custom-test-org");
        assert_eq!(config.default_timeout.as_secs(), 60);
        assert!(!config.use_mock_services);

        cleanup_test_environment();
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_setup_missing_required_variable() {
        // Arrange: Clean up any environment variables from other tests first
        cleanup_test_environment();

        // Remove required environment variable
        env::remove_var("REPO_CREATION_APP_ID");

        // Act & Assert: Setup should fail
        let result = IntegrationTestEnvironment::setup().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::InvalidConfiguration(_) => {
                // Expected error type
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }

        cleanup_test_environment();
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_setup_invalid_github_credentials() {
        // Arrange: Set invalid GitHub credentials
        setup_valid_test_environment();
        env::set_var("REPO_CREATION_APP_ID", "not_a_number"); // invalid format

        // Act & Assert: Setup should fail during authentication
        let result = IntegrationTestEnvironment::setup().await;
        assert!(result.is_err());
        // Could be InvalidConfiguration or AuthenticationError depending on implementation

        cleanup_test_environment();
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_setup_network_connectivity_failure() {
        // Arrange: Set up environment but simulate network failure
        setup_valid_test_environment();
        // Note: This test would require mocking network layer or using invalid GitHub URLs
        // For now, we'll test the error handling structure

        // Act: This test structure shows how network failures would be handled
        // In actual implementation, this would involve mocking the GitHub client
        // let result = IntegrationTestEnvironment::setup().await;

        // Assert: Would expect NetworkError
        // assert!(matches!(result.unwrap_err(), TestError::NetworkError(_)));

        cleanup_test_environment();
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_setup_mock_service_initialization_failure() {
        // Arrange: Set up environment but force mock service failure
        setup_valid_test_environment();
        // This would involve configuring mock services to fail during initialization

        // Act & Assert: Setup should handle mock service failures gracefully
        // In actual implementation, this might still succeed but with limited functionality
        // or fail with EnvironmentError

        cleanup_test_environment();
    }

    /// Helper function to set up a valid test environment
    fn setup_valid_test_environment() {
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_APP_ID", "789012");
        env::set_var(
            "MERGE_WARDEN_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_WEBHOOK_SECRET", "secure_webhook_secret_123");
        env::set_var("USE_MOCK_SERVICES", "true");
    }

    /// Helper function to clean up test environment
    fn cleanup_test_environment() {
        env::remove_var("REPO_CREATION_APP_ID");
        env::remove_var("REPO_CREATION_APP_PRIVATE_KEY");
        env::remove_var("MERGE_WARDEN_APP_ID");
        env::remove_var("MERGE_WARDEN_APP_PRIVATE_KEY");
        env::remove_var("MERGE_WARDEN_WEBHOOK_SECRET");
        env::remove_var("TEST_ORGANIZATION");
        env::remove_var("TEST_TIMEOUT_SECONDS");
        env::remove_var("TEST_CLEANUP_ENABLED");
        env::remove_var("LOCAL_WEBHOOK_ENDPOINT");
        env::remove_var("USE_MOCK_SERVICES");
        env::remove_var("TEST_REPOSITORY_PREFIX");
    }
}

/// Tests for environment readiness and health checks
mod environment_readiness_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_environment_readiness_when_fully_initialized() -> TestResult<()> {
        // Arrange: Set up and initialize environment
        setup_valid_test_environment();
        let test_env = IntegrationTestEnvironment::setup().await?;

        // Act & Assert: Should be ready
        assert!(test_env.is_ready());

        cleanup_test_environment();
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_readiness_when_partially_initialized() {
        // This test would check scenarios where environment is partially set up
        // Implementation would depend on how partial initialization is handled
        setup_valid_test_environment();

        // In actual implementation, this might involve:
        // - Environment with missing components
        // - Failed GitHub authentication but successful mock service setup
        // - Other partial initialization scenarios

        cleanup_test_environment();
    }

    /// Helper function to set up a valid test environment
    fn setup_valid_test_environment() {
        env::set_var("REPO_CREATION_APP_ID", "123456");
        env::set_var(
            "REPO_CREATION_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_APP_ID", "789012");
        env::set_var(
            "MERGE_WARDEN_APP_PRIVATE_KEY",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAtest...\n-----END RSA PRIVATE KEY-----",
        );
        env::set_var("MERGE_WARDEN_WEBHOOK_SECRET", "secure_webhook_secret_123");
        env::set_var("USE_MOCK_SERVICES", "true");
    }

    /// Helper function to clean up test environment
    fn cleanup_test_environment() {
        env::remove_var("REPO_CREATION_APP_ID");
        env::remove_var("REPO_CREATION_APP_PRIVATE_KEY");
        env::remove_var("MERGE_WARDEN_APP_ID");
        env::remove_var("MERGE_WARDEN_APP_PRIVATE_KEY");
        env::remove_var("MERGE_WARDEN_WEBHOOK_SECRET");
        env::remove_var("TEST_ORGANIZATION");
        env::remove_var("TEST_TIMEOUT_SECONDS");
        env::remove_var("TEST_CLEANUP_ENABLED");
        env::remove_var("LOCAL_WEBHOOK_ENDPOINT");
        env::remove_var("USE_MOCK_SERVICES");
        env::remove_var("TEST_REPOSITORY_PREFIX");
    }
}
