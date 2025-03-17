//! # Merge Warden Core
//!
//! Core business logic for validating and managing pull requests according to
//! configurable rules.
//!
//! Merge Warden helps enforce consistent PR practices by validating:
//! - PR titles follow the Conventional Commits format
//! - PR descriptions reference work items or issues
//! - Automatic labeling based on PR content
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use merge_warden_core::{MergeWarden, GitProvider, config::ValidationConfig};
//! use anyhow::Result;
//!
//! async fn validate_pr<P: GitProvider>(provider: P) -> Result<()> {
//!     // Create a MergeWarden instance with default configuration
//!     let warden = MergeWarden::new(provider);
//!
//!     // Process a pull request
//!     let result = warden.process_pull_request("owner", "repo", 123).await?;
//!
//!     // Check the validation results
//!     if result.title_valid && result.work_item_referenced {
//!         println!("PR is valid and can be merged!");
//!     } else {
//!         println!("PR has validation issues that need to be fixed");
//!     }
//!
//!     // Labels added to the PR
//!     println!("Labels: {:?}", result.labels);
//!
//!     Ok(())
//! }
//!
//! // With custom configuration
//! async fn validate_pr_custom<P: GitProvider>(provider: P) -> Result<()> {
//!     // Create a custom configuration
//!     let config = ValidationConfig {
//!         enforce_conventional_commits: true,
//!         require_work_item_references: false, // Don't require work item references
//!         auto_label: true,
//!     };
//!
//!     // Create a MergeWarden instance with custom configuration
//!     let warden = MergeWarden::with_config(provider, config);
//!
//!     // Process a pull request
//!     let result = warden.process_pull_request("owner", "repo", 123).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod checks;
pub mod config;
pub mod errors;
pub mod labels;
pub mod models;

use anyhow::Result;
use async_trait::async_trait;
use models::{Comment, Label, PullRequest};

/// Result of processing a pull request through Merge Warden.
///
/// Contains information about the validation status and any labels that were added.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Whether the PR title follows the Conventional Commits format
    pub title_valid: bool,

    /// Whether the PR description references a work item or issue
    pub work_item_referenced: bool,

    /// Labels that were added to the PR based on its content
    pub labels: Vec<String>,
}

/// Trait for interacting with Git hosting providers (e.g., GitHub, GitLab).
///
/// Implementations of this trait provide the necessary functionality to
/// interact with pull requests, comments, labels, and other Git provider features.
///
/// # Example Implementation
///
/// ```rust,no_run
/// use merge_warden_core::{GitProvider, models::{Comment, Label, PullRequest}};
/// use anyhow::Result;
/// use async_trait::async_trait;
///
/// struct GitHubProvider {
///     // Fields for authentication, etc.
///     token: String,
/// }
///
/// #[async_trait]
/// impl GitProvider for GitHubProvider {
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
///     # async fn update_pr_mergeable_state(&self, _: &str, _: &str, _: u64, _: bool) -> Result<()> { unimplemented!() }
/// }
/// ```
#[async_trait]
pub trait GitProvider {
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

    /// Updates the mergeable state of a pull request.
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
    async fn update_pr_mergeable_state(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        mergeable: bool,
    ) -> Result<()>;
}

use config::ValidationConfig;

/// Main struct for validating and managing pull requests.
///
/// `MergeWarden` is responsible for validating pull requests against configurable
/// rules and managing the associated side effects (comments, labels, etc.).
///
/// # Examples
///
/// ```rust,no_run
/// use merge_warden_core::{MergeWarden, GitProvider};
/// use anyhow::Result;
///
/// async fn example<P: GitProvider>(provider: P) -> Result<()> {
///     // Create a new MergeWarden instance with default configuration
///     let warden = MergeWarden::new(provider);
///
///     // Process a pull request
///     let result = warden.process_pull_request("owner", "repo", 123).await?;
///
///     println!("PR validation result: {:?}", result);
///     Ok(())
/// }
/// ```
pub struct MergeWarden<P: GitProvider> {
    provider: P,
    config: ValidationConfig,
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

impl<P: GitProvider> MergeWarden<P> {
    /// Checks if the PR title follows the Conventional Commits format.
    ///
    /// This is a wrapper around the `checks::title::check_pr_title` function.
    ///
    /// # Arguments
    ///
    /// * `pr` - The pull request to check
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating whether the title is valid
    fn check_title(&self, pr: &PullRequest) -> Result<bool> {
        checks::title::check_pr_title(pr)
    }

    /// Checks if the PR description references a work item or issue.
    ///
    /// This is a wrapper around the `checks::work_item::check_work_item_reference` function.
    ///
    /// # Arguments
    ///
    /// * `pr` - The pull request to check
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating whether a work item is referenced
    fn check_work_item_reference(&self, pr: &PullRequest) -> Result<bool> {
        checks::work_item::check_work_item_reference(pr)
    }

    /// Handles side effects for PR title validation.
    ///
    /// This method:
    /// - Adds or removes the invalid title label based on validation result
    /// - Adds or removes comments with suggestions for fixing the title
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr` - The pull request to validate
    /// * `is_valid_title` - Whether the PR title is valid
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn communicate_pr_title_validity_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr: &PullRequest,
        is_valid_title: bool,
    ) -> Result<()> {
        use models::{TITLE_COMMENT_MARKER, TITLE_INVALID_LABEL};

        // Skip if conventional commits are not enforced
        if !self.config.enforce_conventional_commits {
            return Ok(());
        }

        if !is_valid_title {
            // Check if PR already has the invalid title label
            let labels = self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await?;
            let has_invalid_title_label =
                labels.iter().any(|label| label.name == TITLE_INVALID_LABEL);

            if !has_invalid_title_label {
                // Add invalid title label
                self.provider
                    .add_labels(
                        repo_owner,
                        repo_name,
                        pr.number,
                        &[TITLE_INVALID_LABEL.to_string()],
                    )
                    .await?;

                // Add comment with suggestions
                let comment = format!(
                    "{}\n## Invalid PR Title Format\n\nYour PR title doesn't follow the [Conventional Commits](https://www.conventionalcommits.org/) format.\n\nPlease update your title to follow this format:\n\n`<type>(<scope>): <description>`\n\nValid types: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test\n\nExamples:\n- `feat(auth): add login with GitHub`\n- `fix: correct typo in readme`\n- `docs: update API documentation`\n- `refactor(api): simplify error handling`",
                    TITLE_COMMENT_MARKER
                );

                self.provider
                    .add_comment(repo_owner, repo_name, pr.number, &comment)
                    .await?;
            }
        } else {
            // Check if PR has the invalid title label to remove it
            let labels = self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await?;
            let has_invalid_title_label =
                labels.iter().any(|label| label.name == TITLE_INVALID_LABEL);

            if has_invalid_title_label {
                // Remove the invalid title label
                self.provider
                    .remove_label(repo_owner, repo_name, pr.number, TITLE_INVALID_LABEL)
                    .await?;

                // Find and remove the comment
                let comments = self
                    .provider
                    .list_comments(repo_owner, repo_name, pr.number)
                    .await?;

                for comment in comments {
                    if comment.body.contains(TITLE_COMMENT_MARKER) {
                        self.provider
                            .delete_comment(repo_owner, repo_name, comment.id)
                            .await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles side effects for work item reference validation.
    ///
    /// This method:
    /// - Adds or removes the missing work item label based on validation result
    /// - Adds or removes comments with suggestions for adding work item references
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr` - The pull request to validate
    /// * `has_work_item` - Whether the PR references a work item
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn communicate_pr_work_item_validity_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr: &PullRequest,
        has_work_item: bool,
    ) -> Result<()> {
        use models::{MISSING_WORK_ITEM_LABEL, WORK_ITEM_COMMENT_MARKER};

        // Skip if work item references are not required
        if !self.config.require_work_item_references {
            return Ok(());
        }

        if !has_work_item {
            // Check if PR already has the missing work item label
            let labels = self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await?;
            let has_missing_work_item_label = labels
                .iter()
                .any(|label| label.name == MISSING_WORK_ITEM_LABEL);

            if !has_missing_work_item_label {
                // Add missing work item label
                self.provider
                    .add_labels(
                        repo_owner,
                        repo_name,
                        pr.number,
                        &[MISSING_WORK_ITEM_LABEL.to_string()],
                    )
                    .await?;

                // Add comment with suggestions
                let comment = format!(
                    "{}\n## Missing Work Item Reference\n\nYour PR description doesn't reference a work item or GitHub issue. Please update it to include a reference using one of the following formats:\n\n- `Fixes #123`\n- `Closes #123`\n- `Resolves #123`\n- `References #123`\n- `Relates to #123`\n\nYou can also use the full URL to the issue.",
                    WORK_ITEM_COMMENT_MARKER
                );

                self.provider
                    .add_comment(repo_owner, repo_name, pr.number, &comment)
                    .await?;
            }
        } else {
            // Check if PR has the missing work item label to remove it
            let labels = self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await?;
            let has_missing_work_item_label = labels
                .iter()
                .any(|label| label.name == MISSING_WORK_ITEM_LABEL);

            if has_missing_work_item_label {
                // Remove the missing work item label
                self.provider
                    .remove_label(repo_owner, repo_name, pr.number, MISSING_WORK_ITEM_LABEL)
                    .await?;

                // Find and remove the comment
                let comments = self
                    .provider
                    .list_comments(repo_owner, repo_name, pr.number)
                    .await?;

                for comment in comments {
                    if comment.body.contains(WORK_ITEM_COMMENT_MARKER) {
                        self.provider
                            .delete_comment(repo_owner, repo_name, comment.id)
                            .await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Determines and adds labels to a PR based on its content.
    ///
    /// This method analyzes the PR title and body to determine appropriate labels
    /// to add, such as feature, bug, documentation, etc. It delegates to the
    /// `labels::determine_labels` function for the actual label determination.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr` - The pull request to analyze
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of labels that were added to the PR
    async fn determine_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr: &PullRequest,
    ) -> Result<Vec<String>> {
        // Skip auto-labeling if disabled
        if !self.config.auto_label {
            return Ok(Vec::new());
        }

        labels::determine_labels(&self.provider, repo_owner, repo_name, pr).await
    }

    /// Creates a new `MergeWarden` instance with default configuration.
    ///
    /// # Arguments
    ///
    /// * `provider` - An implementation of the `GitProvider` trait
    ///
    /// # Returns
    ///
    /// A new `MergeWarden` instance with default configuration
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use anyhow::Result;
    /// use async_trait::async_trait;
    /// use merge_warden_core::MergeWarden;
    /// use merge_warden_core::GitProvider;
    ///
    /// use merge_warden_core::models::{Comment, Label, PullRequest};
    ///
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl GitProvider for MyProvider {
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
    ///     # async fn update_pr_mergeable_state(&self, _: &str, _: &str, _: u64, _: bool) -> Result<()> { unimplemented!() }
    /// }
    ///
    /// fn example() {
    ///     let provider = MyProvider;
    ///     let warden = MergeWarden::new(provider);
    /// }
    ///
    /// fn main() {}
    /// ```
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            config: ValidationConfig::default(),
        }
    }

    /// Processes a pull request, validating it against the configured rules.
    ///
    /// This method:
    /// 1. Validates the PR title against the Conventional Commits format (if enabled)
    /// 2. Checks if the PR description references a work item or issue (if enabled)
    /// 3. Adds or removes labels and comments based on validation results
    /// 4. Updates the PR's mergeable state
    /// 5. Adds automatic labels based on PR content (if enabled)
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository (e.g., "octocat")
    /// * `repo_name` - The name of the repository (e.g., "hello-world")
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a `CheckResult` with the validation results
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use async_trait::async_trait;
    /// use merge_warden_core::MergeWarden;
    /// use anyhow::Result;
    /// use merge_warden_core::GitProvider;
    ///
    /// use merge_warden_core::models::{Comment, Label, PullRequest};
    ///
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl GitProvider for MyProvider {
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
    ///     # async fn update_pr_mergeable_state(&self, _: &str, _: &str, _: u64, _: bool) -> Result<()> { unimplemented!() }
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let provider = MyProvider;
    ///     let warden = MergeWarden::new(provider);
    ///
    ///     let result = warden.process_pull_request("owner", "repo", 123).await?;
    ///
    ///     if result.title_valid && result.work_item_referenced {
    ///         println!("PR is valid and can be merged!");
    ///     } else {
    ///         println!("PR has validation issues that need to be fixed");
    ///     }
    ///
    ///     Ok(())
    /// }
    ///
    /// fn main() {}
    /// ```
    pub async fn process_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<CheckResult> {
        // Get the PR
        let pr = self
            .provider
            .get_pull_request(repo_owner, repo_name, pr_number)
            .await?;

        // Check PR title follows the conventional commit structure if enabled
        let is_title_valid = if self.config.enforce_conventional_commits {
            self.check_title(&pr)?
        } else {
            true
        };

        // Check that the PR body has a reference to a work item if enabled
        let is_work_item_referenced = if self.config.require_work_item_references {
            self.check_work_item_reference(&pr)?
        } else {
            true
        };

        // Apply labels and comments based on the title validation results
        self.communicate_pr_title_validity_status(repo_owner, repo_name, &pr, is_title_valid)
            .await?;

        // Apply labels and comment based on the work item validation results
        self.communicate_pr_work_item_validity_status(
            repo_owner,
            repo_name,
            &pr,
            is_work_item_referenced,
        )
        .await?;

        // Determine labels
        let labels = self.determine_labels(repo_owner, repo_name, &pr).await?;

        // Update PR mergeability
        if is_title_valid && is_work_item_referenced {
            self.provider
                .update_pr_mergeable_state(repo_owner, repo_name, pr_number, true)
                .await?;
        } else {
            self.provider
                .update_pr_mergeable_state(repo_owner, repo_name, pr_number, false)
                .await?;
        }

        Ok(CheckResult {
            title_valid: is_title_valid,
            work_item_referenced: is_work_item_referenced,
            labels,
        })
    }

    /// Creates a new `MergeWarden` instance with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `provider` - An implementation of the `GitProvider` trait
    /// * `config` - A custom `ValidationConfig` instance
    ///
    /// # Returns
    ///
    /// A new `MergeWarden` instance with the specified configuration
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use anyhow::Result;
    /// use async_trait::async_trait;
    /// use merge_warden_core::{MergeWarden, config::ValidationConfig};
    /// use merge_warden_core::GitProvider;
    ///
    /// use merge_warden_core::models::{Comment, Label, PullRequest};
    ///
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl GitProvider for MyProvider {
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
    ///     # async fn update_pr_mergeable_state(&self, _: &str, _: &str, _: u64, _: bool) -> Result<()> { unimplemented!() }
    /// }
    ///
    /// fn example() {
    ///     let provider = MyProvider;
    ///     let config = ValidationConfig {
    ///         enforce_conventional_commits: true,
    ///         require_work_item_references: false,
    ///         auto_label: true,
    ///     };
    ///
    ///     let warden = MergeWarden::with_config(provider, config);
    /// }
    ///
    /// fn main() {}
    /// ```
    pub fn with_config(provider: P, config: ValidationConfig) -> Self {
        Self { provider, config }
    }
}
