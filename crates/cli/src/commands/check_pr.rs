use async_trait::async_trait;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use axum_macros::debug_handler;
use clap::Args;
use github_bot_sdk::{
    auth::{GitHubAppId, InstallationId, PrivateKey, SecretProvider},
    client::{ClientConfig, GitHubClient},
    error::SecretError,
    events::{EventEnvelope, EventProcessor, ProcessorConfig},
    webhook::{WebhookHandler, WebhookReceiver, WebhookRequest},
};
use keyring::Entry;
use merge_warden_core::config::{
    load_merge_warden_config, CurrentPullRequestValidationConfiguration,
};
use merge_warden_core::MergeWarden;
use merge_warden_developer_platforms::app_auth::AppAuthProvider;
use merge_warden_developer_platforms::github::GitHubProvider;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use crate::commands::auth::{
    KEY_RING_APP_ID, KEY_RING_APP_PRIVATE_KEY_PATH, KEY_RING_SERVICE_NAME,
};
use crate::config::{get_config_path, AppConfig};
use crate::errors::CliError;

use super::auth::KEY_RING_WEB_HOOK_SECRET;

/// Application state for the PR checking functionality
pub struct AppState {
    /// Webhook receiver: validates HMAC-SHA256 signatures and dispatches events
    pub receiver: WebhookReceiver,
    /// Event processor, kept separately for the signature-bypass dev path
    pub processor: EventProcessor,
    /// The merge-warden business logic handler, kept separately for the bypass path
    pub handler: Arc<dyn WebhookHandler>,
    /// When true (set via MERGE_WARDEN_SKIP_SIGNATURE_VALIDATION env var), signature
    /// validation is skipped. For development proxy scenarios only.
    pub skip_signature_validation: bool,
}

/// `SecretProvider` backed by a webhook secret already loaded from the keyring.
///
/// `SignatureValidator` (inside `WebhookReceiver`) only calls `get_webhook_secret`.
/// The other methods are not invoked here and return `SecretError::NotFound`.
struct WebhookSecretProvider {
    /// The webhook secret retrieved from the system keyring at startup
    webhook_secret: String,
}

#[async_trait]
impl SecretProvider for WebhookSecretProvider {
    async fn get_private_key(&self) -> Result<PrivateKey, SecretError> {
        Err(SecretError::NotFound {
            key: "private_key".to_string(),
        })
    }

    async fn get_app_id(&self) -> Result<GitHubAppId, SecretError> {
        Err(SecretError::NotFound {
            key: "app_id".to_string(),
        })
    }

    async fn get_webhook_secret(&self) -> Result<String, SecretError> {
        Ok(self.webhook_secret.clone())
    }

    fn cache_duration(&self) -> chrono::Duration {
        chrono::Duration::hours(1)
    }
}

/// `WebhookHandler` implementation that runs Merge Warden's PR validation logic.
///
/// Registered on the `WebhookReceiver` at startup. Handles `pull_request` events
/// by creating an installation-scoped GitHub client and running `process_pull_request`.
struct MergeWardenWebhookHandler {
    /// App-level GitHub client; used to create installation-scoped clients per event
    github_client: GitHubClient,
    /// Application configuration loaded from the config file
    config: AppConfig,
}

#[async_trait]
impl WebhookHandler for MergeWardenWebhookHandler {
    async fn handle_event(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if envelope.event_type != "pull_request" {
            return Ok(());
        }

        let action = envelope.payload.raw()["action"].as_str().unwrap_or("");
        match action {
            "opened" | "edited" | "ready_for_review" | "reopened" | "unlocked" | "synchronize" => {}
            _ => {
                info!(action, "Pull request action does not require processing");
                return Ok(());
            }
        }

        let pr_number = envelope
            .entity_id
            .as_deref()
            .and_then(|id| id.parse::<u32>().ok())
            .or_else(|| {
                envelope.payload.raw()["pull_request"]["number"]
                    .as_u64()
                    .map(|n| n as u32)
            });

        let pr_number = match pr_number {
            Some(n) => n,
            None => {
                error!(
                    repository = envelope.repository.full_name.as_str(),
                    "Webhook payload missing pull request number"
                );
                return Err(Box::from("Missing pull request number in webhook payload"));
            }
        };

        let installation_id = match envelope.payload.raw()["installation"]["id"].as_u64() {
            Some(id) => id,
            None => {
                error!(
                    repository = envelope.repository.full_name.as_str(),
                    pull_request = pr_number,
                    "Webhook payload missing installation ID"
                );
                return Err(Box::from("Missing installation ID in webhook payload"));
            }
        };

        let repo_owner = &envelope.repository.owner.login;
        let repo_name = &envelope.repository.name;

        info!(
            repository_owner = repo_owner.as_str(),
            repository = repo_name.as_str(),
            pull_request = pr_number,
            action,
            "Processing pull request"
        );

        let installation_client = self
            .github_client
            .installation_by_id(InstallationId::new(installation_id))
            .await
            .map_err(|e| {
                error!(
                    repository_owner = repo_owner.as_str(),
                    repository = repo_name.as_str(),
                    pull_request = pr_number,
                    error = e.to_string(),
                    "Failed to create installation client"
                );
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        let provider = GitHubProvider::new(installation_client);

        let merge_warden_config_path = ".github/merge-warden.toml";
        let validation_config = match load_merge_warden_config(
            repo_owner,
            repo_name,
            merge_warden_config_path,
            &provider,
            &self.config.policies,
        )
        .await
        {
            Ok(merge_warden_config) => {
                info!(
                    "Loaded merge-warden config from {}",
                    merge_warden_config_path
                );
                merge_warden_config.to_validation_config(&self.config.policies.bypass_rules)
            }
            Err(e) => {
                warn!(
                    "Failed to load merge-warden config from {}: {}. Falling back to defaults.",
                    merge_warden_config_path, e
                );
                CurrentPullRequestValidationConfiguration {
                    enforce_title_convention: self.config.policies.enable_title_validation,
                    title_pattern: self.config.policies.default_title_pattern.clone(),
                    invalid_title_label: self.config.policies.default_invalid_title_label.clone(),
                    enforce_work_item_references: self.config.policies.enable_work_item_validation,
                    work_item_reference_pattern: self
                        .config
                        .policies
                        .default_work_item_pattern
                        .clone(),
                    missing_work_item_label: self
                        .config
                        .policies
                        .default_missing_work_item_label
                        .clone(),
                    pr_size_check: merge_warden_core::config::PrSizeCheckConfig::default(),
                    change_type_labels: Some(
                        merge_warden_core::config::ChangeTypeLabelConfig::default(),
                    ),
                    bypass_rules: self.config.policies.bypass_rules.clone(),
                    wip_check: self.config.policies.wip_check.clone(),
                    pr_state_labels: self.config.policies.pr_state_labels.clone(),
                    issue_propagation: Default::default(),
                }
            }
        };

        let warden = MergeWarden::with_config(provider, validation_config);

        info!(
            message = "Processing pull request",
            pull_request = pr_number,
            repository = repo_name.as_str()
        );

        warden
            .process_pull_request(repo_owner, repo_name, pr_number.into())
            .await
            .map_err(|e| {
                error!(
                    repository_owner = repo_owner.as_str(),
                    repository = repo_name.as_str(),
                    pull_request = pr_number,
                    error = e.to_string(),
                    "Failed to process pull request"
                );
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        Ok(())
    }
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
/// Returns a `Result` containing a `GitHubClient` instance if successful, or a `CliError`
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
async fn create_github_app(config: &AppConfig) -> Result<GitHubClient, CliError> {
    debug!("Creating GitHub app client");
    match config.authentication.auth_method.as_str() {
        "token" => {
            let err = CliError::InvalidArguments(
                format!(
                    "Unsupported authentication method: {}",
                    config.authentication.auth_method
                )
                .to_string(),
            );
            error!(message = "Failed to create GitHub app client", error = ?err);
            Err(err)
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

            let auth = AppAuthProvider::new(app_id_number, &app_key, "https://api.github.com")
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to create GitHub App auth provider: {}", e))
                })?;

            let client = GitHubClient::builder(auth)
                .config(ClientConfig::default())
                .build()
                .map_err(|e| {
                    CliError::AuthError(format!("Failed to build GitHub client: {}", e))
                })?;

            debug!(message = "GitHub App client created successfully");
            Ok(client)
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
            Err(err)
        }
    }
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
    let config_path = get_config_path(args.config.as_deref());
    let config = AppConfig::load(&config_path)
        .map_err(|e| CliError::ConfigError(format!("Failed to load configuration: {}", e)))?;

    let github_client = create_github_app(&config).await?;
    let webhook_secret = retrieve_webhook_secret()?;

    let skip_validation = env::var("MERGE_WARDEN_SKIP_SIGNATURE_VALIDATION")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if skip_validation {
        warn!("Signature validation bypassed via MERGE_WARDEN_SKIP_SIGNATURE_VALIDATION=true — do not use in production");
    }

    let addr = format!("0.0.0.0:{}", config.webhooks.port);

    let secret_provider: Arc<dyn SecretProvider> =
        Arc::new(WebhookSecretProvider { webhook_secret });
    let processor_config = ProcessorConfig::default();
    let bypass_processor = EventProcessor::new(processor_config.clone());
    let receiver_processor = EventProcessor::new(processor_config);
    let handler: Arc<dyn WebhookHandler> = Arc::new(MergeWardenWebhookHandler {
        github_client,
        config,
    });
    let mut receiver = WebhookReceiver::new(secret_provider, receiver_processor);
    receiver.add_handler(handler.clone()).await;

    let state = Arc::new(AppState {
        receiver,
        processor: bypass_processor,
        handler,
        skip_signature_validation: skip_validation,
    });

    let app = Router::new()
        .route("/api/merge_warden", post(handle_webhook))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

/// Handles incoming GitHub webhook requests in CLI webhook server mode.
///
/// In normal operation the HMAC-SHA256 signature is validated by `WebhookReceiver`.
/// When `MERGE_WARDEN_SKIP_SIGNATURE_VALIDATION=true` (dev-proxy mode only),
/// the signature check is bypassed and the event is dispatched directly.
///
/// # Arguments
///
/// * `state` - Application state containing the receiver, processor, and handler
/// * `headers` - HTTP request headers including the event type and optional signature
/// * `body` - Raw webhook payload bytes
///
/// # Returns
///
/// * `Ok(StatusCode::OK)` - Successfully accepted the webhook
/// * `Err(StatusCode::UNAUTHORIZED)` - Invalid or missing webhook signature
/// * `Err(StatusCode::BAD_REQUEST)` - Malformed payload or missing event header
#[debug_handler]
#[instrument(skip(state, headers, body))]
async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    info!("Received webhook call from Github");

    // Dev-proxy bypass: skip signature validation but still parse and dispatch.
    if state.skip_signature_validation {
        let event_type = headers
            .get("x-github-event")
            .or_else(|| headers.get("X-GitHub-Event"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let delivery_id = headers
            .get("x-github-delivery")
            .or_else(|| headers.get("X-GitHub-Delivery"))
            .and_then(|v| v.to_str().ok());

        let envelope = state
            .processor
            .process_webhook(event_type, &body, delivery_id)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        if let Err(e) = state.handler.handle_event(&envelope).await {
            error!(error = e.to_string(), "Handler failed in bypass mode");
        }
        // Always return 200 in bypass mode: GitHub must receive a fast response
        // within its 10-second timeout, and the bypass path is only used in
        // dev-proxy scenarios where delivery acknowledgement is decoupled from
        // processing outcome.
        return Ok(StatusCode::OK);
    }

    // Normal path: full signature validation + fire-and-forget dispatch.
    let request = WebhookRequest::new(
        headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|v| (k.as_str().to_lowercase(), v.to_string()))
            })
            .collect::<HashMap<String, String>>(),
        body,
    );

    let response = state.receiver.receive_webhook(request).await;
    match response.status_code() {
        200 => Ok(StatusCode::OK),
        401 => Err(StatusCode::UNAUTHORIZED),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// Retrieves the webhook secret from the system keyring.
///
/// This function accesses the stored webhook secret that was previously saved during
/// authentication. The secret is used to verify the authenticity of incoming webhook payloads.
///
/// # Returns
///
/// Returns `Ok(String)` containing the webhook secret, or an error if the secret
/// cannot be retrieved from the keyring.
///
/// # Errors
///
/// Returns `CliError::AuthError` if:
/// - The keyring entry cannot be created
/// - The webhook secret is not found in the keyring
/// - The keyring access fails
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
