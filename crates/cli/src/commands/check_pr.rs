use anyhow::Result;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use clap::Args;
use hmac::{Hmac, Mac};
use keyring::Entry;
use merge_warden_core::config::ValidationConfig;
use merge_warden_core::MergeWarden;
use merge_warden_developer_platforms::github::{
    authenticate_with_access_token, create_app_client, create_token_client, GitHubProvider,
};
use merge_warden_developer_platforms::models::{
    Installation, Organization, PullRequest, Repository,
};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use crate::commands::auth::{
    KEY_RING_APP_ID, KEY_RING_APP_TOKEN, KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN,
};
use crate::config::{get_config_path, Config};
use crate::errors::CliError;

use super::auth::KEY_RING_WEB_HOOK_SECRET;

struct AppState {
    octocrab: Octocrab,
    config: Config,
    webhook_secret: String,
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

#[derive(Deserialize)]
struct WebhookPayload {
    action: String,
    pull_request: Option<PullRequest>,
    organization: Option<Organization>,
    repository: Option<Repository>,
    installation: Option<Installation>,
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
async fn create_github_app(config: &Config) -> Result<Octocrab, CliError> {
    let provider = match config.authentication.auth_method.as_str() {
        "token" => {
            let github_token = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN)
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    CliError::AuthError(format!(
                        "Failed to get the user token from the keyring: {}",
                        e
                    ))
                })?;

            info!("Using GitHub token authentication");
            create_token_client(&github_token).map_err(|e| {
                CliError::AuthError(
                    format!("Failed to load the GitHub provider. Error was: {}", e).to_string(),
                )
            })?
        }
        "app" => {
            let app_id = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID)
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to get app ID from the keyring: {}", e))
                })?;

            let app_key = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_TOKEN)
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to get app key from the keyring: {}", e))
                })?;

            info!("Using GitHub token authentication");
            let app_id_number = app_id.parse::<u64>().map_err(|e| {
                CliError::InvalidArguments(
                    format!(
                        "Failed to parse the app ID. Expected a number, got {}. Error was: {}.",
                        app_id, e
                    )
                    .to_string(),
                )
            })?;
            create_app_client(app_id_number, &app_key)
                .await
                .map_err(|e| {
                    CliError::AuthError(
                        format!("Failed to load the GitHub provider. Error was: {}", e).to_string(),
                    )
                })?
        }
        _ => {
            return Err(CliError::InvalidArguments(
                format!(
                    "Unsupported authentication method: {}",
                    config.authentication.auth_method
                )
                .to_string(),
            ))
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
pub async fn execute(args: CheckPrArgs) -> Result<(), CliError> {
    // Load configuration
    let config_path = get_config_path(args.config.as_deref());
    let config = Config::load(&config_path)
        .map_err(|e| CliError::ConfigError(format!("Failed to load configuration: {}", e)))?;

    let octocrab = create_github_app(&config).await?;
    let webhook_secret = retrieve_webhook_secret()?;

    let addr = format!("0.0.0.0:{}", config.pr_validation.port);

    let state = Arc::new(AppState {
        octocrab,
        config,
        webhook_secret,
    });

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[instrument(skip(state, headers, body))]
async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, StatusCode> {
    info!("Received webhook call from Github");

    if !verify_github_signature(&state.webhook_secret, &headers, &body) {
        warn!("Webhook did not have valid signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!("Webhook has valid signature. Processing information ...");

    let payload: WebhookPayload =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    debug!("Github action is {}", payload.action.as_str());
    let action = payload.action.as_str();
    if action == "closed" || action == "converted_to_draft" || action == "locked" {
        return Ok(StatusCode::OK);
    }

    let Some(installation) = payload.installation else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let installation_id = installation.id;

    let Some(repository) = payload.repository else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let Some(pr) = payload.pull_request else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let organization = payload.organization.map_or(String::new(), |o| o.name);

    let api_with_pat = match authenticate_with_access_token(
        &state.octocrab,
        installation_id,
        organization.as_str(),
        &repository.name,
    )
    .await
    {
        Ok(o) => o,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let provider = GitHubProvider::new(api_with_pat);

    // Get pull request
    // Create a custom configuration
    let config = ValidationConfig {
        enforce_conventional_commits: state.config.rules.enforce_title_convention.unwrap_or(false),
        require_work_item_references: state.config.rules.require_work_items,
        auto_label: true,
    };

    // Create a MergeWarden instance with custom configuration
    let warden = MergeWarden::with_config(provider, config);

    // Process a pull request
    let _ = warden
        .process_pull_request(&organization, &repository.name, pr.number)
        .await;

    Ok(StatusCode::OK)
}

fn retrieve_webhook_secret() -> Result<String, CliError> {
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

    Ok(webhook_secret)
}

#[instrument]
fn verify_github_signature(secret: &str, headers: &HeaderMap, body: &str) -> bool {
    let signature = match headers.get("X-Hub-Signature-256") {
        Some(value) => value.to_str().unwrap_or(""),
        None => return false,
    };

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(body.as_bytes());
    let result = mac.finalize();
    let computed_signature = format!("sha256={}", hex::encode(result.into_bytes()));

    signature == computed_signature
}
