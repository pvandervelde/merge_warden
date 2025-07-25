use async_trait::async_trait;
use base64::Engine;
use jsonwebtoken::EncodingKey;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    errors::Error,
    models::{Comment, Label, PullRequest, PullRequestFile, User},
    ConfigFetcher, PullRequestProvider,
};

#[cfg(test)]
#[path = "github_tests.rs"]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
/// JWT claims structure for GitHub App authentication.
///
/// This struct represents the claims that are included in a JSON Web Token (JWT)
/// when authenticating as a GitHub App. The JWT is used to obtain installation
/// access tokens from GitHub's API.
///
/// # JWT Authentication Flow
///
/// 1. Create a JWT with these claims signed by the app's private key
/// 2. Use the JWT to authenticate with GitHub's API
/// 3. Exchange the JWT for an installation access token
/// 4. Use the installation token for API operations
struct JWTClaims {
    /// Issued at timestamp (Unix timestamp in seconds).
    ///
    /// This field indicates when the JWT was created. GitHub requires
    /// this to be no more than 60 seconds in the past.
    iat: u64,

    /// Expiration timestamp (Unix timestamp in seconds).
    ///
    /// This field indicates when the JWT expires. GitHub requires
    /// JWTs to expire within 10 minutes of the issued at time.
    exp: u64,

    /// Issuer - the GitHub App ID.
    ///
    /// This field contains the numeric ID of the GitHub App that
    /// is making the authentication request. This can be found
    /// in the app's settings page on GitHub.
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

/// Creates a GitHub client authenticated with a personal access token.
///
/// # Arguments
///
/// * `token` - The personal access token for authentication
///
/// # Returns
///
/// A `Result` containing the authenticated GitHub client
///
/// # Errors
///
/// Returns `Error::ApiError` if the client cannot be created
#[instrument(skip(token))]
pub fn create_token_client(token: &str) -> Result<Octocrab, Error> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|_| Error::ApiError())
}

/// Logs detailed error information from Octocrab GitHub API errors.
///
/// This private function takes an Octocrab error and logs it with appropriate
/// detail based on the error type. It extracts specific information like
/// error messages, backtraces, and error context to provide comprehensive
/// debugging information.
///
/// # Arguments
///
/// * `message` - A contextual message describing what operation failed
/// * `e` - The Octocrab error to log
///
/// # Error Types Handled
///
/// - `GitHub` - GitHub API errors with detailed error messages
/// - `UriParse` - URI parsing failures
/// - `Uri` - URI construction failures
/// - `InvalidHeaderValue` - HTTP header validation errors
/// - `InvalidUtf8` - UTF-8 encoding errors
/// - Other - Generic error logging for unmatched types
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

/// GitHub implementation of the developer platform traits.
///
/// This struct provides GitHub-specific implementations for interacting with
/// pull requests, comments, labels, and other GitHub features.
#[derive(Debug, Default)]
pub struct GitHubProvider {
    /// The authenticated GitHub client
    client: Octocrab,
}

impl GitHubProvider {
    /// Fetches the default branch name for a repository.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    ///
    /// # Returns
    ///
    /// A `Result` containing the default branch name
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be found or accessed
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

    /// Fetches the content of a file from a repository.
    ///
    /// This method retrieves the content of a specific file from a GitHub repository
    /// at the given path and reference (branch/commit/tag). The content is automatically
    /// decoded from GitHub's base64 encoding.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `path` - The path to the file within the repository
    /// * `reference` - Optional branch, commit SHA, or tag (defaults to "main")
    ///
    /// # Returns
    ///
    /// * `Ok(Some(content))` - If the file exists and was successfully retrieved
    /// * `Ok(None)` - If the file does not exist (404 error)
    /// * `Err(Error)` - If there was an API error or decoding failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository cannot be accessed
    /// - The file content cannot be decoded from base64
    /// - The decoded content is not valid UTF-8
    /// - There's a network or API error (other than 404)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::github::GitHubProvider;
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     // Fetch README from main branch
    ///     if let Some(content) = provider.fetch_file_content(
    ///         "owner",
    ///         "repo",
    ///         "README.md",
    ///         None
    ///     ).await? {
    ///         println!("README content: {}", content);
    ///     }
    ///
    ///     // Fetch specific file from a branch
    ///     if let Some(content) = provider.fetch_file_content(
    ///         "owner",
    ///         "repo",
    ///         "src/main.rs",
    ///         Some("develop")
    ///     ).await? {
    ///         println!("Source code: {}", content);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Creates a new GitHubProvider with the given authenticated client.
    ///
    /// # Arguments
    ///
    /// * `client` - An authenticated Octocrab client for GitHub API access
    ///
    /// # Returns
    ///
    /// A new GitHubProvider instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::github::GitHubProvider;
    /// use octocrab::Octocrab;
    ///
    /// let client = Octocrab::builder().personal_token("token".to_string()).build().unwrap();
    /// let provider = GitHubProvider::new(client);
    /// ```
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

        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            path = path,
            branch = default_branch_name,
            "Fetching configuration file from default branch in repository ...",
        );
        self.fetch_file_content(repo_owner, repo_name, path, Some(&default_branch_name))
            .await
    }
}

#[async_trait]
impl PullRequestProvider for GitHubProvider {
    /// Adds a comment to a pull request.
    ///
    /// This method posts a new comment to the specified pull request using
    /// GitHub's Issues API (since PR comments use the same endpoint as issue comments).
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `comment` - The comment text to add (supports Markdown formatting)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the comment was successfully added, or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to comment
    /// - There's a network or API error
    /// - The comment content violates GitHub's content policies
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     provider.add_comment(
    ///         "owner",
    ///         "repo",
    ///         123,
    ///         "Thanks for the contribution! Please update the PR title."
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Adds multiple labels to a pull request.
    ///
    /// This method adds the specified labels to a pull request. If any of the labels
    /// don't exist in the repository, they will be ignored by GitHub's API.
    /// Existing labels on the PR are preserved.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `labels` - Array of label names to add
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the labels were successfully added, or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to modify labels
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     let labels = vec!["bug".to_string(), "high-priority".to_string()];
    ///     provider.add_labels("owner", "repo", 123, &labels).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Deletes a specific comment from a pull request.
    ///
    /// This method removes a comment from a pull request using the comment's unique ID.
    /// Only the comment author or users with sufficient permissions can delete comments.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `comment_id` - The unique ID of the comment to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the comment was successfully deleted, or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The comment doesn't exist
    /// - The authenticated user lacks permission to delete the comment
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     // Delete comment with ID 456789
    ///     provider.delete_comment("owner", "repo", 456789).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Retrieves detailed information about a specific pull request.
    ///
    /// This method fetches comprehensive information about a pull request including
    /// its title, description, draft status, and author information.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a `PullRequest` struct containing the PR information, or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to access the repository
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     let pr = provider.get_pull_request("owner", "repo", 123).await?;
    ///     println!("PR Title: {}", pr.title);
    ///     println!("Is Draft: {}", pr.draft);
    ///
    ///     Ok(())
    /// }
    /// ```
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
                author: pr.user.map(|user| User {
                    id: user.id.0,
                    login: user.login,
                }),
            }),
            Err(e) => {
                log_octocrab_error("Failed to get pull request information", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    /// Retrieves the list of files changed in a pull request.
    ///
    /// This method fetches information about all files that were modified, added, or deleted
    /// in the pull request, including line count statistics and change status for each file.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a vector of `PullRequestFile` structs containing file change information,
    /// or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to access the repository
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     let files = provider.get_pull_request_files("owner", "repo", 123).await?;
    ///     for file in files {
    ///         println!("File: {} (+{} -{} lines)", file.filename, file.additions, file.deletions);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[instrument]
    async fn get_pull_request_files(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestFile>, Error> {
        match self
            .client
            .pulls(repo_owner, repo_name)
            .list_files(pr_number)
            .await
        {
            Ok(response) => {
                let files: Vec<PullRequestFile> = response
                    .items
                    .into_iter()
                    .map(|file| PullRequestFile {
                        filename: file.filename,
                        additions: file.additions as u32,
                        deletions: file.deletions as u32,
                        changes: file.changes as u32,
                        status: match file.status {
                            octocrab::models::repos::DiffEntryStatus::Added => "added".to_string(),
                            octocrab::models::repos::DiffEntryStatus::Removed => {
                                "removed".to_string()
                            }
                            octocrab::models::repos::DiffEntryStatus::Modified => {
                                "modified".to_string()
                            }
                            octocrab::models::repos::DiffEntryStatus::Renamed => {
                                "renamed".to_string()
                            }
                            octocrab::models::repos::DiffEntryStatus::Copied => {
                                "copied".to_string()
                            }
                            octocrab::models::repos::DiffEntryStatus::Changed => {
                                "changed".to_string()
                            }
                            octocrab::models::repos::DiffEntryStatus::Unchanged => {
                                "unchanged".to_string()
                            }
                            _ => "unknown".to_string(),
                        },
                    })
                    .collect();

                debug!(
                    "Retrieved {} file(s) for PR #{} in {}/{}",
                    files.len(),
                    pr_number,
                    repo_owner,
                    repo_name
                );

                Ok(files)
            }
            Err(e) => {
                log_octocrab_error("Failed to get pull request files", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    /// Lists all labels currently applied to a pull request.
    ///
    /// This method retrieves all labels that are currently attached to the specified
    /// pull request, including both labels added automatically and those added manually.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a vector of `Label` structs containing the applied labels,
    /// or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to access the repository
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     let labels = provider.list_applied_labels("owner", "repo", 123).await?;
    ///     for label in labels {
    ///         println!("Applied label: {}", label.name);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[instrument]
    async fn list_applied_labels(
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

        let result = labels
            .into_iter()
            .map(|l| Label {
                name: l.name,
                description: l.description,
            })
            .collect();

        Ok(result)
    }

    /// Lists all labels available in the repository.
    ///
    /// This method retrieves all labels that have been defined in the repository
    /// and can potentially be applied to pull requests and issues.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    ///
    /// # Returns
    ///
    /// Returns a vector of `Label` structs containing all available labels,
    /// or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository doesn't exist
    /// - The authenticated user lacks permission to access the repository
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     let labels = provider.list_available_labels("owner", "repo").await?;
    ///     for label in labels {
    ///         println!("Available label: {} - {}", label.name,
    ///                 label.description.unwrap_or("No description".to_string()));
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[instrument]
    async fn list_available_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<Vec<Label>, Error> {
        let mut current_page = match self
            .client
            .issues(repo_owner, repo_name)
            .list_labels_for_repo()
            .send()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log_octocrab_error("Failed to list all repository labels", e);
                return Err(Error::InvalidResponse);
            }
        };

        let mut labels = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            labels.extend(new_page.take_items());

            current_page = new_page;
        }

        let result = labels
            .into_iter()
            .map(|l| Label {
                name: l.name,
                description: l.description,
            })
            .collect();

        Ok(result)
    }

    /// Lists all comments on a pull request.
    ///
    /// This method retrieves all comments that have been posted on the specified
    /// pull request, including both automated and manual comments. The comments
    /// are returned in chronological order.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a vector of `Comment` structs containing all comments,
    /// or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to access the repository
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     let comments = provider.list_comments("owner", "repo", 123).await?;
    ///     for comment in comments {
    ///         println!("Comment by {}: {}", comment.user.login, comment.body);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Removes a specific label from a pull request.
    ///
    /// This method removes the specified label from a pull request if it's currently applied.
    /// If the label is not applied to the PR, the operation succeeds without error.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `label` - The name of the label to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the label was successfully removed (or wasn't applied),
    /// or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user lacks permission to modify labels
    /// - There's a network or API error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     // Remove the "bug" label from PR #123
    ///     provider.remove_label("owner", "repo", 123, "bug").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Updates or creates a check run status for a pull request.
    ///
    /// This method creates or updates a GitHub check run for the pull request,
    /// which appears in the PR's status checks section. This is useful for
    /// reporting the results of automated validation or CI processes.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `conclusion` - The conclusion status ("success", "failure", "cancelled", etc.)
    /// * `output_title` - The title shown in the check run details
    /// * `output_summary` - A brief summary of the check results
    /// * `output_text` - Detailed text output (supports Markdown)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the check run was successfully created/updated,
    /// or an `Error` if the operation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The repository or pull request doesn't exist
    /// - The authenticated user/app lacks permission to create check runs
    /// - There's a network or API error
    /// - The check run data is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
    /// use octocrab::Octocrab;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Octocrab::builder().personal_token("token".to_string()).build()?;
    ///     let provider = GitHubProvider::new(client);
    ///
    ///     provider.update_pr_check_status(
    ///         "owner",
    ///         "repo",
    ///         123,
    ///         "success",
    ///         "All validations passed",
    ///         "PR title and description meet all requirements",
    ///         "✅ Title follows conventional commits format\n✅ Work item referenced"
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[instrument]
    async fn update_pr_check_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        conclusion: &str,
        output_title: &str,
        output_summary: &str,
        output_text: &str,
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
                "summary": output_summary,
                "text": output_text,
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
