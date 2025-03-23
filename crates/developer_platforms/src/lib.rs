use anyhow::{anyhow, Result};
use async_trait::async_trait;
//use octocrab::{params::pulls::PullRequestState, Octocrab};

mod auth;

pub mod models;
use models::{Comment, Label, PullRequest};

/// Trait for interacting with Git hosting providers (e.g., GitHub, GitLab).
///
/// Implementations of this trait provide the necessary functionality to
/// interact with pull requests, comments, labels, and other Git provider features.
///
/// # Example Implementation
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::{PullRequestProvider, models::{Comment, Label, PullRequest}};
/// use anyhow::Result;
/// use async_trait::async_trait;
///
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
///     ) -> Result<PullRequest> {
///         // Implementation to fetch PR from GitHub API
///         // ...
///         # unimplemented!()
///     }
///
///     // Implement other required methods...
///     # async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<()> { unimplemented!() }
///     # async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<()> { unimplemented!() }
///     # async fn list_comments(&self, _: &str, _: &str, _: u64) -> Result<Vec<Comment>> { unimplemented!() }
///     # async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<()> { unimplemented!() }
///     # async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<()> { unimplemented!() }
///     # async fn list_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<Label>> { unimplemented!() }
///     # async fn update_pr_blocking_review(&self, _: &str, _: &str, _: u64, _: bool) -> Result<()> { unimplemented!() }
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
    ) -> Result<PullRequest>;

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
    ) -> Result<()>;

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
    ) -> Result<()>;

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
    ) -> Result<Vec<Comment>>;

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
    ) -> Result<()>;

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
    ) -> Result<()>;

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
    ) -> Result<Vec<Label>>;

    /// Updates a blocking review on the pull request. The update may be adding a blocking review,
    /// updating the contents of the blocking review, or removing the review. The review should never
    /// be an approving review.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `mergeable` - Whether the PR should be mergeable
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn update_pr_blocking_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        mergeable: bool,
    ) -> Result<()>;
}

// pub struct GitHubProvider {
//     client: Octocrab,
// }

// impl GitHubProvider {
//     pub fn new(client: Octocrab) -> Self {
//         Self { client }
//     }

//     pub async fn from_app(
//         app_id: u64,
//         private_key: &str,
//         installation_id: Option<u64>,
//     ) -> Result<Self> {
//         let client = auth::create_app_client(app_id, private_key, installation_id).await?;
//         Ok(Self::new(client))
//     }

//     pub fn from_token(token: &str) -> Result<Self> {
//         let client = auth::create_token_client(token)?;
//         Ok(Self::new(client))
//     }
// }

// #[async_trait]
// impl PullRequestProvider for GitHubProvider {
//     async fn get_pull_request(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//     ) -> Result<PullRequest> {
//         let pr = self
//             .client
//             .pulls(repo_owner, repo_name)
//             .get(pr_number)
//             .await
//             .map_err(|e| anyhow!("Failed to get PR: {}", e))?;

//         Ok(PullRequest {
//             number: pr.number,
//             title: pr.title,
//             body: pr.body,
//         })
//     }

//     async fn add_comment(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//         comment: &str,
//     ) -> Result<()> {
//         self.client
//             .issues(repo_owner, repo_name)
//             .create_comment(pr_number, comment)
//             .await
//             .map_err(|e| anyhow!("Failed to add comment: {}", e))?;

//         Ok(())
//     }

//     async fn delete_comment(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         comment_id: u64,
//     ) -> Result<()> {
//         self.client
//             .issues(repo_owner, repo_name)
//             .delete_comment(comment_id)
//             .await
//             .map_err(|e| anyhow!("Failed to delete comment: {}", e))?;

//         Ok(())
//     }

//     async fn list_comments(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//     ) -> Result<Vec<Comment>> {
//         let comments = self
//             .client
//             .issues(repo_owner, repo_name)
//             .list_comments(pr_number)
//             .await
//             .map_err(|e| anyhow!("Failed to list comments: {}", e))?;

//         let result = comments
//             .items
//             .into_iter()
//             .map(|c| Comment {
//                 id: c.id.0,
//                 body: c.body.unwrap_or_default(),
//             })
//             .collect();

//         Ok(result)
//     }

//     async fn add_labels(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//         labels: &[String],
//     ) -> Result<()> {
//         self.client
//             .issues(repo_owner, repo_name)
//             .add_labels(pr_number, labels)
//             .await
//             .map_err(|e| anyhow!("Failed to add labels: {}", e))?;

//         Ok(())
//     }

//     async fn remove_label(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//         label: &str,
//     ) -> Result<()> {
//         self.client
//             .issues(repo_owner, repo_name)
//             .remove_label(pr_number, label)
//             .await
//             .map_err(|e| anyhow!("Failed to remove label: {}", e))?;

//         Ok(())
//     }

//     async fn list_labels(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//     ) -> Result<Vec<Label>> {
//         let labels = self
//             .client
//             .issues(repo_owner, repo_name)
//             .list_labels_for_issue(pr_number)
//             .await
//             .map_err(|e| anyhow!("Failed to list labels: {}", e))?;

//         let result = labels
//             .items
//             .into_iter()
//             .map(|l| Label { name: l.name })
//             .collect();

//         Ok(result)
//     }

//     async fn update_pr_blocking_review(
//         &self,
//         repo_owner: &str,
//         repo_name: &str,
//         pr_number: u64,
//         mergeable: bool,
//     ) -> Result<()> {
//         // GitHub's API doesn't directly support changing the mergeable state
//         // Instead, we can use the "draft" status as a proxy
//         self.client
//             .pulls(repo_owner, repo_name)
//             .update(pr_number)
//             .draft(!mergeable) // Set to draft if not mergeable
//             .send()
//             .await
//             .map_err(|e| anyhow!("Failed to update PR state: {}", e))?;

//         Ok(())
//     }
// }
