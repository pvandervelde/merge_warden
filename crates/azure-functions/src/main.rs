use axum::{extract::State, routing::get, routing::post, Router};
use axum_macros::debug_handler;
use azure_core::credentials::TokenCredential;
use azure_identity::ManagedIdentityCredentialOptions;
use azure_security_keyvault_secrets::SecretClient;
use hmac::{Hmac, Mac};
use merge_warden_core::{
    config::{
        load_merge_warden_config, ApplicationDefaults, CurrentPullRequestValidationConfiguration,
        CONVENTIONAL_COMMIT_REGEX, WORK_ITEM_REGEX,
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

mod errors;
use errors::AzureFunctionsError;

mod telemetry;

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

struct AppConfig {
    app_id: u64,
    app_private_key: String,
    webhook_secret: String,
    port_number: u16,
    require_work_items: bool,
    default_title_pattern: Option<String>,
    default_invalid_title_label: Option<String>,

    enforce_title_convention: bool,
    default_work_item_pattern: Option<String>,
    default_missing_work_item_label: Option<String>,
}

pub struct AppState {
    pub octocrab: Octocrab,
    pub user: User,
    pub policies: ApplicationDefaults,
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
async fn create_github_app(config: &AppConfig) -> Result<(Octocrab, User), AzureFunctionsError> {
    info!("Creating GitHub app client");
    info!(message = "Using GitHub App authentication");
    let app_id = config.app_id;
    let app_key = config.app_private_key.as_str();

    let provider = create_app_client(app_id, app_key).await.map_err(|e| {
        AzureFunctionsError::AuthError(
            format!("Failed to load the GitHub provider. Error was: {}", e).to_string(),
        )
    })?;
    debug!(message = "GitHub App client created successfully");

    Ok(provider)
}

async fn get_azure_config() -> Result<AppConfig, AzureFunctionsError> {
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
        "Fetching configuration from Azure Key Vault",
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

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };
    debug!(port = port, "Got the port from the environment variables",);

    let enforce_title_convention_key = "ENFORCE_TITLE_CONVENTION";
    let enforce_title_convention = match env::var(enforce_title_convention_key) {
        Ok(val) => match val.parse() {
            Ok(b) => b,
            Err(_) => {
                error!(
                    input = val,
                    "Failed to parse the {} key", enforce_title_convention_key
                );
                false
            }
        },
        Err(e) => {
            error!(
                error = e.to_string(),
                "Failed to parse the {} key", enforce_title_convention_key
            );
            false
        }
    };

    let title_convention_pattern_key = "TITLE_CONVENTION_PATTERN";
    let title_convention_pattern = match env::var(title_convention_pattern_key) {
        Ok(val) => Some(val),
        Err(e) => {
            info!(
                error = e.to_string(),
                "Failed to parse the {} key", title_convention_pattern_key
            );
            None
        }
    };

    let missing_title_label_key = "MISSING_TITLE_LABEL";
    let missing_title_label = match env::var(missing_title_label_key) {
        Ok(val) => Some(val),
        Err(e) => {
            info!(
                error = e.to_string(),
                "Failed to parse the {} key", missing_title_label_key
            );
            None
        }
    };

    let require_work_items_key = "REQUIRE_WORK_ITEMS";
    let require_work_items = match env::var(require_work_items_key) {
        Ok(val) => match val.parse() {
            Ok(b) => b,
            Err(_) => {
                error!(
                    input = val,
                    "Failed to parse the {} key", require_work_items_key
                );
                false
            }
        },
        Err(e) => {
            error!(
                error = e.to_string(),
                "Failed to parse the {} key", require_work_items_key
            );
            false
        }
    };

    let work_item_pattern_key = "WORK_ITEM_PATTERN";
    let work_item_convention_pattern = match env::var(work_item_pattern_key) {
        Ok(val) => Some(val),
        Err(e) => {
            info!(
                error = e.to_string(),
                "Failed to parse the {} key", work_item_pattern_key
            );
            None
        }
    };

    let missing_work_item_label_key = "MISSING_WORK_ITEM_LABEL";
    let missing_work_item_label = match env::var(missing_work_item_label_key) {
        Ok(val) => Some(val),
        Err(e) => {
            info!(
                error = e.to_string(),
                "Failed to parse the {} key", missing_work_item_label_key
            );
            None
        }
    };

    let config = AppConfig {
        app_id: app_id_to_number,
        app_private_key,
        webhook_secret,
        port_number: port,
        enforce_title_convention,
        default_title_pattern: title_convention_pattern,
        default_invalid_title_label: missing_title_label,
        require_work_items,
        default_work_item_pattern: work_item_convention_pattern,
        default_missing_work_item_label: missing_work_item_label,
    };

    Ok(config)
}

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

#[instrument(skip(_state, _headers, _body))]
async fn handle_get_request(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
    _body: String,
) -> Result<StatusCode, StatusCode> {
    info!("Received get request ...");

    Ok(StatusCode::OK)
}

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
            merge_warden_config.to_validation_config()
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

    debug!("Loading Azure configs ...");
    let app_config = get_azure_config().await?;

    let (octocrab, user) = create_github_app(&app_config).await?;

    let addr = format!("0.0.0.0:{}", app_config.port_number);

    let state = Arc::new(AppState {
        octocrab,
        user,
        policies: ApplicationDefaults {
            enable_title_validation: app_config.enforce_title_convention,
            default_title_pattern: if let Some(pattern) = &app_config.default_title_pattern {
                pattern.to_string()
            } else {
                CONVENTIONAL_COMMIT_REGEX.to_string()
            },
            default_invalid_title_label: app_config.default_invalid_title_label.clone(),
            enable_work_item_validation: app_config.require_work_items,
            default_work_item_pattern: if let Some(pattern) = &app_config.default_work_item_pattern
            {
                pattern.to_string()
            } else {
                WORK_ITEM_REGEX.to_string()
            },
            default_missing_work_item_label: app_config.default_missing_work_item_label.clone(),
        },
        webhook_secret: app_config.webhook_secret,
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
