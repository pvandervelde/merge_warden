use std::{
    fs,
    path::{Path, PathBuf},
};

use merge_warden_core::{config::ApplicationDefaults, errors::MergeWardenError};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = ".merge-warden.toml";

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

/// Configuration for Merge Warden CLI
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Default settings
    #[serde(default)]
    pub default: DefaultConfig,

    /// Rules for validation
    #[serde(default)]
    pub policies: ApplicationDefaults,

    #[serde(default)]
    pub authentication: AuthenticationConfig,

    #[serde(default)]
    pub webhooks: WebHookConfig,
}

impl AppConfig {
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

        let config: AppConfig = toml::from_str(&content).map_err(|e| {
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default: DefaultConfig::new(),
            policies: ApplicationDefaults::default(),
            authentication: AuthenticationConfig::new(),
            webhooks: WebHookConfig::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    #[serde(default = "AuthenticationConfig::default_auth_method")]
    pub auth_method: String,
}

impl AuthenticationConfig {
    fn default_auth_method() -> String {
        "token".to_string()
    }

    pub fn new() -> Self {
        AuthenticationConfig {
            auth_method: Self::default_auth_method(),
        }
    }
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            auth_method: AuthenticationConfig::default_auth_method(),
        }
    }
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Default Git provider
    #[serde(default = "DefaultConfig::default_provider")]
    pub provider: String,
}

impl DefaultConfig {
    fn default_provider() -> String {
        "github".to_string()
    }

    pub fn new() -> Self {
        DefaultConfig {
            provider: Self::default_provider(),
        }
    }
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            provider: DefaultConfig::default_provider(),
        }
    }
}

/// Configuration for listening to webhook calls
#[derive(Debug, Serialize, Deserialize)]
pub struct WebHookConfig {
    /// The port on which webhooks will be received
    #[serde(default = "default_port")]
    pub port: u32,
}

impl WebHookConfig {
    pub fn new() -> Self {
        WebHookConfig {
            port: default_port(),
        }
    }
}

impl Default for WebHookConfig {
    fn default() -> Self {
        Self {
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
