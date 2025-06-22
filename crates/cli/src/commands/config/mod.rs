use anyhow::Result;
use clap::Subcommand;
use tracing::{debug, error, info, instrument};

use crate::config::{get_config_path, AppConfig};
use crate::errors::CliError;

pub mod bypass;

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

    /// Manage bypass rules
    #[command(subcommand)]
    Bypass(bypass::BypassCommands),
}

/// Execute the config command
#[instrument]
pub async fn execute(cmd: ConfigCommands) -> Result<(), CliError> {
    match cmd {
        ConfigCommands::Init { path } => init_config(path.as_deref()),
        ConfigCommands::Validate { path } => validate_config(path.as_deref()),
        ConfigCommands::Get { path, key } => get_config(path.as_deref(), key.as_deref()),
        ConfigCommands::Set { path, key, value } => set_config(path.as_deref(), &key, &value),
        ConfigCommands::Bypass(bypass_cmd) => bypass::execute(bypass_cmd).await,
    }
}

/// Initialize a new configuration file
#[instrument]
fn init_config(path: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(message = "Initializing configuration", path = ?config_path);

    if config_path.exists() {
        let err = CliError::ConfigError(format!(
            "Configuration file already exists at {:?}",
            config_path
        ));
        error!(
            message = "Configuration file already exists",
            path = ?config_path,
            error = ?err
        );
        return Err(err);
    }

    let config = AppConfig::default();
    if let Err(e) = config.save(&config_path) {
        error!(message = "Failed to save configuration", path = ?config_path, error = ?e);
        return Err(CliError::ConfigError(
            "Failed to save configuration".to_string(),
        ));
    }

    info!(message = "Configuration initialized", path = ?config_path);
    println!("Configuration initialized at {:?}", config_path);
    Ok(())
}

/// Validate a configuration file
#[instrument]
fn validate_config(path: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(message = "Validating configuration", path = ?config_path);

    match AppConfig::load(&config_path) {
        Ok(_) => {
            info!(message = "Configuration is valid", path = ?config_path);
            println!("Configuration is valid");
            Ok(())
        }
        Err(e) => {
            error!(
                message = "Configuration is invalid",
                path = ?config_path,
                error = ?e
            );
            Err(CliError::ConfigError(
                "The configuration is invalid".to_string(),
            ))
        }
    }
}

/// Get a configuration value
#[instrument]
fn get_config(path: Option<&str>, key: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(message = "Getting configuration", path = ?config_path, key = ?key);

    let config = match AppConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(CliError::ConfigError(
                "Failed to load the configuration".to_string(),
            ));
        }
    };

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
#[instrument]
fn set_config(path: Option<&str>, key: &str, value: &str) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(
        message = "Setting configuration",
        path = ?config_path,
        key = key,
        value = value
    );

    // Load existing config or create a new one
    let mut config = match if config_path.exists() {
        AppConfig::load(&config_path)
    } else {
        Ok(AppConfig::default())
    } {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(CliError::ConfigError(
                "Failed to load the configuration".to_string(),
            ));
        }
    };

    // Update the config
    if let Err(e) = set_config_value(&mut config, key, value) {
        error!(message = "Failed to set configuration value", key = key, value = value, error = ?e);
        return Err(e);
    }

    // Save the updated config
    if let Err(e) = config.save(&config_path) {
        error!(message = "Failed to save configuration", path = ?config_path, error = ?e);
        return Err(CliError::ConfigError(
            "Failed to save configuration".to_string(),
        ));
    }

    info!(message = "Configuration updated", key = key, value = value);
    println!("Configuration updated: {} = {}", key, value);
    Ok(())
}

/// Get a value from the configuration by key path
fn get_config_value(config: &AppConfig, key: &str) -> Result<String, CliError> {
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
            Some(&"enforceWorkItemValidation") => {
                Ok(config.policies.enable_work_item_validation.to_string())
            }
            Some(&"enforceTitleValidation") => {
                Ok(config.policies.enable_title_validation.to_string())
            }
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        "policies" => match parts.get(1) {
            Some(&"bypassRules") => get_bypass_rule_value(&config.policies, &parts[2..]),
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
fn set_config_value(config: &mut AppConfig, key: &str, value: &str) -> Result<(), CliError> {
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
            Some(&"enforceWorkItemValidation") => {
                config.policies.enable_work_item_validation = value.parse().map_err(|_| {
                    CliError::InvalidArguments(format!(
                        "Invalid value for require_work_items: {}",
                        value
                    ))
                })?;
                Ok(())
            }
            Some(&"enforceTitleValidation") => {
                config.policies.enable_title_validation = value.parse().map_err(|_| {
                    CliError::InvalidArguments(format!(
                        "Invalid value for require_work_items: {}",
                        value
                    ))
                })?;
                Ok(())
            }
            _ => Err(CliError::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        "policies" => match parts.get(1) {
            Some(&"bypassRules") => set_bypass_rule_value(&mut config.policies, &parts[2..], value),
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

/// Get a bypass rule value by path
fn get_bypass_rule_value(
    policies: &merge_warden_core::config::ApplicationDefaults,
    parts: &[&str],
) -> Result<String, CliError> {
    if parts.is_empty() {
        return Err(CliError::InvalidArguments(
            "Invalid bypass rule path".to_string(),
        ));
    }

    match parts[0] {
        "titleConvention" => match parts.get(1) {
            Some(&"enabled") => Ok(policies.bypass_rules.title_convention.enabled.to_string()),
            Some(&"users") => Ok(format!(
                "{:?}",
                policies.bypass_rules.title_convention.users
            )),
            _ => Err(CliError::InvalidArguments(
                "Invalid title convention bypass path. Use 'enabled' or 'users'".to_string(),
            )),
        },
        "workItems" => match parts.get(1) {
            Some(&"enabled") => Ok(policies.bypass_rules.work_items.enabled.to_string()),
            Some(&"users") => Ok(format!("{:?}", policies.bypass_rules.work_items.users)),
            _ => Err(CliError::InvalidArguments(
                "Invalid work items bypass path. Use 'enabled' or 'users'".to_string(),
            )),
        },
        _ => Err(CliError::InvalidArguments(format!(
            "Invalid bypass rule type: {}. Use 'titleConvention' or 'workItems'",
            parts[0]
        ))),
    }
}

/// Set a bypass rule value by path
fn set_bypass_rule_value(
    policies: &mut merge_warden_core::config::ApplicationDefaults,
    parts: &[&str],
    value: &str,
) -> Result<(), CliError> {
    if parts.is_empty() {
        return Err(CliError::InvalidArguments(
            "Invalid bypass rule path".to_string(),
        ));
    }

    match parts[0] {
        "titleConvention" => match parts.get(1) {
            Some(&"enabled") => {
                policies.bypass_rules.title_convention.enabled = value.parse().map_err(|_| {
                    CliError::InvalidArguments(format!(
                        "Invalid boolean value for enabled: {}",
                        value
                    ))
                })?;
                Ok(())
            }
            Some(&"users") => {
                // Parse comma-separated user list
                let users: Vec<String> = if value.trim().is_empty() {
                    Vec::new()
                } else {
                    value
                        .split(',')
                        .map(|u| u.trim().to_string())
                        .filter(|u| !u.is_empty())
                        .collect()
                };
                policies.bypass_rules.title_convention.users = users;
                Ok(())
            }
            _ => Err(CliError::InvalidArguments(
                "Invalid title convention bypass path. Use 'enabled' or 'users'".to_string(),
            )),
        },
        "workItems" => match parts.get(1) {
            Some(&"enabled") => {
                policies.bypass_rules.work_items.enabled = value.parse().map_err(|_| {
                    CliError::InvalidArguments(format!(
                        "Invalid boolean value for enabled: {}",
                        value
                    ))
                })?;
                Ok(())
            }
            Some(&"users") => {
                // Parse comma-separated user list
                let users: Vec<String> = if value.trim().is_empty() {
                    Vec::new()
                } else {
                    value
                        .split(',')
                        .map(|u| u.trim().to_string())
                        .filter(|u| !u.is_empty())
                        .collect()
                };
                policies.bypass_rules.work_items.users = users;
                Ok(())
            }
            _ => Err(CliError::InvalidArguments(
                "Invalid work items bypass path. Use 'enabled' or 'users'".to_string(),
            )),
        },
        _ => Err(CliError::InvalidArguments(format!(
            "Invalid bypass rule type: {}. Use 'titleConvention' or 'workItems'",
            parts[0]
        ))),
    }
}
