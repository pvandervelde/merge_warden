//! # Title Validation
//!
//! This module provides functionality for validating pull request titles
//! against the Conventional Commits format.
//!
//! The Conventional Commits specification is a lightweight convention on top of commit
//! messages. It provides an easy set of rules for creating an explicit commit history,
//! which makes it easier to write automated tools on top of.

use crate::{config::CONVENTIONAL_COMMIT_REGEX, models::PullRequest};
use anyhow::Result;

#[cfg(test)]
#[path = "title_tests.rs"]
mod tests;

/// Validates that the PR title follows the Conventional Commits format.
///
/// # Conventional Commits Format
///
/// The format is: `<type>(<scope>): <description>` where:
/// - **type**: The type of change being made (feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert)
/// - **scope**: (optional) The scope of the change, usually the area of the codebase affected
/// - **description**: A short summary of the change
///
/// # Examples of Valid Titles
///
/// - `feat: add new feature`
/// - `fix(auth): correct login issue`
/// - `docs: update README`
/// - `refactor(api): simplify error handling`
/// - `chore: update dependencies`
/// - `feat!: introduce breaking change`
///
/// # Arguments
///
/// * `pr` - The pull request to validate
///
/// # Returns
///
/// A `Result` containing a boolean indicating whether the title is valid
///
/// # Examples
///
/// ```
/// use merge_warden_core::{models::PullRequest, checks::title::check_pr_title};
///
/// let pr = PullRequest {
///     number: 123,
///     title: "feat(auth): add GitHub login".to_string(),
///     body: Some("This PR adds GitHub login functionality.".to_string()),
/// };
///
/// let is_valid = check_pr_title(&pr).unwrap();
/// assert!(is_valid);
/// ```
pub fn check_pr_title(pr: &PullRequest) -> Result<bool> {
    // Use the pre-compiled regex from config
    let is_valid_title = CONVENTIONAL_COMMIT_REGEX.is_match(&pr.title);
    Ok(is_valid_title)
}
