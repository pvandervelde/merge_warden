use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::errors::CliError;

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = ".merge-warden.toml";

/// Configuration for Merge Warden CLI
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Default settings
    #[serde(default)]
    pub default: DefaultConfig,

    /// Rules for validation
    #[serde(default)]
    pub rules: RulesConfig,

    #[serde(default)]
    pub authentication: AuthenticationConfig,
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DefaultConfig {
    /// Default Git provider
    #[serde(default = "default_provider")]
    pub provider: String,
}

fn default_provider() -> String {
    "github".to_string()
}

/// Rules configuration
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RulesConfig {
    /// Require work items to be linked
    #[serde(default)]
    pub require_work_items: bool,

    /// Enforce title convention
    #[serde(default)]
    pub enforce_title_convention: Option<String>,

    /// Minimum number of approvals required
    #[serde(default)]
    pub min_approvals: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthenticationConfig {
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
}

fn default_auth_method() -> String {
    "token".to_string()
}

impl Config {
    /// Load configuration from the specified file
    pub fn load(path: &Path) -> Result<Self, CliError> {
        debug!("Loading configuration from {:?}", path);

        if !path.exists() {
            return Err(CliError::ConfigError(format!(
                "Configuration file not found: {:?}",
                path
            )));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            CliError::ConfigError(format!("Failed to read configuration file: {}", e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            CliError::ConfigError(format!("Failed to parse configuration file: {}", e))
        })?;

        Ok(config)
    }

    /// Save configuration to the specified file
    pub fn save(&self, path: &Path) -> Result<(), CliError> {
        debug!("Saving configuration to {:?}", path);

        let content = toml::to_string_pretty(self).map_err(|e| {
            CliError::ConfigError(format!("Failed to serialize configuration: {}", e))
        })?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CliError::ConfigError(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, content).map_err(|e| {
            CliError::ConfigError(format!("Failed to write configuration file: {}", e))
        })?;

        info!("Configuration saved to {:?}", path);
        Ok(())
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Config {
            default: DefaultConfig::default(),
            rules: RulesConfig::default(),
            authentication: AuthenticationConfig::default(),
        }
    }
}

/// Get the path to the configuration file
pub fn get_config_path(config_path: Option<&str>) -> PathBuf {
    if let Some(path) = config_path {
        PathBuf::from(path)
    } else {
        // Look for config in current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join(DEFAULT_CONFIG_FILENAME)
    }
}
