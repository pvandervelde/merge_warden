use std::{
    fs,
    path::{Path, PathBuf},
};

use merge_warden_core::{
    config::{AuthenticationConfig, DefaultConfig, RulesConfig},
    errors::MergeWardenError,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = ".merge-warden.toml";

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

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

    #[serde(default)]
    pub pr_validation: PRValidationConfig,
}

impl Config {
    /// Load configuration from the specified file
    pub fn load(path: &Path) -> Result<Self, MergeWardenError> {
        debug!("Loading configuration from {:?}", path);

        if !path.exists() {
            return Err(MergeWardenError::ConfigError(format!(
                "Configuration file not found: {:?}",
                path
            )));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            MergeWardenError::ConfigError(format!("Failed to read configuration file: {}", e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            MergeWardenError::ConfigError(format!("Failed to parse configuration file: {}", e))
        })?;

        Ok(config)
    }

    /// Save configuration to the specified file
    pub fn save(&self, path: &Path) -> Result<(), MergeWardenError> {
        debug!("Saving configuration to {:?}", path);

        let content = toml::to_string_pretty(self).map_err(|e| {
            MergeWardenError::ConfigError(format!("Failed to serialize configuration: {}", e))
        })?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MergeWardenError::ConfigError(format!("Failed to create directory: {}", e))
            })?;
        }

        fs::write(path, content).map_err(|e| {
            MergeWardenError::ConfigError(format!("Failed to write configuration file: {}", e))
        })?;

        info!("Configuration saved to {:?}", path);
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: DefaultConfig::new(),
            rules: RulesConfig::new(),
            authentication: AuthenticationConfig::new(),
            pr_validation: PRValidationConfig::new(),
        }
    }
}

/// Pull Request Validation configuration
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PRValidationConfig {
    /// The port on which webhooks will be received
    #[serde(default = "default_port")]
    pub port: u32,
}

impl PRValidationConfig {
    pub fn new() -> Self {
        PRValidationConfig {
            port: default_port(),
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

fn default_port() -> u32 {
    3100
}
