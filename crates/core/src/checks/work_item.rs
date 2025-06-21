//! # Work Item Reference Validation
//!
//! This module provides functionality for validating that pull request descriptions
//! contain references to work items or issues.
//!
//! Work item references are important for traceability, allowing teams to connect
//! code changes to the issues or tasks they address. This helps with project management,
//! release notes generation, and understanding the purpose of changes.

use crate::{
    bypass::can_bypass_work_item_validation,
    config::{BypassRule, WORK_ITEM_REGEX},
};
use merge_warden_developer_platforms::models::PullRequest;

#[cfg(test)]
#[path = "work_item_tests.rs"]
mod tests;

/// Checks if the PR body contains a reference to a work item or GitHub issue.
///
/// # Valid Reference Formats
///
/// The function recognizes the following formats:
/// - `Fixes #123`
/// - `Closes #123`
/// - `Resolves #123`
/// - `References #123`
/// - `Relates to #123`
/// - Full GitHub issue URLs (e.g., `https://github.com/owner/repo/issues/123`)
///
/// # Why Work Item References Matter
///
/// Work item references provide:
/// - Traceability between code changes and issues
/// - Automatic issue closing when PRs are merged (for certain keywords)
/// - Better context for reviewers
/// - Improved release notes generation
///
/// # Arguments
///
/// * `pr` - The pull request to check
///
/// # Returns
///
/// A `Result` containing a boolean indicating whether a work item reference was found
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::PullRequest;
/// use merge_warden_core::checks::work_item::check_work_item_reference;
///
/// // PR with a work item reference
/// let pr_with_reference = PullRequest {
///     number: 123,
///     title: "feat: add new feature".to_string(),
///     draft: false,
///     body: Some("This PR adds a new feature.\n\nFixes #42".to_string()),
///     author: None,
/// };
///
/// let has_reference = check_work_item_reference(&pr_with_reference);
/// assert!(has_reference);
///
/// // PR without a work item reference
/// let pr_without_reference = PullRequest {
///     number: 124,
///     title: "feat: another feature".to_string(),
///     draft: false,
///     body: Some("This PR adds another feature.".to_string()),
///     author: None,
/// };
///
/// let has_reference = check_work_item_reference(&pr_without_reference);
/// assert!(!has_reference);
/// ```
pub fn check_work_item_reference(pr: &PullRequest) -> bool {
    // Use the pre-compiled regex from config
    match &pr.body {
        Some(body) => WORK_ITEM_REGEX.is_match(body),
        None => false,
    }
}

/// Checks if the PR body contains a reference to a work item or GitHub issue,
/// with support for bypass rules.
///
/// This function first checks if the PR author can bypass work item validation
/// according to the configured bypass rules. If bypass is allowed, the function
/// returns `true` immediately. Otherwise, it performs the standard work item
/// reference validation.
///
/// # Arguments
///
/// * `pr` - The pull request to check
/// * `bypass_rules` - The bypass rules configuration
///
/// # Returns
///
/// A `Result` containing a boolean indicating whether a work item reference was found
/// or if the validation was bypassed
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequest, User};
/// use merge_warden_core::checks::work_item::check_work_item_reference_with_bypass;
/// use merge_warden_core::config::BypassRule;
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
/// let bypass_rule = BypassRule {
///     enabled: true,
///     users: vec!["bypass-user".to_string()],
/// };
///
/// let result = check_work_item_reference_with_bypass(&pr_with_bypass, &bypass_rule);
/// assert!(result); // Bypassed, so returns true
/// ```
pub fn check_work_item_reference_with_bypass(pr: &PullRequest, bypass_rules: &BypassRule) -> bool {
    // Check if the user can bypass work item validation
    if can_bypass_work_item_validation(pr.author.as_ref(), bypass_rules) {
        return true;
    }

    // If no bypass, perform normal validation
    check_work_item_reference(pr)
}
