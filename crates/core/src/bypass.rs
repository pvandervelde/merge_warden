//! # Bypass Logic Module
//!
//! This module provides functionality to check if users can bypass specific validation rules
//! based on configuration settings.

use merge_warden_developer_platforms::models::{PullRequest, User};

use crate::config::{BypassRule, BypassRules};

/// Checks if a user can bypass title convention validation
///
/// # Arguments
///
/// * `user` - The user to check for bypass permissions (None if no author info available)
/// * `bypass_rule` - The bypass rule configuration for title validation
///
/// # Returns
///
/// `true` if the user can bypass title validation, `false` otherwise
///
/// # Examples
///
/// ```
/// use merge_warden_core::bypass::can_bypass_title_validation;
/// use merge_warden_core::config::BypassRule;
/// use merge_warden_developer_platforms::models::User;
///
/// let user = Some(User {
///     id: 123,
///     login: "release-bot".to_string(),
/// });
///
/// let bypass_rule = BypassRule {
///     enabled: true,
///     users: vec!["release-bot".to_string()],
/// };
///
/// assert!(can_bypass_title_validation(user.as_ref(), &bypass_rule));
/// ```
pub fn can_bypass_title_validation(user: Option<&User>, bypass_rule: &BypassRule) -> bool {
    can_bypass_validation(user, bypass_rule)
}

/// Checks if a user can bypass work item validation
///
/// # Arguments
///
/// * `user` - The user to check for bypass permissions (None if no author info available)
/// * `bypass_rule` - The bypass rule configuration for work item validation
///
/// # Returns
///
/// `true` if the user can bypass work item validation, `false` otherwise
///
/// # Examples
///
/// ```
/// use merge_warden_core::bypass::can_bypass_work_item_validation;
/// use merge_warden_core::config::BypassRule;
/// use merge_warden_developer_platforms::models::User;
///
/// let user = Some(User {
///     id: 456,
///     login: "hotfix-team".to_string(),
/// });
///
/// let bypass_rule = BypassRule {
///     enabled: true,
///     users: vec!["hotfix-team".to_string(), "admin".to_string()],
/// };
///
/// assert!(can_bypass_work_item_validation(user.as_ref(), &bypass_rule));
/// ```
pub fn can_bypass_work_item_validation(user: Option<&User>, bypass_rule: &BypassRule) -> bool {
    can_bypass_validation(user, bypass_rule)
}

/// Checks if a pull request author can bypass specific validation rules
///
/// # Arguments
///
/// * `pr` - The pull request to check
/// * `bypass_rules` - The complete bypass rules configuration
///
/// # Returns
///
/// A tuple containing:
/// - `bool`: whether title validation can be bypassed
/// - `bool`: whether work item validation can be bypassed
///
/// # Examples
///
/// ```
/// use merge_warden_core::bypass::can_bypass_validations;
/// use merge_warden_core::config::{BypassRule, BypassRules};
/// use merge_warden_developer_platforms::models::{PullRequest, User};
///
/// let pr = PullRequest {
///     number: 123,
///     title: "hotfix: urgent security fix".to_string(),
///     draft: false,
///     body: Some("Emergency fix for security vulnerability".to_string()),
///     author: Some(User {
///         id: 789,
///         login: "security-team".to_string(),
///     }),
/// };
///
/// let bypass_rules = BypassRules {
///     title_convention: BypassRule {
///         enabled: true,
///         users: vec!["security-team".to_string()],
///     },
///     work_items: BypassRule {
///         enabled: true,
///         users: vec!["security-team".to_string()],
///     },
/// };
///
/// let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);
/// assert!(can_bypass_title);
/// assert!(can_bypass_work_item);
/// ```
pub fn can_bypass_validations(pr: &PullRequest, bypass_rules: &BypassRules) -> (bool, bool) {
    let user = pr.author.as_ref();

    let can_bypass_title = can_bypass_title_validation(user, &bypass_rules.title_convention);
    let can_bypass_work_item = can_bypass_work_item_validation(user, &bypass_rules.work_items);

    (can_bypass_title, can_bypass_work_item)
}

/// Core bypass validation logic
///
/// # Arguments
///
/// * `user` - The user to check for bypass permissions (None if no author info available)
/// * `bypass_rule` - The bypass rule configuration
///
/// # Returns
///
/// `true` if the user can bypass the validation, `false` otherwise
///
/// # Logic
///
/// Returns `true` if:
/// 1. The bypass rule is enabled AND
/// 2. A user is provided AND
/// 3. The user's login is in the bypass users list
///
/// Returns `false` if:
/// - The bypass rule is disabled
/// - No user information is available
/// - The user's login is not in the bypass users list
fn can_bypass_validation(user: Option<&User>, bypass_rule: &BypassRule) -> bool {
    // If bypass is disabled, no one can bypass
    if !bypass_rule.enabled {
        return false;
    }

    // If no user information is available, cannot bypass
    let user = match user {
        Some(u) => u,
        None => return false,
    };

    // Check if user is in the bypass list
    bypass_rule
        .users
        .iter()
        .any(|bypass_user| bypass_user == &user.login)
}

#[cfg(test)]
#[path = "bypass_tests.rs"]
mod tests;
