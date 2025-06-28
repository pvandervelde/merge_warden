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
use merge_warden_developer_platforms::models::{PullRequest, PullRequestFile};
use regex::Regex;

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;

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
    let regex = match Regex::new(&current_configuration.title_pattern.as_str()) {
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
/// and calculating the total lines modified. It supports file exclusion patterns
/// and can optionally fail the check for oversized PRs.
///
/// # Arguments
///
/// * `pr_files` - List of files changed in the pull request
/// * `bypass_rule` - Bypass rule for size validation (not currently used as size checking doesn't support bypasses)
/// * `config` - Current validation configuration containing size check settings
///
/// # Returns
///
/// A `ValidationResult` indicating the size validation status
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::PullRequestFile;
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
/// let bypass_rule = BypassRule::default();
/// let mut config = CurrentPullRequestValidationConfiguration::default();
/// config.pr_size_check.enabled = true;
/// config.pr_size_check.excluded_file_patterns = vec!["*.md".to_string()];
///
/// let result = check_pr_size(&files, &bypass_rule, &config);
/// // Only src/main.rs counts (15 lines), README.md is excluded
/// assert!(result.is_valid()); // 15 lines is XS, should be valid
/// ```
pub fn check_pr_size(
    pr_files: &[PullRequestFile],
    _bypass_rule: &BypassRule, // Size validation doesn't currently support bypasses
    config: &CurrentPullRequestValidationConfiguration,
) -> ValidationResult {
    // If size checking is disabled, always return valid
    if !config.pr_size_check.enabled {
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
