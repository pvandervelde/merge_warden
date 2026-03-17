use async_trait::async_trait;
use base64::Engine;
use github_bot_sdk::{
    client::{CreateCommentRequest, InstallationClient},
    error::ApiError,
};
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

/// Maps a `github_bot_sdk` [`ApiError`] to the crate-local [`Error`] type.
///
/// Provides a consistent, single-purpose mapping between the SDK error hierarchy and
/// the error types used throughout this crate. Auth errors, rate limit signals, and
/// token failures are mapped to their semantic equivalents; all other SDK errors fall
/// back to the generic [`Error::ApiError`].
fn map_api_error(e: ApiError) -> Error {
    match e {
        ApiError::AuthenticationFailed | ApiError::AuthorizationFailed => {
            Error::AuthError(e.to_string())
        }
        ApiError::RateLimitExceeded { .. } | ApiError::SecondaryRateLimit => {
            Error::RateLimitExceeded
        }
        // 429 after retry exhaustion: execute_with_retry converts to HttpError { status: 429 }
        ApiError::HttpError { status: 429, .. } => Error::RateLimitExceeded,
        ApiError::NotFound => Error::InvalidResponse,
        ApiError::TokenGenerationFailed { message } | ApiError::TokenExchangeFailed { message } => {
            Error::TokenRefreshFailed(0, message)
        }
        _ => Error::ApiError(),
    }
}

/// GitHub implementation of developer platform traits.
///
/// Wraps an installation-scoped [`InstallationClient`] to expose it through the
/// [`PullRequestProvider`] and [`ConfigFetcher`] interfaces.  All operations are
/// performed within the permission context of the bound GitHub App installation.
///
/// # Construction
///
/// Obtain an [`InstallationClient`] from
/// [`github_bot_sdk::client::GitHubClient::installation_by_id`] and pass it here:
///
/// ```rust,no_run
/// use github_bot_sdk::{client::{GitHubClient, ClientConfig}, auth::InstallationId};
/// use merge_warden_developer_platforms::github::GitHubProvider;
///
/// # async fn example(github_client: GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
/// let installation_client = github_client
///     .installation_by_id(InstallationId::new(12345))
///     .await?;
/// let provider = GitHubProvider::new(installation_client);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct GitHubProvider {
    /// Installation-scoped GitHub API client.
    client: InstallationClient,
}

impl GitHubProvider {
    /// Creates a `GitHubProvider` from an installation-scoped client.
    ///
    /// # Arguments
    ///
    /// * `client` - An [`InstallationClient`] authenticated for a specific GitHub App installation.
    pub fn new(client: InstallationClient) -> Self {
        Self { client }
    }

    /// Fetches the default branch name for a repository.
    ///
    /// Uses a raw `GET /repos/{owner}/{repo}` request and extracts the
    /// `default_branch` field from the JSON response.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidResponse`] if the repository cannot be reached or
    /// the response cannot be parsed.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name))]
    async fn fetch_default_branch(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<String, Error> {
        let path = format!("/repos/{}/{}", repo_owner, repo_name);

        let response = match self.client.get(&path).await {
            Ok(r) => r,
            Err(e) => {
                error!(
                    owner = repo_owner,
                    repo = repo_name,
                    error = %e,
                    "Failed to get repository info"
                );
                return Err(map_api_error(e));
            }
        };

        if !response.status().is_success() {
            error!(
                owner = repo_owner,
                repo = repo_name,
                status = response.status().as_u16(),
                "Non-success status fetching repository"
            );
            return Err(Error::InvalidResponse);
        }

        let json: serde_json::Value = response.json().await.map_err(|_| Error::InvalidResponse)?;

        let branch = match json["default_branch"].as_str() {
            Some(b) => b.to_string(),
            None => {
                error!(
                    owner = repo_owner,
                    repo = repo_name,
                    "GitHub API response is missing 'default_branch' field"
                );
                return Err(Error::InvalidResponse);
            }
        };

        debug!(owner = repo_owner, repo = repo_name, branch = %branch, "Resolved default branch");
        Ok(branch)
    }

    /// Fetches the raw string content of a file from a repository at a given ref.
    ///
    /// Uses `GET /repos/{owner}/{repo}/contents/{path}?ref={reference}`.  GitHub
    /// returns file contents as base64-encoded strings, which this function decodes
    /// automatically.
    ///
    /// Returns `Ok(None)` when the file does not exist (HTTP 404).
    ///
    /// # Errors
    ///
    /// Returns an error for any API failure other than 404, or if the base64
    /// content cannot be decoded to valid UTF-8.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, path, reference))]
    async fn fetch_file_content(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
        reference: &str,
    ) -> Result<Option<String>, Error> {
        // Percent-encode path segments so that branch names or file paths
        // containing characters such as `#`, `+`, or spaces produce a valid URL.
        // The `path` component uses standard percent-encoding; the `ref` query
        // parameter value uses form-encoded rules (spaces → `%20`, not `+`).
        let encoded_path = path
            .split('/')
            .map(|s| urlencoding::encode(s).into_owned())
            .collect::<Vec<_>>()
            .join("/");
        let encoded_ref = urlencoding::encode(reference);
        let url_path = format!(
            "/repos/{}/{}/contents/{}?ref={}",
            repo_owner, repo_name, encoded_path, encoded_ref
        );

        let response = match self.client.get(&url_path).await {
            Ok(r) => r,
            Err(ApiError::NotFound) => {
                debug!(
                    owner = repo_owner,
                    repo = repo_name,
                    path,
                    "File not found (404)"
                );
                return Ok(None);
            }
            Err(e) => {
                error!(
                    owner = repo_owner,
                    repo = repo_name,
                    path,
                    error = %e,
                    "Error fetching file content"
                );
                return Err(map_api_error(e));
            }
        };

        if !response.status().is_success() {
            // 404 is already handled above via ApiError::NotFound → Ok(None).
            // Other non-success codes (403 permission denied, 500 server error, etc.)
            // are not "file not found" and should not be silently treated as such.
            return Err(Error::InvalidResponse);
        }

        let json: serde_json::Value = response.json().await.map_err(|_| Error::InvalidResponse)?;

        let content = match json["content"].as_str() {
            Some(c) => c.to_string(),
            None => return Ok(None),
        };

        // GitHub encodes file content as base64 with embedded newlines — strip them.
        let cleaned = content.replace(['\n', ' '], "");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&cleaned)
            .map_err(|e| {
                error!(error = %e, "Failed to decode base64 file content");
                Error::InvalidResponse
            })?;

        let text = String::from_utf8(decoded).map_err(|e| {
            error!(error = %e, "File content is not valid UTF-8");
            Error::InvalidResponse
        })?;

        debug!(
            owner = repo_owner,
            repo = repo_name,
            path,
            "Successfully fetched file content"
        );
        Ok(Some(text))
    }
}

#[async_trait]
impl ConfigFetcher for GitHubProvider {
    /// Fetches the content of a configuration file from the repository's default branch.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` — Repository owner.
    /// * `repo_name` — Repository name.
    /// * `path` — Path to the configuration file.
    ///
    /// # Returns
    ///
    /// `Ok(Some(content))` if found, `Ok(None)` if not found, `Err` on failure.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, path))]
    async fn fetch_config(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
    ) -> Result<Option<String>, Error> {
        let default_branch = self.fetch_default_branch(repo_owner, repo_name).await?;
        info!(
            owner = repo_owner,
            repo = repo_name,
            path,
            branch = %default_branch,
            "Fetching configuration file from default branch"
        );
        self.fetch_file_content(repo_owner, repo_name, path, &default_branch)
            .await
    }
}

#[async_trait]
impl PullRequestProvider for GitHubProvider {
    /// Adds a comment to a pull request.
    ///
    /// Posts a new comment using GitHub's Issues API (PR comments share the same
    /// endpoint as issue comments).
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
    /// Returns `Ok(())` if the comment was successfully added.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FailedToUpdatePullRequest`] if the API call fails for any reason.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn add_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        comment: &str,
    ) -> Result<(), Error> {
        self.client
            .create_issue_comment(
                repo_owner,
                repo_name,
                pr_number,
                CreateCommentRequest {
                    body: comment.to_string(),
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| {
                warn!(
                    owner = repo_owner,
                    repo = repo_name,
                    pr = pr_number,
                    error = %e,
                    "Failed to add pull request comment"
                );
                Error::FailedToUpdatePullRequest("Failed to add comment".to_string())
            })
    }

    /// Adds multiple labels to a pull request.
    ///
    /// Adds the specified labels to a pull request. Existing labels on the PR are preserved.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `labels` - Slice of label names to add
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the labels were successfully added.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FailedToUpdatePullRequest`] if the API call fails.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn add_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        self.client
            .add_labels_to_pull_request(repo_owner, repo_name, pr_number, labels.to_vec())
            .await
            .map(|_| ())
            .map_err(|e| {
                warn!(
                    owner = repo_owner,
                    repo = repo_name,
                    pr = pr_number,
                    error = %e,
                    "Failed to add labels to pull request"
                );
                Error::FailedToUpdatePullRequest("Failed to add labels".to_string())
            })
    }

    /// Deletes a specific comment from a pull request.
    ///
    /// Removes a comment using its unique ID.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `comment_id` - The unique ID of the comment to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the comment was successfully deleted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FailedToUpdatePullRequest`] if the API call fails.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, comment = comment_id))]
    async fn delete_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error> {
        self.client
            .delete_issue_comment(repo_owner, repo_name, comment_id)
            .await
            .map_err(|e| {
                warn!(
                    owner = repo_owner,
                    repo = repo_name,
                    comment = comment_id,
                    error = %e,
                    "Failed to delete pull request comment"
                );
                Error::FailedToUpdatePullRequest(format!("Failed to delete comment: {}", e))
            })
    }

    /// Retrieves detailed information about a specific pull request.
    ///
    /// Fetches comprehensive PR information including title, description, draft status,
    /// and author information.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a [`PullRequest`] struct containing the PR information.
    ///
    /// # Errors
    ///
    /// Returns an error mapping through [`map_api_error`] if the API call fails
    /// (including `NotFound` → `InvalidResponse`, rate limit and auth errors).
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn get_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<PullRequest, Error> {
        let pr = self
            .client
            .get_pull_request(repo_owner, repo_name, pr_number)
            .await
            .map_err(|e| {
                error!(
                    owner = repo_owner,
                    repo = repo_name,
                    pr = pr_number,
                    error = %e,
                    "Failed to get pull request"
                );
                map_api_error(e)
            })?;

        Ok(PullRequest {
            number: pr.number,
            title: pr.title,
            draft: pr.draft,
            body: pr.body,
            author: Some(User {
                id: pr.user.id,
                login: pr.user.login,
            }),
        })
    }

    /// Retrieves the list of files changed in a pull request.
    ///
    /// Uses a raw `GET /repos/{owner}/{repo}/pulls/{number}/files` request and
    /// parses the JSON array response.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a vector of [`PullRequestFile`] structs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidResponse`] if the API call fails or the response
    /// cannot be parsed.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn get_pull_request_files(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestFile>, Error> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/files",
            repo_owner, repo_name, pr_number
        );

        let response = self.client.get(&path).await.map_err(|e| {
            error!(
                owner = repo_owner,
                repo = repo_name,
                pr = pr_number,
                error = %e,
                "Failed to get pull request files"
            );
            map_api_error(e)
        })?;

        if !response.status().is_success() {
            error!(
                owner = repo_owner,
                repo = repo_name,
                pr = pr_number,
                status = response.status().as_u16(),
                "Non-success status fetching pull request files"
            );
            return Err(Error::InvalidResponse);
        }

        let items: Vec<serde_json::Value> =
            response.json().await.map_err(|_| Error::InvalidResponse)?;

        let files: Vec<PullRequestFile> = items
            .into_iter()
            .map(|v| PullRequestFile {
                filename: v["filename"].as_str().unwrap_or_default().to_string(),
                additions: v["additions"].as_u64().unwrap_or_default() as u32,
                deletions: v["deletions"].as_u64().unwrap_or_default() as u32,
                changes: v["changes"].as_u64().unwrap_or_default() as u32,
                status: v["status"].as_str().unwrap_or_default().to_string(),
            })
            .collect();

        debug!(
            owner = repo_owner,
            repo = repo_name,
            pr = pr_number,
            count = files.len(),
            "Fetched pull request files"
        );

        Ok(files)
    }

    /// Lists all labels currently applied to a pull request.
    ///
    /// Fetches the pull request and extracts its `labels` field.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a vector of [`Label`] structs currently applied to the PR.
    ///
    /// # Errors
    ///
    /// Returns an error (via [`map_api_error`]) if the pull request cannot be fetched.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn list_applied_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        let pr = self
            .client
            .get_pull_request(repo_owner, repo_name, pr_number)
            .await
            .map_err(|e| {
                error!(
                    owner = repo_owner,
                    repo = repo_name,
                    pr = pr_number,
                    error = %e,
                    "Failed to get pull request for applied labels"
                );
                map_api_error(e)
            })?;

        Ok(pr
            .labels
            .into_iter()
            .map(|l| Label {
                name: l.name,
                description: l.description,
            })
            .collect())
    }

    /// Lists all labels available in the repository.
    ///
    /// Uses `GET /repos/{owner}/{repo}/labels` via the SDK.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The repository name
    ///
    /// # Returns
    ///
    /// Returns a vector of all [`Label`] structs defined in the repository.
    ///
    /// # Errors
    ///
    /// Returns an error (via [`map_api_error`]) if the API call fails.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name))]
    async fn list_available_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<Vec<Label>, Error> {
        self.client
            .list_labels(repo_owner, repo_name)
            .await
            .map(|labels| {
                labels
                    .into_iter()
                    .map(|l| Label {
                        name: l.name,
                        description: l.description,
                    })
                    .collect()
            })
            .map_err(map_api_error)
    }

    /// Lists all comments on a pull request.
    ///
    /// Uses `GET /repos/{owner}/{repo}/issues/{number}/comments` via the SDK.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// Returns a vector of [`Comment`] structs.
    ///
    /// # Errors
    ///
    /// Returns an error (via [`map_api_error`]) if the API call fails.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn list_comments(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        self.client
            .list_issue_comments(repo_owner, repo_name, pr_number)
            .await
            .map(|comments| {
                comments
                    .into_iter()
                    .map(|c| Comment {
                        id: c.id,
                        body: c.body,
                        user: User {
                            id: c.user.id,
                            login: c.user.login,
                        },
                    })
                    .collect()
            })
            .map_err(map_api_error)
    }

    /// Removes a specific label from a pull request.
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
    /// Returns `Ok(())` if the label was successfully removed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FailedToUpdatePullRequest`] if the API call fails.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
    async fn remove_label(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        label: &str,
    ) -> Result<(), Error> {
        self.client
            .remove_label_from_pull_request(repo_owner, repo_name, pr_number, label)
            .await
            .map_err(|e| {
                warn!(
                    owner = repo_owner,
                    repo = repo_name,
                    pr = pr_number,
                    label,
                    error = %e,
                    "Failed to remove label from pull request"
                );
                Error::FailedToUpdatePullRequest(format!("Failed to remove label '{}'", label))
            })
    }

    /// Creates or updates a GitHub check run for the pull request.
    ///
    /// Fetches the PR head commit SHA and then POSTs to
    /// `POST /repos/{owner}/{repo}/check-runs`.
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
    /// Returns `Ok(())` if the check run was successfully created/updated.
    ///
    /// # Errors
    ///
    /// Returns an error if the pull request cannot be fetched or the check run
    /// POST fails.
    #[instrument(skip(self), fields(owner = repo_owner, repo = repo_name, pr = pr_number))]
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
        // Fetch the PR to get the head commit SHA for the check run.
        let pr = self
            .client
            .get_pull_request(repo_owner, repo_name, pr_number)
            .await
            .map_err(|e| {
                error!(
                    owner = repo_owner,
                    repo = repo_name,
                    pr = pr_number,
                    error = %e,
                    "Failed to get PR head SHA for check run"
                );
                map_api_error(e)
            })?;

        let head_sha = pr.head.sha;

        let url = format!("/repos/{}/{}/check-runs", repo_owner, repo_name);
        let payload = json!({
            "name": "MergeWarden",
            "head_sha": head_sha,
            "status": "completed",
            "conclusion": conclusion,
            "output": {
                "title": output_title,
                "summary": output_summary,
                "text": output_text,
            }
        });

        let response = self.client.post(&url, &payload).await.map_err(|e| {
            error!(
                owner = repo_owner,
                repo = repo_name,
                pr = pr_number,
                error = %e,
                "Failed to post check run"
            );
            map_api_error(e)
        })?;

        if !response.status().is_success() {
            error!(
                owner = repo_owner,
                repo = repo_name,
                pr = pr_number,
                status = response.status().as_u16(),
                "Non-success status creating check run"
            );
            return Err(Error::FailedToUpdatePullRequest(
                "Failed to create/update check run".to_string(),
            ));
        }

        info!(
            owner = repo_owner,
            repo = repo_name,
            pr = pr_number,
            conclusion,
            "Successfully updated PR check run status"
        );

        Ok(())
    }
}
