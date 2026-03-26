//! Azure Functions implementation for the Merge Warden service.
//!
//! This crate provides the Azure Functions implementation of the Merge Warden webhook handler,
//! including configuration management, secret handling, and telemetry integration.

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use async_trait::async_trait;
use axum::{body::Bytes, extract::State, http::HeaderMap, routing::get, routing::post, Router};
use axum_macros::debug_handler;
use azure_core::credentials::TokenCredential;
use azure_identity::ManagedIdentityCredentialOptions;
use azure_security_keyvault_secrets::SecretClient;
use github_bot_sdk::{
    auth::{GitHubAppId, InstallationId, PrivateKey, SecretProvider},
    client::{ClientConfig, GitHubClient},
    error::SecretError,
    events::{EventEnvelope, EventProcessor, ProcessorConfig},
    webhook::{WebhookHandler, WebhookReceiver, WebhookRequest},
};
use merge_warden_core::{
    config::{
        load_merge_warden_config, ApplicationDefaults, CurrentPullRequestValidationConfiguration,
    },
    MergeWarden,
};
use merge_warden_developer_platforms::{app_auth::AppAuthProvider, github::GitHubProvider};
use reqwest::StatusCode;
use std::{env, sync::Arc};
use tracing::{debug, error, info, instrument, warn};

/// Error handling for Azure Functions specific operations
mod errors;
use errors::AzureFunctionsError;

/// Azure App Configuration client for retrieving application settings
mod app_config_client;
/// Telemetry and observability configuration
mod telemetry;

use app_config_client::AppConfigClient;

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

/// Application secrets retrieved from Azure Key Vault
struct AppSecrets {
    /// GitHub App ID for authentication
    app_id: u64,
    /// GitHub App private key for JWT token generation
    app_private_key: String,
    /// Webhook secret for payload verification
    webhook_secret: String,
}

/// Application state shared across Azure Function handlers
pub struct AppState {
    /// Webhook receiver that validates signatures and dispatches events to handlers
    pub receiver: WebhookReceiver,
}

/// `SecretProvider` backed by a webhook secret already loaded into memory at startup.
///
/// `SignatureValidator` (used internally by `WebhookReceiver`) only calls
/// `get_webhook_secret`. The other methods are not invoked in this context and
/// return `SecretError::NotFound` as a safe default.
struct WebhookSecretProvider {
    /// The webhook secret used to validate HMAC-SHA256 signatures
    webhook_secret: String,
}

#[async_trait]
impl SecretProvider for WebhookSecretProvider {
    async fn get_private_key(&self) -> Result<PrivateKey, SecretError> {
        // Not called by SignatureValidator — only needed for JWT signing, which
        // is handled separately by AppAuthProvider.
        Err(SecretError::NotFound {
            key: "private_key".to_string(),
        })
    }

    async fn get_app_id(&self) -> Result<GitHubAppId, SecretError> {
        // Not called by SignatureValidator.
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
/// Registered on the `WebhookReceiver` during startup. Each `pull_request` event
/// is dispatched here asynchronously after the HTTP response has been sent to
/// GitHub, satisfying GitHub's 10-second webhook timeout requirement.
struct MergeWardenHandler {
    /// App-level GitHub client; used to create installation-scoped clients per event
    github_client: GitHubClient,
    /// Application policy defaults loaded from Azure App Configuration
    policies: ApplicationDefaults,
}

#[async_trait]
impl WebhookHandler for MergeWardenHandler {
    async fn handle_event(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if envelope.event_type != "pull_request" {
            return Ok(());
        }

        // Filter to the subset of actions that require PR re-validation.
        let action = envelope.payload.raw()["action"].as_str().unwrap_or("");
        match action {
            "opened" | "edited" | "ready_for_review" | "reopened" | "unlocked" | "synchronize" => {}
            _ => {
                info!(action, "Pull request action does not require processing");
                return Ok(());
            }
        }

        // entity_id is set by EventProcessor for pull_request events.
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
            &self.policies,
        )
        .await
        {
            Ok(merge_warden_config) => {
                info!(
                    "Loaded merge-warden config from {}",
                    merge_warden_config_path
                );
                merge_warden_config.to_validation_config(&self.policies.bypass_rules)
            }
            Err(e) => {
                warn!(
                    "Failed to load merge-warden config from {}: {}. Falling back to defaults.",
                    merge_warden_config_path, e
                );
                CurrentPullRequestValidationConfiguration {
                    enforce_title_convention: self.policies.enable_title_validation,
                    title_pattern: self.policies.default_title_pattern.clone(),
                    invalid_title_label: self.policies.default_invalid_title_label.clone(),
                    enforce_work_item_references: self.policies.enable_work_item_validation,
                    work_item_reference_pattern: self.policies.default_work_item_pattern.clone(),
                    missing_work_item_label: self.policies.default_missing_work_item_label.clone(),
                    pr_size_check: self.policies.pr_size_check.clone(),
                    change_type_labels: Some(self.policies.change_type_labels.clone()),
                    bypass_rules: self.policies.bypass_rules.clone(),
                    wip_check: self.policies.wip_check.clone(),
                    pr_state_labels: self.policies.pr_state_labels.clone(),
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

/// Creates a GitHub application client from the provided secrets.
async fn create_github_app(secrets: &AppSecrets) -> Result<GitHubClient, AzureFunctionsError> {
    info!("Creating GitHub app client");
    info!(message = "Using GitHub App authentication");

    let auth = AppAuthProvider::new(
        secrets.app_id,
        &secrets.app_private_key,
        "https://api.github.com",
    )
    .map_err(|e| {
        AzureFunctionsError::AuthError(format!("Failed to create GitHub App auth provider: {}", e))
    })?;

    let client = GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .map_err(|e| {
            AzureFunctionsError::AuthError(format!("Failed to build GitHub client: {}", e))
        })?;

    debug!(message = "GitHub App client created successfully");
    Ok(client)
}

/// Retrieves application secrets from Azure Key Vault.
///
/// This function connects to Azure Key Vault using managed identity and retrieves
/// the necessary GitHub App credentials and webhook secret.
///
/// # Returns
///
/// Returns `Ok(AppSecrets)` containing the GitHub App ID, private key, and webhook secret,
/// or an error if any secrets cannot be retrieved or parsed.
///
/// # Errors
///
/// Returns `AzureFunctionsError::ConfigError` if:
/// - The KEY_VAULT_NAME environment variable is not set
/// - Any required secret cannot be retrieved from Key Vault
/// - The GitHub App ID cannot be parsed as a number
async fn get_azure_secrets() -> Result<AppSecrets, AzureFunctionsError> {
    let key_vault_name = env::var("KEY_VAULT_NAME").map_err(|e| {
        error!(
            error = e.to_string(),
            "Failed to get the KeyVault name from the environment variables"
        );
        AzureFunctionsError::ConfigError(
            "Failed to get the KeyVault name from the environment variables".to_string(),
        )
    })?;
    let key_vault_url = format!("https://{}.vault.azure.net", key_vault_name);

    info!(
        keyvault = key_vault_url.as_str(),
        "Fetching secrets from Azure Key Vault",
    );
    let app_id = get_secret_from_keyvault(key_vault_url.as_str(), "GithubAppId").await?;
    let app_private_key =
        get_secret_from_keyvault(key_vault_url.as_str(), "GithubAppPrivateKey").await?;
    let app_id_to_number = app_id.parse::<u64>().map_err(|e| {
        error!(
            error = e.to_string(),
            app_id = app_id,
            "Failed to parse the app ID",
        );
        AzureFunctionsError::ConfigError("The app ID was not a number".to_string())
    })?;
    debug!(
        keyvault = key_vault_url.as_str(),
        app_id = app_id_to_number,
        "Got app key from Azure Key Vault",
    );

    let webhook_secret =
        get_secret_from_keyvault(key_vault_url.as_str(), "GithubWebhookSecret").await?;
    debug!(
        keyvault = key_vault_url.as_str(),
        "Got webhook secret from Azure Key Vault",
    );

    let secrets = AppSecrets {
        app_id: app_id_to_number,
        app_private_key,
        webhook_secret,
    };

    Ok(secrets)
}

/// Retrieves application configuration from Azure App Configuration.
///
/// This function connects to Azure App Configuration using managed identity and retrieves
/// the application defaults and policies for the Merge Warden service.
///
/// # Returns
///
/// Returns `Ok(ApplicationDefaults)` containing the loaded configuration,
/// or an error if the configuration cannot be retrieved or parsed.
///
/// # Errors
///
/// Returns `AzureFunctionsError::ConfigError` if:
/// - The APP_CONFIG_ENDPOINT environment variable is not set
/// - The App Configuration client cannot be created
/// - Configuration values cannot be retrieved or parsed
async fn get_application_config() -> Result<ApplicationDefaults, AzureFunctionsError> {
    let app_config_endpoint = env::var("APP_CONFIG_ENDPOINT").map_err(|e| {
        error!(
            error = e.to_string(),
            "Failed to get the App Configuration endpoint from the environment variables"
        );
        AzureFunctionsError::ConfigError(
            "Failed to get the App Configuration endpoint from the environment variables"
                .to_string(),
        )
    })?;

    info!(
        endpoint = app_config_endpoint.as_str(),
        "Loading configuration from Azure App Configuration",
    );

    let app_config_client =
        AppConfigClient::new(&app_config_endpoint, std::time::Duration::from_secs(600)).map_err(
            |e| {
                error!(
                    error = e.to_string(),
                    "Failed to create App Configuration client"
                );
                AzureFunctionsError::ConfigError(format!(
                    "Failed to create App Configuration client: {}",
                    e
                ))
            },
        )?;

    let application_defaults = app_config_client
        .load_application_defaults()
        .await
        .map_err(|e| {
            warn!(
                error = e.to_string(),
                "Failed to load configuration from App Configuration, using fallback defaults"
            );
            // Return default configuration instead of failing
            AzureFunctionsError::ConfigError(format!("Failed to load configuration: {}", e))
        })?;

    info!("Successfully loaded configuration from Azure App Configuration");
    Ok(application_defaults)
}

/// Retrieves a secret from Azure Key Vault using managed identity.
///
/// This function authenticates with Azure Key Vault using managed identity credentials
/// and retrieves the specified secret value.
///
/// # Arguments
///
/// * `key_vault_url` - The URL of the Azure Key Vault (e.g., "https://vault-name.vault.azure.net")
/// * `secret_name` - The name of the secret to retrieve
///
/// # Returns
///
/// Returns `Ok(String)` containing the secret value, or an error if the secret
/// cannot be retrieved.
///
/// # Errors
///
/// Returns `AzureFunctionsError` if:
/// - Managed identity credentials cannot be created
/// - Authentication with Key Vault fails
/// - The secret does not exist or cannot be accessed
/// - The Key Vault client cannot be created
async fn get_secret_from_keyvault(
    key_vault_url: &str,
    secret_name: &str,
) -> Result<String, AzureFunctionsError> {
    // Use ManagedIdentityCredential for Azure Functions in production
    // correct resource for Key Vault
    let credential = azure_identity::ManagedIdentityCredential::new(Some(
        ManagedIdentityCredentialOptions::default(),
    ))
    .map_err(|e| {
        error!(
            error = e.to_string(),
            "Failed to create the managed credential."
        );
        AzureFunctionsError::AuthError("Failed to create the managed credential.".to_string())
    })?;

    // Ask for a token for Key Vault
    let token_response = credential
        .get_token(&["https://vault.azure.net/.default"])
        .await
        .map_err(|e| {
            error!("Failed to get token: {}", e);
            AzureFunctionsError::Other(format!("token error: {}", e))
        })?;

    debug!("Access Token acquired:");
    debug!("Token: {}", token_response.token.secret());
    debug!("Expires on: {:?}", token_response.expires_on);

    let client = SecretClient::new(key_vault_url, credential, None).map_err(|e| {
        error!(
            error = e.to_string(),
            "Failed to create an Azure KeyVault client."
        );
        AzureFunctionsError::AuthError("Failed to create an Azure KeyVault client.".to_string())
    })?;

    let secret = client
        .get_secret(secret_name, "", None)
        .await
        .map_err(|e| {
            error!(
                error = e.to_string(),
                "Failed to get a secret from the KeyVault."
            );
            AzureFunctionsError::AuthError("Failed to get a secret from the KeyVault.".to_string())
        })?;
    let value = secret.into_body().await.map_err(|e| {
        error!(
            error = e.to_string(),
            "Failed to get a secret from the KeyVault."
        );
        AzureFunctionsError::AuthError(
            "Failed to extract the secret from the data obtained from the KeyVault.".to_string(),
        )
    })?;
    Ok(value.value.unwrap_or_default())
}

/// Handles HTTP GET requests to the Azure Function endpoint.
///
/// This function serves as a health check endpoint that returns HTTP 200 OK
/// for GET requests. It's primarily used for monitoring and service health verification.
///
/// # Arguments
///
/// * `_state` - Application state (unused for GET requests)
/// * `_headers` - HTTP request headers (unused for GET requests)
/// * `_body` - Request body (unused for GET requests)
///
/// # Returns
///
/// Returns `Ok(StatusCode::OK)` for successful health checks
#[instrument(skip(_state, _headers, _body))]
async fn handle_get_request(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
    _body: String,
) -> Result<StatusCode, StatusCode> {
    info!("Received get request ...");

    Ok(StatusCode::OK)
}

/// Handles HTTP POST requests containing GitHub webhook payloads.
///
/// Converts the axum HTTP request into a `WebhookRequest` and delegates to the
/// `WebhookReceiver`, which validates the HMAC-SHA256 signature via the SDK's
/// `SignatureValidator`, parses the payload into an `EventEnvelope`, and dispatches
/// to the registered `MergeWardenHandler` asynchronously (fire-and-forget) so
/// that GitHub receives its HTTP response within the 10-second timeout.
#[debug_handler]
#[instrument(skip(state, headers, body))]
async fn handle_post_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, StatusCode> {
    info!("Received post request ...");

    let request = WebhookRequest::new(
        headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|v| (k.as_str().to_lowercase(), v.to_string()))
            })
            .collect(),
        Bytes::from(body.into_bytes()),
    );

    let response = state.receiver.receive_webhook(request).await;
    match response.status_code() {
        200 => Ok(StatusCode::OK),
        401 => Err(StatusCode::UNAUTHORIZED),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

#[tokio::main]
async fn main() -> Result<(), AzureFunctionsError> {
    telemetry::init_console_logging()?;

    info!("Starting application");

    debug!("Loading Azure secrets ...");
    let app_secrets = get_azure_secrets().await?;

    debug!("Loading application configuration ...");
    let application_config = get_application_config().await.unwrap_or_else(|e| {
        warn!(
            error = e.to_string(),
            "Failed to load configuration from App Configuration, using fallback defaults"
        );
        ApplicationDefaults::default()
    });

    let github_client = create_github_app(&app_secrets).await?;

    // Build WebhookReceiver using the SDK's SignatureValidator for HMAC-SHA256
    // verification, EventProcessor for JSON parsing, and MergeWardenHandler for
    // the actual PR validation business logic.
    let secret_provider = Arc::new(WebhookSecretProvider {
        webhook_secret: app_secrets.webhook_secret,
    });
    let processor = EventProcessor::new(ProcessorConfig::default());
    let handler = Arc::new(MergeWardenHandler {
        github_client,
        policies: application_config,
    });
    let mut receiver = WebhookReceiver::new(secret_provider, processor);
    receiver.add_handler(handler).await;

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };
    debug!(port = port, "Got the port from the environment variables");

    let addr = format!("0.0.0.0:{}", port);

    let state = Arc::new(AppState { receiver });

    let app = Router::new()
        .route("/api/merge_warden", get(handle_get_request))
        .route("/api/merge_warden", post(handle_post_request))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
