use anyhow::Result;
use clap::Subcommand;
use tracing::{debug, error, info, instrument};

use crate::config::{get_config_path, AppConfig};
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
}

/// Execute the config command
#[instrument]
pub async fn execute(cmd: ConfigCommands) -> Result<(), CliError> {
    match cmd {
        ConfigCommands::Init { path } => init_config(path.as_deref()),
        ConfigCommands::Validate { path } => validate_config(path.as_deref()),
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
