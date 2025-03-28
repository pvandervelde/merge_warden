use anyhow::Result;
use clap::Subcommand;
use keyring::Entry;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::config::{get_config_path, Config};
use crate::errors::CliError;

pub const KEY_RING_SERVICE_NAME: &str = "merge_warden_cli";
pub const KEY_RING_APP_ID: &str = "github_app_id";
pub const KEY_RING_APP_TOKEN: &str = "github_private_key_path";
pub const KEY_RING_USER_TOKEN: &str = "github_token";

/// Subcommands for the auth command
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Authenticate with GitHub
    GitHub {
        /// Authentication method (app or token)
        #[arg(default_value = "token")]
        method: String,
    },
}

/// Execute the auth command
pub async fn execute(cmd: AuthCommands) -> Result<(), CliError> {
    match cmd {
        AuthCommands::GitHub { method } => auth_github(&method).await,
        _ => Err(CliError::InvalidArguments(
            "Unknown authentication type.".to_string(),
        )),
    }
}

/// Authenticate with GitHub
async fn auth_github(method: &str) -> Result<(), CliError> {
    debug!("Authenticating with GitHub using method: {}", method);

    let config_path = get_config_path(None);
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        Config::default()
    };

    match method {
        "app" => {
            // GitHub App authentication
            println!("GitHub App Authentication");
            println!("------------------------");
            println!("Please provide the following information:");

            // Get App ID
            println!("App ID:");
            let mut app_id = String::new();
            std::io::stdin()
                .read_line(&mut app_id)
                .map_err(|e| CliError::AuthError(format!("Failed to read input: {}", e)))?;

            // Get private key path
            println!("Path to private key file:");
            let mut key_path = String::new();
            std::io::stdin()
                .read_line(&mut key_path)
                .map_err(|e| CliError::AuthError(format!("Failed to read input: {}", e)))?;
            let key_path = key_path.trim();

            // Verify the key file exists
            let key_path_buf = PathBuf::from(key_path);
            if !key_path_buf.exists() {
                return Err(CliError::AuthError(format!(
                    "Private key file not found: {}",
                    key_path
                )));
            }

            let keyring_app_id =
                Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID).map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_app_id
                .set_password(&app_id.to_string())
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to save app_id to keyring: {}", e))
                })?;

            let keyring_key_path =
                Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_TOKEN).map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_key_path.set_password(key_path).map_err(|e| {
                CliError::AuthError(format!("Failed to save private_key_path to keyring: {}", e))
            })?;

            config.authentication.auth_method = "app".to_string();
            config.save(&config_path)?;

            println!("GitHub App authentication configured successfully!");
        }
        "token" => {
            // Personal Access Token authentication
            println!("GitHub Personal Access Token Authentication");
            println!("------------------------------------------");
            println!("Please provide your GitHub Personal Access Token:");
            println!("(Token will not be displayed as you type)");

            // Get token (in a real implementation, this would use a secure input method)
            let mut token = String::new();
            std::io::stdin()
                .read_line(&mut token)
                .map_err(|e| CliError::AuthError(format!("Failed to read input: {}", e)))?;
            let token = token.trim();

            if token.is_empty() {
                return Err(CliError::AuthError("Token cannot be empty".to_string()));
            }

            let keyring = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN).map_err(|e| {
                CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
            })?;
            keyring.set_password(token).map_err(|e| {
                CliError::AuthError(format!("Failed to save token to keyring: {}", e))
            })?;

            config.authentication.auth_method = "token".to_string();
            config.save(&config_path)?;

            println!("GitHub token authentication configured successfully!");
        }
        _ => {
            return Err(CliError::InvalidArguments(format!(
                "Unsupported authentication method: {}",
                method
            )));
        }
    }

    Ok(())
}
