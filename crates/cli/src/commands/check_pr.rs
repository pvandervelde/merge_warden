use anyhow::Result;
use clap::Args;
use merge_warden_developer_platforms::github::GitHubProvider;
use merge_warden_developer_platforms::PullRequestProvider;
use serde::Serialize;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::commands::auth::{
    KEY_RING_APP_ID, KEY_RING_APP_TOKEN, KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN,
};
use crate::config::{get_config_path, Config};
use crate::errors::CliError;

/// Arguments for the check-pr command
#[derive(Args, Debug)]
pub struct CheckPrArgs {
    /// Git provider (github)
    #[arg(short, long)]
    pub provider: String,

    /// Repository in format: owner/repo
    #[arg(short, long)]
    pub repo: String,

    /// Pull request number
    #[arg(short, long)]
    pub pr: u64,

    /// Output results in JSON format
    #[arg(short, long)]
    pub json: bool,

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

/// Execute the check-pr command
pub async fn execute(args: CheckPrArgs) -> Result<ValidationResult, CliError> {
    debug!("Executing check-pr command with args: {:?}", args);

    // Parse repository owner and name
    let repo_parts: Vec<&str> = args.repo.split('/').collect();
    if repo_parts.len() != 2 {
        return Err(CliError::InvalidArguments(
            "Repository must be in format: owner/repo".to_string(),
        ));
    }
    let owner = repo_parts[0];
    let repo = repo_parts[1];

    // Load configuration
    let config_path = get_config_path(args.config.as_deref());
    let config = Config::load(&config_path)
        .map_err(|e| CliError::ConfigError(format!("Failed to load configuration: {}", e)))?;

    use keyring::Entry;

    // Create provider based on the specified provider
    let provider: Box<dyn PullRequestProvider> = match args.provider.as_str() {
        "github" => match config.authentication.auth_method.as_str() {
            "token" => {
                let github_token = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN)
                    .map_err(|e| {
                        CliError::AuthError(format!(
                            "Failed to create an entry in the keyring: {}",
                            e
                        ))
                    })?
                    .get_password()
                    .map_err(|e| {
                        CliError::AuthError(format!("Failed to get token from keyring: {}", e))
                    })?;

                info!("Using GitHub token authentication");
                let provider = GitHubProvider::from_token(&github_token).map_err(|e| {
                    CliError::AuthError("Failed to load the GitHub provider".to_string())
                })?;

                Box::new(provider)
            }
            "app" => {
                let app_id = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID)
                    .map_err(|e| {
                        CliError::AuthError(format!(
                            "Failed to create an entry in the keyring: {}",
                            e
                        ))
                    })?
                    .get_password()
                    .map_err(|e| {
                        CliError::AuthError(format!("Failed to get app id from keyring: {}", e))
                    })?;

                let app_key = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_TOKEN)
                    .map_err(|e| {
                        CliError::AuthError(format!(
                            "Failed to create an entry in the keyring: {}",
                            e
                        ))
                    })?
                    .get_password()
                    .map_err(|e| {
                        CliError::AuthError(format!("Failed to get app key from keyring: {}", e))
                    })?;

                info!("Using GitHub token authentication");
                let provider = GitHubProvider::from_app(&app_id, &app_key).map_err(|e| {
                    CliError::AuthError("Failed to load the GitHub provider".to_string())
                })?;

                Box::new(provider)
            }
            _ => {
                return Err(CliError::InvalidArguments(format!(
                    "Unsupported authentication method: {}",
                    config.authentication.auth_method
                )))
            }
        },
        _ => {
            return Err(CliError::InvalidArguments(format!(
                "Unsupported provider: {}",
                args.provider
            )));
        }
    };

    // Get pull request
    info!("Fetching pull request {}/{}/#{}", owner, repo, args.pr);
    let pull_request = provider
        .get_pull_request(owner, repo, args.pr)
        .await
        .map_err(|e| CliError::NetworkError(format!("Failed to fetch pull request: {}", e)))?;

    // Validate pull request
    info!("Validating pull request");
    let mut failures = Vec::new();

    // Check work items if required
    if config.rules.require_work_items {
        // Check if the PR body contains work item references
        let has_work_item = if let Some(body) = &pull_request.body {
            // Simple check for work item references like "Fixes #123" or "Closes #456"
            body.to_lowercase().contains("fix") && body.contains("#")
                || body.to_lowercase().contains("close") && body.contains("#")
                || body.to_lowercase().contains("resolve") && body.contains("#")
                || body.to_lowercase().contains("reference") && body.contains("#")
        } else {
            false
        };

        if !has_work_item {
            failures.push("No work items linked to the pull request".to_string());
        }
    }

    // Check title convention if required
    if let Some(convention) = &config.rules.enforce_title_convention {
        match convention.as_str() {
            "conventional" => {
                // Simple check for conventional commits format
                let title = &pull_request.title;
                if !title.contains(':') || !title.split(':').next().unwrap().contains('(') {
                    failures.push(format!(
                        "Pull request title '{}' does not follow conventional commits format",
                        title
                    ));
                }
            }
            _ => {
                failures.push(format!("Unknown title convention: {}", convention));
            }
        }
    }

    // Note: We can't check approvals here as the PullRequest model doesn't include that information
    // In a real implementation, we would need to fetch the reviews separately
    if let Some(min_approvals) = config.rules.min_approvals {
        failures.push(format!(
            "Cannot check approvals in CLI mode. Minimum required: {}",
            min_approvals
        ));
    }

    // Create result
    let result = ValidationResult {
        passed: failures.is_empty(),
        failures,
    };

    // Output result in JSON format if requested
    if args.json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }

    Ok(result)
}
