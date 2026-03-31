//! # Developer Platforms Crate
//!
//! This crate provides abstractions and implementations for interacting with
//! various developer platforms (like GitHub, GitLab) that host pull requests.
//!
//! The crate defines traits for platform-agnostic operations on pull requests,
//! comments, labels, and other Git provider features. It includes a GitHub
//! implementation and models for representing platform data.
//!
//! # Key Components
//!
//! - [`PullRequestProvider`] - Core trait for pull request operations
//! - [`IssueMetadataProvider`] - Trait for fetching and updating issue metadata
//! - [`ConfigFetcher`] - Trait for fetching configuration files
//! - [`models`] - Data models for pull requests, comments, labels, etc.
//! - [`github`] - GitHub implementation of the provider traits
//! - [`errors`] - Error types for the crate
//!
//! # Examples
//!
//! ```rust,no_run
//! use merge_warden_developer_platforms::{PullRequestProvider, github::GitHubProvider};
//! use merge_warden_developer_platforms::errors::Error;
//! use github_bot_sdk::{client::{GitHubClient, ClientConfig}, auth::InstallationId};
//!
//! async fn example(github_client: GitHubClient) -> Result<(), Error> {
//!     let installation_client = github_client
//!         .installation_by_id(InstallationId::new(12345))
//!         .await
//!         .map_err(|_| Error::ApiError())?;
//!     let github = GitHubProvider::new(installation_client);
//!     let pr = github.get_pull_request("owner", "repo", 123).await?;
//!     println!("PR title: {}", pr.title);
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use async_trait::async_trait;

/// GitHub App authentication provider for webhook-driven deployments.
pub mod app_auth;

/// Error types for developer platform operations.
pub mod errors;

/// GitHub implementation of developer platform traits.
pub mod github;

/// Data models for pull requests, comments, labels, and other platform entities.
pub mod models;

#[cfg(test)]
mod lib_tests;

use errors::Error;
use models::{Comment, IssueMetadata, Label, PullRequest, PullRequestFile, Review};

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
///     # async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<Label>, Error> { unimplemented!() }
///     # async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
///     # async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
///     # async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<Label>, Error> { unimplemented!() }
///     # async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
///     # async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
///     # async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
/// }
/// ```
#[async_trait]
pub trait PullRequestProvider {
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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn add_comment(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    ///     comment: &str,
    /// ) -> Result<(), Error> {
    ///     // Implementation to add comment to pull request
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn add_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        comment: &str,
    ) -> Result<(), Error>;

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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn add_labels(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    ///     labels: &[String],
    /// ) -> Result<(), Error> {
    ///     // Implementation to add labels to pull request
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn add_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        labels: &[String],
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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn delete_comment(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     comment_id: u64,
    /// ) -> Result<(), Error> {
    ///     // Implementation to delete comment from pull request
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn delete_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error>;

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
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
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

    /// Lists all labels currently applied to a pull request.
    ///
    /// This method fetches the labels that are currently attached to a specific
    /// pull request, which is useful for determining what labels are already present
    /// before adding or removing labels.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of labels currently applied to the pull request
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn list_applied_labels(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    /// ) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> {
    ///     // Implementation to list labels currently applied to pull request
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn list_applied_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Label>, Error>;

    /// Lists all labels available in the repository.
    ///
    /// This method fetches all labels that are defined in the repository,
    /// which is essential for smart label discovery and validation. It provides
    /// the complete set of labels that can be applied to pull requests.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all available repository labels
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn list_available_labels(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    /// ) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> {
    ///     // Implementation to list all available repository labels
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn list_available_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<Vec<Label>, Error>;

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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn list_comments(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    /// ) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> {
    ///     // Implementation to list comments on pull request
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn list_comments(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Comment>, Error>;

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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn remove_label(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    ///     label: &str,
    /// ) -> Result<(), Error> {
    ///     // Implementation to remove label from pull request
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
    async fn remove_label(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        label: &str,
    ) -> Result<(), Error>;

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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// # use merge_warden_developer_platforms::errors::Error;
    /// # use async_trait::async_trait;
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl PullRequestProvider for MyProvider {
    /// #     async fn get_pull_request(&self, _: &str, _: &str, _: u64) -> Result<merge_warden_developer_platforms::models::PullRequest, Error> { unimplemented!() }
    /// #     async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// #     async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    /// #     async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<merge_warden_developer_platforms::models::Label>, Error> { unimplemented!() }
    /// #     async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Comment>, Error> { unimplemented!() }
    /// #     async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    /// #     async fn list_pr_reviews(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> { unimplemented!() }
    ///
    /// async fn update_pr_check_status(
    ///     &self,
    ///     repo_owner: &str,
    ///     repo_name: &str,
    ///     pr_number: u64,
    ///     conclusion: &str,
    ///     output_title: &str,
    ///     output_summary: &str,
    ///     output_text: &str,
    /// ) -> Result<(), Error> {
    ///     // Implementation to update pull request check status
    ///     # unimplemented!()
    /// }
    /// # }
    /// ```
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

    /// Lists all reviews submitted on a pull request.
    ///
    /// Returns the reviews in the order they were submitted. Each review contains
    /// the reviewer, the review state (e.g., `"approved"`, `"changes_requested"`,
    /// `"commented"`), and a unique review ID.
    ///
    /// This is used to determine whether the PR has at least one approved review
    /// for state-based lifecycle labeling.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of [`Review`]s, ordered oldest-first
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the API call fails
    async fn list_pr_reviews(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Review>, Error>;
}

/// Provides read access to issue metadata for propagation to pull requests.
///
/// Implementations retrieve milestone and project information from an issue so
/// that `merge_warden_core` can copy that information onto the associated pull
/// request.
///
/// # Platform Support
///
/// The default implementation is [`github::GitHubIssueMetadataProvider`].
/// Teams using external issue trackers (Jira, Linear, etc.) that do not wish
/// to implement this trait can simply leave both propagation flags disabled in
/// their configuration, which is the default.
///
/// # Examples
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::{IssueMetadataProvider, errors::Error};
/// use merge_warden_developer_platforms::models::{IssueMetadata, IssueMilestone, IssueProject};
/// use async_trait::async_trait;
///
/// struct MyIssueProvider;
///
/// #[async_trait]
/// impl IssueMetadataProvider for MyIssueProvider {
///     async fn get_issue_metadata(
///         &self,
///         _repo_owner: &str,
///         _repo_name: &str,
///         _issue_number: u64,
///     ) -> Result<Option<IssueMetadata>, Error> {
///         Ok(Some(IssueMetadata {
///             milestone: Some(IssueMilestone { number: 1, title: "v1.0".to_string() }),
///             projects: vec![],
///         }))
///     }
///     async fn set_pull_request_milestone(
///         &self,
///         _repo_owner: &str,
///         _repo_name: &str,
///         _pr_number: u64,
///         _milestone_number: Option<u64>,
///     ) -> Result<(), Error> {
///         Ok(())
///     }
///     async fn add_pull_request_to_project(
///         &self,
///         _repo_owner: &str,
///         _repo_name: &str,
///         _pr_number: u64,
///         _project_number: u64,
///         _project_owner_login: &str,
///     ) -> Result<(), Error> {
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait IssueMetadataProvider: std::fmt::Debug + Sync + Send {
    /// Fetch milestone and project metadata for a single issue.
    ///
    /// # Arguments
    ///
    /// * `repo_owner`   - Owner of the repository where the issue lives.
    /// * `repo_name`    - Name of the repository where the issue lives.
    /// * `issue_number` - Issue number within that repository.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(metadata))` — issue exists and metadata was fetched.
    /// - `Ok(None)` — issue does not exist (404).
    /// - `Err(e)` — transient or permission error.
    async fn get_issue_metadata(
        &self,
        repo_owner: &str,
        repo_name: &str,
        issue_number: u64,
    ) -> Result<Option<IssueMetadata>, Error>;

    /// Set the milestone on a pull request.
    ///
    /// Overwrites any existing milestone on the PR. Pass `milestone_number: None`
    /// to clear — but note that the propagation logic never calls this with `None`.
    ///
    /// # Arguments
    ///
    /// * `repo_owner`       - Owner of the repository containing the PR.
    /// * `repo_name`        - Name of that repository.
    /// * `pr_number`        - Pull request number.
    /// * `milestone_number` - Milestone number to apply, or `None` to clear.
    async fn set_pull_request_milestone(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        milestone_number: Option<u64>,
    ) -> Result<(), Error>;

    /// Add a pull request to a GitHub Projects v2 project.
    ///
    /// Resolves the project's GraphQL node ID from `project_owner_login` and
    /// `project_number`, fetches the PR's global node ID, then calls the
    /// `addProjectV2ItemById` GraphQL mutation to attach the PR to the project.
    ///
    /// # Arguments
    ///
    /// * `repo_owner`          - Owner of the repository containing the PR.
    /// * `repo_name`           - Name of that repository.
    /// * `pr_number`           - Pull request number.
    /// * `project_number`      - Project number (owner-scoped).
    /// * `project_owner_login` - Login of the project owner (org or user).
    async fn add_pull_request_to_project(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        project_number: u64,
        project_owner_login: &str,
    ) -> Result<(), Error>;
}
