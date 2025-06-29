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
use merge_warden_developer_platforms::models::{Label, PullRequest};
use merge_warden_developer_platforms::PullRequestProvider;
use regex::Regex;
use tracing::warn;

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

/// Manages size labels for a pull request based on file changes using smart label discovery.
///
/// This function implements the smart label discovery strategy from the spec:
/// 1. Discovers existing size labels in the repository using multiple detection patterns
/// 2. Removes any existing size labels (exclusive labeling)
/// 3. Applies the appropriate size label based on the PR's categorization
/// 4. Falls back to creating new labels if none are found
///
/// # Arguments
///
/// * `provider` - The Git provider implementation
/// * `owner` - The owner of the repository
/// * `repo` - The name of the repository
/// * `pr_number` - The PR number
/// * `size_info` - Information about the PR's size and categorization
///
/// # Returns
///
/// A `Result` containing the size label that was applied, or None if size labeling failed
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
) -> Result<Option<String>, MergeWardenError> {
    // Step 1: Discover existing size labels in the repository
    let discovered_labels = LabelDiscovery::discover_size_labels(provider, owner, repo).await?;

    // Step 2: Get current labels on the PR
    let current_pr_labels = provider
        .list_labels(owner, repo, pr_number)
        .await
        .map_err(|_| {
            MergeWardenError::FailedToUpdatePullRequest(
                "Failed to list current PR labels".to_string(),
            )
        })?;

    // Step 3: Remove any existing size labels (exclusive labeling)
    for existing_label in &current_pr_labels {
        // Check if this label is one of our discovered size labels
        if discovered_labels.all_discovered_labels().contains(&&existing_label.name) {
            provider
                .remove_label(owner, repo, pr_number, &existing_label.name)
                .await
                .map_err(|_| {
                    MergeWardenError::FailedToUpdatePullRequest(
                        "Failed to remove existing size label".to_string(),
                    )
                })?;
        }
    }

    // Step 4: Apply the new size label
    if let Some(label_name) = discovered_labels.get_label_for_category(&size_info.size_category) {
        // Use discovered label
        provider
            .add_labels(owner, repo, pr_number, &[label_name.clone()])
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest("Failed to add size label".to_string())
            })?;

        Ok(Some(label_name.clone()))
    } else {
        // Fallback: create new label with standard format
        let fallback_label = format!("size: {}", size_info.size_category.as_str());
        
        // Log that we're falling back to creating a new label
        // In a real implementation, we might want to check if the repository allows
        // label creation or provide more sophisticated fallback logic
        warn!(
            "No existing size label found for category '{}' in repository {}/{}. Using fallback label: '{}'",
            size_info.size_category.as_str(),
            owner,
            repo,
            fallback_label
        );

        provider
            .add_labels(owner, repo, pr_number, &[fallback_label.clone()])
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest(
                    "Failed to add fallback size label".to_string(),
                )
            })?;

        Ok(Some(fallback_label))
    }
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
/// let comment = generate_oversized_pr_comment(&size_info);
/// assert!(comment.contains("XXL"));
/// assert!(comment.contains("550 lines"));
/// ```
pub fn generate_oversized_pr_comment(size_info: &PrSizeInfo) -> String {
    format!(
        r#"## 📏 Pull Request Size Notice

This PR has been labeled as `{category}` as it contains **{total_lines} lines** of changes across {file_count} files.

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

/// Discovered size labels in the repository using smart discovery
#[derive(Debug, Clone)]
pub struct DiscoveredSizeLabels {
    pub xs: Option<String>,
    pub s: Option<String>,
    pub m: Option<String>,
    pub l: Option<String>,
    pub xl: Option<String>,
    pub xxl: Option<String>,
}

impl DiscoveredSizeLabels {
    /// Create a new empty discovery result
    pub fn new() -> Self {
        Self {
            xs: None,
            s: None,
            m: None,
            l: None,
            xl: None,
            xxl: None,
        }
    }

    /// Get the label name for a specific size category
    pub fn get_label_for_category(&self, category: &PrSizeCategory) -> Option<&String> {
        match category {
            PrSizeCategory::XS => self.xs.as_ref(),
            PrSizeCategory::S => self.s.as_ref(),
            PrSizeCategory::M => self.m.as_ref(),
            PrSizeCategory::L => self.l.as_ref(),
            PrSizeCategory::XL => self.xl.as_ref(),
            PrSizeCategory::XXL => self.xxl.as_ref(),
        }
    }

    /// Get all discovered labels that might be size labels
    pub fn all_discovered_labels(&self) -> Vec<&String> {
        [&self.xs, &self.s, &self.m, &self.l, &self.xl, &self.xxl]
            .iter()
            .filter_map(|opt| opt.as_ref())
            .collect()
    }
}

/// Label discovery system that implements smart label detection
pub struct LabelDiscovery;

impl LabelDiscovery {
    /// Discover existing size labels in the repository using smart detection algorithms
    ///
    /// This implements the label detection algorithm from the spec:
    /// 1. Name-based detection using regex patterns
    /// 2. Description-based detection for metadata
    /// 3. Priority-based selection
    pub async fn discover_size_labels<P: PullRequestProvider>(
        provider: &P,
        owner: &str,
        repo: &str,
    ) -> Result<DiscoveredSizeLabels, MergeWardenError> {
        let all_labels = provider
            .list_repository_labels(owner, repo)
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest(
                    "Failed to fetch repository labels".to_string(),
                )
            })?;

        let mut discovered = DiscoveredSizeLabels::new();

        // Define the size categories we're looking for
        let categories = ["XS", "S", "M", "L", "XL", "XXL"];

        for category in &categories {
            let best_label = Self::find_best_label_for_category(&all_labels, category);
            match category {
                &"XS" => discovered.xs = best_label,
                &"S" => discovered.s = best_label,
                &"M" => discovered.m = best_label,
                &"L" => discovered.l = best_label,
                &"XL" => discovered.xl = best_label,
                &"XXL" => discovered.xxl = best_label,
                _ => {}
            }
        }

        Ok(discovered)
    }

    /// Find the best matching label for a size category using priority-based selection
    fn find_best_label_for_category(labels: &[Label], category: &str) -> Option<String> {
        // Priority 1: Exact size match - size/XS, size/S, etc.
        if let Some(label) = Self::find_exact_size_match(labels, category) {
            return Some(label.name.clone());
        }

        // Priority 2: Size with separator - size-M, size_L, size: M, etc.
        if let Some(label) = Self::find_size_with_separator(labels, category) {
            return Some(label.name.clone());
        }

        // Priority 3: Standalone size - XS, M, XXL, etc.
        if let Some(label) = Self::find_standalone_size(labels, category) {
            return Some(label.name.clone());
        }

        // Priority 4: Description-based - any label with (size: M) in description
        if let Some(label) = Self::find_description_based(labels, category) {
            return Some(label.name.clone());
        }

        None
    }

    /// Find exact size match: size/XS, size/S, etc.
    fn find_exact_size_match<'a>(labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"^size/{}$", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        labels
            .iter()
            .find(|label| regex.is_match(&label.name.to_uppercase()))
    }

    /// Find size with separator: size-M, size_L, size: M, etc.
    fn find_size_with_separator<'a>(labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"^size[_\-:\s]+{}$", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        labels
            .iter()
            .find(|label| regex.is_match(&label.name.to_uppercase()))
    }

    /// Find standalone size: XS, M, XXL, etc.
    fn find_standalone_size<'a>(labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"^{}$", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        labels
            .iter()
            .find(|label| regex.is_match(&label.name.to_uppercase()))
    }

    /// Find description-based: any label with (size: M) in description
    fn find_description_based<'a>(labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"\(size:\s*{}\)", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        labels.iter().find(|label| {
            if let Some(description) = &label.description {
                regex.is_match(&description.to_uppercase())
            } else {
                false
            }
        })
    }
}
