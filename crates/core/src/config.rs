//! Configuration settings for the Merge Warden core functionality.
//!
//! This module centralizes configuration constants and settings used throughout
//! the crate, making it easier to modify behavior in one place.

use lazy_static::lazy_static;
use regex::Regex;

/// Label applied to PRs missing work item references
pub const MISSING_WORK_ITEM_LABEL: &str = "missing-work-item";

/// HTML comment marker for title validation comments
pub const TITLE_COMMENT_MARKER: &str = "<!-- PR_TITLE_CHECK -->";

/// Label applied to PRs with invalid title format
pub const TITLE_INVALID_LABEL: &str = "invalid-title-format";

/// Valid PR types for conventional commits
pub const VALID_PR_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

/// HTML comment marker for work item validation comments
pub const WORK_ITEM_COMMENT_MARKER: &str = "<!-- PR_WORK_ITEM_CHECK -->";

lazy_static! {
    /// Pre-compiled regex for conventional commit format validation
    pub static ref CONVENTIONAL_COMMIT_REGEX: Regex = Regex::new(
        r"^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9/-]+\))?!?: .+"
    ).expect("Failed to compile conventional commit regex");

    /// Pre-compiled regex for extracting scope from PR title
    pub static ref PR_SCOPE_REGEX: Regex = Regex::new(
        r"\(([a-z0-9/-]+)\)"
    ).expect("Failed to compile PR scope regex");

    /// Pre-compiled regex for extracting PR type from title
    pub static ref PR_TYPE_REGEX: Regex = Regex::new(
        r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)"
    ).expect("Failed to compile PR type regex");

    /// Pre-compiled regex for work item reference validation
    pub static ref WORK_ITEM_REGEX: Regex = Regex::new(
        r"(?i)(fixes|closes|resolves|references|relates to)\s+(#\d+|GH-\d+|https://github\.com/[^/]+/[^/]+/issues/\d+)"
    ).expect("Failed to compile work item regex");
}

/// Configuration for PR validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to enforce conventional commit format for PR titles
    pub enforce_conventional_commits: bool,

    /// Whether to require work item references in PR descriptions
    pub require_work_item_references: bool,

    /// Whether to automatically add labels based on PR content
    pub auto_label: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enforce_conventional_commits: true,
            require_work_item_references: true,
            auto_label: true,
        }
    }
}
