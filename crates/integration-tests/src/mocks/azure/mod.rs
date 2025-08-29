//! Azure service mocks for integration testing.
//!
//! This module provides mock implementations of Azure services used by Merge Warden,
//! including App Configuration and Key Vault services. These mocks allow comprehensive
//! testing without requiring actual Azure service dependencies.

pub mod app_config;
pub mod key_vault;

pub use app_config::MockAppConfigService;
pub use key_vault::MockKeyVaultService;
