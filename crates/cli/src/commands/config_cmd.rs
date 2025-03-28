use anyhow::Result;
use clap::Subcommand;
use tracing::debug;

use crate::config::{get_config_path, Config};
use crate::errors::CliError;

/// Subcommands for the config command
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Create initial configuration file
    Init {
        /// Path to save the configuration file
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Check configuration syntax
    Validate {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Show current configuration
    Get {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Configuration key to get (e.g., "rules.require_work_items")
        key: Option<String>,
    },

    /// Update configuration values
    Set {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Configuration key to set (e.g., "rules.require_work_items")
        key: String,

        /// Value to set
        value: String,
    },
}

/// Execute the config command
pub async fn execute(cmd: ConfigCommands) -> Result<(), CliError> {
    match cmd {
        ConfigCommands::Init { path } => init_config(path.as_deref()),
        ConfigCommands::Validate { path } => validate_config(path.as_deref()),
        ConfigCommands::Get { path, key } => get_config(path.as_deref(), key.as_deref()),
        ConfigCommands::Set { path, key, value } => set_config(path.as_deref(), &key, &value),
    }
}

/// Initialize a new configuration file
fn init_config(path: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!("Initializing configuration at {:?}", config_path);

    if config_path.exists() {
        return Err(CliError::ConfigError(format!(
            "Configuration file already exists at {:?}",
            config_path
        )));
    }

    let config = Config::default();
    config.save(&config_path)?;

    println!("Configuration initialized at {:?}", config_path);
    Ok(())
}

/// Validate a configuration file
fn validate_config(path: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!("Validating configuration at {:?}", config_path);

    match Config::load(&config_path) {
        Ok(_) => {
            println!("Configuration is valid");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Get a configuration value
fn get_config(path: Option<&str>, key: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!("Getting configuration from {:?}", config_path);

    let config = Config::load(&config_path)?;

    if let Some(key) = key {
        // Get specific key
        let value = get_config_value(&config, key)?;
        println!("{}: {}", key, value);
    } else {
        // Print entire config
        let config_str = toml::to_string_pretty(&config).map_err(|e| {
            CliError::ConfigError(format!("Failed to serialize configuration: {}", e))
        })?;
        println!("{}", config_str);
    }

    Ok(())
}

/// Set a configuration value
fn set_config(path: Option<&str>, key: &str, value: &str) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!("Setting configuration at {:?}", config_path);

    // Load existing config or create a new one
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        Config::default()
    };

    // Update the config
    set_config_value(&mut config, key, value)?;

    // Save the updated config
    config.save(&config_path)?;

    println!("Configuration updated: {} = {}", key, value);
    Ok(())
}

/// Get a value from the configuration by key path
fn get_config_value(config: &Config, key: &str) -> Result<String, CliError> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(CliError::InvalidArguments(
            "Invalid configuration key".to_string(),
        ));
    }

    match parts[0] {
        "default" => match parts.get(1) {
            Some(&"provider") => Ok(config.default.provider.clone()),
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        "rules" => match parts.get(1) {
            Some(&"require_work_items") => Ok(config.rules.require_work_items.to_string()),
            Some(&"enforce_title_convention") => Ok(config
                .rules
                .enforce_title_convention
                .clone()
                .unwrap_or_default()),
            Some(&"min_approvals") => Ok(config
                .rules
                .min_approvals
                .map(|n| n.to_string())
                .unwrap_or_default()),
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        "authentication" => match parts.get(1) {
            Some(&"auth_method") => Ok(config.authentication.auth_method.clone()),
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        _ => Err(CliError::InvalidArguments(format!(
            "Invalid configuration key: {}",
            key
        ))),
    }
}

/// Set a value in the configuration by key path
fn set_config_value(config: &mut Config, key: &str, value: &str) -> Result<(), CliError> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(CliError::InvalidArguments(
            "Invalid configuration key".to_string(),
        ));
    }

    match parts[0] {
        "default" => match parts.get(1) {
            Some(&"provider") => {
                config.default.provider = value.to_string();
                Ok(())
            }
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        "rules" => match parts.get(1) {
            Some(&"require_work_items") => {
                config.rules.require_work_items = value.parse().map_err(|_| {
                    CliError::InvalidArguments(format!(
                        "Invalid value for require_work_items: {}",
                        value
                    ))
                })?;
                Ok(())
            }
            Some(&"enforce_title_convention") => {
                if value.is_empty() {
                    config.rules.enforce_title_convention = None;
                } else {
                    config.rules.enforce_title_convention = Some(value.to_string());
                }
                Ok(())
            }
            Some(&"min_approvals") => {
                if value.is_empty() {
                    config.rules.min_approvals = None;
                } else {
                    config.rules.min_approvals = Some(value.parse().map_err(|_| {
                        CliError::InvalidArguments(format!(
                            "Invalid value for min_approvals: {}",
                            value
                        ))
                    })?);
                }
                Ok(())
            }
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        "authentication" => match parts.get(1) {
            Some(&"auth_method") => {
                if value.is_empty() {
                    config.authentication.auth_method = String::new();
                } else {
                    config.authentication.auth_method = value.to_string();
                }
                Ok(())
            }
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        _ => Err(CliError::InvalidArguments(format!(
            "Invalid configuration key: {}",
            key
        ))),
    }
}
