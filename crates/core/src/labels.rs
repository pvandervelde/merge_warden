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

use crate::config::{
    ChangeTypeLabelConfig, CurrentPullRequestValidationConfiguration, CONVENTIONAL_COMMIT_REGEX,
};
use crate::errors::MergeWardenError;
use crate::size::{PrSizeCategory, PrSizeInfo};
use merge_warden_developer_platforms::models::{Label, PullRequest};
use merge_warden_developer_platforms::PullRequestProvider;
use regex::Regex;
use tracing::{debug, info, warn};

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
    set_pull_request_labels_with_config(provider, owner, repo, pr, None).await
}

/// Sets pull request labels with smart detection configuration.
///
/// This is the main implementation that supports both legacy behavior (when config is None)
/// and smart label detection (when config is provided with change_type_labels).
///
/// # Arguments
///
/// * `provider` - The Git provider implementation
/// * `owner` - The owner of the repository
/// * `repo` - The name of the repository
/// * `pr` - The pull request to analyze
/// * `config` - Optional configuration with smart label detection settings
///
/// # Returns
///
/// A `Result` containing a vector of labels that were applied to the PR
pub async fn set_pull_request_labels_with_config<P: PullRequestProvider>(
    provider: &P,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
    config: Option<&CurrentPullRequestValidationConfiguration>,
) -> Result<Vec<String>, MergeWardenError> {
    // This is the implementation we created earlier - delegate to the main function
    // but include the logic in a new internal function to avoid circular calls
    let mut labels = Vec::new();
    let mut smart_detection_applied = false;

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

        // Use smart label detection if configured, otherwise fall back to hardcoded labels
        if let Some(config) = config {
            if let Some(ref change_type_config) = config.change_type_labels {
                if change_type_config.enabled {
                    let detection_start = std::time::Instant::now();

                    info!(
                        repository_owner = owner,
                        repository = repo,
                        pr_number = pr.number,
                        commit_type = pr_type,
                        detection_enabled = change_type_config.enabled,
                        "Starting smart label detection for change type"
                    );

                    // Use smart label detection with LabelManager
                    let label_manager = LabelManager::new(Some(change_type_config.clone()));

                    match label_manager
                        .apply_change_type_label(provider, owner, repo, pr.number, pr_type)
                        .await
                    {
                        Ok(result) => {
                            let detection_duration = detection_start.elapsed();
                            labels.extend(result.all_applied_labels());
                            smart_detection_applied = true;

                            info!(
                                repository_owner = owner,
                                repository = repo,
                                pr_number = pr.number,
                                commit_type = pr_type,
                                applied_labels = ?result.all_applied_labels(),
                                labels_count = result.all_applied_labels().len(),
                                detection_duration_ms = detection_duration.as_millis(),
                                detection_method = "smart_detection",
                                used_fallback = result.used_fallback_creation(),
                                "Successfully applied smart change type labels"
                            );

                            // Audit log for each applied label
                            for label in result.all_applied_labels() {
                                debug!(
                                    repository_owner = owner,
                                    repository = repo,
                                    pr_number = pr.number,
                                    label_name = label,
                                    commit_type = pr_type,
                                    detection_method = "smart_detection",
                                    source = if result.used_fallback_creation() {
                                        "fallback_creation"
                                    } else {
                                        "repository_detection"
                                    },
                                    "Applied smart label to pull request"
                                );
                            }
                        }
                        Err(e) => {
                            let detection_duration = detection_start.elapsed();

                            warn!(
                                repository_owner = owner,
                                repository = repo,
                                pr_number = pr.number,
                                commit_type = pr_type,
                                error = %e,
                                detection_duration_ms = detection_duration.as_millis(),
                                "Smart label detection failed, falling back to hardcoded labels"
                            );

                            // Fall back to hardcoded labels and log the decision
                            add_hardcoded_type_label(&mut labels, pr_type);

                            info!(
                                repository_owner = owner,
                                repository = repo,
                                pr_number = pr.number,
                                commit_type = pr_type,
                                applied_labels = ?labels,
                                detection_method = "hardcoded_fallback",
                                fallback_reason = "smart_detection_failure",
                                "Applied hardcoded fallback labels after smart detection failure"
                            );
                        }
                    }
                } else {
                    // Smart label configuration exists but is disabled, use hardcoded labels
                    add_hardcoded_type_label(&mut labels, pr_type);

                    debug!(
                        repository_owner = owner,
                        repository = repo,
                        pr_number = pr.number,
                        commit_type = pr_type,
                        applied_labels = ?labels,
                        detection_method = "hardcoded_disabled",
                        "Applied hardcoded labels (smart detection disabled)"
                    );
                }
            } else {
                // No smart label configuration, use hardcoded labels
                add_hardcoded_type_label(&mut labels, pr_type);

                debug!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr.number,
                    commit_type = pr_type,
                    applied_labels = ?labels,
                    detection_method = "hardcoded_no_config",
                    "Applied hardcoded labels (no smart detection configured)"
                );
            }
        } else {
            // No configuration provided, use hardcoded labels
            add_hardcoded_type_label(&mut labels, pr_type);

            debug!(
                repository_owner = owner,
                repository = repo,
                pr_number = pr.number,
                commit_type = pr_type,
                applied_labels = ?labels,
                detection_method = "hardcoded_no_config",
                "Applied hardcoded labels (no configuration provided)"
            );
        }
    }

    // Collect additional labels that need to be applied
    let mut additional_labels = Vec::new();

    // Check if PR is a breaking change
    let breaking_change_label = "breaking-change".to_string();
    if pr.title.contains("!:") || pr.title.to_lowercase().contains("breaking change") {
        additional_labels.push(breaking_change_label.clone());
        labels.push(breaking_change_label.clone());
    }

    // Check PR description for keywords
    if let Some(body) = &pr.body {
        let body_lower = body.to_lowercase();

        if body_lower.contains("breaking change") && !labels.contains(&breaking_change_label) {
            additional_labels.push(breaking_change_label.clone());
            labels.push(breaking_change_label.clone());
        }

        if body_lower.contains("security") || body_lower.contains("vulnerability") {
            additional_labels.push("security".to_string());
            labels.push("security".to_string());
        }

        if body_lower.contains("hotfix") {
            additional_labels.push("hotfix".to_string());
            labels.push("hotfix".to_string());
        }

        if body_lower.contains("technical debt") || body_lower.contains("tech debt") {
            additional_labels.push("tech-debt".to_string());
            labels.push("tech-debt".to_string());
        }
    }

    // Apply labels to the PR
    if smart_detection_applied {
        // Smart detection already applied its labels, only apply additional labels
        if !additional_labels.is_empty() {
            if let Err(e) = provider
                .add_labels(owner, repo, pr.number, &additional_labels)
                .await
            {
                warn!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr.number,
                    labels = ?additional_labels,
                    error = %e,
                    "Failed to add additional labels to pull request, continuing with validation"
                );
                // Don't propagate the error, just log it and continue
            }
        }
    } else {
        // Apply all labels (hardcoded + additional)
        if !labels.is_empty() {
            if let Err(e) = provider.add_labels(owner, repo, pr.number, &labels).await {
                warn!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr.number,
                    labels = ?labels,
                    error = %e,
                    "Failed to add labels to pull request, continuing with validation"
                );
                // Don't propagate the error, just log it and continue
                // Return empty labels vector since none were successfully applied
                return Ok(vec![]);
            }
        }
    }

    Ok(labels)
}

/// Add hardcoded type-based label mapping (legacy behavior)
fn add_hardcoded_type_label(labels: &mut Vec<String>, pr_type: &str) {
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
    info!(
        "Starting size label management for PR {}/{}/{}. Size category: {}, Total changes: {}",
        owner,
        repo,
        pr_number,
        size_info.size_category.as_str(),
        size_info.total_lines_changed
    );

    // Step 1: Discover existing size labels in the repository
    debug!(
        "Step 1: Discovering existing size labels in repository {}/{}",
        owner, repo
    );
    let detector = LabelDetector::new_for_size_labels();
    let discovered_labels = detector.discover_size_labels(provider, owner, repo).await?;
    info!(
        "Label discovery completed. Found {} discovered labels",
        discovered_labels.count_discovered()
    );

    // Step 2: Get current labels on the PR
    debug!(
        "Step 2: Getting current labels on PR {}/{}/{}",
        owner, repo, pr_number
    );
    let current_pr_labels = provider
        .list_applied_labels(owner, repo, pr_number)
        .await
        .map_err(|_| {
            MergeWardenError::FailedToUpdatePullRequest(
                "Failed to list current PR labels".to_string(),
            )
        })?;

    debug!(
        "Found {} current labels on PR: {:?}",
        current_pr_labels.len(),
        current_pr_labels
            .iter()
            .map(|l| &l.name)
            .collect::<Vec<_>>()
    );

    // Step 3: Remove any existing size labels (exclusive labeling)
    debug!("Step 3: Removing any existing size labels from PR");
    let mut removed_labels = Vec::new();
    for existing_label in &current_pr_labels {
        // Check if this label is one of our discovered size labels
        if discovered_labels
            .all_discovered_labels()
            .contains(&&existing_label.name)
        {
            debug!("Removing existing size label: {}", existing_label.name);
            provider
                .remove_label(owner, repo, pr_number, &existing_label.name)
                .await
                .map_err(|_| {
                    MergeWardenError::FailedToUpdatePullRequest(
                        "Failed to remove existing size label".to_string(),
                    )
                })?;
            removed_labels.push(&existing_label.name);
        }
    }
    if !removed_labels.is_empty() {
        info!(
            "Removed {} existing size labels: {:?}",
            removed_labels.len(),
            removed_labels
        );
    } else {
        debug!("No existing size labels found to remove");
    }

    // Step 4: Apply the new size label
    debug!(
        "Step 4: Applying new size label for category: {}",
        size_info.size_category.as_str()
    );
    if let Some(label_name) = discovered_labels.get_label_for_category(&size_info.size_category) {
        // Use discovered label
        info!(
            "Using discovered label '{}' for size category '{}'",
            label_name,
            size_info.size_category.as_str()
        );
        provider
            .add_labels(owner, repo, pr_number, &[label_name.clone()])
            .await
            .map_err(|e| {
                warn!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr_number,
                    label_name = label_name,
                    error = %e,
                    "Failed to add size label to pull request"
                );
                MergeWardenError::FailedToUpdatePullRequest("Failed to add size label".to_string())
            })?;

        info!("Successfully applied size label: {}", label_name);
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
            .map_err(|e| {
                warn!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr_number,
                    fallback_label = fallback_label,
                    error = %e,
                    "Failed to add fallback size label to pull request"
                );
                MergeWardenError::FailedToUpdatePullRequest(
                    "Failed to add fallback size label".to_string(),
                )
            })?;

        info!(
            "Successfully applied fallback size label: {}",
            fallback_label
        );
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
    /// Label name for extra small PRs (typically < 10 lines changed)
    pub xs: Option<String>,
    /// Label name for small PRs (typically 10-100 lines changed)
    pub s: Option<String>,
    /// Label name for medium PRs (typically 100-300 lines changed)
    pub m: Option<String>,
    /// Label name for large PRs (typically 300-500 lines changed)
    pub l: Option<String>,
    /// Label name for extra large PRs (typically 500-800 lines changed)
    pub xl: Option<String>,
    /// Label name for extra extra large PRs (typically > 800 lines changed)
    pub xxl: Option<String>,
}

impl Default for DiscoveredSizeLabels {
    fn default() -> Self {
        Self::new()
    }
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

    /// Count how many size labels were discovered
    pub fn count_discovered(&self) -> usize {
        self.all_discovered_labels().len()
    }
}

/// Unified label detector for both size and change type labels
///
/// This struct provides intelligent label detection using repository-specific patterns
/// and supports both size labels (XS, S, M, L, XL, XXL) and change type labels
/// (feat, fix, docs, etc.) with configurable search strategies.
pub struct LabelDetector {
    /// Configuration for change type label detection
    change_type_config: Option<ChangeTypeLabelConfig>,
}

impl LabelDetector {
    /// Create a new label detector for size labels only
    pub fn new_for_size_labels() -> Self {
        Self {
            change_type_config: None,
        }
    }

    /// Create a new label detector with change type configuration
    pub fn new_for_change_type_labels(config: ChangeTypeLabelConfig) -> Self {
        Self {
            change_type_config: Some(config),
        }
    }

    /// Create a new label detector that handles both size and change type labels
    pub fn new(config: Option<ChangeTypeLabelConfig>) -> Self {
        Self {
            change_type_config: config,
        }
    }
    /// Discover existing size labels in the repository using smart detection algorithms
    ///
    /// This implements the label detection algorithm from the spec:
    /// 1. Name-based detection using regex patterns
    /// 2. Description-based detection for metadata
    /// 3. Priority-based selection
    pub async fn discover_size_labels<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
    ) -> Result<DiscoveredSizeLabels, MergeWardenError> {
        info!(
            repository_owner = owner,
            repository = repo,
            "Starting smart label discovery for size labels"
        );

        let all_labels = provider
            .list_available_labels(owner, repo)
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest(
                    "Failed to fetch repository labels".to_string(),
                )
            })?;

        info!(
            repository_owner = owner,
            repository = repo,
            total_labels = all_labels.len(),
            "Retrieved repository labels for analysis"
        );

        // Log all labels for debugging
        for label in &all_labels {
            debug!(
                repository_owner = owner,
                repository = repo,
                label_name = %label.name,
                label_description = ?label.description,
                "Processing repository label"
            );
        }

        let mut discovered = DiscoveredSizeLabels::new();

        // Define the size categories we're looking for
        let categories = ["XS", "S", "M", "L", "XL", "XXL"];

        for category in &categories {
            debug!(
                repository_owner = owner,
                repository = repo,
                category = category,
                "Searching for size labels matching category"
            );

            let best_label =
                self.find_best_label_for_size_category(&all_labels, category, owner, repo);

            if let Some(ref label_name) = best_label {
                info!(
                    repository_owner = owner,
                    repository = repo,
                    category = category,
                    discovered_label = %label_name,
                    "Found size label for category"
                );
            } else {
                warn!(
                    repository_owner = owner,
                    repository = repo,
                    category = category,
                    "No size label found for category"
                );
            }

            match *category {
                "XS" => discovered.xs = best_label,
                "S" => discovered.s = best_label,
                "M" => discovered.m = best_label,
                "L" => discovered.l = best_label,
                "XL" => discovered.xl = best_label,
                "XXL" => discovered.xxl = best_label,
                _ => {}
            }
        }

        let total_discovered = discovered.all_discovered_labels().len();
        info!(
            repository_owner = owner,
            repository = repo,
            total_discovered_labels = total_discovered,
            discovered_xs = ?discovered.xs,
            discovered_s = ?discovered.s,
            discovered_m = ?discovered.m,
            discovered_l = ?discovered.l,
            discovered_xl = ?discovered.xl,
            discovered_xxl = ?discovered.xxl,
            "Completed smart label discovery"
        );

        Ok(discovered)
    }

    /// Find the best matching label for a size category using priority-based selection
    fn find_best_label_for_size_category(
        &self,
        labels: &[Label],
        category: &str,
        owner: &str,
        repo: &str,
    ) -> Option<String> {
        debug!(
            repository_owner = owner,
            repository = repo,
            category = category,
            "Starting priority-based label search"
        );

        // Priority 1: Exact size match - size/XS, size/S, etc.
        if let Some(label) = self.find_exact_size_match(labels, category) {
            info!(
                repository_owner = owner,
                repository = repo,
                category = category,
                found_label = %label.name,
                detection_method = "exact_size_match",
                pattern = format!("^size/{}$", category),
                "Found size label using exact match"
            );
            return Some(label.name.clone());
        }

        // Priority 2: Size with separator - size-M, size_L, size: M, etc.
        if let Some(label) = self.find_size_with_separator(labels, category) {
            info!(
                repository_owner = owner,
                repository = repo,
                category = category,
                found_label = %label.name,
                detection_method = "size_with_separator",
                pattern = format!("^size[_\\-:\\s]+{}$", category),
                "Found size label using separator match"
            );
            return Some(label.name.clone());
        }

        // Priority 3: Standalone size - XS, M, XXL, etc.
        if let Some(label) = self.find_standalone_size(labels, category) {
            info!(
                repository_owner = owner,
                repository = repo,
                category = category,
                found_label = %label.name,
                detection_method = "standalone_size",
                pattern = format!("^{}$", category),
                "Found size label using standalone match"
            );
            return Some(label.name.clone());
        }

        // Priority 4: Description-based - any label with (size: M) in description
        if let Some(label) = self.find_description_based(labels, category) {
            info!(
                repository_owner = owner,
                repository = repo,
                category = category,
                found_label = %label.name,
                detection_method = "description_based",
                pattern = format!("\\(size:\\s*{}\\)", category),
                label_description = ?label.description,
                "Found size label using description match"
            );
            return Some(label.name.clone());
        }

        debug!(
            repository_owner = owner,
            repository = repo,
            category = category,
            total_labels_checked = labels.len(),
            "No matching size label found for category after checking all patterns"
        );

        None
    }

    /// Find exact size match: size/XS, size/S, etc.
    fn find_exact_size_match<'a>(&self, labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"^size/{}$", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        debug!(
            "Checking exact size match pattern: '{}' against {} labels",
            pattern,
            labels.len()
        );

        for label in labels {
            let matches = regex.is_match(&label.name);
            debug!(
                "Exact size match check: label '{}' against pattern '{}' = {}",
                label.name, pattern, matches
            );
            if matches {
                debug!("Found exact size match: {}", label.name);
                return Some(label);
            }
        }

        debug!("No exact size match found for pattern: {}", pattern);
        None
    }

    /// Find size with separator: size-M, size_L, size: M, etc.
    fn find_size_with_separator<'a>(
        &self,
        labels: &'a [Label],
        category: &str,
    ) -> Option<&'a Label> {
        let pattern = format!(r"^size[_\-:\s]+{}$", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        debug!(
            "Checking size with separator pattern: '{}' against {} labels",
            pattern,
            labels.len()
        );

        for label in labels {
            let matches = regex.is_match(&label.name);
            debug!(
                "Size with separator check: label '{}' against pattern '{}' = {}",
                label.name, pattern, matches
            );
            if matches {
                debug!("Found size with separator match: {}", label.name);
                return Some(label);
            }
        }

        debug!(
            "No size with separator match found for pattern: {}",
            pattern
        );
        None
    }

    /// Find standalone size: XS, M, XXL, etc.
    fn find_standalone_size<'a>(&self, labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"^{}$", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        debug!(
            "Checking standalone size pattern: '{}' against {} labels",
            pattern,
            labels.len()
        );

        for label in labels {
            let matches = regex.is_match(&label.name.to_uppercase());
            debug!(
                "Standalone size check: label '{}' against pattern '{}' = {}",
                label.name, pattern, matches
            );
            if matches {
                debug!("Found standalone size match: {}", label.name);
                return Some(label);
            }
        }

        debug!("No standalone size match found for pattern: {}", pattern);
        None
    }

    /// Find description-based: any label with (size: M) in description
    fn find_description_based<'a>(&self, labels: &'a [Label], category: &str) -> Option<&'a Label> {
        let pattern = format!(r"(?i)\(size:\s*{}\)", regex::escape(category));
        let regex = Regex::new(&pattern).ok()?;

        debug!(
            "Checking description-based pattern: '{}' against {} labels",
            pattern,
            labels.len()
        );

        for label in labels {
            if let Some(description) = &label.description {
                let matches = regex.is_match(description);
                debug!(
                    "Description-based check: label '{}' (description: '{}') against pattern '{}' = {}",
                    label.name, description, pattern, matches
                );
                if matches {
                    debug!(
                        "Found description-based match: {} (description: {})",
                        label.name, description
                    );
                    return Some(label);
                }
            } else {
                debug!(
                    "Description-based check: label '{}' has no description, skipping",
                    label.name
                );
            }
        }

        debug!("No description-based match found for pattern: {}", pattern);
        None
    }

    /// Detect the best matching label for a conventional commit type
    ///
    /// This implements the three-tier search strategy:
    /// 1. Exact match detection - looks for exact matches of mapped label names
    /// 2. Prefix match detection - looks for labels with type prefixes (e.g., "feat:", "fix:")
    /// 3. Description match detection - looks for commit types in label descriptions
    ///
    /// # Arguments
    ///
    /// * `provider` - The Git provider implementation
    /// * `owner` - The owner of the repository
    /// * `repo` - The name of the repository
    /// * `commit_type` - The conventional commit type to search for
    ///
    /// # Returns
    ///
    /// A `Result` containing the detection result
    pub async fn detect_change_type_label<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
        commit_type: &str,
    ) -> Result<DiscoveredChangeTypeLabels, MergeWardenError> {
        let config = self.change_type_config.as_ref().ok_or_else(|| {
            MergeWardenError::ConfigError(
                "Change type configuration not provided to LabelDetector".to_string(),
            )
        })?;

        info!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "Starting change type label detection"
        );

        // Get repository labels
        let all_labels = provider
            .list_available_labels(owner, repo)
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest(
                    "Failed to fetch repository labels".to_string(),
                )
            })?;

        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            total_labels = all_labels.len(),
            "Retrieved repository labels for change type detection"
        );

        // Get mapped label names for this commit type
        let mapped_labels = self.get_mapped_label_names(commit_type, config);

        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            mapped_labels = ?mapped_labels,
            "Retrieved mapped label names for commit type"
        );

        // Tier 1: Exact match detection
        if config.detection_strategy.exact_match {
            if let Some(label) =
                self.find_exact_match(&all_labels, &mapped_labels, owner, repo, commit_type)
            {
                return Ok(DiscoveredChangeTypeLabels {
                    label_name: Some(label),
                    commit_type: commit_type.to_string(),
                    should_create_fallback: false,
                });
            }
        }

        // Tier 2: Prefix match detection
        if config.detection_strategy.prefix_match {
            if let Some(label) = self.find_prefix_match(&all_labels, commit_type, owner, repo) {
                return Ok(DiscoveredChangeTypeLabels {
                    label_name: Some(label),
                    commit_type: commit_type.to_string(),
                    should_create_fallback: false,
                });
            }
        }

        // Tier 3: Description match detection
        if config.detection_strategy.description_match {
            if let Some(label) = self.find_description_match(&all_labels, commit_type, owner, repo)
            {
                return Ok(DiscoveredChangeTypeLabels {
                    label_name: Some(label),
                    commit_type: commit_type.to_string(),
                    should_create_fallback: false,
                });
            }
        }

        // No match found - should create fallback if enabled
        warn!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "No existing label found for commit type"
        );

        Ok(DiscoveredChangeTypeLabels {
            label_name: None,
            commit_type: commit_type.to_string(),
            should_create_fallback: config.fallback_label_settings.create_if_missing,
        })
    }

    /// Get mapped label names for a conventional commit type
    fn get_mapped_label_names(
        &self,
        commit_type: &str,
        config: &ChangeTypeLabelConfig,
    ) -> Vec<String> {
        let mut mapped_labels = Vec::new();

        // Add mappings from the configuration based on commit type
        let config_mappings = match commit_type {
            "feat" => &config.conventional_commit_mappings.feat,
            "fix" => &config.conventional_commit_mappings.fix,
            "docs" => &config.conventional_commit_mappings.docs,
            "style" => &config.conventional_commit_mappings.style,
            "refactor" => &config.conventional_commit_mappings.refactor,
            "perf" => &config.conventional_commit_mappings.perf,
            "test" => &config.conventional_commit_mappings.test,
            "chore" => &config.conventional_commit_mappings.chore,
            "ci" => &config.conventional_commit_mappings.ci,
            "build" => &config.conventional_commit_mappings.build,
            "revert" => &config.conventional_commit_mappings.revert,
            _ => &Vec::new(), // Return empty vector for unknown types
        };

        // Add configured mappings
        mapped_labels.extend(config_mappings.clone());

        // Add default mappings if no configuration mappings exist
        if mapped_labels.is_empty() {
            mapped_labels.extend(self.get_default_mappings(commit_type));
        }

        mapped_labels
    }

    /// Get default mappings for common conventional commit types
    fn get_default_mappings(&self, commit_type: &str) -> Vec<String> {
        match commit_type {
            "feat" => vec!["feature".to_string(), "enhancement".to_string()],
            "fix" => vec!["bug".to_string(), "bugfix".to_string()],
            "docs" => vec!["documentation".to_string()],
            "style" => vec!["style".to_string()],
            "refactor" => vec!["refactor".to_string()],
            "perf" => vec!["performance".to_string()],
            "test" => vec!["testing".to_string(), "tests".to_string()],
            "build" => vec!["build".to_string()],
            "ci" => vec!["ci".to_string()],
            "chore" => vec!["chore".to_string()],
            "revert" => vec!["revert".to_string()],
            _ => vec![],
        }
    }

    /// Find exact match for mapped label names
    fn find_exact_match(
        &self,
        labels: &[Label],
        mapped_labels: &[String],
        owner: &str,
        repo: &str,
        commit_type: &str,
    ) -> Option<String> {
        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "Starting exact match detection"
        );

        for mapped_label in mapped_labels {
            for label in labels {
                if label.name.eq_ignore_ascii_case(mapped_label) {
                    info!(
                        repository_owner = owner,
                        repository = repo,
                        commit_type = commit_type,
                        found_label = %label.name,
                        mapped_label = %mapped_label,
                        detection_method = "exact_match",
                        "Found exact match for commit type"
                    );
                    return Some(label.name.clone());
                }
            }
        }

        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "No exact match found"
        );
        None
    }

    /// Find prefix match for commit type
    fn find_prefix_match(
        &self,
        labels: &[Label],
        commit_type: &str,
        owner: &str,
        repo: &str,
    ) -> Option<String> {
        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "Starting prefix match detection"
        );

        // Define possible prefix patterns
        let prefix_patterns = vec![
            format!("{}:", commit_type),
            format!("{}-", commit_type),
            format!("{}_{}", commit_type, ""),
            format!("type: {}", commit_type),
            format!("type-{}", commit_type),
            format!("type_{}", commit_type),
            format!("kind: {}", commit_type),
            format!("kind-{}", commit_type),
            format!("kind_{}", commit_type),
        ];

        for pattern in &prefix_patterns {
            for label in labels {
                if label
                    .name
                    .to_lowercase()
                    .starts_with(&pattern.to_lowercase())
                {
                    info!(
                        repository_owner = owner,
                        repository = repo,
                        commit_type = commit_type,
                        found_label = %label.name,
                        prefix_pattern = %pattern,
                        detection_method = "prefix_match",
                        "Found prefix match for commit type"
                    );
                    return Some(label.name.clone());
                }
            }
        }

        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "No prefix match found"
        );
        None
    }

    /// Find description match for commit type
    fn find_description_match(
        &self,
        labels: &[Label],
        commit_type: &str,
        owner: &str,
        repo: &str,
    ) -> Option<String> {
        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "Starting description match detection"
        );

        let commit_type_lower = commit_type.to_lowercase();

        for label in labels {
            if let Some(ref description) = label.description {
                let description_lower = description.to_lowercase();

                // Look for the commit type in the description
                if description_lower.contains(&commit_type_lower) {
                    info!(
                        repository_owner = owner,
                        repository = repo,
                        commit_type = commit_type,
                        found_label = %label.name,
                        label_description = %description,
                        detection_method = "description_match",
                        "Found description match for commit type"
                    );
                    return Some(label.name.clone());
                }
            }
        }

        debug!(
            repository_owner = owner,
            repository = repo,
            commit_type = commit_type,
            "No description match found"
        );
        None
    }
}

/// Result of change type label detection
#[derive(Debug, Clone)]
pub struct DiscoveredChangeTypeLabels {
    /// The detected label name, if any
    pub label_name: Option<String>,
    /// The conventional commit type that was searched for
    pub commit_type: String,
    /// Whether fallback label creation should be used
    pub should_create_fallback: bool,
}

/// Result of label management operation
#[derive(Debug, Clone)]
pub struct LabelManagementResult {
    /// Labels that were successfully applied
    pub applied_labels: Vec<String>,
    /// Labels that were removed
    pub removed_labels: Vec<String>,
    /// Labels that were created as fallbacks
    pub created_fallback_labels: Vec<String>,
    /// Any labels that failed to be applied
    pub failed_labels: Vec<String>,
    /// Error messages for any failures (non-blocking)
    pub error_messages: Vec<String>,
}

impl Default for LabelManagementResult {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelManagementResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            applied_labels: Vec::new(),
            removed_labels: Vec::new(),
            created_fallback_labels: Vec::new(),
            failed_labels: Vec::new(),
            error_messages: Vec::new(),
        }
    }

    /// Check if the operation was completely successful
    pub fn is_success(&self) -> bool {
        self.failed_labels.is_empty() && self.error_messages.is_empty()
    }

    /// Check if any labels were applied (including fallbacks)
    pub fn has_applied_labels(&self) -> bool {
        !self.applied_labels.is_empty() || !self.created_fallback_labels.is_empty()
    }

    /// Get all labels that were successfully applied (including fallbacks)
    pub fn all_applied_labels(&self) -> Vec<String> {
        let mut all_labels = self.applied_labels.clone();
        all_labels.extend(self.created_fallback_labels.clone());
        all_labels
    }

    /// Check if fallback labels were created during the operation
    pub fn used_fallback_creation(&self) -> bool {
        !self.created_fallback_labels.is_empty()
    }
}

/// Label manager that coordinates intelligent label detection and application
///
/// This manager orchestrates the complete labeling workflow:
/// - Uses LabelDetector to find existing repository labels
/// - Applies intelligent label selection based on detection results
/// - Creates fallback labels with consistent formatting when needed
/// - Handles label application, removal, and updates intelligently
/// - Provides graceful error handling and comprehensive logging
pub struct LabelManager {
    /// Configuration for change type label detection and management
    config: Option<ChangeTypeLabelConfig>,
}

impl LabelManager {
    /// Create a new label manager with the specified configuration
    pub fn new(config: Option<ChangeTypeLabelConfig>) -> Self {
        Self { config }
    }

    /// Apply labeling to a pull request based on conventional commit type
    ///
    /// This method implements the complete labeling workflow:
    /// 1. Detects existing labels in the repository
    /// 2. Applies intelligent label selection based on commit type
    /// 3. Creates fallback labels if no existing labels are found
    /// 4. Handles label application with proper error handling
    ///
    /// # Arguments
    ///
    /// * `provider` - The Git provider implementation
    /// * `owner` - The owner of the repository
    /// * `repo` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `commit_type` - The conventional commit type (feat, fix, docs, etc.)
    ///
    /// # Returns
    ///
    /// A `Result` containing the label result with details about applied labels
    pub async fn apply_change_type_label<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_type: &str,
    ) -> Result<LabelManagementResult, MergeWardenError> {
        let mut result = LabelManagementResult::new();

        info!(
            repository_owner = owner,
            repository = repo,
            pr_number = pr_number,
            commit_type = commit_type,
            "Starting smart label management for change type"
        );

        // Step 1: Detect existing labels using LabelDetector
        if let Some(ref config) = self.config {
            let detector = LabelDetector::new_for_change_type_labels(config.clone());

            match detector
                .detect_change_type_label(provider, owner, repo, commit_type)
                .await
            {
                Ok(detection_result) => {
                    if let Some(label_name) = detection_result.label_name {
                        // Step 2: Apply the detected label
                        info!(
                            repository_owner = owner,
                            repository = repo,
                            pr_number = pr_number,
                            commit_type = commit_type,
                            detected_label = %label_name,
                            "Found existing label for commit type, applying"
                        );

                        match self
                            .apply_label(provider, owner, repo, pr_number, &label_name)
                            .await
                        {
                            Ok(()) => {
                                result.applied_labels.push(label_name);
                                info!(
                                    repository_owner = owner,
                                    repository = repo,
                                    pr_number = pr_number,
                                    commit_type = commit_type,
                                    applied_label = %result.applied_labels[0],
                                    "Successfully applied detected label"
                                );
                            }
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to apply detected label '{}': {}",
                                    label_name, e
                                );
                                warn!(
                                    repository_owner = owner,
                                    repository = repo,
                                    pr_number = pr_number,
                                    commit_type = commit_type,
                                    failed_label = %label_name,
                                    error = %e,
                                    "Failed to apply detected label"
                                );
                                result.failed_labels.push(label_name);
                                result.error_messages.push(error_msg);
                            }
                        }
                    } else if detection_result.should_create_fallback {
                        // Step 3: Create and apply fallback label
                        info!(
                            repository_owner = owner,
                            repository = repo,
                            pr_number = pr_number,
                            commit_type = commit_type,
                            "No existing label found, creating fallback label"
                        );

                        match self
                            .create_and_apply_fallback_label(
                                provider,
                                owner,
                                repo,
                                pr_number,
                                commit_type,
                                config,
                            )
                            .await
                        {
                            Ok(fallback_label) => {
                                result.created_fallback_labels.push(fallback_label.clone());
                                info!(
                                    repository_owner = owner,
                                    repository = repo,
                                    pr_number = pr_number,
                                    commit_type = commit_type,
                                    fallback_label = %fallback_label,
                                    "Successfully created and applied fallback label"
                                );
                            }
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to create fallback label for '{}': {}",
                                    commit_type, e
                                );
                                warn!(
                                    repository_owner = owner,
                                    repository = repo,
                                    pr_number = pr_number,
                                    commit_type = commit_type,
                                    error = %e,
                                    "Failed to create fallback label"
                                );
                                result.error_messages.push(error_msg);
                            }
                        }
                    } else {
                        debug!(
                            repository_owner = owner,
                            repository = repo,
                            pr_number = pr_number,
                            commit_type = commit_type,
                            "No existing label found and fallback creation disabled"
                        );
                    }
                }
                Err(e) => {
                    let error_msg = format!("Label detection failed for '{}': {}", commit_type, e);
                    warn!(
                        repository_owner = owner,
                        repository = repo,
                        pr_number = pr_number,
                        commit_type = commit_type,
                        error = %e,
                        "Label detection failed, attempting fallback"
                    );
                    result.error_messages.push(error_msg);

                    // Fallback to default behavior if detection fails
                    if let Some(default_label) = self.get_default_label_for_commit_type(commit_type)
                    {
                        match self
                            .apply_label(provider, owner, repo, pr_number, &default_label)
                            .await
                        {
                            Ok(()) => {
                                result.applied_labels.push(default_label);
                                info!(
                                    repository_owner = owner,
                                    repository = repo,
                                    pr_number = pr_number,
                                    commit_type = commit_type,
                                    fallback_label = %result.applied_labels[0],
                                    "Applied default fallback label after detection failure"
                                );
                            }
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to apply default fallback label '{}': {}",
                                    default_label, e
                                );
                                result.failed_labels.push(default_label);
                                result.error_messages.push(error_msg);
                            }
                        }
                    }
                }
            }
        } else {
            // No configuration provided, use hardcoded defaults
            debug!(
                repository_owner = owner,
                repository = repo,
                pr_number = pr_number,
                commit_type = commit_type,
                "No smart label configuration provided, using hardcoded defaults"
            );

            if let Some(default_label) = self.get_default_label_for_commit_type(commit_type) {
                match self
                    .apply_label(provider, owner, repo, pr_number, &default_label)
                    .await
                {
                    Ok(()) => {
                        result.applied_labels.push(default_label);
                    }
                    Err(e) => {
                        let error_msg =
                            format!("Failed to apply default label '{}': {}", default_label, e);
                        result.failed_labels.push(default_label);
                        result.error_messages.push(error_msg);
                    }
                }
            }
        }

        let final_success = result.is_success();
        let applied_count = result.all_applied_labels().len();

        info!(
            repository_owner = owner,
            repository = repo,
            pr_number = pr_number,
            commit_type = commit_type,
            success = final_success,
            applied_labels_count = applied_count,
            removed_labels_count = result.removed_labels.len(),
            failed_labels_count = result.failed_labels.len(),
            "Completed smart label management for change type"
        );

        Ok(result)
    }

    /// Apply a single label to a pull request
    async fn apply_label<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
        pr_number: u64,
        label_name: &str,
    ) -> Result<(), MergeWardenError> {
        provider
            .add_labels(owner, repo, pr_number, &[label_name.to_string()])
            .await
            .map_err(|_| {
                MergeWardenError::FailedToUpdatePullRequest(format!(
                    "Failed to add label '{}'",
                    label_name
                ))
            })
    }

    /// Create and apply a fallback label when no existing label is found
    async fn create_and_apply_fallback_label<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_type: &str,
        config: &ChangeTypeLabelConfig,
    ) -> Result<String, MergeWardenError> {
        // Generate fallback label name based on configuration
        let fallback_label = self.generate_fallback_label_name(commit_type, config);

        debug!(
            repository_owner = owner,
            repository = repo,
            pr_number = pr_number,
            commit_type = commit_type,
            fallback_label = %fallback_label,
            "Generated fallback label name"
        );

        // Apply the fallback label (this will create it if it doesn't exist)
        self.apply_label(provider, owner, repo, pr_number, &fallback_label)
            .await?;

        Ok(fallback_label)
    }

    /// Generate a fallback label name based on configuration and commit type
    fn generate_fallback_label_name(
        &self,
        commit_type: &str,
        config: &ChangeTypeLabelConfig,
    ) -> String {
        // Use the configured fallback format, defaulting to a standard pattern
        let format = &config.fallback_label_settings.name_format;

        // Replace {change_type} placeholder with the actual commit type
        let fallback_label = format.replace("{change_type}", commit_type);

        debug!(
            commit_type = commit_type,
            format_template = %format,
            generated_label = %fallback_label,
            "Generated fallback label using configured format"
        );

        fallback_label
    }

    /// Get default hardcoded label for a commit type (fallback when no configuration)
    fn get_default_label_for_commit_type(&self, commit_type: &str) -> Option<String> {
        let default_label = match commit_type {
            "feat" => Some("feature".to_string()),
            "fix" => Some("bug".to_string()),
            "docs" => Some("documentation".to_string()),
            "style" => Some("style".to_string()),
            "refactor" => Some("refactor".to_string()),
            "perf" => Some("performance".to_string()),
            "test" => Some("testing".to_string()),
            "build" => Some("build".to_string()),
            "ci" => Some("ci".to_string()),
            "chore" => Some("chore".to_string()),
            "revert" => Some("revert".to_string()),
            _ => None,
        };

        if let Some(ref label) = default_label {
            debug!(
                commit_type = commit_type,
                default_label = %label,
                "Using hardcoded default label for commit type"
            );
        } else {
            debug!(
                commit_type = commit_type,
                "No hardcoded default label available for commit type"
            );
        }

        default_label
    }

    /// Apply smart labeling for breaking changes detection
    ///
    /// This method handles the special case of breaking change detection,
    /// which can be determined from PR title or body content.
    ///
    /// # Arguments
    ///
    /// * `provider` - The Git provider implementation
    /// * `owner` - The owner of the repository
    /// * `repo` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `pr_title` - The pull request title
    /// * `pr_body` - The pull request body (optional)
    ///
    /// # Returns
    ///
    /// A `Result` containing the smart label result for breaking change labels
    pub async fn apply_breaking_change_label<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_title: &str,
        pr_body: Option<&str>,
    ) -> Result<LabelManagementResult, MergeWardenError> {
        let mut result = LabelManagementResult::new();

        // Check if this PR indicates a breaking change
        let is_breaking_change = pr_title.contains("!:")
            || pr_title.to_lowercase().contains("breaking change")
            || pr_body.map_or(false, |body| {
                body.to_lowercase().contains("breaking change")
            });

        if !is_breaking_change {
            debug!(
                repository_owner = owner,
                repository = repo,
                pr_number = pr_number,
                "No breaking change indicators found in PR title or body"
            );
            return Ok(result);
        }

        info!(
            repository_owner = owner,
            repository = repo,
            pr_number = pr_number,
            "Breaking change detected, applying smart labeling"
        );

        // Try to find an existing breaking change label
        let breaking_change_label = if let Some(ref config) = self.config {
            // Use detection to find existing breaking change labels
            let detector = LabelDetector::new_for_change_type_labels(config.clone());

            // For breaking changes, we look for common patterns
            let breaking_change_candidates =
                vec!["breaking-change", "breaking", "major", "bc", "BREAKING"];

            // Try to detect any existing breaking change label
            let mut found_label = None;
            for candidate in &breaking_change_candidates {
                match detector
                    .detect_change_type_label(provider, owner, repo, candidate)
                    .await
                {
                    Ok(detection_result) => {
                        if detection_result.label_name.is_some() {
                            found_label = detection_result.label_name;
                            break;
                        }
                    }
                    Err(_) => continue, // Try next candidate
                }
            }

            found_label.unwrap_or_else(|| {
                // Fallback to configured format
                self.generate_fallback_label_name("breaking", config)
            })
        } else {
            // No configuration, use hardcoded default
            "breaking-change".to_string()
        };

        // Apply the breaking change label
        match self
            .apply_label(provider, owner, repo, pr_number, &breaking_change_label)
            .await
        {
            Ok(()) => {
                result.applied_labels.push(breaking_change_label.clone());
                info!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr_number,
                    breaking_change_label = %breaking_change_label,
                    "Successfully applied breaking change label"
                );
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to apply breaking change label '{}': {}",
                    breaking_change_label, e
                );
                warn!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr_number,
                    breaking_change_label = %breaking_change_label,
                    error = %e,
                    "Failed to apply breaking change label"
                );
                result.failed_labels.push(breaking_change_label);
                result.error_messages.push(error_msg);
            }
        }

        Ok(result)
    }

    /// Apply smart labeling for keyword-based labels (security, hotfix, tech-debt)
    ///
    /// This method handles keyword detection in PR body for special labels.
    ///
    /// # Arguments
    ///
    /// * `provider` - The Git provider implementation
    /// * `owner` - The owner of the repository
    /// * `repo` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `pr_body` - The pull request body (optional)
    ///
    /// # Returns
    ///
    /// A `Result` containing the smart label result for keyword-based labels
    pub async fn apply_keyword_labels<P: PullRequestProvider>(
        &self,
        provider: &P,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_body: Option<&str>,
    ) -> Result<LabelManagementResult, MergeWardenError> {
        let mut result = LabelManagementResult::new();

        let Some(body) = pr_body else {
            debug!(
                repository_owner = owner,
                repository = repo,
                pr_number = pr_number,
                "No PR body provided for keyword label detection"
            );
            return Ok(result);
        };

        let body_lower = body.to_lowercase();

        // Define keyword mappings
        let keyword_mappings = vec![
            (vec!["security", "vulnerability"], "security"),
            (vec!["hotfix"], "hotfix"),
            (vec!["technical debt", "tech debt"], "tech-debt"),
        ];

        for (keywords, default_label) in keyword_mappings {
            let has_keyword = keywords.iter().any(|keyword| body_lower.contains(keyword));

            if has_keyword {
                info!(
                    repository_owner = owner,
                    repository = repo,
                    pr_number = pr_number,
                    keywords = ?keywords,
                    default_label = default_label,
                    "Found keyword match, applying smart labeling"
                );

                let label_to_apply = if let Some(ref config) = self.config {
                    // Try to detect existing label using detection
                    let detector = LabelDetector::new_for_change_type_labels(config.clone());

                    match detector
                        .detect_change_type_label(provider, owner, repo, default_label)
                        .await
                    {
                        Ok(detection_result) => detection_result.label_name.unwrap_or_else(|| {
                            self.generate_fallback_label_name(default_label, config)
                        }),
                        Err(_) => default_label.to_string(),
                    }
                } else {
                    default_label.to_string()
                };

                // Apply the keyword-based label
                match self
                    .apply_label(provider, owner, repo, pr_number, &label_to_apply)
                    .await
                {
                    Ok(()) => {
                        result.applied_labels.push(label_to_apply.clone());
                        info!(
                            repository_owner = owner,
                            repository = repo,
                            pr_number = pr_number,
                            applied_label = %label_to_apply,
                            matched_keywords = ?keywords,
                            "Successfully applied keyword-based label"
                        );
                    }
                    Err(e) => {
                        let error_msg =
                            format!("Failed to apply keyword label '{}': {}", label_to_apply, e);
                        warn!(
                            repository_owner = owner,
                            repository = repo,
                            pr_number = pr_number,
                            failed_label = %label_to_apply,
                            error = %e,
                            "Failed to apply keyword-based label"
                        );
                        result.failed_labels.push(label_to_apply);
                        result.error_messages.push(error_msg);
                    }
                }
            }
        }

        Ok(result)
    }
}
