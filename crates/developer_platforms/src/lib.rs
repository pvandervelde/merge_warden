use async_trait::async_trait;

pub mod errors;

pub mod github;

pub mod models;
use errors::Error;
use models::{Comment, Label, PullRequest, PullRequestFile};

/// Trait to fetch configuration files from remote repositories.
#[async_trait]
pub trait ConfigFetcher: Sync + Send {
    /// Fetch the content of a configuration file at the given path.
    /// Returns Ok(Some(content)) if found, Ok(None) if not found, or Err on error.
    async fn fetch_config(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
    ) -> Result<Option<String>, Error>;
}

/// Trait for interacting with developer platforms that provide pull requests (e.g., GitHub, GitLab).
///
/// Implementations of this trait provide the necessary functionality to
/// interact with pull requests, comments, labels, and other Git provider features.
///
/// # Example Implementation
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::{PullRequestProvider, errors::Error, models::{Comment, Label, PullRequest}};
/// use anyhow::Result;
/// use async_trait::async_trait;
///
/// #[derive(Debug)]
/// struct GitHubProvider {
///     // Fields for authentication, etc.
///     token: String,
/// }
///
/// #[async_trait]
/// impl PullRequestProvider for GitHubProvider {
///     async fn get_pull_request(
///         &self,
///         repo_owner: &str,
///         repo_name: &str,
///         pr_number: u64,
///     ) -> Result<PullRequest, Error> {
///         // Implementation to fetch PR from GitHub API
///         // ...
///         # unimplemented!()
///     }
///
///     // Implement other required methods...
///     # async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
///     # async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
///     # async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<Comment>, Error> { unimplemented!() }
///     # async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
///     # async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
///     # async fn list_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<Label>, Error> { unimplemented!() }
///     # async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
/// }
/// ```
#[async_trait]
pub trait PullRequestProvider {
    /// Retrieves a pull request from the Git provider.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing the pull request information
    async fn get_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<PullRequest, Error>;

    /// Gets the list of files changed in a pull request.
    ///
    /// This method fetches all files that have been modified, added, deleted, or renamed
    /// as part of the pull request. The returned data includes line change counts and
    /// file status information needed for PR size analysis.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of file changes
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::{PullRequestProvider, models::PullRequestFile};
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    ///
    /// async fn get_pull_request_files(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    /// ) -> Result<Vec<PullRequestFile>, Error> {
    ///     // Implementation to fetch file changes from the Git provider
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn get_pull_request_files(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestFile>, Error>;

    /// Adds a comment to a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `comment` - The comment text to add
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn add_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        comment: &str,
    ) -> Result<(), Error>;

    /// Deletes a comment from a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `comment_id` - The ID of the comment to delete
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn delete_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error>;

    /// Lists all comments on a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of comments
    async fn list_comments(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Comment>, Error>;

    /// Adds labels to a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `labels` - The labels to add
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn add_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        labels: &[String],
    ) -> Result<(), Error>;

    /// Removes a label from a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `label` - The label to remove
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn remove_label(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        label: &str,
    ) -> Result<(), Error>;

    /// Lists all labels on a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of labels
    async fn list_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Label>, Error>;

    /// Updates the GitHub check run status for the pull request. This should be used to report
    /// the result of MergeWarden's validation as a GitHub check (success/failure, with details).
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `conclusion` - The check run conclusion (e.g., "success", "failure")
    /// * `output_title` - The title for the check run output
    /// * `output_summary` - The summary for the check run output
    /// * `output_text` - The text for the check run output. Supports Markdown
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    #[allow(clippy::too_many_arguments)]
    async fn update_pr_check_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        conclusion: &str,
        output_title: &str,
        output_summary: &str,
        output_text: &str,
    ) -> Result<(), Error>;
}
