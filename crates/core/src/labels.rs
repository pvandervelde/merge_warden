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
use crate::errors::MergeWardenError;
use crate::size::{PrSizeCategory, PrSizeInfo};
use merge_warden_developer_platforms::models::PullRequest;
use merge_warden_developer_platforms::PullRequestProvider;
use regex::Regex;

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
/// use merge_warden_developer_platforms::PullRequestProvider;
/// use merge_warden_developer_platforms::models::PullRequest;
/// use merge_warden_core::labels::set_pull_request_labels;
/// use anyhow::Result;
///
/// async fn example<P: PullRequestProvider>(provider: &P) -> Result<()> {
///     let pr = PullRequest {
///         number: 123,
///         title: "feat(auth): add GitHub login".to_string(),
///         draft: false,
///         body: Some("This PR adds GitHub login functionality.".to_string()),
///         author: None,
///     };
///
///     let labels = set_pull_request_labels(provider, "owner", "repo", &pr).await?;
///     println!("Applied labels: {:?}", labels);
///
///     Ok(())
/// }
/// ```
pub async fn set_pull_request_labels<P: PullRequestProvider>(
    provider: &P,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
) -> Result<Vec<String>, MergeWardenError> {
    let mut labels = Vec::new();

    // Extract type from PR title using pre-compiled regex
    let regex = match Regex::new(CONVENTIONAL_COMMIT_REGEX) {
        Ok(r) => r,
        Err(_e) => {
            return Err(MergeWardenError::ConfigError(
                "Failed to create a title extraction regex.".to_string(),
            ))
        }
    };

    if let Some(captures) = regex.captures(&pr.title) {
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
        provider
            .add_labels(owner, repo, pr.number, &labels)
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest("Failed to add label".to_string())
            })?;
    }

    Ok(labels)
}

/// Manages size labels for a pull request based on file changes.
///
/// This function calculates the PR size, removes any existing size labels with the
/// configured prefix, and applies the appropriate size label based on the PR's
/// line count.
///
/// # Arguments
///
/// * `provider` - The Git provider implementation
/// * `owner` - The owner of the repository
/// * `repo` - The name of the repository
/// * `pr_number` - The PR number
/// * `size_info` - Information about the PR's size and categorization
/// * `label_prefix` - The prefix for size labels (e.g., "size/")
///
/// # Returns
///
/// A `Result` containing the size label that was applied, or None if size labeling is disabled
///
/// # Examples
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::PullRequestProvider;
/// use merge_warden_core::labels::manage_size_labels;
/// use merge_warden_core::size::{PrSizeInfo, SizeThresholds};
/// use merge_warden_developer_platforms::models::PullRequestFile;
/// use anyhow::Result;
///
/// async fn example<P: PullRequestProvider>(provider: &P) -> Result<()> {
///     let files = vec![
///         PullRequestFile {
///             filename: "src/main.rs".to_string(),
///             additions: 10,
///             deletions: 5,
///             changes: 15,
///             status: "modified".to_string(),
///         },
///     ];
///     let thresholds = SizeThresholds::default();
///     let size_info = PrSizeInfo::from_files_with_exclusions(&files, &thresholds, &[]);
///
///     let label = manage_size_labels(
///         provider,
///         "owner",
///         "repo",
///         123,
///         &size_info,
///         "size/"
///     ).await?;
///
///     println!("Applied size label: {:?}", label);
///     Ok(())
/// }
/// ```
pub async fn manage_size_labels<P: PullRequestProvider>(
    provider: &P,
    owner: &str,
    repo: &str,
    pr_number: u64,
    size_info: &PrSizeInfo,
    label_prefix: &str,
) -> Result<Option<String>, MergeWardenError> {
    // First, remove any existing size labels with this prefix
    let existing_labels = provider
        .list_labels(owner, repo, pr_number)
        .await
        .map_err(|_| {
            MergeWardenError::FailedToUpdatePullRequest(
                "Failed to list existing labels".to_string(),
            )
        })?;

    // Remove existing size labels
    for label in existing_labels {
        if label.name.starts_with(label_prefix) {
            provider
                .remove_label(owner, repo, pr_number, &label.name)
                .await
                .map_err(|_| {
                    MergeWardenError::FailedToUpdatePullRequest(
                        "Failed to remove existing size label".to_string(),
                    )
                })?;
        }
    }

    // Add the new size label
    let size_label = format!("{}{}", label_prefix, size_info.size_category.as_str());
    provider
        .add_labels(owner, repo, pr_number, &[size_label.clone()])
        .await
        .map_err(|_| {
            MergeWardenError::FailedToUpdatePullRequest("Failed to add size label".to_string())
        })?;

    Ok(Some(size_label))
}

/// Generates an educational comment for oversized pull requests.
///
/// This function creates a helpful comment that explains why the PR is considered
/// oversized and provides suggestions for breaking it into smaller, more reviewable
/// pieces.
///
/// # Arguments
///
/// * `size_info` - Information about the PR's size and categorization
/// * `label_prefix` - The prefix used for size labels
///
/// # Returns
///
/// A formatted comment string explaining the size issue and providing guidance
///
/// # Examples
///
/// ```
/// use merge_warden_core::labels::generate_oversized_pr_comment;
/// use merge_warden_core::size::{PrSizeInfo, SizeThresholds};
/// use merge_warden_developer_platforms::models::PullRequestFile;
///
/// let files = vec![
///     PullRequestFile {
///         filename: "src/large_file.rs".to_string(),
///         additions: 300,
///         deletions: 250,
///         changes: 550,
///         status: "modified".to_string(),
///     },
/// ];
/// let thresholds = SizeThresholds::default();
/// let size_info = PrSizeInfo::from_files_with_exclusions(&files, &thresholds, &[]);
///
/// let comment = generate_oversized_pr_comment(&size_info, "size/");
/// assert!(comment.contains("XXL"));
/// assert!(comment.contains("550 lines"));
/// ```
pub fn generate_oversized_pr_comment(size_info: &PrSizeInfo, label_prefix: &str) -> String {
    format!(
        r#"## 📏 Pull Request Size Notice

This PR has been labeled as `{prefix}{category}` as it contains **{total_lines} lines** of changes across {file_count} files.

### Why does PR size matter?

Research shows that smaller PRs are:
- ✅ **Reviewed more thoroughly** - reviewers can focus better on smaller changes
- ✅ **Catch more bugs** - defect detection rates decrease significantly for large PRs
- ✅ **Merged faster** - less time in review cycles
- ✅ **Easier to understand** - simpler to reason about the changes

### 💡 Consider breaking this PR into smaller pieces

Large PRs can be challenging to review effectively. Consider:

1. **Separate concerns** - Split unrelated changes into different PRs
2. **Incremental changes** - Break features into smaller, logical steps
3. **Preparatory PRs** - Create setup/refactoring PRs before the main feature
4. **Documentation separately** - Move documentation updates to separate PRs

### Size Breakdown
- **Total lines changed**: {total_lines}
- **Files modified**: {file_count}
- **Category**: {category} ({category_description})

*This is an automated message to help improve code review quality. If you believe this PR cannot be reasonably split, please add a comment explaining why.*"#,
        prefix = label_prefix,
        category = size_info.size_category.as_str(),
        total_lines = size_info.total_lines_changed,
        file_count = size_info.included_files.len(),
        category_description = get_category_description(&size_info.size_category)
    )
}

/// Get a human-readable description for a size category.
fn get_category_description(category: &PrSizeCategory) -> &'static str {
    match category {
        PrSizeCategory::XS => "Extra Small - Very easy to review",
        PrSizeCategory::S => "Small - Easy to review thoroughly",
        PrSizeCategory::M => "Medium - Manageable review scope",
        PrSizeCategory::L => "Large - Approaching review complexity limits",
        PrSizeCategory::XL => "Extra Large - Difficult to review effectively",
        PrSizeCategory::XXL => "Extra Extra Large - Should be split for better reviewability",
    }
}
