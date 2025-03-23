//! # Labels
//!
//! This module provides functionality for automatically determining and applying
//! labels to pull requests based on their content.
//!
//! The module analyzes the PR title and body to determine appropriate labels, such as:
//! - Type-based labels (feature, bug, documentation, etc.)
//! - Scope-based labels
//! - Breaking change indicators
//! - Special labels based on PR description keywords

use crate::config::CONVENTIONAL_COMMIT_REGEX;
use anyhow::Result;
use merge_warden_developer_platforms::models::PullRequest;
use merge_warden_developer_platforms::GitProvider;

#[cfg(test)]
#[path = "labels_tests.rs"]
mod tests;

/// Determines and applies labels to a pull request based on its content.
///
/// This function analyzes the PR title and body to determine appropriate labels
/// and applies them to the PR using the provided Git provider.
///
/// # Label Categories
///
/// - **Type-based labels**: Derived from the PR type (feat → feature, fix → bug, etc.)
/// - **Scope-based labels**: Derived from the PR scope if present (e.g., "scope:auth")
/// - **Breaking change**: Applied if the PR title contains "!:" or "breaking change"
/// - **Special labels**: Applied based on keywords in the PR description:
///   - "security" or "vulnerability" → security label
///   - "hotfix" → hotfix label
///   - "technical debt" or "tech debt" → tech-debt label
///
/// # Arguments
///
/// * `provider` - The Git provider implementation
/// * `owner` - The owner of the repository
/// * `repo` - The name of the repository
/// * `pr` - The pull request to analyze
///
/// # Returns
///
/// A `Result` containing a vector of labels that were added to the PR
///
/// # Examples
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::GitProvider;
/// use merge_warden_developer_platforms::models::PullRequest;
/// use merge_warden_core::labels::set_pull_request_labels;
/// use anyhow::Result;
///
/// async fn example<P: GitProvider>(provider: &P) -> Result<()> {
///     let pr = PullRequest {
///         number: 123,
///         title: "feat(auth): add GitHub login".to_string(),
///         body: Some("This PR adds GitHub login functionality.".to_string()),
///     };
///
///     let labels = set_pull_request_labels(provider, "owner", "repo", &pr).await?;
///     println!("Applied labels: {:?}", labels);
///
///     Ok(())
/// }
/// ```
pub async fn set_pull_request_labels<P: GitProvider>(
    provider: &P,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
) -> Result<Vec<String>> {
    let mut labels = Vec::new();

    // Extract type from PR title using pre-compiled regex
    if let Some(captures) = CONVENTIONAL_COMMIT_REGEX.captures(&pr.title) {
        let pr_type = captures.get(1).unwrap().as_str();

        // Add type-based label
        match pr_type {
            "feat" => labels.push("feature".to_string()),
            "fix" => labels.push("bug".to_string()),
            "docs" => labels.push("documentation".to_string()),
            "style" => labels.push("style".to_string()),
            "refactor" => labels.push("refactor".to_string()),
            "perf" => labels.push("performance".to_string()),
            "test" => labels.push("testing".to_string()),
            "build" => labels.push("build".to_string()),
            "ci" => labels.push("ci".to_string()),
            "chore" => labels.push("chore".to_string()),
            "revert" => labels.push("revert".to_string()),
            _ => {}
        }
    }

    // Check if PR is a breaking change
    let breaking_change_label = "breaking-change".to_string();
    if pr.title.contains("!:") || pr.title.to_lowercase().contains("breaking change") {
        labels.push(breaking_change_label.clone());
    }

    // Check PR description for keywords
    if let Some(body) = &pr.body {
        let body_lower = body.to_lowercase();

        if body_lower.contains("breaking change") && !labels.contains(&breaking_change_label) {
            labels.push(breaking_change_label.clone());
        }

        if body_lower.contains("security") || body_lower.contains("vulnerability") {
            labels.push("security".to_string());
        }

        if body_lower.contains("hotfix") {
            labels.push("hotfix".to_string());
        }

        if body_lower.contains("technical debt") || body_lower.contains("tech debt") {
            labels.push("tech-debt".to_string());
        }
    }

    // Add the labels to the PR
    if !labels.is_empty() {
        provider.add_labels(owner, repo, pr.number, &labels).await?;
    }

    Ok(labels)
}
