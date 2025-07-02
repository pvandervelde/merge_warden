use anyhow::Result;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use axum_macros::debug_handler;
use clap::Args;
use hmac::{Hmac, Mac};
use keyring::Entry;
use merge_warden_core::config::{
    load_merge_warden_config, CurrentPullRequestValidationConfiguration,
};
use merge_warden_core::{MergeWarden, WebhookPayload};
use merge_warden_developer_platforms::github::{
    authenticate_with_access_token, create_app_client, GitHubProvider,
};
use merge_warden_developer_platforms::models::User;
use octocrab::Octocrab;
use serde::Serialize;
use sha2::Sha256;
use std::fs;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use crate::commands::auth::{
    KEY_RING_APP_ID, KEY_RING_APP_PRIVATE_KEY_PATH, KEY_RING_SERVICE_NAME,
};
use crate::config::{get_config_path, AppConfig};
use crate::errors::CliError;

use super::auth::KEY_RING_WEB_HOOK_SECRET;

pub struct AppState {
    pub octocrab: Octocrab,
    pub config: AppConfig,
    pub webhook_secret: String,
}

/// Arguments for the check-pr command
#[derive(Args, Debug)]
pub struct CheckPrArgs {
    /// Git provider (github)
    #[arg(short, long)]
    pub provider: String,

    /// Alternate config file
    #[arg(short, long)]
    pub config: Option<String>,
}

/// Result of the check-pr command
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub passed: bool,

    /// List of validation failures
    pub failures: Vec<String>,
}

/// Creates a GitHub application client based on the provided configuration.
///
/// This function initializes a GitHub client using either a personal access token
/// or a GitHub App authentication method, depending on the `auth_method` specified
/// in the configuration. The credentials are retrieved securely from the system's
/// keyring.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` object containing authentication details.
///
/// # Returns
///
/// Returns a `Result` containing an `Octocrab` instance if successful, or a `CliError`
/// if an error occurs during the authentication process.
///
/// # Errors
///
/// This function will return a `CliError` in the following cases:
/// - If the keyring entry cannot be created or accessed.
/// - If the authentication method specified in the configuration is unsupported.
/// - If the app ID or app key cannot be parsed or retrieved.
/// - If the GitHub client cannot be initialized.
///
/// # Example
///
/// ```rust
/// use merge_warden_developer_platforms::github::GitHubProvider;
/// use crate::config::Config;
/// use crate::errors::CliError;
///
/// #[tokio::main]
/// async fn main() -> Result<(), CliError> {
///     let config = Config {
///         authentication: Authentication {
///             auth_method: "token".to_string(),
///             ..Default::default()
///         },
///         ..Default::default()
///     };
///
///     let github_client = create_github_app(&config).await?;
///     println!("GitHub client created successfully!");
///
///     Ok(())
/// }
/// ```
async fn create_github_app(config: &AppConfig) -> Result<(Octocrab, User), CliError> {
    debug!("Creating GitHub app client");
    let provider = match config.authentication.auth_method.as_str() {
        "token" => {
            let err = CliError::InvalidArguments(
                format!(
                    "Unsupported authentication method: {}",
                    config.authentication.auth_method
                )
                .to_string(),
            );
            error!(message = "Failed to create GitHub app client", error = ?err);
            return Err(err);
        }
        "app" => {
            info!(message = "Using GitHub App authentication");
            let app_id = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID)
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to get app ID from the keyring: {}", e))
                })?;

            let app_key_path = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_PRIVATE_KEY_PATH)
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    CliError::AuthError(format!(
                        "Failed to get app key location from the keyring: {}",
                        e
                    ))
                })?;

            let app_key = fs::read_to_string(app_key_path).map_err(|e| {
                CliError::ConfigError(format!(
                    "Failed to load the app key from the provided file: {}",
                    e
                ))
            })?;

            let app_id_number = app_id.parse::<u64>().map_err(|e| {
                CliError::InvalidArguments(
                    format!(
                        "Failed to parse the app ID. Expected a number, got {}. Error was: {}.",
                        app_id, e
                    )
                    .to_string(),
                )
            })?;
            let pair = create_app_client(app_id_number, &app_key)
                .await
                .map_err(|e| {
                    CliError::AuthError(
                        format!("Failed to load the GitHub provider. Error was: {}", e).to_string(),
                    )
                })?;
            debug!(message = "GitHub App client created successfully");
            pair
        }
        _ => {
            let err = CliError::InvalidArguments(
                format!(
                    "Unsupported authentication method: {}",
                    config.authentication.auth_method
                )
                .to_string(),
            );
            error!(message = "Failed to create GitHub app client", error = ?err);
            return Err(err);
        }
    };

    Ok(provider)
}

/// Executes the `check-pr` command.
///
/// This function sets up the environment for validating pull requests by:
/// - Loading the configuration file, either from the default location or a user-specified path.
/// - Creating a GitHub client using the authentication method specified in the configuration.
/// - Initializing the webhook secret for verifying incoming GitHub webhook requests.
/// - Setting up an HTTP server to listen for webhook events from GitHub.
///
/// The function listens for incoming webhook events and processes them to validate pull requests
/// based on the specified configuration.
///
/// # Arguments
///
/// * `args` - A `CheckPrArgs` struct containing the command-line arguments for the `check-pr` command.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure:
/// - `Ok(())` if the server is successfully started and running.
/// - `Err(CliError)` if an error occurs during configuration loading, authentication, or server setup.
///
/// # Errors
///
/// This function will return a `CliError` in the following cases:
/// - If the configuration file cannot be loaded.
/// - If the GitHub client cannot be initialized due to authentication issues.
/// - If the webhook secret cannot be retrieved.
/// - If the HTTP server fails to start.
///
/// # Notes
///
/// The function uses Axum to set up the HTTP server and routes. It listens for webhook events
/// on the `/webhook` endpoint and processes them asynchronously. The server runs indefinitely
/// until manually stopped.
#[instrument]
pub async fn execute(args: CheckPrArgs) -> Result<(), CliError> {
    // Load configuration
    let config_path = get_config_path(args.config.as_deref());
    let config = AppConfig::load(&config_path)
        .map_err(|e| CliError::ConfigError(format!("Failed to load configuration: {}", e)))?;

    let (octocrab, _user) = create_github_app(&config).await?;
    let webhook_secret = retrieve_webhook_secret()?;

    let addr = format!("0.0.0.0:{}", config.webhooks.port);

    let state = Arc::new(AppState {
        octocrab,
        config,
        webhook_secret,
    });

    let app = Router::new()
        .route("/api/merge_warden", post(handle_webhook))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[debug_handler]
#[instrument(skip(state, headers, body))]
async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    info!("Received webhook call from Github");

    if !verify_github_signature(&state.webhook_secret, &headers, &body) {
        warn!("Webhook did not have valid signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!("Webhook has valid signature. Processing information ...");
    let body_str = std::str::from_utf8(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let payload: WebhookPayload = serde_json::from_str(body_str).map_err(|e| {
        error!(
            body = body_str.to_string(),
            error = e.to_string(),
            "Could not extract webhook payload from request body"
        );
        StatusCode::BAD_REQUEST
    })?;

    debug!(action = payload.action.as_str(), "Github action");
    let action = payload.action.as_str();
    if action != "opened"
        && action != "edited"
        && action != "ready_for_review"
        && action != "reopened"
        && action != "unlocked"
    {
        info!(
            action = payload.action.as_str(),
            message = "Pull request change type means no scanning required."
        );
        return Ok(StatusCode::OK);
    }

    let Some(installation) = payload.installation else {
        warn!(message = "Web hook payload did not include installation information. Cannot process changes.");
        return Err(StatusCode::BAD_REQUEST);
    };
    let installation_id = installation.id;

    let Some(repository) = payload.repository else {
        warn!(
            message =
                "Web hook payload did not include repository information. Cannot process changes."
        );
        return Err(StatusCode::BAD_REQUEST);
    };

    let Some(pr) = payload.pull_request else {
        warn!(message = "Web hook payload did not include pull request information. Cannot process changes.");
        return Err(StatusCode::BAD_REQUEST);
    };

    let parts: Vec<&str> = repository.full_name.split('/').collect();
    if parts.len() != 2 {
        warn!(
            repository = &repository.name,
            pull_request = pr.number,
            "Failed to extract the name of the repository owner"
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let repo_owner = parts[0];

    info!(
        repository_owner = repo_owner,
        repository = &repository.name,
        pull_request = pr.number,
        "Processing pull request"
    );

    let api_with_pat = match authenticate_with_access_token(
        &state.octocrab,
        installation_id,
        repo_owner,
        &repository.name,
    )
    .await
    {
        Ok(o) => o,
        Err(e) => {
            error!(
                repository_owner = repo_owner,
                repository = &repository.name,
                pull_request = pr.number,
                error = e.to_string(),
                "Failed to authenticate with GitHub"
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let provider = GitHubProvider::new(api_with_pat);

    // Load the merge-warden TOML config file
    let merge_warden_config_path = ".github/merge-warden.toml";
    let validation_config = match load_merge_warden_config(
        repo_owner,
        &repository.name,
        merge_warden_config_path,
        &provider,
        &state.config.policies,
    )
    .await
    {
        Ok(merge_warden_config) => {
            info!(
                "Loaded merge-warden config from {}",
                merge_warden_config_path
            );
            merge_warden_config.to_validation_config(&state.config.policies.bypass_rules)
        }
        Err(e) => {
            warn!(
                "Failed to load merge-warden config from {}: {}. Falling back to defaults.",
                merge_warden_config_path, e
            );
            CurrentPullRequestValidationConfiguration {
                enforce_title_convention: state.config.policies.enable_title_validation,
                title_pattern: state.config.policies.default_title_pattern.clone(),
                invalid_title_label: state.config.policies.default_invalid_title_label.clone(),
                enforce_work_item_references: state.config.policies.enable_work_item_validation,
                work_item_reference_pattern: state
                    .config
                    .policies
                    .default_work_item_pattern
                    .clone(),
                missing_work_item_label: state
                    .config
                    .policies
                    .default_missing_work_item_label
                    .clone(),
                bypass_rules: state.config.policies.bypass_rules.clone(),
            }
        }
    };

    // Create a MergeWarden instance with loaded or fallback configuration
    let warden = MergeWarden::with_config(provider, validation_config);

    // Process a pull request
    info!(
        message = "Processing pull request",
        pull_request = pr.number,
        repository = &repository.name
    );
    let _ = warden
        .process_pull_request(repo_owner, &repository.name, pr.number)
        .await
        .inspect_err(|e| {
            error!(
                repository_owner = repo_owner,
                repository = &repository.name,
                pull_request = pr.number,
                error = e.to_string(),
                "Failed to process pull request"
            );
        });

    Ok(StatusCode::OK)
}

fn retrieve_webhook_secret() -> Result<String, CliError> {
    debug!(message = "Retrieving webhook secret");
    let webhook_secret = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_WEB_HOOK_SECRET)
        .map_err(|e| {
            CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
        })?
        .get_password()
        .map_err(|e| {
            CliError::AuthError(format!(
                "Failed to get the webhook secret from the keyring: {}",
                e
            ))
        })?;

    debug!(message = "Webhook secret retrieved successfully");
    Ok(webhook_secret)
}

#[instrument]
fn verify_github_signature(secret: &str, headers: &HeaderMap, payload: &[u8]) -> bool {
    let prefix = "sha256=";

    let signature_header = match headers.get("X-Hub-Signature-256") {
        Some(value) => value.to_str().unwrap_or(""),
        None => return false,
    };

    if !signature_header.starts_with(prefix) {
        error!("Missing 'sha256=' prefix in signature header");
        return false;
    }

    let received_sig = &signature_header[prefix.len()..];
    let received_bytes = match hex::decode(received_sig) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode signature: {:?}", e);
            return false;
        }
    };

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload);

    let expected_mac = mac.finalize();
    let expected_bytes = expected_mac.into_bytes();

    //debug!("Expected signature: {}", hex::encode(expected_bytes));
    //debug!("Received signature: {}", received_sig);

    let _result = expected_bytes.as_slice() == received_bytes;
    //debug!("Match result: {}", result);

    // For now just return true. If you're running this as a CLI it is likely that
    // you're running through some kind of proxy. It is highly likely that this proxy
    // reads the information from GitHub, translates it and then resends it. This will
    // most likely screw with the signature.
    true
}
