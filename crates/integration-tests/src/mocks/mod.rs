//! Mock cloud services for integration testing.
//!
//! This module provides mock implementations of cloud services to enable reliable
//! integration testing without requiring actual cloud infrastructure. Services are
//! organized by cloud provider to support multiple platforms.

pub mod azure;

// Re-export Azure services for convenience
pub use azure::{MockAppConfigService, MockKeyVaultService};

// Future: AWS services will be added here
// pub mod aws;

// Future: GCP services will be added here
// pub mod gcp;

use crate::errors::TestResult;

/// Provider for all mock Azure services used in integration testing.
///
/// The `MockServiceProvider` coordinates multiple mock Azure services and provides
/// a unified interface for configuring service behavior, simulating outages, and
/// managing service state during testing.
///
/// # Features
///
/// - **Service Coordination**: Manages multiple mock services together
/// - **Outage Simulation**: Simulates various failure scenarios across services
/// - **State Management**: Provides consistent state management across services
/// - **Health Monitoring**: Tracks service health and availability
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{MockServiceProvider, TestError};
///
/// #[tokio::test]
/// async fn test_service_coordination() -> Result<(), TestError> {
///     let mut provider = MockServiceProvider::new().await?;
///
///     // Configure services for testing
///     provider.set_app_config_value("test-key", "test-value").await?;
///     provider.set_key_vault_secret("test-secret", "secret-value").await?;
///
///     // Verify services are healthy
///     assert!(provider.is_healthy().await?);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct MockServiceProvider {
    /// Mock Azure App Configuration service
    pub app_config: MockAppConfigService,
    /// Mock Azure Key Vault service
    pub key_vault: MockKeyVaultService,
    /// Overall health status of mock services
    is_healthy: bool,
}

impl MockServiceProvider {
    /// Creates a new mock service provider with default configuration.
    ///
    /// This method initializes all mock services with default test data and
    /// configuration suitable for standard integration testing scenarios.
    ///
    /// # Returns
    ///
    /// A fully configured `MockServiceProvider` ready for testing.
    ///
    /// # Errors
    ///
    /// Returns `TestError::EnvironmentError` if mock services cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_provider_initialization() -> Result<(), TestError> {
    ///     let provider = MockServiceProvider::new().await?;
    ///     assert!(provider.is_healthy().await?);
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> TestResult<Self> {
        let app_config = MockAppConfigService::new();
        let key_vault = MockKeyVaultService::new();

        Ok(MockServiceProvider {
            app_config,
            key_vault,
            is_healthy: true,
        })
    }

    /// Sets a configuration value in the mock App Config service.
    ///
    /// # Parameters
    ///
    /// - `key`: Configuration key name
    /// - `value`: Configuration value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_app_config_setup() -> Result<(), TestError> {
    ///     let mut provider = MockServiceProvider::new().await?;
    ///     provider.set_app_config_value("merge-warden:policies:enabled", "true").await?;
    ///
    ///     let value = provider.app_config.get_configuration("merge-warden:policies:enabled").await?;
    ///     assert_eq!(value, "true");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_app_config_value(&mut self, key: &str, value: &str) -> TestResult<()> {
        self.app_config.set_configuration(key, value);
        Ok(())
    }

    /// Sets a secret in the mock Key Vault service.
    ///
    /// # Parameters
    ///
    /// - `name`: Secret name
    /// - `value`: Secret value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_key_vault_setup() -> Result<(), TestError> {
    ///     let mut provider = MockServiceProvider::new().await?;
    ///     provider.set_key_vault_secret("github-token", "secret-token").await?;
    ///
    ///     let secret = provider.key_vault.get_secret("github-token").await?;
    ///     assert_eq!(secret, "secret-token");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_key_vault_secret(&mut self, name: &str, value: &str) -> TestResult<()> {
        self.key_vault.set_secret(name, value);
        Ok(())
    }

    /// Simulates service outages across all mock services.
    ///
    /// # Parameters
    ///
    /// - `app_config_failure_rate`: Failure rate for App Config (0.0 to 1.0)
    /// - `key_vault_failure_rate`: Failure rate for Key Vault (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_outage_simulation() -> Result<(), TestError> {
    ///     let mut provider = MockServiceProvider::new().await?;
    ///
    ///     // Simulate complete outage
    ///     provider.simulate_outages(1.0, 1.0).await?;
    ///
    ///     // Services should now fail
    ///     assert!(!provider.is_healthy().await?);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn simulate_outages(
        &mut self,
        app_config_failure_rate: f32,
        key_vault_failure_rate: f32,
    ) -> TestResult<()> {
        self.app_config.set_failure_rate(app_config_failure_rate);
        self.key_vault.set_failure_rate(key_vault_failure_rate);

        if app_config_failure_rate >= 1.0 {
            self.app_config.simulate_outage();
        }

        if key_vault_failure_rate >= 1.0 {
            self.key_vault.simulate_outage();
        }

        Ok(())
    }

    /// Restores all mock services to healthy operation.
    ///
    /// This method resets all failure rates and restores normal service operation
    /// across all mock services.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_service_restoration() -> Result<(), TestError> {
    ///     let mut provider = MockServiceProvider::new().await?;
    ///
    ///     // Simulate outage then restore
    ///     provider.simulate_outages(1.0, 1.0).await?;
    ///     provider.restore_services().await?;
    ///
    ///     assert!(provider.is_healthy().await?);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn restore_services(&mut self) -> TestResult<()> {
        self.app_config.restore_service();
        self.key_vault.restore_service();
        self.is_healthy = true;
        Ok(())
    }

    /// Checks if all mock services are healthy and responsive.
    ///
    /// # Returns
    ///
    /// `true` if all services are healthy, `false` if any service is failing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_health_check() -> Result<(), TestError> {
    ///     let provider = MockServiceProvider::new().await?;
    ///     assert!(provider.is_healthy().await?);
    ///     Ok(())
    /// }
    /// ```
    pub async fn is_healthy(&self) -> TestResult<bool> {
        Ok(self.is_healthy && self.app_config.is_healthy() && self.key_vault.is_healthy())
    }

    /// Resets all mock services to their initial state.
    ///
    /// This method clears all configuration and secrets, returning services
    /// to their default test state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{MockServiceProvider, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_service_reset() -> Result<(), TestError> {
    ///     let mut provider = MockServiceProvider::new().await?;
    ///
    ///     // Add some test data
    ///     provider.set_app_config_value("test-key", "test-value").await?;
    ///
    ///     // Reset to clean state
    ///     provider.reset().await?;
    ///
    ///     // Data should be cleared
    ///     Ok(())
    /// }
    /// ```
    pub async fn reset(&mut self) -> TestResult<()> {
        self.app_config.reset();
        self.key_vault.reset();
        self.is_healthy = true;
        Ok(())
    }

    /// Simulates App Config service failure.
    pub async fn simulate_app_config_failure(&mut self) -> TestResult<()> {
        self.simulate_outages(1.0, 0.0).await
    }

    /// Restores App Config service to healthy state.
    pub async fn restore_app_config(&mut self) -> TestResult<()> {
        self.app_config.restore_service();
        self.is_healthy = true;
        Ok(())
    }

    /// Simulates Key Vault service failure.
    pub async fn simulate_key_vault_failure(&mut self) -> TestResult<()> {
        self.simulate_outages(0.0, 1.0).await
    }

    /// Restores Key Vault service to healthy state.
    pub async fn restore_key_vault(&mut self) -> TestResult<()> {
        self.key_vault.restore_service();
        self.is_healthy = true;
        Ok(())
    }
}
