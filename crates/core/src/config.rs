//! Configuration settings for the Merge Warden core functionality.
//!
//! This module centralizes configuration constants and settings used throughout
//! the crate, making it easier to modify behavior in one place.
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::errors::ConfigLoadError;

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

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
        r"^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+"
    ).expect("Failed to compile conventional commit regex");

    /// Pre-compiled regex for work item reference validation
    pub static ref WORK_ITEM_REGEX: Regex = Regex::new(
        r"(?i)(fixes|closes|resolves|references|relates to)\s+(#\d+|GH-\d+|https://github\.com/[^/]+/[^/]+/issues/\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\d+)"
    ).expect("Failed to compile work item regex");
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthenticationConfig {
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
}

impl AuthenticationConfig {
    pub fn new() -> Self {
        AuthenticationConfig {
            auth_method: default_auth_method(),
        }
    }
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DefaultConfig {
    /// Default Git provider
    #[serde(default = "default_provider")]
    pub provider: String,
}

impl DefaultConfig {
    pub fn new() -> Self {
        DefaultConfig {
            provider: default_provider(),
        }
    }
}

/// Top-level configuration struct for merge-warden
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeWardenConfig {
    pub schemaVersion: u32,
    #[serde(default)]
    pub policies: PoliciesConfig,
}

/// Convert a MergeWardenConfig (TOML) to a ValidationConfig (runtime enforcement)
impl MergeWardenConfig {
    pub fn to_validation_config(&self) -> ValidationConfig {
        // For now, only support the main PR policies (title, work item)
        let pr_policies = &self.policies.pull_requests;
        let enforce_conventional_commits = pr_policies.prTitle.format == "conventional-commits";
        let require_work_item_references = pr_policies.workItem.required;
        // Auto-labeling is always enabled for now (could be made configurable later)
        ValidationConfig {
            enforce_conventional_commits,
            require_work_item_references,
            auto_label: true,
        }
    }
}

impl Default for MergeWardenConfig {
    fn default() -> Self {
        Self {
            schemaVersion: 1,
            policies: PoliciesConfig::default(),
        }
    }
}

/// Policies configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PoliciesConfig {
    #[serde(default, rename = "pullRequests")]
    pub pull_requests: PullRequestsPoliciesConfig,
}

/// Pull request policies configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PullRequestsPoliciesConfig {
    #[serde(default)]
    pub prTitle: PullRequestsTitlePolicyConfig,

    #[serde(default)]
    pub workItem: WorkItemPolicyConfig,
}

/// Configuration for PR title policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestsTitlePolicyConfig {
    /// PR title format (e.g., "conventional-commits")
    #[serde(default = "PullRequestsTitlePolicyConfig::default_format")]
    pub format: String,
}

impl PullRequestsTitlePolicyConfig {
    fn default_format() -> String {
        "conventional-commits".to_string()
    }
}

impl Default for PullRequestsTitlePolicyConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
        }
    }
}

/// Rules configuration
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RulesConfig {
    /// Require work items to be linked
    #[serde(default)]
    pub require_work_items: bool,

    /// Enforce title convention
    #[serde(default)]
    pub enforce_title_convention: Option<bool>,

    /// Minimum number of approvals required
    #[serde(default)]
    pub min_approvals: Option<u32>,
}

impl RulesConfig {
    pub fn new() -> Self {
        RulesConfig {
            require_work_items: false,
            enforce_title_convention: Some(false),
            min_approvals: Some(1),
        }
    }
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

/// Configuration for work item policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkItemPolicyConfig {
    /// Whether a work item reference is required in the PR description
    #[serde(default = "WorkItemPolicyConfig::default_required")]
    pub required: bool,
    /// Regex pattern for work item references
    #[serde(default = "WorkItemPolicyConfig::default_pattern")]
    pub pattern: String,
}

impl WorkItemPolicyConfig {
    fn default_required() -> bool {
        true
    }
    fn default_pattern() -> String {
        "#\\d+".to_string()
    }
}

impl Default for WorkItemPolicyConfig {
    fn default() -> Self {
        Self {
            required: Self::default_required(),
            pattern: Self::default_pattern(),
        }
    }
}

fn default_auth_method() -> String {
    "token".to_string()
}

fn default_provider() -> String {
    "github".to_string()
}

/// Loads the merge-warden configuration from the given path.
//
/// If the file is missing, malformed, or has an unsupported schema version,
/// this function returns a default configuration and logs a warning.
///
/// # Arguments
/// * `path` - Path to the configuration file
///
/// # Returns
/// * `Ok(MergeWardenConfig)` if loaded and valid
/// * `Err(ConfigLoadError)` if there is a problem
pub fn load_merge_warden_config<P: AsRef<Path>>(
    path: P,
) -> Result<MergeWardenConfig, ConfigLoadError> {
    let path_ref = path.as_ref();
    let content = match fs::read_to_string(path_ref) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ConfigLoadError::NotFound(path_ref.display().to_string()));
        }
        Err(e) => return Err(ConfigLoadError::Io(e)),
    };
    let config: MergeWardenConfig = toml::from_str(&content)?;
    if config.schemaVersion != 1 {
        return Err(ConfigLoadError::UnsupportedSchemaVersion(
            config.schemaVersion,
        ));
    }
    Ok(config)
}
