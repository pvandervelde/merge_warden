//! Configuration settings for the Merge Warden core functionality.
//!
//! This module centralizes configuration constants and settings used throughout
//! the crate, making it easier to modify behavior in one place.
use lazy_static::lazy_static;
use merge_warden_developer_platforms::ConfigFetcher;
use regex::Regex;
use serde::{Deserialize, Serialize};

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
    ///
    /// This regex enforces the Conventional Commits specification (https://conventionalcommits.org/)
    /// which provides a lightweight convention on top of commit messages to create an explicit
    /// commit history that makes it easier to write automated tools on top of.
    ///
    /// ## Pattern Breakdown
    ///
    /// ```text
    /// ^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+
    /// ```
    ///
    /// - `^` - Anchors the match to the start of the string
    /// - `(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)` - **Type** (required)
    ///   - Captures one of the standard commit types
    /// - `(\([a-z0-9_-]+\))?` - **Scope** (optional)
    ///   - Parentheses-wrapped scope using lowercase letters, numbers, underscores, or hyphens
    ///   - Examples: `(auth)`, `(ui)`, `(user-service)`
    /// - `!?` - **Breaking change indicator** (optional)
    ///   - Exclamation mark indicates a breaking change
    /// - `:` - **Separator** (required)
    ///   - Literal colon character
    /// - ` ` - **Space** (required)
    ///   - Single space after the colon
    /// - `.+` - **Description** (required)
    ///   - One or more characters describing the change
    ///
    /// ## Commit Types
    ///
    /// | Type       | Description                                               |
    /// |------------|-----------------------------------------------------------|
    /// | `build`    | Changes that affect the build system or external deps    |
    /// | `chore`    | Other changes that don't modify src or test files        |
    /// | `ci`       | Changes to CI configuration files and scripts           |
    /// | `docs`     | Documentation only changes                               |
    /// | `feat`     | A new feature                                            |
    /// | `fix`      | A bug fix                                                |
    /// | `perf`     | A code change that improves performance                  |
    /// | `refactor` | A code change that neither fixes a bug nor adds a feature|
    /// | `revert`   | Reverts a previous commit                                |
    /// | `style`    | Changes that don't affect the meaning of the code       |
    /// | `test`     | Adding missing tests or correcting existing tests       |
    ///
    /// ## Valid Examples
    ///
    /// ```text
    /// feat: add user authentication system
    /// fix(auth): resolve login validation bug
    /// feat!: remove deprecated API endpoints
    /// docs(readme): update installation instructions
    /// perf(database): optimize query performance
    /// chore: update dependencies
    /// ```
    ///
    /// ## Invalid Examples
    ///
    /// ```text
    /// Add new feature                    // Missing type and colon
    /// feat:missing space after colon    // Missing required space
    /// FEAT: uppercase type not allowed  // Type must be lowercase
    /// feat(AUTH): uppercase scope       // Scope must be lowercase
    /// feat(spa ces): spaces in scope    // Only a-z, 0-9, _, - allowed in scope
    /// feat:                             // Empty description
    /// ```
    ///
    /// ## Usage
    ///
    /// ```rust
    /// use regex::Regex;
    ///
    /// let conventional_commit_regex = Regex::new(
    ///     r"^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+"
    /// ).unwrap();
    ///
    /// assert!(conventional_commit_regex.is_match("feat: add new user dashboard"));
    /// assert!(conventional_commit_regex.is_match("fix(auth): resolve token expiry"));
    /// assert!(!conventional_commit_regex.is_match("Add new feature"));
    /// ```
    pub static ref CONVENTIONAL_COMMIT_REGEX: Regex = Regex::new(
        r"^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+"
    ).expect("Failed to compile conventional commit regex");

    /// Regular expression pattern for matching issue reference patterns in commit messages and PRs.
    ///
    /// This regex identifies references to GitHub issues and other issue tracking systems,
    /// enabling automatic linking and closure of issues when commits are merged. It supports
    /// multiple reference formats and action keywords that GitHub recognizes for automated
    /// issue management.
    ///
    /// ## Pattern Breakdown
    ///
    /// ```text
    /// (?i)(fixes|closes|resolves|references|relates to)\s+(#\d+|GH-\d+|https://github\.com/[^/]+/[^/]+/issues/\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\d+)
    /// ```
    ///
    /// - `(?i)` - **Case-insensitive flag**
    ///   - Matches regardless of capitalization (e.g., "Fixes", "CLOSES", "resolves")
    /// - `(fixes|closes|resolves|references|relates to)` - **Action keywords**
    ///   - Determines how the issue should be handled when the commit/PR is merged
    /// - `\s+` - **Whitespace separator** (required)
    ///   - One or more spaces, tabs, or newlines between keyword and issue reference
    /// - Issue reference alternatives (one of the following formats):
    ///   - `#\d+` - Simple hash-number format
    ///   - `GH-\d+` - GitHub-prefixed format
    ///   - `https://github\.com/[^/]+/[^/]+/issues/\d+` - Full GitHub URL
    ///   - `[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\d+` - Owner/repository#issue format
    ///
    /// ## Action Keywords
    ///
    /// | Keyword      | Behavior                           | GitHub Auto-Close |
    /// |--------------|------------------------------------|--------------------|
    /// | `fixes`      | Links and closes issue when merged | ✅                |
    /// | `closes`     | Links and closes issue when merged | ✅                |
    /// | `resolves`   | Links and closes issue when merged | ✅                |
    /// | `references` | Links to issue without closing     | ❌                |
    /// | `relates to` | Links to issue without closing     | ❌                |
    ///
    /// ## Supported Issue Reference Formats
    ///
    /// ### 1. Simple Hash Format (`#\d+`)
    /// References an issue in the same repository using just the issue number.
    /// ```text
    /// fixes #123
    /// closes #4567
    /// ```
    ///
    /// ### 2. GitHub Prefix Format (`GH-\d+`)
    /// Alternative GitHub-style reference format.
    /// ```text
    /// resolves GH-456
    /// references GH-789
    /// ```
    ///
    /// ### 3. Full GitHub URL Format
    /// Complete URL to a GitHub issue, useful for cross-repository references.
    /// ```text
    /// fixes https://github.com/rust-lang/rust/issues/98765
    /// relates to https://github.com/microsoft/vscode/issues/12345
    /// ```
    ///
    /// ### 4. Owner/Repository Format (`owner/repo#issue`)
    /// Concise format for referencing issues in other repositories.
    /// ```text
    /// closes microsoft/typescript#4567
    /// references facebook/react#8901
    /// ```
    ///
    /// ## Valid Examples
    ///
    /// ```text
    /// fixes #42
    /// Closes GH-123
    /// RESOLVES #999
    /// references https://github.com/rust-lang/cargo/issues/5678
    /// Relates to microsoft/vscode#1234
    /// fixes    #567           // Multiple spaces allowed
    /// closes #123 and #456    // Multiple references (each matched separately)
    /// ```
    ///
    /// ## Invalid Examples
    ///
    /// ```text
    /// fix #42                              // Wrong keyword (not in approved list)
    /// fixes#42                             // Missing required whitespace
    /// fixes issue #42                      // Extra words between keyword and reference
    /// closes github.com/owner/repo#123     // Missing https:// protocol
    /// resolves #                           // Missing issue number
    /// fixes owner/repo/issues/123          // Wrong format (not a GitHub URL)
    /// ```
    ///
    /// ## Usage
    ///
    /// ```rust
    /// use regex::Regex;
    ///
    /// let issue_ref_regex = Regex::new(
    ///     r"(?i)(fixes|closes|resolves|references|relates to)\s+(#\d+|GH-\d+|https://github\.com/[^/]+/[^/]+/issues/\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\d+)"
    /// ).unwrap();
    ///
    /// // Test cases
    /// assert!(issue_ref_regex.is_match("fixes #123"));
    /// assert!(issue_ref_regex.is_match("Closes GH-456"));
    /// assert!(issue_ref_regex.is_match("resolves https://github.com/owner/repo/issues/789"));
    /// assert!(issue_ref_regex.is_match("References microsoft/vscode#1011"));
    /// assert!(!issue_ref_regex.is_match("fix #123")); // Wrong keyword
    ///
    /// // Extract all matches from a commit message
    /// let commit_msg = "fixes #123 and resolves microsoft/typescript#456";
    /// let matches: Vec<_> = issue_ref_regex.find_iter(commit_msg).collect();
    /// assert_eq!(matches.len(), 2);
    /// ```
    ///
    /// ## Integration with GitHub
    ///
    /// When this pattern is found in commit messages or pull request descriptions,
    /// GitHub will automatically:
    /// - Create links between the commit/PR and the referenced issues
    /// - Close issues when using closing keywords (`fixes`, `closes`, `resolves`)
    /// - Add timeline entries to the referenced issues
    /// - Enable cross-repository issue linking when using full URLs or owner/repo format
    pub static ref WORK_ITEM_REGEX: Regex = Regex::new(
        r"(?i)(fixes|closes|resolves|references|relates to)\s+(#\d+|GH-\d+|https://github\.com/[^/]+/[^/]+/issues/\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\d+)"
    ).expect("Failed to compile work item regex");
}

/// Application-level default settings for merge-warden configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDefaults {
    /// Whether the pull request title should follow a convention
    #[serde(
        rename = "enforceTitleValidation",
        default = "ApplicationDefaults::default_title_required"
    )]
    pub enable_title_validation: bool,

    #[serde(
        rename = "titlePattern",
        default = "ApplicationDefaults::default_title_pattern"
    )]
    pub default_title_pattern: String,

    #[serde(
        rename = "labelIfTitleInvalid",
        default = "ApplicationDefaults::default_title_invalid_label"
    )]
    pub default_invalid_title_label: Option<String>,

    #[serde(
        rename = "enforceWorkItemValidation",
        default = "ApplicationDefaults::default_work_item_required"
    )]
    pub enable_work_item_validation: bool,

    #[serde(
        rename = "workItemPattern",
        default = "ApplicationDefaults::default_work_item_pattern"
    )]
    pub default_work_item_pattern: String,

    #[serde(
        rename = "labelIfWorkItemMissing",
        default = "ApplicationDefaults::default_work_item_missing_label"
    )]
    pub default_missing_work_item_label: Option<String>,
}

impl ApplicationDefaults {
    fn default_title_invalid_label() -> Option<String> {
        None
    }

    fn default_title_pattern() -> String {
        CONVENTIONAL_COMMIT_REGEX.to_string()
    }

    fn default_title_required() -> bool {
        false
    }

    fn default_work_item_missing_label() -> Option<String> {
        None
    }

    fn default_work_item_pattern() -> String {
        WORK_ITEM_REGEX.to_string()
    }

    fn default_work_item_required() -> bool {
        false
    }
}

impl Default for ApplicationDefaults {
    fn default() -> Self {
        Self {
            enable_title_validation: ApplicationDefaults::default_title_required(),
            default_title_pattern: ApplicationDefaults::default_title_pattern(),
            default_invalid_title_label: ApplicationDefaults::default_title_invalid_label(),
            enable_work_item_validation: ApplicationDefaults::default_work_item_required(),
            default_work_item_pattern: ApplicationDefaults::default_work_item_pattern(),
            default_missing_work_item_label: ApplicationDefaults::default_work_item_missing_label(),
        }
    }
}

/// Top-level configuration struct for merge-warden repository level configuration.
/// This configuration data is read from the merge-warden.toml file in the .github directory of the
/// repository
/// The expected configuration file should look like
///
/// ---- File format ----
/// schemaVersion = 1
///
/// # Define the pull request policies pertaining to the pull request title.
/// [policies.pullRequests.prTitle]
/// # Indicate if the pull request title should follow a specific format.
/// required = true
/// # The regular expression pattern that the pull request title must match. By default it follows the conventional commit
/// # specification.
/// pattern = "^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\\([a-z0-9_-]+\\))?!?: .+"
/// # Define the label that will be applied to the pull request if the title does not match the specified pattern.
/// # If the label is not specified, no label will be applied.
/// label_if_missing = "invalid-title-format"
///
/// [policies.pullRequests.workItem]
/// # Indicate if the pull request description should contain a work item reference.
/// required = true
/// # The regular expression pattern that the pull request description must match to reference a work item.
/// # By default, it matches issue references like `#123`, `GH-123`, or full URLs to GitHub issues.
/// pattern = "(?i)(fixes|closes|resolves|references|relates to)\\s+(#\\d+|GH-\\d+|https://github\\.com/[^/]+/[^/]+/issues/\\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\\d+)"
/// # Define the label that will be applied to the pull request if it does not contain a work item reference.
/// # If the label is not specified, no label will be applied.
/// label_if_missing = "missing-work-item"
/// ---- ----------- ----
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryProvidedConfig {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,

    #[serde(default)]
    pub policies: PoliciesConfig,
}

/// Convert a RepositoryConfig (TOML) to a ValidationConfig (runtime enforcement)
impl RepositoryProvidedConfig {
    pub fn to_validation_config(&self) -> CurrentPullRequestValidationConfiguration {
        // For now, only support the main PR policies (title, work item)
        let pr_policies = &self.policies.pull_requests;

        let enforce_title_convention = pr_policies.title_policies.required;
        let title_pattern = pr_policies.title_policies.pattern.clone();
        let invalid_title_label = pr_policies.title_policies.label_if_missing.clone();

        let enforce_work_item_references = pr_policies.work_item_policies.required;
        let work_item_reference_pattern = pr_policies.work_item_policies.pattern.clone();
        let missing_work_item_label = pr_policies.work_item_policies.label_if_missing.clone();

        CurrentPullRequestValidationConfiguration {
            enforce_title_convention,
            title_pattern,
            invalid_title_label,
            enforce_work_item_references,
            work_item_reference_pattern,
            missing_work_item_label,
        }
    }
}

impl Default for RepositoryProvidedConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
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
    #[serde(default, rename = "prTitle")]
    pub title_policies: PullRequestsTitlePolicyConfig,

    #[serde(default, rename = "workItem")]
    pub work_item_policies: WorkItemPolicyConfig,
}

/// Configuration for PR title policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestsTitlePolicyConfig {
    /// Whether the pull request title should follow a convention
    #[serde(default = "WorkItemPolicyConfig::default_required")]
    pub required: bool,

    /// Regex pattern for the pull request title
    #[serde(default = "PullRequestsTitlePolicyConfig::default_pattern")]
    pub pattern: String,

    // Indicates the label that should be applied if the title doesn't match the pattern
    #[serde(default = "PullRequestsTitlePolicyConfig::default_label")]
    pub label_if_missing: Option<String>,
}

impl PullRequestsTitlePolicyConfig {
    fn default_label() -> Option<String> {
        None
    }

    fn default_pattern() -> String {
        CONVENTIONAL_COMMIT_REGEX.to_string()
    }

    fn default_required() -> bool {
        true
    }
}

impl Default for PullRequestsTitlePolicyConfig {
    fn default() -> Self {
        Self {
            required: Self::default_required(),
            pattern: Self::default_pattern(),
            label_if_missing: Self::default_label(),
        }
    }
}

/// Configuration for the validation of the current pull request.
#[derive(Debug, Clone)]
pub struct CurrentPullRequestValidationConfiguration {
    /// Whether to enforce conventional commit format for PR titles
    pub enforce_title_convention: bool,

    /// The regular expression used to determine if the pull request title is valid
    pub title_pattern: String,

    /// The label to apply when an invalid title is found. No label will be applied if set to `None`.
    pub invalid_title_label: Option<String>,

    /// Whether to require work item references in PR descriptions
    pub enforce_work_item_references: bool,

    /// The regular expression used to determine if a work item reference exists
    pub work_item_reference_pattern: String,

    /// The label to apply when no work item reference is found. No label will be applied if set to `None`.
    pub missing_work_item_label: Option<String>,
}

impl CurrentPullRequestValidationConfiguration {
    fn new(
        enforce_title_convention: bool,
        title_pattern: Option<String>,
        invalid_title_label: Option<String>,
        enforce_work_item_references: bool,
        work_item_reference_pattern: Option<String>,
        missing_work_item_label: Option<String>,
    ) -> Self {
        Self {
            enforce_title_convention,
            title_pattern: if let Some(pattern) = title_pattern {
                pattern
            } else {
                CONVENTIONAL_COMMIT_REGEX.to_string()
            },
            invalid_title_label,
            enforce_work_item_references,
            work_item_reference_pattern: if let Some(pattern) = work_item_reference_pattern {
                pattern
            } else {
                WORK_ITEM_REGEX.to_string()
            },
            missing_work_item_label,
        }
    }
}

impl Default for CurrentPullRequestValidationConfiguration {
    fn default() -> Self {
        Self {
            enforce_title_convention: true,
            title_pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
            invalid_title_label: Some(TITLE_INVALID_LABEL.to_string()),
            enforce_work_item_references: true,
            work_item_reference_pattern: WORK_ITEM_REGEX.to_string(),
            missing_work_item_label: Some(MISSING_WORK_ITEM_LABEL.to_string()),
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

    // Indicates the label that should be applied if the work item is missing
    #[serde(default = "WorkItemPolicyConfig::default_label")]
    pub label_if_missing: Option<String>,
}

impl WorkItemPolicyConfig {
    fn default_label() -> Option<String> {
        None
    }

    fn default_pattern() -> String {
        WORK_ITEM_REGEX.to_string()
    }

    fn default_required() -> bool {
        true
    }
}

impl Default for WorkItemPolicyConfig {
    fn default() -> Self {
        Self {
            required: Self::default_required(),
            pattern: Self::default_pattern(),
            label_if_missing: Self::default_label(),
        }
    }
}

/// Loads the merge-warden configuration from the given path.
//
/// If the file is missing, malformed, or has an unsupported schema version,
/// this function returns a default configuration and logs a warning.
///
/// # Arguments
/// * `repo_owner` - The name of the user or organisation that owns the repository in which the configuration is stored
/// * `repo_name` - The name of the repository in which the configuration is stored
/// * `path` - Path to the configuration file
/// * `fetch_repo_config` - The config fetcher used to get the config from the repository
/// * `app_defaults` - The default setting values for the application
///
/// # Returns
/// * `Ok(RepositoryConfig)` if loaded and valid
/// * `Err(ConfigLoadError)` if there is a problem
pub async fn load_merge_warden_config(
    repo_owner: &str,
    repo_name: &str,
    path_relative_to_repository_root: &str,
    fetch_repo_config: &dyn ConfigFetcher,
    app_defaults: &ApplicationDefaults,
) -> Result<RepositoryProvidedConfig, ConfigLoadError> {
    let potential_content = match fetch_repo_config
        .fetch_config(repo_owner, repo_name, path_relative_to_repository_root)
        .await
    {
        Ok(c) => c,
        Err(e) => return Err(ConfigLoadError::NotFound(e.to_string())),
    };

    let content = potential_content.unwrap_or(String::new());
    let mut config: RepositoryProvidedConfig = toml::from_str(&content)?;
    if config.schema_version != 1 {
        return Err(ConfigLoadError::UnsupportedSchemaVersion(
            config.schema_version,
        ));
    }

    // Enforce application-level enables for validations
    if app_defaults.enable_title_validation {
        config.policies.pull_requests.title_policies.required = true;
    }

    if app_defaults.enable_work_item_validation {
        config.policies.pull_requests.work_item_policies.required = true;
    }

    // Use repository config labels and patterns if present, else fallback to defaults
    if config
        .policies
        .pull_requests
        .title_policies
        .pattern
        .is_empty()
    {
        config.policies.pull_requests.title_policies.pattern =
            app_defaults.default_title_pattern.clone();
    }

    if config
        .policies
        .pull_requests
        .title_policies
        .label_if_missing
        .is_none()
    {
        config
            .policies
            .pull_requests
            .title_policies
            .label_if_missing = app_defaults.default_invalid_title_label.clone();
    }

    if config
        .policies
        .pull_requests
        .work_item_policies
        .pattern
        .is_empty()
    {
        config.policies.pull_requests.work_item_policies.pattern =
            app_defaults.default_work_item_pattern.clone();
    }

    if config
        .policies
        .pull_requests
        .work_item_policies
        .label_if_missing
        .is_none()
    {
        config
            .policies
            .pull_requests
            .work_item_policies
            .label_if_missing = app_defaults.default_missing_work_item_label.clone();
    }

    Ok(config)
}
