//! # Validation Checks
//!
//! This module contains the validation checks that are performed on pull requests.
//!
//! The checks are organized into submodules:
//! - `title`: Validates that PR titles follow the Conventional Commits format
//! - `work_item`: Validates that PR descriptions reference a work item or issue
//!
//! These checks are used by the `MergeWarden` to determine if a PR is valid
//! and can be merged.

use crate::{
    config::{BypassRule, CurrentPullRequestValidationConfiguration},
    size::PrSizeInfo,
    validation_result::{BypassInfo, BypassRuleType, ValidationResult},
};
use merge_warden_developer_platforms::models::{PullRequest, PullRequestFile, User};
use regex::Regex;

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;

/// A parsed issue reference extracted from a pull request body.
///
/// Carries enough information to fetch the issue from the appropriate repository,
/// which may differ from the repository the PR lives in.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::IssueReference;
///
/// let same = IssueReference::SameRepo { issue_number: 42 };
/// assert_eq!(same.issue_number(), 42);
///
/// let cross = IssueReference::CrossRepo {
///     owner: "acme".to_string(),
///     repo: "widgets".to_string(),
///     issue_number: 7,
/// };
/// assert_eq!(cross.issue_number(), 7);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueReference {
    /// Issue in the same repository as the PR.
    SameRepo {
        /// Issue number.
        issue_number: u64,
    },
    /// Issue in a different repository.
    CrossRepo {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// Issue number.
        issue_number: u64,
    },
}

impl IssueReference {
    /// Returns the issue number regardless of reference kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::checks::IssueReference;
    ///
    /// let r = IssueReference::SameRepo { issue_number: 99 };
    /// assert_eq!(r.issue_number(), 99);
    /// ```
    pub fn issue_number(&self) -> u64 {
        match self {
            Self::SameRepo { issue_number } | Self::CrossRepo { issue_number, .. } => *issue_number,
        }
    }
}

/// Validates that the PR title follows the Conventional Commits format with bypass support.
///
/// This function checks if the PR title follows the Conventional Commits format.
/// If bypass rules are provided and the PR author is allowed to bypass title validation,
/// the function will return a successful result with bypass information.
///
/// # Arguments
///
/// * `pr` - The pull request to validate
/// * `bypass_rule` - The bypass rule for title validation
///
/// # Returns
///
/// A `ValidationResult` indicating whether the title is valid and any bypass information
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequest, User};
/// use merge_warden_core::checks::check_pr_title;
/// use merge_warden_core::config::{BypassRule, CurrentPullRequestValidationConfiguration};
///
/// // Regular validation
/// let pr = PullRequest {
///     number: 123,
///     title: "feat(auth): add GitHub login".to_string(),
///     draft: false,
///     body: Some("This PR adds GitHub login functionality.".to_string()),
///     author: None,
/// };
///
/// let bypass_rule = BypassRule::default(); // Disabled bypass
/// let config = CurrentPullRequestValidationConfiguration::default();
/// let result = check_pr_title(&pr, &bypass_rule, &config);
/// assert!(result.is_valid());
/// assert!(!result.was_bypassed());
///
/// // Bypass validation for authorized user with invalid title
/// let pr_with_bad_title = PullRequest {
///     number: 124,
///     title: "fix urgent bug".to_string(), // Invalid format
///     draft: false,
///     body: Some("Emergency fix".to_string()),
///     author: Some(User {
///         id: 123,
///         login: "emergency-bot".to_string(),
///     }),
/// };
///
/// let bypass_rule = BypassRule::new(true, vec!["emergency-bot".to_string()]);
/// let result = check_pr_title(&pr_with_bad_title, &bypass_rule, &config);
/// assert!(result.is_valid()); // Bypass allows invalid title
/// assert!(result.was_bypassed());
/// ```
pub fn check_pr_title(
    pr: &PullRequest,
    bypass_rule: &BypassRule,
    current_configuration: &CurrentPullRequestValidationConfiguration,
) -> ValidationResult {
    let user = pr.author.as_ref();

    // Check if user can bypass title validation
    if bypass_rule.can_bypass_validation(user) {
        let bypass_info = BypassInfo {
            rule_type: BypassRuleType::TitleConvention,
            user: user.unwrap().login.clone(), // Safe unwrap since can_bypass_validation checks user existence
        };

        return ValidationResult::bypassed(bypass_info);
    }

    // Otherwise, perform normal validation
    let regex = match Regex::new(&current_configuration.title_pattern) {
        Ok(r) => r,
        Err(_) => return ValidationResult::invalid(),
    };

    if regex.is_match(&pr.title) {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid()
    }
}

/// Checks if the PR body contains a reference to a work item or GitHub issue,
/// with support for bypass rules.
///
/// This function first checks if the PR author can bypass work item validation
/// according to the configured bypass rules. If bypass is allowed, the function
/// returns a successful result with bypass information. Otherwise, it performs
/// the standard work item reference validation.
///
/// # Arguments
///
/// * `pr` - The pull request to check
/// * `bypass_rules` - The bypass rules configuration
///
/// # Returns
///
/// A `ValidationResult` indicating whether a work item reference was found
/// or if the validation was bypassed
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequest, User};
/// use merge_warden_core::checks::check_work_item_reference;
/// use merge_warden_core::config::{BypassRule, CurrentPullRequestValidationConfiguration};
///
/// // PR author who can bypass validation
/// let bypass_user = User {
///     id: 123,
///     login: "bypass-user".to_string(),
/// };
///
/// let pr_with_bypass = PullRequest {
///     number: 123,
///     title: "feat: emergency fix".to_string(),
///     draft: false,
///     body: Some("Emergency fix without work item".to_string()),
///     author: Some(bypass_user),
/// };
///
/// let bypass_rule = BypassRule::new(true, vec!["bypass-user".to_string()]);
/// let config = CurrentPullRequestValidationConfiguration::default();
///
/// let result = check_work_item_reference(&pr_with_bypass, &bypass_rule, &config);
/// assert!(result.is_valid()); // Bypassed, so returns true
/// assert!(result.was_bypassed()); // Indicates bypass was used
/// ```
pub fn check_work_item_reference(
    pr: &PullRequest,
    bypass_rules: &BypassRule,
    current_configuration: &CurrentPullRequestValidationConfiguration,
) -> ValidationResult {
    // Check if the user can bypass work item validation
    let user = pr.author.as_ref();
    if bypass_rules.can_bypass_validation(user) {
        let bypass_info = BypassInfo {
            rule_type: BypassRuleType::WorkItemReference,
            user: user.unwrap().login.clone(), // Safe unwrap since can_bypass_validation checks user existence
        };

        return ValidationResult::bypassed(bypass_info);
    }

    // If no bypass, perform normal validation
    match &pr.body {
        Some(body) => {
            let regex = match Regex::new(current_configuration.work_item_reference_pattern.as_str())
            {
                Ok(r) => r,
                Err(_) => return ValidationResult::invalid(),
            };

            if regex.is_match(body) {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid()
            }
        }
        None => ValidationResult::invalid(),
    }
}

/// Validates PR size based on file changes and configuration.
///
/// This function analyzes the size of a pull request by examining the files changed
/// and calculating the total lines modified. It supports file exclusion patterns,
/// can optionally fail the check for oversized PRs, and supports bypass rules for
/// automated tools that may legitimately create large PRs.
///
/// # Arguments
///
/// * `pr_files` - List of files changed in the pull request
/// * `user` - The user who created the pull request (for bypass checking)
/// * `bypass_rule` - Bypass rule for size validation (allows specific users to bypass size checks)
/// * `config` - Current validation configuration containing size check settings
///
/// # Returns
///
/// A `ValidationResult` indicating the size validation status
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequestFile, User};
/// use merge_warden_core::checks::check_pr_size;
/// use merge_warden_core::config::{BypassRule, CurrentPullRequestValidationConfiguration, PrSizeCheckConfig};
///
/// let files = vec![
///     PullRequestFile {
///         filename: "src/main.rs".to_string(),
///         additions: 10,
///         deletions: 5,
///         changes: 15,
///         status: "modified".to_string(),
///     },
///     PullRequestFile {
///         filename: "README.md".to_string(),
///         additions: 2,
///         deletions: 1,
///         changes: 3,
///         status: "modified".to_string(),
///     },
/// ];
///
/// let user = User {
///     login: "developer".to_string(),
///     id: 123,
/// };
/// let bypass_rule = BypassRule::default();
/// let mut config = CurrentPullRequestValidationConfiguration::default();
/// config.pr_size_check.enabled = true;
/// config.pr_size_check.excluded_file_patterns = vec!["*.md".to_string()];
///
/// let result = check_pr_size(&files, Some(&user), &bypass_rule, &config);
/// // Only src/main.rs counts (15 lines), README.md is excluded
/// assert!(result.is_valid()); // 15 lines is XS, should be valid
/// ```
pub fn check_pr_size(
    pr_files: &[PullRequestFile],
    user: Option<&User>,
    bypass_rule: &BypassRule,
    config: &CurrentPullRequestValidationConfiguration,
) -> ValidationResult {
    // If size checking is disabled, always return valid
    if !config.pr_size_check.enabled {
        return ValidationResult::valid();
    }

    // Check if the user can bypass size validation
    if bypass_rule.can_bypass_validation(user) {
        return ValidationResult::valid();
    }

    // Calculate size info with file exclusions
    let size_info = PrSizeInfo::from_files_with_exclusions(
        pr_files,
        &config.pr_size_check.get_effective_thresholds(),
        &config.pr_size_check.excluded_file_patterns,
    );

    // Check if we should fail for oversized PRs
    if config.pr_size_check.fail_on_oversized && size_info.is_oversized() {
        ValidationResult::invalid()
    } else {
        ValidationResult::valid()
    }
}

/// Extracts the first closing-keyword issue reference from a pull request body.
///
/// Scans `body` for `fixes`, `closes`, or `resolves` references in all supported
/// formats. Returns the first match found, or `None` if no closing reference is
/// present. Informational keywords (`references`, `relates to`) are intentionally
/// excluded — they satisfy the work-item link check but are not used for metadata
/// propagation.
///
/// # Arguments
///
/// * `body` - The pull request body text to scan.
///
/// # Returns
///
/// The first closing-keyword issue reference found, or `None`.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::{extract_closing_issue_reference, IssueReference};
///
/// assert_eq!(
///     extract_closing_issue_reference("fixes #42"),
///     Some(IssueReference::SameRepo { issue_number: 42 }),
/// );
///
/// // Informational keywords are not closing references
/// assert_eq!(extract_closing_issue_reference("relates to #99"), None);
/// ```
pub fn extract_closing_issue_reference(body: &str) -> Option<IssueReference> {
    todo!("implement extract_closing_issue_reference")
}
