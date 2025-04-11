use async_trait::async_trait;

pub mod errors;

pub mod github;

pub mod models;
use errors::Error;
use models::{Comment, Label, PullRequest};

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
///     # async fn update_pr_blocking_review(&self, _: &str, _: &str, _: u64, _: bool) -> Result<(), Error> { unimplemented!() }
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

    /// Updates a blocking review on the pull request. The update may be adding a blocking review,
    /// updating the contents of the blocking review, or removing the review. The review should never
    /// be an approving review.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository.
    /// * `repo_name` - The name of the repository.
    /// * `pr_number` - The pull request number.
    /// * `is_approved` - Whether the PR should be approved or 'rejected'.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn update_pr_blocking_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        is_approved: bool,
    ) -> Result<(), Error>;
}
