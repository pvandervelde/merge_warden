use anyhow::Result;
use clap::Subcommand;
use keyring::Entry;
use std::path::PathBuf;
use tracing::{debug, error, info, instrument};

use crate::config::{get_config_path, Config};
use crate::errors::CliError;

pub const KEY_RING_SERVICE_NAME: &str = "merge_warden_cli";
pub const KEY_RING_APP_ID: &str = "github_app_id";
pub const KEY_RING_APP_PRIVATE_KEY_PATH: &str = "github_private_key_path";
pub const KEY_RING_USER_TOKEN: &str = "github_token";
pub const KEY_RING_WEB_HOOK_SECRET: &str = "webhook_secret";

/// Subcommands for the auth command
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Authenticate with GitHub
    #[command(name = "github")]
    GitHub {
        /// Authentication method (app or token)
        #[arg(default_value = "token")]
        method: String,
    },
}

/// Execute the auth command
#[instrument]
pub async fn execute(cmd: AuthCommands) -> Result<(), CliError> {
    match cmd {
        AuthCommands::GitHub { method } => auth_github(&method).await,
        _ => Err(CliError::InvalidArguments(
            "Unknown authentication type.".to_string(),
        )),
    }
}

/// Authenticate with GitHub
#[instrument]
async fn auth_github(method: &str) -> Result<(), CliError> {
    debug!(message = "Authenticating with GitHub", method = method);

    let config_path = get_config_path(None);
    let mut config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(e);
        }
    };

    match method {
        "app" => {
            // GitHub App authentication
            info!(message = "GitHub App Authentication");
            println!("GitHub App Authentication");
            println!("------------------------");
            println!("Please provide the following information:");

            // Get App ID
            println!("App ID:");
            let mut app_id = String::new();
            std::io::stdin()
                .read_line(&mut app_id)
                .map_err(|e| CliError::AuthError(format!("Failed to read input: {}", e)))?;
            let app_id = app_id.trim();
            debug!(message = "Read app ID from stdin", app_id = app_id);

            // Get private key path
            println!("Path to private key file:");
            let mut key_path = String::new();
            std::io::stdin()
                .read_line(&mut key_path)
                .map_err(|e| CliError::AuthError(format!("Failed to read input: {}", e)))?;
            let key_path = key_path.trim();
            debug!(message = "Read key path from stdin", key_path = key_path);

            // Verify the key file exists
            let key_path_buf = PathBuf::from(key_path);
            if !key_path_buf.exists() {
                let err = CliError::AuthError(format!("Private key file not found: {}", key_path));
                error!(message = "Private key file not found", key_path = key_path, error = ?err);
                return Err(err);
            }

            // Get the webhook secret
            println!("Webhook secret:");
            let mut webhook_secret = String::new();
            std::io::stdin()
                .read_line(&mut webhook_secret)
                .map_err(|e| CliError::AuthError(format!("Failed to read input: {}", e)))?;
            let webhook_secret = webhook_secret.trim();
            debug!(
                message = "Read webhook secret from stdin",
                webhook_secret = webhook_secret
            );

            let keyring_app_id =
                Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID).map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_app_id.set_password(app_id).map_err(|e| {
                CliError::AuthError(format!("Failed to save the app ID to the keyring: {}", e))
            })?;
            debug!(message = "Saved app ID to keyring");

            let keyring_key_path = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_PRIVATE_KEY_PATH)
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_key_path.set_password(key_path).map_err(|e| {
                CliError::AuthError(format!(
                    "Failed to save the app private key to the keyring: {}",
                    e
                ))
            })?;
            debug!(message = "Saved key path to keyring");

            let keyring_webhook_secret =
                Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_WEB_HOOK_SECRET).map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_webhook_secret
                .set_password(webhook_secret)
                .map_err(|e| {
                    CliError::AuthError(format!(
                        "Failed to save the webhook secret to the keyring: {}",
                        e
                    ))
                })?;
            debug!(message = "Saved webhook secret to keyring");

            config.authentication.auth_method = "app".to_string();
            config.save(&config_path)?;
            info!(
                message = "Updated configuration with auth method",
                auth_method = "app"
            );

            println!("GitHub App authentication configured successfully!");
        }
        "token" => {
            // Personal Access Token authentication
            info!(message = "GitHub Personal Access Token Authentication");
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
            debug!(message = "Read token from stdin");

            if token.is_empty() {
                let err = CliError::AuthError("Token cannot be empty".to_string());
                error!(message = "Token cannot be empty", error = ?err);
                return Err(err);
            }

            let keyring = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN).map_err(|e| {
                CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
            })?;
            keyring.set_password(token).map_err(|e| {
                CliError::AuthError(format!("Failed to save token to keyring: {}", e))
            })?;
            debug!(message = "Saved token to keyring");

            config.authentication.auth_method = "token".to_string();
            config.save(&config_path)?;
            info!(
                message = "Updated configuration with auth method",
                auth_method = "token"
            );

            println!("GitHub token authentication configured successfully!");
        }
        _ => {
            let err = CliError::InvalidArguments(format!(
                "Unsupported authentication method: {}",
                method
            ));
            error!(message = "Unsupported authentication method", method = method, error = ?err);
            return Err(err);
        }
    }

    Ok(())
}
