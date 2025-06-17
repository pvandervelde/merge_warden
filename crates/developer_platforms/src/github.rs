use async_trait::async_trait;
use base64::Engine;
use jsonwebtoken::EncodingKey;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    errors::Error,
    models::{Comment, Label, PullRequest, User},
    ConfigFetcher, PullRequestProvider,
};

#[cfg(test)]
#[path = "github_tests.rs"]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    iat: u64,
    exp: u64,
    iss: u64,
}

/// Authenticates with GitHub using an installation access token for a specific app installation.
///
/// This function retrieves an access token for a GitHub App installation and creates a new
/// `Octocrab` client authenticated with that token. It is useful for performing API operations
/// on behalf of a GitHub App installation.
///
/// # Arguments
///
/// * `octocrab` - An existing `Octocrab` client instance.
/// * `installation_id` - The ID of the GitHub App installation.
/// * `repository_owner` - The owner of the repository associated with the installation.
/// * `source_repository` - The name of the repository associated with the installation.
///
/// # Returns
///
/// A `Result` containing a new `Octocrab` client authenticated with the installation access token,
/// or an `Error` if the operation fails.
///
/// # Errors
///
/// This function returns an `Error` in the following cases:
/// - If the app installation cannot be found.
/// - If the access token cannot be created.
/// - If the new `Octocrab` client cannot be built.
///
/// # Example
///
/// ```rust,no_run
/// use anyhow::Result;
/// use octocrab::Octocrab;
/// use merge_warden_developer_platforms::github::authenticate_with_access_token;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let octocrab = Octocrab::builder().build().unwrap();
///     let installation_id = 12345678; // Replace with your installation ID
///     let repository_owner = "example-owner";
///     let source_repository = "example-repo";
///
///     let authenticated_client = authenticate_with_access_token(
///         &octocrab,
///         installation_id,
///         repository_owner,
///         source_repository,
///     )
///     .await?;
///
///     // Use `authenticated_client` to perform API operations
///     Ok(())
/// }
/// ```
#[instrument]
pub async fn authenticate_with_access_token(
    octocrab: &Octocrab,
    installation_id: u64,
    repository_owner: &str,
    source_repository: &str,
) -> Result<Octocrab, Error> {
    debug!(
        repository_owner = repository_owner,
        repository = source_repository,
        installation_id,
        "Finding installation"
    );

    let (api_with_token, _) = octocrab
        .installation_and_token(installation_id.into())
        .await
        .map_err(|_| {
            error!(
                repository_owner = repository_owner,
                repository = source_repository,
                installation_id,
                "Failed to create a token for the installation",
            );

            Error::InvalidResponse
        })?;

    info!(
        repository_owner = repository_owner,
        repository = source_repository,
        installation_id,
        "Created access token for installation",
    );

    Ok(api_with_token)
}

/// Creates an `Octocrab` client authenticated as a GitHub App using a JWT token.
///
/// This function generates a JSON Web Token (JWT) for the specified GitHub App ID and private key,
/// and uses it to create an authenticated `Octocrab` client. The client can then be used to perform
/// API operations on behalf of the GitHub App.
///
/// # Arguments
///
/// * `app_id` - The ID of the GitHub App.
/// * `private_key` - The private key associated with the GitHub App, in PEM format.
///
/// # Returns
///
/// A `Result` containing an authenticated `Octocrab` client, or an `Error` if the operation fails.
///
/// # Errors
///
/// This function returns an `Error` in the following cases:
/// - If the private key cannot be parsed.
/// - If the JWT token cannot be created.
/// - If the `Octocrab` client cannot be built.
///
/// # Example
///
/// ```rust,no_run
/// use anyhow::Result;
/// use merge_warden_developer_platforms::github::create_app_client;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let app_id = 123456; // Replace with your GitHub App ID
///     let private_key = r#"
/// -----BEGIN RSA PRIVATE KEY-----
/// ...
/// -----END RSA PRIVATE KEY-----
/// "#; // Replace with your GitHub App private key
///
///     let client = create_app_client(app_id, private_key).await?;
///
///     // Use `client` to perform API operations
///     Ok(())
/// }
/// ```
#[instrument(skip(private_key))]
pub async fn create_app_client(app_id: u64, private_key: &str) -> Result<(Octocrab, User), Error> {
    //let app_id_struct = AppId::from(app_id);
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|e| {
        Error::AuthError(
            format!("Failed to translate the private key. Error was: {}", e).to_string(),
        )
    })?;

    let octocrab = Octocrab::builder()
        .app(app_id.into(), key)
        .build()
        .map_err(|_| {
            Error::AuthError("Failed to get a personal token for the app install.".to_string())
        })?;

    info!("Created access token for the GitHub app",);

    let author = match octocrab.current().app().await {
        Ok(a) => a,
        Err(e) => {
            log_octocrab_error(
                "Failed to retreive App information for the currently authenticated app",
                e,
            );
            return Err(Error::InvalidResponse);
        }
    };

    let user = User {
        id: author.id.into_inner(),
        login: author.name,
    };

    Ok((octocrab, user))
}

#[instrument(skip(token))]
pub fn create_token_client(token: &str) -> Result<Octocrab, Error> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|_| Error::ApiError())
}

fn log_octocrab_error(message: &str, e: octocrab::Error) {
    match e {
        octocrab::Error::GitHub { source, backtrace } => {
            let err = *source;
            error!(
                error_message = err.message,
                backtrace = backtrace.to_string(),
                "{}. Received an error from GitHub",
                message
            )
        }
        octocrab::Error::UriParse { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. Failed to parse URI.",
            message
        ),

        octocrab::Error::Uri { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}, Failed to parse URI.",
            message
        ),
        octocrab::Error::InvalidHeaderValue { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. One of the header values was invalid.",
            message
        ),
        octocrab::Error::InvalidUtf8 { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. The message wasn't valid UTF-8.",
            message,
        ),
        _ => error!(error_message = e.to_string(), message),
    };
}

#[derive(Debug, Default)]
pub struct GitHubProvider {
    client: Octocrab,
}

impl GitHubProvider {
    pub async fn fetch_default_branch(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<String, crate::errors::Error> {
        let repo = match self.client.repos(repo_owner, repo_name).get().await {
            Ok(r) => r,
            Err(e) => {
                log_octocrab_error("Failed to get repository information", e);
                return Err(Error::InvalidResponse);
            }
        };
        let branch = repo.default_branch.unwrap_or("main".to_string());

        Ok(branch)
    }

    /// Fetch the content of a file from the repository at the given path.
    /// Returns Ok(Some(content)) if found, Ok(None) if not found, or Err on error.
    pub async fn fetch_file_content(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
        reference: Option<&str>,
    ) -> Result<Option<String>, crate::errors::Error> {
        let content_result = self
            .client
            .repos(repo_owner, repo_name)
            .get_content()
            .path(path)
            .r#ref(reference.unwrap_or("main"))
            .send()
            .await;

        match content_result {
            Ok(response) => {
                if let Some(file) = response.items.into_iter().next() {
                    if let Some(content) = file.content {
                        // GitHub API returns base64 encoded content with newlines
                        let decoded = base64::engine::general_purpose::STANDARD
                            .decode(content.replace('\n', ""))
                            .map_err(|_| crate::errors::Error::InvalidResponse)?;
                        let content_str = String::from_utf8(decoded)
                            .map_err(|_| crate::errors::Error::InvalidResponse)?;
                        Ok(Some(content_str))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                // If 404, treat as not found
                if e.to_string().contains("404") {
                    Ok(None)
                } else {
                    Err(crate::errors::Error::ApiError())
                }
            }
        }
    }

    pub fn new(client: Octocrab) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ConfigFetcher for GitHubProvider {
    #[instrument]
    async fn fetch_config(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
    ) -> Result<Option<String>, Error> {
        let default_branch_name = self.fetch_default_branch(repo_owner, repo_name).await?;
        self.fetch_file_content(repo_owner, repo_name, path, Some(&default_branch_name))
            .await
    }
}

#[async_trait]
impl PullRequestProvider for GitHubProvider {
    #[instrument]
    async fn get_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<PullRequest, Error> {
        match self
            .client
            .pulls(repo_owner, repo_name)
            .get(pr_number)
            .await
        {
            Ok(pr) => Ok(PullRequest {
                number: pr.number,
                title: pr.title.unwrap_or(String::new()),
                draft: pr.draft.unwrap_or_default(),
                body: pr.body,
            }),
            Err(e) => {
                log_octocrab_error("Failed to get pull request information", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    #[instrument]
    async fn add_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        comment: &str,
    ) -> Result<(), Error> {
        match self
            .client
            .issues(repo_owner, repo_name)
            .create_comment(pr_number, comment)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log_octocrab_error("Failed to add pull request comment", e);
                return Err(Error::FailedToUpdatePullRequest(
                    "Failed to add comment".to_string(),
                ));
            }
        }
    }

    #[instrument]
    async fn delete_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error> {
        self.client
            .issues(repo_owner, repo_name)
            .delete_comment(comment_id.into())
            .await
            .map_err(|e| {
                warn!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    comment = comment_id,
                    "Failed to delete pr comment",
                );
                Error::FailedToUpdatePullRequest(format!("Failed to delete comment: {}", e))
            })
    }

    #[instrument]
    async fn list_comments(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        let mut current_page = match self
            .client
            .issues(repo_owner, repo_name)
            .list_comments(pr_number)
            .send()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log_octocrab_error("Failed to list comments for pull request", e);
                return Err(Error::InvalidResponse);
            }
        };

        let mut comments = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            comments.extend(new_page.take_items());

            current_page = new_page;
        }

        let result = comments
            .into_iter()
            .map(|c| Comment {
                id: c.id.0,
                body: c.body.unwrap_or_default(),
                user: User {
                    id: c.user.id.into_inner(),
                    login: c.user.login,
                },
            })
            .collect();

        Ok(result)
    }

    #[instrument]
    async fn add_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        match self
            .client
            .issues(repo_owner, repo_name)
            .add_labels(pr_number, labels)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log_octocrab_error("Failed to add new labels", e);
                return Err(Error::FailedToUpdatePullRequest(
                    "Failed to add labels".to_string(),
                ));
            }
        }
    }

    #[instrument]
    async fn remove_label(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        label: &str,
    ) -> Result<(), Error> {
        match self
            .client
            .issues(repo_owner, repo_name)
            .remove_label(pr_number, label)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log_octocrab_error("Failed to remove label", e);
                return Err(Error::FailedToUpdatePullRequest(
                    "Failed to remove label".to_string(),
                ));
            }
        }
    }

    #[instrument]
    async fn list_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        let mut current_page = match self
            .client
            .issues(repo_owner, repo_name)
            .list_labels_for_issue(pr_number)
            .send()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log_octocrab_error("Failed to list all labels for pull request", e);
                return Err(Error::InvalidResponse);
            }
        };

        let mut labels = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            labels.extend(new_page.take_items());

            current_page = new_page;
        }

        let result = labels.into_iter().map(|l| Label { name: l.name }).collect();

        Ok(result)
    }

    #[instrument]
    async fn update_pr_check_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        conclusion: &str,
        output_title: &str,
        output_summary: &str,
    ) -> Result<(), Error> {
        // Get the commit SHA for the PR head
        let pr_data = self
            .client
            .pulls(repo_owner, repo_name)
            .get(pr_number)
            .await
            .map_err(|e| {
                log_octocrab_error("Failed to get PR for check run", e);
                Error::InvalidResponse
            })?;
        let head_sha = pr_data.head.sha;

        // Prepare the check run payload
        let check_name = "MergeWarden";
        let url = format!("/repos/{}/{}/check-runs", repo_owner, repo_name);
        let payload = json!({
            "name": check_name,
            "head_sha": head_sha,
            "status": "completed",
            "conclusion": conclusion,
            "output": {
                "title": output_title,
                "summary": output_summary
            }
        });

        // POST the check run
        self.client._post(url, Some(&payload)).await.map_err(|e| {
            log_octocrab_error("Failed to create/update check run", e);
            Error::FailedToUpdatePullRequest("Failed to create/update check run".to_string())
        })?;
        Ok(())
    }
}
