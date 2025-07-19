#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

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
//! use merge_warden_core::{
//!     MergeWarden,
//!     config::{ BypassRules, CONVENTIONAL_COMMIT_REGEX, CurrentPullRequestValidationConfiguration, MISSING_WORK_ITEM_LABEL, TITLE_INVALID_LABEL, WORK_ITEM_REGEX }};
//! use anyhow::Result;
//!
//! async fn validate_pr<P: PullRequestProvider + std::fmt::Debug>(provider: P) -> Result<()> {
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
//! async fn validate_pr_custom<P: PullRequestProvider + std::fmt::Debug>(provider: P) -> Result<()> {
//!     // Create a custom configuration
//!     let config = CurrentPullRequestValidationConfiguration {
//!         enforce_title_convention: true,
//!         title_pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
//!         invalid_title_label: Some(TITLE_INVALID_LABEL.to_string()),
//!         enforce_work_item_references: true,
//!         work_item_reference_pattern: WORK_ITEM_REGEX.to_string(),
//!         missing_work_item_label: Some(MISSING_WORK_ITEM_LABEL.to_string()),
//!         bypass_rules: BypassRules::default(),
//!         pr_size_check: Default::default(),
//!         change_type_labels: None,
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
use merge_warden_developer_platforms::models::{Installation, PullRequest, Repository};
use merge_warden_developer_platforms::PullRequestProvider;

pub mod checks;
pub mod config;
use config::CurrentPullRequestValidationConfiguration;
use config::TITLE_COMMENT_MARKER;
use config::WORK_ITEM_COMMENT_MARKER;

/// Error types and utilities for Merge Warden operations.
///
/// This module contains error types that can occur during pull request
/// validation, configuration parsing, and Git provider interactions.
pub mod errors;
use errors::MergeWardenError;
use serde::Deserialize;
use tracing::{debug, error, info, instrument, warn};

pub mod labels;
pub mod size;
pub mod validation_result;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// Result of processing a pull request through Merge Warden.
///
/// Contains information about the validation status, any labels that were added,
/// and details about any bypass rules that were used during validation.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Whether the PR title follows the Conventional Commits format or was bypassed
    pub title_valid: bool,

    /// Whether the PR description references a work item or issue
    pub work_item_referenced: bool,

    /// Whether the PR size validation passed
    pub size_valid: bool,

    /// Labels that were added to the PR based on its content
    pub labels: Vec<String>,

    /// Information about any bypasses that were used during validation
    pub bypasses_used: Vec<validation_result::BypassInfo>,
}

/// Webhook payload structure for GitHub webhook events.
///
/// This struct represents the JSON payload received from GitHub webhooks
/// when pull request events occur. It contains the essential information
/// needed to process pull request events.
///
/// # Fields
///
/// * `action` - The type of action that triggered the webhook (e.g., "opened", "synchronize")
/// * `pull_request` - The pull request data, if available
/// * `repository` - The repository information
/// * `installation` - The GitHub App installation information, if applicable
///
/// # Examples
///
/// ```rust
/// use merge_warden_core::WebhookPayload;
/// use serde_json::from_str;
///
/// let json = r#"{
///     "action": "opened",
///     "pull_request": null,
///     "repository": null,
///     "installation": null
/// }"#;
///
/// let payload: WebhookPayload = from_str(json).expect("Failed to parse webhook payload");
/// assert_eq!(payload.action, "opened");
/// ```
#[derive(Deserialize)]
pub struct WebhookPayload {
    /// The action that triggered the webhook event
    pub action: String,

    /// The pull request data, if available in the webhook payload
    pub pull_request: Option<PullRequest>,

    /// The repository information from the webhook
    pub repository: Option<Repository>,

    /// The GitHub App installation information, if applicable
    pub installation: Option<Installation>,
}

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
/// async fn example<P: PullRequestProvider + std::fmt::Debug>(provider: P) -> Result<()> {
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
#[derive(Debug)]
pub struct MergeWarden<P: PullRequestProvider + std::fmt::Debug> {
    /// The pull request provider implementation (GitHub, GitLab, etc.)
    provider: P,

    /// The validation configuration settings for this instance
    config: CurrentPullRequestValidationConfiguration,
}

impl<P: PullRequestProvider + std::fmt::Debug> MergeWarden<P> {
    /// Checks if the PR title follows the Conventional Commits format.
    ///
    /// This is a wrapper around the `checks::check_pr_title` function that returns
    /// detailed validation results including any bypass information.
    ///
    /// # Arguments
    ///
    /// * `pr` - The pull request to check
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing validation status and bypass information
    #[instrument]
    fn check_title(&self, pr: &PullRequest) -> validation_result::ValidationResult {
        debug!(pull_request = pr.number, "Checking PR title");
        checks::check_pr_title(
            pr,
            self.config.bypass_rules.title_convention(),
            &self.config,
        )
    }

    /// Checks if the PR description references a work item or issue.
    ///
    /// This is a wrapper around the `checks::check_work_item_reference` function that returns
    /// detailed validation results including any bypass information.
    ///
    /// # Arguments
    ///
    /// * `pr` - The pull request to check
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing validation status and bypass information
    #[instrument]
    fn check_work_item_reference(&self, pr: &PullRequest) -> validation_result::ValidationResult {
        debug!(
            pull_request = pr.number,
            "Checking work item reference in PR description"
        );
        checks::check_work_item_reference(
            pr,
            self.config.bypass_rules.work_item_convention(),
            &self.config,
        )
    }

    /// Checks the size of the PR based on file changes.
    ///
    /// This is a wrapper around the `checks::check_pr_size` function that returns
    /// detailed validation results.
    ///
    /// # Arguments
    ///
    /// * `pr_files` - List of files changed in the pull request
    /// * `user` - The user who created the pull request (for bypass checking)
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing size validation status
    #[instrument]
    fn check_pr_size(
        &self,
        pr_files: &[merge_warden_developer_platforms::models::PullRequestFile],
        user: Option<&merge_warden_developer_platforms::models::User>,
    ) -> validation_result::ValidationResult {
        debug!("Checking PR size");
        checks::check_pr_size(
            pr_files,
            user,
            self.config.bypass_rules.size(),
            &self.config,
        )
    }

    /// Handles side effects for PR title validation.
    ///
    /// This method:
    /// - Adds or removes the invalid title label based on validation result
    /// - Adds or removes comments with suggestions for fixing the title
    /// - Logs bypass usage for audit trails when applicable
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr` - The pull request to validate
    /// * `validation_result` - The result of title validation including bypass information
    ///
    /// # Returns
    ///
    /// A `String` containing the comment text that was generated
    #[instrument]
    async fn communicate_pr_title_validity_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr: &PullRequest,
        validation_result: &validation_result::ValidationResult,
    ) -> String {
        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr.number,
            is_valid = validation_result.is_valid(),
            bypass_used = validation_result.was_bypassed(),
            "Updating the pull request to indicate title validation status",
        );

        // Skip if conventional commits are not enforced
        if !self.config.enforce_title_convention {
            debug!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr.number,
                "Not validating the title of pull request",
            );
            return String::new();
        }

        // Log bypass usage for audit trail
        if let Some(bypass_info) = validation_result.bypass_info() {
            info!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr.number,
                user = bypass_info.user,
                rule_type = ?bypass_info.rule_type,
                pr_title = pr.title,
                pr_author = pr.author.as_ref().map(|u| &u.login),
                "Validation bypass used"
            );
        }

        let is_valid = validation_result.is_valid();
        let was_bypassed = validation_result.was_bypassed();

        if !is_valid {
            // Check if PR already has the invalid title label
            let labels = (self
                .provider
                .list_applied_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();
            debug!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr.number,
                count = labels.len(),
                "Searched for existing labels",
            );

            if let Some(title_label) = &self.config.invalid_title_label {
                let has_invalid_title_label = labels.iter().any(|label| &label.name == title_label);

                if !has_invalid_title_label {
                    // Add invalid title label
                    let result = self
                        .provider
                        .add_labels(repo_owner, repo_name, pr.number, &[title_label.to_string()])
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            "The pull request title is invalid. Added a label to indicate the issue."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "The pull request title is invalid. Failed to add a label to indicate the issue."
                        );
                    }
                }
            }

            let comments = (self
                .provider
                .list_comments(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            let mut has_comment = false;
            for comment in comments {
                if comment.body.contains(TITLE_COMMENT_MARKER) {
                    has_comment = true;
                    break;
                }
            }

            // Add comment with suggestions
            let comment_text = r#"
The pull request title needs correction:

Your PR title does not follow the [Conventional Commits](https://www.conventionalcommits.org/) message format.
- Supported types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
- Expected format: `<type>(<optional scope>): <description>`
- Examples:
    * feat(auth): add login functionality
    * fix: resolve null pointer exception
- For full details, see: https://www.conventionalcommits.org/

Please update the PR title to match the conventional commit message guidelines."#;

            let comment = format!(
                "{prefix}{text}",
                prefix = TITLE_COMMENT_MARKER,
                text = comment_text
            );
            if !has_comment {
                let result = self
                    .provider
                    .add_comment(repo_owner, repo_name, pr.number, &comment)
                    .await;

                if result.is_ok() {
                    info!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr.number,
                        "The pull request title is invalid. Added a comment to indicate the issue."
                    );
                } else {
                    let e = result.unwrap_err();
                    warn!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr.number,
                        error = e.to_string(),
                        "The pull request title is invalid. Failed to add a comment to indicate the issue."
                    );
                }
            }

            comment_text.to_string()
        } else {
            // Title validation passed (either valid or bypassed)

            // Check if PR has the invalid title label to remove it
            let labels = (self
                .provider
                .list_applied_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            debug!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr.number,
                count = labels.len(),
                "Searched for existing labels",
            );

            if let Some(title_label) = &self.config.invalid_title_label {
                let has_invalid_title_label = labels.iter().any(|label| &label.name == title_label);

                if has_invalid_title_label {
                    // Remove the invalid title label
                    let result = self
                        .provider
                        .remove_label(repo_owner, repo_name, pr.number, title_label)
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            "The pull request title is valid. Removed a label that was indicating the issue."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "The pull request title is valid. Failed to remove a label that was indicating the issue."
                        );
                    }
                }
            }

            // Find and remove any existing invalid title comments
            let comments = (self
                .provider
                .list_comments(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            for comment in comments {
                if comment.body.contains(TITLE_COMMENT_MARKER) {
                    let result = self
                        .provider
                        .delete_comment(repo_owner, repo_name, comment.id)
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            "Removed existing title validation comment."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "Failed to remove existing title validation comment."
                        );
                    }
                    break;
                }
            }

            // If validation was bypassed, add a bypass notification comment
            if was_bypassed {
                if let Some(bypass_info) = validation_result.bypass_info() {
                    let bypass_comment_text = formatdoc!(
                        r#"
                        ⚠️ **Title Validation Bypassed**

                        The PR title validation was bypassed for user `{user}`.
                        - Original title: `{title}`
                        - Bypass rule: {rule_type}
                        - Bypassed by: {user}

                        **Note**: This PR may not follow conventional commit format but was allowed due to bypass permissions.

                        ---
                        For more information about conventional commits, see: https://www.conventionalcommits.org/
                        "#,
                        user = bypass_info.user,
                        title = pr.title,
                        rule_type = bypass_info.rule_type
                    );

                    let comment = format!(
                        "{prefix}{text}",
                        prefix = TITLE_COMMENT_MARKER,
                        text = bypass_comment_text
                    );

                    let result = self
                        .provider
                        .add_comment(repo_owner, repo_name, pr.number, &comment)
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            user = bypass_info.user,
                            "Added bypass notification comment for title validation."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "Failed to add bypass notification comment."
                        );
                    }

                    return bypass_comment_text;
                }
            }

            String::new()
        }
    }

    /// Handles side effects for work item reference validation.
    ///
    /// This method:
    /// - Adds or removes the missing work item label based on validation result
    /// - Adds or removes comments with suggestions for adding work item references
    /// - Logs bypass usage for audit trails when applicable
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr` - The pull request to validate
    /// * `validation_result` - The result of work item validation including bypass information
    ///
    /// # Returns
    ///
    /// A `String` containing the comment text that was generated
    #[instrument]
    async fn communicate_pr_work_item_validity_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr: &PullRequest,
        validation_result: &validation_result::ValidationResult,
    ) -> String {
        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr.number,
            is_valid = validation_result.is_valid(),
            bypass_used = validation_result.was_bypassed(),
            "Updating the pull request to indicate work item validation status",
        );

        // Skip if work item references are not required
        if !self.config.enforce_work_item_references {
            return String::new();
        }

        // Log bypass usage for audit trail
        if let Some(bypass_info) = validation_result.bypass_info() {
            info!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr.number,
                user = bypass_info.user,
                rule_type = ?bypass_info.rule_type,
                pr_title = pr.title,
                pr_author = pr.author.as_ref().map(|u| &u.login),
                "Validation bypass used"
            );
        }

        let is_valid = validation_result.is_valid();
        let was_bypassed = validation_result.was_bypassed();

        if !is_valid {
            // Check if PR already has the missing work item label
            let labels = (self
                .provider
                .list_applied_labels(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();
            debug!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr.number,
                count = labels.len(),
                "Searched for existing labels",
            );

            if let Some(work_item_label) = &self.config.missing_work_item_label {
                let has_missing_work_item_label =
                    labels.iter().any(|label| &label.name == work_item_label);

                if !has_missing_work_item_label {
                    // Add missing work item label
                    let result = self
                        .provider
                        .add_labels(
                            repo_owner,
                            repo_name,
                            pr.number,
                            &[work_item_label.to_string()],
                        )
                        .await;
                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            "The pull request does not have a work item reference. Added a label to indicate the issue."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "The pull request does not have a work item reference. Failed to add a label to indicate the issue."
                        );
                    }
                }
            }

            let comments = (self
                .provider
                .list_comments(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            let mut has_comment = false;
            for comment in comments {
                if comment.body.contains(WORK_ITEM_COMMENT_MARKER) {
                    has_comment = true;
                    break;
                }
            }

            let comment_text = r#"
The pull request body needs improvement:

The PR body is missing a valid work item reference.
- Supported formats:
    * Prefixes: fixes, closes, resolves, references, relates to
    * Work Item Identifiers: #XXX or GH-XXX
- Examples:
    * fixes #1234
    * closes GH-5678
    * resolves #9012
    * references GH-3456
    * relates to #7890

Please update the PR body to include a valid work item reference."#;
            let comment = format!(
                "{prefix}{text}",
                prefix = WORK_ITEM_COMMENT_MARKER,
                text = comment_text,
            );
            if !has_comment {
                // Add comment with suggestions

                let result = self
                    .provider
                    .add_comment(repo_owner, repo_name, pr.number, &comment)
                    .await;

                if result.is_ok() {
                    info!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr.number,
                        "The pull request does not have a work item reference. Added a comment to indicate the issue."
                    );
                } else {
                    let e = result.unwrap_err();
                    warn!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr.number,
                        error = e.to_string(),
                        "The pull request does not have a work item reference. Failed to add a comment to indicate the issue."
                    );
                }
            }

            comment_text.to_string()
        } else {
            // Work item validation passed (either valid or bypassed)

            // Check if PR has the missing work item label to remove it
            if let Some(work_item_label) = &self.config.missing_work_item_label {
                let labels = (self
                    .provider
                    .list_applied_labels(repo_owner, repo_name, pr.number)
                    .await)
                    .unwrap_or_default();

                debug!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr.number,
                    count = labels.len(),
                    "Searched for existing labels",
                );

                let has_missing_work_item_label =
                    labels.iter().any(|label| &label.name == work_item_label);

                if has_missing_work_item_label {
                    // Remove the missing work item label
                    let result = self
                        .provider
                        .remove_label(repo_owner, repo_name, pr.number, work_item_label)
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            "The pull request has a work item reference. Removed a label that was indicating the issue."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "The pull request has a work item reference. Failed to remove a label that was indicating the issue."
                        );
                    }
                }
            }

            // Find and remove any existing missing work item comments
            let comments = (self
                .provider
                .list_comments(repo_owner, repo_name, pr.number)
                .await)
                .unwrap_or_default();

            for comment in comments {
                if comment.body.contains(WORK_ITEM_COMMENT_MARKER) {
                    let result = self
                        .provider
                        .delete_comment(repo_owner, repo_name, comment.id)
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            "Removed existing work item validation comment."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "Failed to remove existing work item validation comment."
                        );
                    }
                    break;
                }
            }

            // If validation was bypassed, add a bypass notification comment
            if was_bypassed {
                if let Some(bypass_info) = validation_result.bypass_info() {
                    let bypass_comment_text = formatdoc!(
                        r#"
                        ⚠️ **Work Item Validation Bypassed**

                        The work item reference validation was bypassed for user `{user}`.
                        - PR description may not contain required work item references
                        - Bypass rule: {rule_type}
                        - Bypassed by: {user}

                        **Note**: This PR was allowed to proceed without work item references due to bypass permissions.
                        "#,
                        user = bypass_info.user,
                        rule_type = bypass_info.rule_type
                    );

                    let comment = format!(
                        "{prefix}{text}",
                        prefix = WORK_ITEM_COMMENT_MARKER,
                        text = bypass_comment_text
                    );

                    let result = self
                        .provider
                        .add_comment(repo_owner, repo_name, pr.number, &comment)
                        .await;

                    if result.is_ok() {
                        info!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            user = bypass_info.user,
                            "Added bypass notification comment for work item validation."
                        );
                    } else {
                        let e = result.unwrap_err();
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr.number,
                            error = e.to_string(),
                            "Failed to add bypass notification comment."
                        );
                    }

                    return bypass_comment_text;
                }
            }

            String::new()
        }
    }

    /// Handles size labeling and comments for a pull request.
    ///
    /// This method:
    /// - Calculates PR size based on file changes
    /// - Applies appropriate size labels
    /// - Adds educational comments for oversized PRs if configured
    /// - Returns a status message for inclusion in the check status
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `pr_files` - List of files changed in the PR
    ///
    /// # Returns
    ///
    /// A status message describing the size analysis result
    #[instrument]
    async fn communicate_pr_size_status(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        pr_files: &[merge_warden_developer_platforms::models::PullRequestFile],
    ) -> String {
        // Calculate size info
        let size_info = crate::size::PrSizeInfo::from_files_with_exclusions(
            pr_files,
            &self.config.pr_size_check.get_effective_thresholds(),
            &self.config.pr_size_check.excluded_file_patterns,
        );

        // Apply size label
        let label_result = labels::manage_size_labels(
            &self.provider,
            repo_owner,
            repo_name,
            pr_number,
            &size_info,
        )
        .await;

        match label_result {
            Ok(Some(label)) => {
                debug!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    label = label,
                    "Applied size label"
                );
            }
            Err(e) => {
                warn!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    error = e.to_string(),
                    "Failed to apply size label"
                );
            }
            _ => {}
        }

        // Add comment for oversized PRs if configured
        if self.config.pr_size_check.add_comment && size_info.is_oversized() {
            let comment = labels::generate_oversized_pr_comment(&size_info);

            match self
                .provider
                .add_comment(repo_owner, repo_name, pr_number, &comment)
                .await
            {
                Ok(_) => {
                    info!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr_number,
                        "Added oversized PR comment"
                    );
                }
                Err(e) => {
                    warn!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr_number,
                        error = e.to_string(),
                        "Failed to add oversized PR comment"
                    );
                }
            }
        }

        // Return status message
        format!(
            "PR size: {} ({} lines across {} files)",
            size_info.size_category.as_str().to_uppercase(),
            size_info.total_lines_changed,
            size_info.included_files.len()
        )
    }

    /// Determines and adds labels to a PR based on its content.
    ///
    /// This method analyzes the PR title and body to determine appropriate labels
    /// to add, such as feature, bug, documentation, etc. It supports both legacy
    /// hardcoded labeling and smart label detection based on repository configuration.
    ///
    /// Features:
    /// - Smart label detection using repository-specific label mappings
    /// - Fallback to hardcoded labels if smart detection fails
    /// - Performance monitoring and audit logging
    /// - Graceful error handling that doesn't block PR processing
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
    #[instrument(
        fields(
            repository_owner = repo_owner,
            repository = repo_name,
            pr_number = pr.number,
            pr_title = %pr.title,
            smart_labeling_enabled = self.config.change_type_labels.is_some()
        )
    )]
    async fn determine_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr: &PullRequest,
    ) -> Result<Vec<String>, MergeWardenError> {
        let start_time = std::time::Instant::now();

        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pr_number = pr.number,
            smart_labeling_enabled = self.config.change_type_labels.is_some(),
            "Starting label determination for pull request"
        );

        // Attempt smart label detection with graceful error handling
        let result = labels::set_pull_request_labels_with_config(
            &self.provider,
            repo_owner,
            repo_name,
            pr,
            Some(&self.config),
        )
        .await;

        let elapsed = start_time.elapsed();

        match result {
            Ok(applied_labels) => {
                info!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pr_number = pr.number,
                    applied_labels = ?applied_labels,
                    labels_count = applied_labels.len(),
                    processing_duration_ms = elapsed.as_millis(),
                    "Successfully determined and applied labels"
                );

                // Log audit trail for applied labels
                for label in applied_labels.iter() {
                    debug!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pr_number = pr.number,
                        label_name = label.clone(),
                        detection_method = if self.config.change_type_labels.is_some() {
                            "smart_detection"
                        } else {
                            "legacy_hardcoded"
                        },
                        "Applied label to pull request"
                    );
                }

                return Ok(applied_labels);
            }
            Err(e) => {
                // Label failures should be logged but not propagate
                // The core validation and status reporting should continue to work
                warn!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pr_number = pr.number,
                    error = %e,
                    processing_duration_ms = elapsed.as_millis(),
                    "Label determination failed, but PR processing will continue"
                );

                // Return empty labels vector instead of failing
                return Ok(Vec::new());
            }
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
    /// #[derive(Debug)]
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
    ///     # async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<Label>, Error> { unimplemented!() }
    ///     # async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    ///     # async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    ///     # async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<Label>, Error> { unimplemented!() }
    ///     # async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    ///     # async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
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
            config: CurrentPullRequestValidationConfiguration::default(),
        }
    }

    /// Processes a pull request, validating it against the configured rules.
    ///
    /// This method:
    /// 1. Validates the PR title against the Conventional Commits format (if enabled)
    /// 2. Checks if the PR description references a work item or issue (if enabled)
    /// 3. Adds or removes labels and comments based on validation results
    /// 4. Updates the PR's check run status (GitHub status check)
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
    /// #[derive(Debug)]
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
    ///     # async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<Label>, Error> { unimplemented!() }
    ///     # async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    ///     # async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    ///     # async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<Label>, Error> { unimplemented!() }
    ///     # async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    ///     # async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
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
    #[instrument]
    pub async fn process_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<CheckResult, MergeWardenError> {
        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr_number,
            "Processing pull request",
        );

        // Get the PR
        let pr = self
            .provider
            .get_pull_request(repo_owner, repo_name, pr_number)
            .await
            .map_err(|e| {
                error!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    error = e.to_string(),
                    "Failed to Find the PR"
                );

                MergeWardenError::GitProviderError(format!(
                    "Failed to find the PR with number [{}] in {}/{}",
                    pr_number, repo_owner, repo_name
                ))
            })?;

        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr_number,
            "Got pull request",
        );

        // If the pull request is a draft then we don't review it initially. We wait until it is ready for review
        let check_title = "Merge Warden";
        if pr.draft {
            info!(message = "Pull request is in draft mode. Will not review pull request until it is marked as ready for review.");

            self.provider
                .update_pr_check_status(
                    repo_owner,
                    repo_name,
                    pr_number,
                    "skipped",
                    check_title,
                    "Pull request is in draft mode. Will not review pull request until it is marked as ready for review.",
                    "",
                )
                .await
                .map_err(|e| {
                    error!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr_number,
                        error = e.to_string(),
                        "Failed to add or update GitHub check run"
                    );
                    MergeWardenError::FailedToUpdatePullRequest(
                        "Failed to add or update GitHub check run".to_string(),
                    )
                })?;
            return Ok(CheckResult {
                title_valid: true,
                work_item_referenced: true,
                size_valid: true,
                labels: Vec::<String>::new(),
                bypasses_used: Vec::new(),
            });
        }

        // Check PR title follows the conventional commit structure if enabled
        let title_result = if self.config.enforce_title_convention {
            self.check_title(&pr)
        } else {
            validation_result::ValidationResult::valid()
        };

        // Check that the PR body has a reference to a work item if enabled
        let work_item_result = if self.config.enforce_work_item_references {
            self.check_work_item_reference(&pr)
        } else {
            validation_result::ValidationResult::valid()
        };

        // Fetch PR files for size analysis if size checking is enabled
        let (size_result, pr_files) = if self.config.pr_size_check.enabled {
            let files = self
                .provider
                .get_pull_request_files(repo_owner, repo_name, pr_number)
                .await
                .map_err(|e| {
                    error!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        pull_request = pr_number,
                        error = e.to_string(),
                        "Failed to fetch PR files for size analysis"
                    );
                    MergeWardenError::GitProviderError("Failed to fetch PR files".to_string())
                })?;

            let result = self.check_pr_size(&files, pr.author.as_ref());
            (result, Some(files))
        } else {
            (validation_result::ValidationResult::valid(), None)
        };

        // Collect bypass information for audit trail
        let mut bypasses_used = Vec::new();
        if let Some(bypass_info) = title_result.bypass_info() {
            bypasses_used.push(bypass_info.clone());
        }
        if let Some(bypass_info) = work_item_result.bypass_info() {
            bypasses_used.push(bypass_info.clone());
        }
        if let Some(bypass_info) = size_result.bypass_info() {
            bypasses_used.push(bypass_info.clone());
        }

        // Extract validity flags for downstream logic
        let is_title_valid = title_result.is_valid();
        let is_work_item_referenced = work_item_result.is_valid();
        let is_size_valid = size_result.is_valid();

        // Apply labels and comments based on the title validation results
        let title_message = if title_result.bypass_info().is_some() {
            "Title validation bypassed".to_string()
        } else {
            self.communicate_pr_title_validity_status(repo_owner, repo_name, &pr, &title_result)
                .await
        };

        // Apply labels and comment based on the work item validation results
        let work_item_message = if work_item_result.bypass_info().is_some() {
            "Work item validation bypassed".to_string()
        } else {
            self.communicate_pr_work_item_validity_status(
                repo_owner,
                repo_name,
                &pr,
                &work_item_result,
            )
            .await
        };

        // Handle size labeling and comments if size checking is enabled
        let size_message = if self.config.pr_size_check.enabled {
            if let Some(files) = &pr_files {
                self.communicate_pr_size_status(repo_owner, repo_name, pr_number, files)
                    .await
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Determine labels with enhanced error handling and monitoring
        let labels = self
            .determine_labels(repo_owner, repo_name, &pr)
            .await
            .unwrap_or_else(|e| {
                warn!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pr_number = pr_number,
                    error = %e,
                    "Label determination failed, continuing with empty labels"
                );
                Vec::new()
            });

        // Generate smart label status message for check reporting
        let smart_label_message = if self.config.change_type_labels.is_some() {
            if labels.is_empty() {
                "⚠️ **Smart Label Detection**: No labels applied (detection may have failed or no matching labels found)".to_string()
            } else {
                format!(
                    "✅ **Smart Label Detection**: Applied {} label(s): {}",
                    labels.len(),
                    labels.join(", ")
                )
            }
        } else if !labels.is_empty() {
            format!(
                "📋 **Legacy Labeling**: Applied {} label(s): {}",
                labels.len(),
                labels.join(", ")
            )
        } else {
            String::new()
        };

        // Determine check conclusion - fail if any validation fails or if size is oversized and fail_on_oversized is true
        let should_fail_on_size = if let Some(files) = &pr_files {
            let size_info = crate::size::PrSizeInfo::from_files_with_exclusions(
                files,
                &self.config.pr_size_check.get_effective_thresholds(),
                &self.config.pr_size_check.excluded_file_patterns,
            );
            self.config.pr_size_check.fail_on_oversized && size_info.is_oversized()
        } else {
            false
        };

        let check_conclusion =
            if is_title_valid && is_work_item_referenced && (is_size_valid || !should_fail_on_size)
            {
                "success"
            } else {
                "failure"
            };

        // Enhanced check summary that includes all validation results and bypass information
        let check_summary = if is_title_valid && is_work_item_referenced && is_size_valid {
            if bypasses_used.is_empty() {
                "All PR requirements satisfied.".to_string()
            } else {
                match bypasses_used.len() {
                    1 => "All PR requirements satisfied (1 validation bypassed).".to_string(),
                    n => format!(
                        "All PR requirements satisfied ({} validations bypassed).",
                        n
                    ),
                }
            }
        } else {
            let mut issues = Vec::new();
            if !is_title_valid {
                issues.push("title is invalid");
            }
            if !is_work_item_referenced {
                issues.push("work item reference is missing");
            }
            if !is_size_valid {
                issues.push("PR size exceeds threshold");
            }

            match issues.len() {
                1 => format!("PR {}.", issues[0]),
                2 => format!("PR {} and {}.", issues[0], issues[1]),
                _ => format!("PR {}, {}, and {}.", issues[0], issues[1], issues[2]),
            }
        };

        // Smart text formatting that includes all messages with separators when content exists
        let text = {
            let mut messages = Vec::new();
            if !title_message.is_empty() {
                messages.push(title_message);
            }
            if !work_item_message.is_empty() {
                messages.push(work_item_message);
            }
            if !size_message.is_empty() {
                messages.push(size_message);
            }
            if !smart_label_message.is_empty() {
                messages.push(smart_label_message);
            }
            messages.join("\n\n---\n\n")
        };
        self.provider
            .update_pr_check_status(
                repo_owner,
                repo_name,
                pr_number,
                check_conclusion,
                check_title,
                &check_summary,
                &text,
            )
            .await
            .map_err(|e| {
                error!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    error = e.to_string(),
                    "Failed to add or update GitHub check run"
                );
                MergeWardenError::FailedToUpdatePullRequest(
                    "Failed to add or update GitHub check run".to_string(),
                )
            })?;
        Ok(CheckResult {
            title_valid: is_title_valid,
            work_item_referenced: is_work_item_referenced,
            size_valid: is_size_valid,
            labels,
            bypasses_used,
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
    /// use merge_warden_core::{
    ///     MergeWarden,
    ///     config::{
    ///         BypassRules, CONVENTIONAL_COMMIT_REGEX, CurrentPullRequestValidationConfiguration,
    ///         MISSING_WORK_ITEM_LABEL, TITLE_INVALID_LABEL, WORK_ITEM_REGEX
    ///     }
    /// };
    /// use merge_warden_developer_platforms::PullRequestProvider;
    /// use merge_warden_developer_platforms::errors::Error;
    /// use merge_warden_developer_platforms::models::{Comment, Label, PullRequest};
    ///
    /// #[derive(Debug)]
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
    ///     # async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<Label>, Error> { unimplemented!() }
    ///     # async fn add_labels(&self, _: &str, _: &str, _: u64, _: &[String]) -> Result<(), Error> { unimplemented!() }
    ///     # async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> { unimplemented!() }
    ///     # async fn list_applied_labels(&self, _: &str, _: &str, _: u64) -> Result<Vec<Label>, Error> { unimplemented!() }
    ///     # async fn update_pr_check_status(&self, _: &str, _: &str, _: u64, _: &str, _: &str, _: &str, _: &str) -> Result<(), Error> { unimplemented!() }
    ///     # async fn get_pull_request_files(&self, _: &str, _: &str, _: u64) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> { unimplemented!() }
    /// }
    ///
    /// fn example() {
    ///     let provider = MyProvider;
    ///     let config = CurrentPullRequestValidationConfiguration {
    ///         enforce_title_convention: true,
    ///         title_pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
    ///         invalid_title_label: Some(TITLE_INVALID_LABEL.to_string()),
    ///         enforce_work_item_references: true,
    ///         work_item_reference_pattern: WORK_ITEM_REGEX.to_string(),
    ///         missing_work_item_label: Some(MISSING_WORK_ITEM_LABEL.to_string()),
    ///         bypass_rules: BypassRules::default(),
    ///         pr_size_check: Default::default(),
    ///         change_type_labels: None,
    ///     };
    ///
    ///     let warden = MergeWarden::with_config(provider, config);
    /// }
    ///
    /// fn main() {}
    /// ```
    pub fn with_config(provider: P, config: CurrentPullRequestValidationConfiguration) -> Self {
        Self { provider, config }
    }
}
