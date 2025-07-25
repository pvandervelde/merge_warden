//! Azure Functions implementation for the Merge Warden service.
//!
//! This crate provides the Azure Functions implementation of the Merge Warden webhook handler,
//! including configuration management, secret handling, and telemetry integration.

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use axum::{extract::State, routing::get, routing::post, Router};
use axum_macros::debug_handler;
use azure_core::credentials::TokenCredential;
use azure_identity::ManagedIdentityCredentialOptions;
use azure_security_keyvault_secrets::SecretClient;
use hmac::{Hmac, Mac};
use merge_warden_core::{
    config::{
        load_merge_warden_config, ApplicationDefaults, CurrentPullRequestValidationConfiguration,
    },
    MergeWarden, WebhookPayload,
};
use merge_warden_developer_platforms::{
    github::{authenticate_with_access_token, create_app_client, GitHubProvider},
    models::User,
};
use octocrab::Octocrab;
use reqwest::{header::HeaderMap, StatusCode};
use sha2::Sha256;
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
    /// Authenticated GitHub API client
    pub octocrab: Octocrab,
    /// GitHub user information for the authenticated app
    pub user: User,
    /// Application-wide policy configurations
    pub policies: ApplicationDefaults,
    /// Secret key for webhook payload verification
    pub webhook_secret: String,
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
async fn create_github_app(secrets: &AppSecrets) -> Result<(Octocrab, User), AzureFunctionsError> {
    info!("Creating GitHub app client");
    info!(message = "Using GitHub App authentication");
    let app_id = secrets.app_id;
    let app_key = secrets.app_private_key.as_str();

    let provider = create_app_client(app_id, app_key).await.map_err(|e| {
        AzureFunctionsError::AuthError(
            format!("Failed to load the GitHub provider. Error was: {}", e).to_string(),
        )
    })?;
    debug!(message = "GitHub App client created successfully");

    Ok(provider)
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
/// This function processes GitHub webhook events for pull request operations.
/// It validates webhook signatures, parses the payload, and delegates processing
/// to the Merge Warden engine for PR validation and management.
///
/// # Arguments
///
/// * `state` - Application state containing GitHub client and configuration
/// * `headers` - HTTP request headers including webhook signature
/// * `body` - JSON payload from GitHub webhook
///
/// # Returns
///
/// * `Ok(StatusCode::OK)` - Successfully processed the webhook
/// * `Err(StatusCode::UNAUTHORIZED)` - Invalid webhook signature
/// * `Err(StatusCode::BAD_REQUEST)` - Malformed payload or missing required fields
///
/// # Security
///
/// All webhook payloads are verified using HMAC-SHA256 signature validation
/// before processing to ensure they originated from GitHub.
#[debug_handler]
#[instrument(skip(state, headers, body))]
async fn handle_post_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, StatusCode> {
    info!("Received post request ...");

    if !verify_github_signature(&state.webhook_secret, &headers, &body) {
        warn!("Webhook did not have valid signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!("Webhook has valid signature. Processing information ...");

    let payload: WebhookPayload = serde_json::from_str(&body).map_err(|e| {
        error!(
            body = body.clone(),
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
        && action != "synchronize"
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
        &state.policies,
    )
    .await
    {
        Ok(merge_warden_config) => {
            info!(
                "Loaded merge-warden config from {}",
                merge_warden_config_path
            );
            merge_warden_config.to_validation_config(&state.policies.bypass_rules)
        }
        Err(e) => {
            warn!(
                "Failed to load merge-warden config from {}: {}. Falling back to defaults.",
                merge_warden_config_path, e
            );
            CurrentPullRequestValidationConfiguration {
                enforce_title_convention: state.policies.enable_title_validation,
                title_pattern: state.policies.default_title_pattern.clone(),
                invalid_title_label: state.policies.default_invalid_title_label.clone(),
                enforce_work_item_references: state.policies.enable_work_item_validation,
                work_item_reference_pattern: state.policies.default_work_item_pattern.clone(),
                missing_work_item_label: state.policies.default_missing_work_item_label.clone(),
                pr_size_check: state.policies.pr_size_check.clone(),
                change_type_labels: Some(state.policies.change_type_labels.clone()),
                bypass_rules: state.policies.bypass_rules.clone(),
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

/// Verifies the authenticity of a GitHub webhook payload using HMAC-SHA256 signature validation.
///
/// This function implements GitHub's webhook security mechanism by computing an HMAC-SHA256
/// signature of the payload using the configured webhook secret and comparing it against
/// the signature provided in the `X-Hub-Signature-256` header.
///
/// # Arguments
///
/// * `secret` - The webhook secret configured in GitHub and stored securely
/// * `headers` - HTTP request headers containing the GitHub signature
/// * `body` - The raw request body that was signed by GitHub
///
/// # Returns
///
/// * `true` - The signature is valid and the payload is authentic
/// * `false` - The signature is invalid, missing, or the payload was tampered with
///
/// # Security
///
/// This function is critical for webhook security. It prevents:
/// - Replay attacks from malicious actors
/// - Payload tampering during transit
/// - Unauthorized webhook submissions
///
/// The GitHub signature format is: `sha256=<hex_encoded_hmac>`
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
    debug!(
        github_signature = signature,
        computed_signature, "Comparing the GitHub signature with the computed signature"
    );

    signature == computed_signature
}

#[tokio::main]
async fn main() -> Result<(), AzureFunctionsError> {
    let instrumentation_connection_string = env::var("APPLICATIONINSIGHTS_CONNECTION_STRING")
        .expect("APPLICATIONINSIGHTS_CONNECTION_STRING not set");
    telemetry::init_telemetry(&instrumentation_connection_string).await?;

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

    let (octocrab, user) = create_github_app(&app_secrets).await?;

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };
    debug!(port = port, "Got the port from the environment variables");

    let addr = format!("0.0.0.0:{}", port);

    let state = Arc::new(AppState {
        octocrab,
        user,
        policies: application_config,
        webhook_secret: app_secrets.webhook_secret,
    });

    let app = Router::new()
        .route("/api/merge_warden", get(handle_get_request))
        .route("/api/merge_warden", post(handle_post_request))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
