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
//! use merge_warden_developer_platforms::PullRequestProvider;
//! use merge_warden_core::{MergeWarden, config::ValidationConfig};
//! use anyhow::Result;
//!
//! async fn validate_pr<P: PullRequestProvider>(provider: P) -> Result<()> {
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
//! async fn validate_pr_custom<P: PullRequestProvider>(provider: P) -> Result<()> {
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

use indoc::formatdoc;
use merge_warden_developer_platforms::models::PullRequest;
use merge_warden_developer_platforms::PullRequestProvider;

pub mod checks;
pub mod config;
pub mod errors;
use errors::MergeWardenError;

pub mod labels;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

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

use config::ValidationConfig;

/// Main struct for validating and managing pull requests.
///
/// `MergeWarden` is responsible for validating pull requests against configurable
/// rules and managing the associated side effects (comments, labels, etc.).
///
/// # Examples
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::PullRequestProvider;
/// use merge_warden_core::MergeWarden;
/// use anyhow::Result;
///
/// async fn example<P: PullRequestProvider>(provider: P) -> Result<()> {
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
pub struct MergeWarden<P: PullRequestProvider> {
    provider: P,
    config: ValidationConfig,
}

impl<P: PullRequestProvider> MergeWarden<P> {
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
    fn check_title(&self, pr: &PullRequest) -> bool {
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
    fn check_work_item_reference(&self, pr: &PullRequest) -> bool {
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
    ) -> Result<(), MergeWardenError> {
        use config::{TITLE_COMMENT_MARKER, TITLE_INVALID_LABEL};

        // Skip if conventional commits are not enforced
        if !self.config.enforce_conventional_commits {
            return Ok(());
        }

        if !is_valid_title {
            // Check if PR already has the invalid title label
            let labels = (self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();
            let has_invalid_title_label =
                labels.iter().any(|label| label.name == TITLE_INVALID_LABEL);

            if !has_invalid_title_label {
                // Add invalid title label
                let _ = self
                    .provider
                    .add_labels(
                        repo_owner,
                        repo_name,
                        pr.number,
                        &[TITLE_INVALID_LABEL.to_string()],
                    )
                    .await;

                // Add comment with suggestions
                let comment = format!(
                    "{}\n## Invalid PR Title Format\n\nYour PR title doesn't follow the [Conventional Commits](https://www.conventionalcommits.org/) format.\n\nPlease update your title to follow this format:\n\n`<type>(<scope>): <description>`\n\nValid types: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test\n\nExamples:\n- `feat(auth): add login with GitHub`\n- `fix: correct typo in readme`\n- `docs: update API documentation`\n- `refactor(api): simplify error handling`",
                    TITLE_COMMENT_MARKER
                );

                self.provider
                    .add_comment(repo_owner, repo_name, pr.number, &comment)
                    .await
                    .map_err(|_| {
                        MergeWardenError::FailedToUpdatePullRequest(
                            "Failed to add comment".to_string(),
                        )
                    })?;
            }
        } else {
            // Check if PR has the invalid title label to remove it
            let labels = (self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            let has_invalid_title_label =
                labels.iter().any(|label| label.name == TITLE_INVALID_LABEL);

            if has_invalid_title_label {
                // Remove the invalid title label
                let _ = self
                    .provider
                    .remove_label(repo_owner, repo_name, pr.number, TITLE_INVALID_LABEL)
                    .await;

                // Find and remove the comment
                let comments = (self
                    .provider
                    .list_comments(repo_owner, repo_name, pr.number)
                    .await)
                    .unwrap_or_default();

                for comment in comments {
                    if comment.body.contains(TITLE_COMMENT_MARKER) {
                        self.provider
                            .delete_comment(repo_owner, repo_name, comment.id)
                            .await
                            .map_err(|_| {
                                MergeWardenError::FailedToUpdatePullRequest(
                                    "Failed to add comment".to_string(),
                                )
                            })?;
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
    ) -> Result<(), MergeWardenError> {
        use config::{MISSING_WORK_ITEM_LABEL, WORK_ITEM_COMMENT_MARKER};

        // Skip if work item references are not required
        if !self.config.require_work_item_references {
            return Ok(());
        }

        if !has_work_item {
            // Check if PR already has the missing work item label
            let labels = (self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();
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
                    .await
                    .map_err(|_| {
                        MergeWardenError::FailedToUpdatePullRequest(
                            "Failed to add label".to_string(),
                        )
                    })?;

                // Add comment with suggestions
                let comment = format!(
                    "{}\n## Missing Work Item Reference\n\nYour PR description doesn't reference a work item or GitHub issue. Please update it to include a reference using one of the following formats:\n\n- `Fixes #123`\n- `Closes #123`\n- `Resolves #123`\n- `References #123`\n- `Relates to #123`\n\nYou can also use the full URL to the issue.",
                    WORK_ITEM_COMMENT_MARKER
                );

                self.provider
                    .add_comment(repo_owner, repo_name, pr.number, &comment)
                    .await
                    .map_err(|_| {
                        MergeWardenError::FailedToUpdatePullRequest(
                            "Failed to add comment".to_string(),
                        )
                    })?;
            }
        } else {
            // Check if PR has the missing work item label to remove it
            let labels = (self
                .provider
                .list_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            let has_missing_work_item_label = labels
                .iter()
                .any(|label| label.name == MISSING_WORK_ITEM_LABEL);

            if has_missing_work_item_label {
                // Remove the missing work item label
                let _ = self
                    .provider
                    .remove_label(repo_owner, repo_name, pr.number, MISSING_WORK_ITEM_LABEL)
                    .await;

                // Find and remove the comment
                let comments = (self
                    .provider
                    .list_comments(repo_owner, repo_name, pr.number)
                    .await)
                    .unwrap_or_default();

                for comment in comments {
                    if comment.body.contains(WORK_ITEM_COMMENT_MARKER) {
                        self.provider
                            .delete_comment(repo_owner, repo_name, comment.id)
                            .await
                            .map_err(|_| {
                                MergeWardenError::FailedToUpdatePullRequest(
                                    "Failed to delete comment".to_string(),
                                )
                            })?;
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
    ) -> Result<Vec<String>, MergeWardenError> {
        // Skip auto-labeling if disabled
        if !self.config.auto_label {
            return Ok(Vec::new());
        }

        labels::set_pull_request_labels(&self.provider, repo_owner, repo_name, pr).await
    }

    /// Generates a review message based on the validation status of PR title and body
    fn generate_review_message(&self, title_valid: bool, body_valid: bool) -> String {
        match (title_valid, body_valid) {
            // Both are incorrect
            (false, false) => formatdoc!(
                "The pull request needs some improvements:

    1. Title Convention: Your PR title does not follow the conventional commit message format.
    - Supported types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
    - Expected format: <type>(<optional scope>): <description>
    - Examples:
        * feat(auth): add login functionality
        * fix: resolve null pointer exception
    - For full details, see: https://www.conventionalcommits.org/

    2. Work Item Tracking: The PR body is missing a valid work item reference.
    - Supported formats:
        * Prefixes: fixes, closes, resolves, references, relates to
        * Work Item Identifiers: #XXX or GH-XXX
    - Examples:
        * fixes #1234
        * closes GH-5678
        * resolves #9012
        * references GH-3456
        * relates to #7890

    Please update both the title and body to meet these requirements."
            ),

            // Title is incorrect, body is valid
            (false, true) => formatdoc!(
                "The pull request title needs correction:

    1. Title Convention: Your PR title does not follow the conventional commit message format.
    - Supported types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
    - Expected format: <type>(<optional scope>): <description>
    - Examples:
        * feat(auth): add login functionality
        * fix: resolve null pointer exception
    - For full details, see: https://www.conventionalcommits.org/

    Please update the PR title to match the conventional commit message guidelines."
            ),

            // Title is valid, body is incorrect
            (true, false) => formatdoc!(
                "The pull request body needs improvement:

    1. Work Item Tracking: The PR body is missing a valid work item reference.
    - Supported formats:
        * Prefixes: fixes, closes, resolves, references, relates to
        * Work Item Identifiers: #XXX or GH-XXX
    - Examples:
        * fixes #1234
        * closes GH-5678
        * resolves #9012
        * references GH-3456
        * relates to #7890

    Please update the PR body to include a valid work item reference."
            ),

            // Both are correct (this will return None, effectively removing the review)
            (true, true) => String::new(),
        }
    }

    /// Creates a new `MergeWarden` instance with default configuration.
    ///
    /// # Arguments
    ///
    /// * `provider` - An implementation of the `PullRequestProvider` trait
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
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// use merge_warden_developer_platforms::errors::Error;
    /// use merge_warden_developer_platforms::models::{Comment, Label, PullRequest};
    ///
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl PullRequestProvider for MyProvider {
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
    ///     # async fn update_pr_blocking_review(&self, _: &str, _: &str, _: u64, _: &str, _: bool) -> Result<(), Error> { unimplemented!() }
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
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// use merge_warden_developer_platforms::errors::Error;
    /// use merge_warden_developer_platforms::models::{Comment, Label, PullRequest};
    ///
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl PullRequestProvider for MyProvider {
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
    ///     # async fn update_pr_blocking_review(&self, _: &str, _: &str, _: u64, _: &str, _: bool) -> Result<(), Error> { unimplemented!() }
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
    ) -> Result<CheckResult, MergeWardenError> {
        // Get the PR
        let pr = self
            .provider
            .get_pull_request(repo_owner, repo_name, pr_number)
            .await
            .map_err(|_| {
                MergeWardenError::GitProviderError(format!(
                    "Failed to find the PR with number [{}] in {}/{}",
                    pr_number, repo_owner, repo_name
                ))
            })?;

        // Check PR title follows the conventional commit structure if enabled
        let is_title_valid = if self.config.enforce_conventional_commits {
            self.check_title(&pr)
        } else {
            true
        };

        // Check that the PR body has a reference to a work item if enabled
        let is_work_item_referenced = if self.config.require_work_item_references {
            self.check_work_item_reference(&pr)
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
        let review_message = self.generate_review_message(is_title_valid, is_work_item_referenced);
        self.provider
            .update_pr_blocking_review(
                repo_owner,
                repo_name,
                pr_number,
                review_message.as_str(),
                is_title_valid && is_work_item_referenced,
            )
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest("Failed to add review".to_string())
            })?;

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
    /// * `provider` - An implementation of the `PullRequestProvider` trait
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
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// use merge_warden_developer_platforms::errors::Error;
    /// use merge_warden_developer_platforms::models::{Comment, Label, PullRequest};
    ///
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl PullRequestProvider for MyProvider {
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
    ///     # async fn update_pr_blocking_review(&self, _: &str, _: &str, _: u64, _: &str, _: bool) -> Result<(), Error> { unimplemented!() }
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
