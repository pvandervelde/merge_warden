//! Configuration settings for the Merge Warden core functionality.
//!
//! This module centralizes configuration constants and settings used throughout
//! the crate, making it easier to modify behavior in one place.
use merge_warden_developer_platforms::{
    models::{RepositoryContext, User},
    ConfigFetcher, RepositoryMetadataProvider,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use crate::errors::ConfigLoadError;
use crate::size::SizeThresholds;

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

/// HTML comment marker for PR size validation comments
pub const SIZE_COMMENT_MARKER: &str = "<!-- PR_SIZE_CHECK -->";

/// HTML comment marker for WIP (Work In Progress) validation comments
pub const WIP_COMMENT_MARKER: &str = "<!-- PR_WIP_CHECK -->";

/// HTML comment marker prefix for keyword-triggered label explanation comments.
///
/// The full per-label marker appends the resolved label name and a closing delimiter,
/// e.g. `<!-- MERGE_WARDEN_KEYWORD_LABEL:breaking-change -->`.  Searching for this
/// prefix is sufficient to locate any keyword-label explanation comment.
pub const KEYWORD_LABEL_COMMENT_MARKER: &str = "<!-- MERGE_WARDEN_KEYWORD_LABEL:";

/// Context string identifying the Renovate stability check in GitHub commit statuses.
pub const RENOVATE_STABILITY_CHECK_CONTEXT: &str = "renovate/stability-days";

/// Default label name applied while the Renovate stability period has not elapsed.
pub const RENOVATE_STABILITY_LABEL: &str = "pr-validation: pending-stability";

/// HTML comment marker used to identify configuration validity status comments.
///
/// Merge Warden uses this marker to find and update (or delete) configuration
/// validation comments so that only one such comment exists on a PR at any time.
pub const CONFIG_COMMENT_MARKER: &str = "<!-- MERGE_WARDEN_CONFIG_CHECK -->";

/// Path to the repository-provided merge-warden configuration file.
///
/// When a PR touches this file, Merge Warden fetches and validates its content and
/// posts an informational comment on the PR.  The comment is purely informational
/// and never affects the check conclusion.
pub const CONFIG_FILE_PATH: &str = ".github/merge-warden.toml";

/// The outcome of validating the content of a repository-provided configuration file.
///
/// This type is returned by [`validate_config_content`] and carries both a boolean
/// validity flag and, when invalid, a list of human-readable error messages that can
/// be surfaced directly in a pull-request comment.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigValidationOutcome {
    /// `true` when the configuration is syntactically valid TOML and passes all
    /// schema checks; `false` otherwise.
    pub valid: bool,
    /// Human-readable error messages when `valid` is `false`, empty when `valid`
    /// is `true`.
    pub errors: Vec<String>,
}

/// Validates the content of a merge-warden configuration file.
///
/// Parses `content` as TOML into a [`RepositoryProvidedConfig`] and then applies
/// schema validation rules.  Returns a [`ConfigValidationOutcome`] that indicates
/// whether the configuration is valid and, if not, lists the reasons why.
///
/// # Validation scope
///
/// Validation is currently limited to:
///
/// 1. The content must be valid TOML that can be deserialized into
///    [`RepositoryProvidedConfig`].
/// 2. The `schemaVersion` field must equal `1`.
///
/// Semantic validation (e.g., verifying that label names, regex patterns, or other
/// field values are meaningful) is **out of scope** for this function.  Callers
/// should not rely on this function to catch application-level misconfiguration.
///
/// # Error message format
///
/// When TOML parsing fails, the error message is taken directly from the [`toml`]
/// crate.  The exact wording and structure of those messages is an implementation
/// detail of the `toml` crate and **may change** across versions.  Callers that
/// display these messages to end-users (e.g., in PR comments) should treat them as
/// opaque, human-readable strings rather than parsing them programmatically.
///
/// # Examples
///
/// ```rust
/// use merge_warden_core::config::validate_config_content;
///
/// // Valid configuration
/// let valid_toml = r#"schemaVersion = 1"#;
/// let outcome = validate_config_content(valid_toml);
/// assert!(outcome.valid);
/// assert!(outcome.errors.is_empty());
///
/// // Invalid TOML
/// let bad_toml = "not = valid = toml";
/// let outcome = validate_config_content(bad_toml);
/// assert!(!outcome.valid);
/// assert!(!outcome.errors.is_empty());
/// ```
pub fn validate_config_content(content: &str) -> ConfigValidationOutcome {
    match toml::from_str::<RepositoryProvidedConfig>(content) {
        Err(e) => ConfigValidationOutcome {
            valid: false,
            errors: vec![e.to_string()],
        },
        Ok(config) if config.schema_version != 1 => ConfigValidationOutcome {
            valid: false,
            errors: vec![format!(
                "schemaVersion must be 1, found {}",
                config.schema_version
            )],
        },
        Ok(_) => ConfigValidationOutcome {
            valid: true,
            errors: vec![],
        },
    }
}

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
pub static CONVENTIONAL_COMMIT_REGEX: &str =
    r"^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+";

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
pub static WORK_ITEM_REGEX: &str = r"(?i)(fixes|closes|resolves|references|relates to)\s+(#\d+|GH-\d+|https://github\.com/[^/]+/[^/]+/issues/\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\d+)";

/// Application-level default settings for merge-warden configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDefaults {
    /// Whether the pull request title should follow a convention
    #[serde(default = "ApplicationDefaults::default_title_required")]
    pub enable_title_validation: bool,

    /// Default regex pattern for validating pull request titles
    #[serde(default = "ApplicationDefaults::default_title_pattern")]
    pub default_title_pattern: String,

    /// Default label to apply when title validation fails
    #[serde(default = "ApplicationDefaults::default_title_invalid_label")]
    pub default_invalid_title_label: Option<String>,

    /// Whether work item reference validation is enabled by default
    #[serde(default = "ApplicationDefaults::default_work_item_required")]
    pub enable_work_item_validation: bool,

    /// Default regex pattern for validating work item references
    #[serde(default = "ApplicationDefaults::default_work_item_pattern")]
    pub default_work_item_pattern: String,

    /// Default label to apply when work item reference is missing
    #[serde(default = "ApplicationDefaults::default_work_item_missing_label")]
    pub default_missing_work_item_label: Option<String>,

    /// Bypass rules for allowing specific users to skip validation
    #[serde(default)]
    pub bypass_rules: BypassRules,

    /// Configuration for PR size checking
    #[serde(default)]
    pub pr_size_check: PrSizeCheckConfig,

    /// Configuration for change type label detection
    #[serde(default)]
    pub change_type_labels: ChangeTypeLabelConfig,

    /// Application-level defaults for WIP detection and blocking
    #[serde(default)]
    pub wip_check: WipCheckConfig,

    /// Application-level defaults for state-based PR lifecycle labels
    #[serde(default)]
    pub pr_state_labels: PrStateLabelsConfig,

    /// Application-level defaults for Renovate stability label management
    #[serde(default)]
    pub renovate_stability: RenovateStabilityConfig,

    /// Bot mention prefix used for comment-based label suppression.
    ///
    /// PR participants post a comment line of the form `<bot_mention> suppress: <label-name>`
    /// to prevent merge-warden from (re-)applying a keyword-triggered label.  Defaults to
    /// `"@merge-warden"`.  Operators running a custom GitHub App installation should set this
    /// to their app's mention handle (e.g. `"@acme-merge-warden[bot]"`).
    #[serde(default = "ApplicationDefaults::default_bot_mention")]
    pub bot_mention: String,

    /// Optional pointer to an org-level policy file.
    ///
    /// When `None`, the system behaves identically to the three-tier configuration
    /// model (application defaults → repo config → system defaults).
    /// When `Some`, a fourth tier is loaded from the specified repository and
    /// merged into the resolution chain via [`resolve_pull_request_config`].
    #[serde(default)]
    pub org_policy_source: Option<OrgPolicySource>,

    /// Optional repository allow/deny scope filter.
    ///
    /// When `None` (default), merge-warden processes events for every
    /// repository the GitHub App is installed on — full backward
    /// compatibility with pre-FR-009 behaviour.
    ///
    /// When `Some`, this is a webhook-ingress-level gate applied by
    /// [`crate::config::is_repository_in_scope`] — it is deliberately NOT
    /// part of the [`PolicySet`] merge chain (see
    /// [`PolicySet::from_application_defaults`]), since it controls whether
    /// an event is processed at all, not how it is validated.
    #[serde(default)]
    pub repository_scope: Option<RepositoryScope>,
}

impl ApplicationDefaults {
    /// Default value for invalid title label (None)
    fn default_title_invalid_label() -> Option<String> {
        None
    }

    /// Default regex pattern for title validation (conventional commits)
    fn default_title_pattern() -> String {
        CONVENTIONAL_COMMIT_REGEX.to_string()
    }

    /// Default value for title validation requirement (false)
    fn default_title_required() -> bool {
        false
    }

    /// Default value for missing work item label (None)
    fn default_work_item_missing_label() -> Option<String> {
        None
    }

    /// Default regex pattern for work item validation
    fn default_work_item_pattern() -> String {
        WORK_ITEM_REGEX.to_string()
    }

    /// Default value for work item validation requirement (false)
    fn default_work_item_required() -> bool {
        false
    }

    /// Default bot mention prefix for comment-based label suppression.
    fn default_bot_mention() -> String {
        "@merge-warden".to_string()
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
            bypass_rules: BypassRules::default(),
            pr_size_check: PrSizeCheckConfig::default(),
            change_type_labels: ChangeTypeLabelConfig::default(),
            wip_check: WipCheckConfig::default(),
            pr_state_labels: PrStateLabelsConfig::default(),
            renovate_stability: RenovateStabilityConfig::default(),
            bot_mention: ApplicationDefaults::default_bot_mention(),
            org_policy_source: None,
            repository_scope: None,
        }
    }
}

/// Locates the org-level policy TOML file within a GitHub repository.
///
/// Added to [`ApplicationDefaults`] as an optional field.
/// When absent, the system uses the three-tier configuration model.
///
/// # TOML example
///
/// ```toml
/// [org_policy_source]
/// owner = "my-org"
/// repo  = "platform-configs"
/// path  = "merge-warden/org-policy.toml"
/// # fail_if_unreachable = false   # optional; default false
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrgPolicySource {
    /// GitHub organisation or user name that owns the policy repository.
    pub owner: String,

    /// Name of the repository that holds the org policy file.
    pub repo: String,

    /// Path to the org policy TOML file within the repository,
    /// relative to the repository root.
    pub path: String,

    /// When `true`, an unreachable or unparseable org policy causes
    /// [`resolve_pull_request_config`] to return
    /// `Err(ConfigLoadError::OrgPolicyUnavailable)`.
    /// When `false` (default), failures degrade gracefully to the
    /// three-tier system.
    ///
    /// # Asymmetry: "file not found" always degrades gracefully
    ///
    /// This flag only governs *fetch errors*, *parse errors*, and *schema
    /// version mismatches*. If the policy file simply does not exist
    /// (`Ok(None)` from the fetcher), the system always falls back to
    /// three-tier mode with a warning — even when `fail_if_unreachable`
    /// is `true`. The rationale: a missing file is an expected bootstrap
    /// state (e.g. the policy file hasn't been created yet), whereas a
    /// fetch error indicates an infrastructure problem that the operator
    /// may want to surface explicitly.
    #[serde(default)]
    pub fail_if_unreachable: bool,
}

/// Repository allow/deny scope filter (FR-009: Repository Scope Filtering).
///
/// Added to [`ApplicationDefaults`] as an optional field. When absent
/// (`None`), every repository the GitHub App is installed on is processed —
/// identical to pre-FR-009 behaviour. When present, [`is_repository_in_scope`]
/// gates webhook processing before any per-repository configuration is
/// loaded.
///
/// This is deliberately NOT part of the [`PolicySet`] merge chain: it
/// controls whether an event is processed at all (an ingress-level
/// decision), not how a processed pull request is validated.
///
/// # Pattern syntax
///
/// Patterns are glob-like, restricted to ASCII letters, digits, `-`, `_`,
/// `.`, and two wildcards:
/// - `*` matches any sequence of characters, including the empty sequence.
/// - `?` matches exactly one character.
///
/// Every other character (e.g. `[`, `]`, `(`, `)`, `\`, whitespace) makes the
/// entire pattern invalid — see [`validate_repository_scope_patterns`].
/// Matching is case-insensitive and anchored to the full repository name
/// (no substring matches).
///
/// # TOML example
///
/// ```toml
/// [repository_scope]
/// include_patterns = ["payments-*", "checkout"]
/// exclude_patterns = ["payments-legacy"]
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryScope {
    /// Repository name patterns that are in scope (OR semantics).
    ///
    /// An empty list means "no repositories are in scope" — this is a
    /// fail-closed default, distinct from `repository_scope` being absent
    /// entirely (which means "all repositories are in scope").
    pub include_patterns: Vec<String>,

    /// Repository name patterns that are excluded from scope (OR semantics).
    ///
    /// Checked after `include_patterns`; a repository matching both an
    /// include and an exclude pattern is excluded (exclude takes
    /// precedence).
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Internal deserialisation type for a single condition inside a conditional policy block.
///
/// Used only during TOML parsing; converted to [`PolicyCondition`] by `load_org_policy`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct PolicyConditionRaw {
    /// Repository topics that satisfy this condition (OR match).
    ///
    /// Comparison is case-insensitive.
    #[serde(default)]
    pub has_any_topic: Vec<String>,

    /// Custom property key-value pairs required for this condition (AND match).
    ///
    /// All entries must be present and equal (case-sensitive value comparison).
    #[serde(default)]
    pub has_custom_property: HashMap<String, String>,
}

/// Internal deserialisation type for one `[[conditional_policies]]` TOML block.
///
/// Converted to [`ConditionalPolicy`] by `load_org_policy`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConditionalPolicyRaw {
    /// Condition that must match for this block to apply.
    #[serde(default)]
    pub condition: PolicyConditionRaw,

    /// Enforced settings applied after the repo tier when the condition matches.
    #[serde(default)]
    pub enforced: OrgPolicySectionRaw,

    /// Default settings applied before the repo tier when the condition matches.
    #[serde(default)]
    pub defaults: OrgPolicySectionRaw,
}

/// Condition used to determine whether a [`ConditionalPolicy`] block applies to a repository.
///
/// A condition matches when **all** of its non-empty criteria are satisfied simultaneously
/// (AND semantics across criteria). Within `has_any_topic` the match uses OR semantics:
/// at least one listed topic must be present.
///
/// An empty condition (no criteria) always matches.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use merge_warden_core::config::PolicyCondition;
/// use merge_warden_developer_platforms::models::RepositoryContext;
///
/// // Matches any repo with the "payments" topic.
/// let cond = PolicyCondition {
///     has_any_topic: vec!["payments".to_string()],
///     has_custom_property: HashMap::new(),
/// };
/// let ctx = RepositoryContext {
///     topics: vec!["payments".to_string(), "backend".to_string()],
///     custom_properties: HashMap::new(),
/// };
/// assert!(cond.matches(&ctx));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyCondition {
    /// Repository must have at least one of these topics (case-insensitive, OR semantics).
    ///
    /// An empty list means "any topics" — i.e., this criterion is considered satisfied.
    #[serde(default)]
    pub has_any_topic: Vec<String>,

    /// Repository must have ALL of these custom properties with the specified values
    /// (AND semantics, case-sensitive value comparison).
    ///
    /// An empty map means "any properties" — i.e., this criterion is considered satisfied.
    #[serde(default)]
    pub has_custom_property: HashMap<String, String>,
}

impl PolicyCondition {
    /// Returns `true` if this condition matches the given [`RepositoryContext`].
    ///
    /// Matching rules:
    /// - `has_any_topic`: satisfied when empty **or** when at least one entry
    ///   matches a repository topic (case-insensitive comparison).
    /// - `has_custom_property`: satisfied when empty **or** when **all** entries
    ///   are present in the repository's custom properties with equal values
    ///   (case-sensitive value comparison).
    /// - Both criteria must be satisfied simultaneously (AND semantics).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use merge_warden_core::config::PolicyCondition;
    /// use merge_warden_developer_platforms::models::RepositoryContext;
    ///
    /// let cond = PolicyCondition::default();
    /// let ctx = RepositoryContext::default();
    /// assert!(cond.matches(&ctx), "empty condition always matches");
    /// ```
    pub fn matches(&self, context: &RepositoryContext) -> bool {
        let topics_match = self.has_any_topic.is_empty()
            || self.has_any_topic.iter().any(|required| {
                context
                    .topics
                    .iter()
                    .any(|repo_topic| repo_topic.eq_ignore_ascii_case(required))
            });

        let props_match = self.has_custom_property.is_empty()
            || self
                .has_custom_property
                .iter()
                .all(|(key, expected_value)| {
                    context.custom_properties.get(key) == Some(expected_value)
                });

        topics_match && props_match
    }
}

/// A conditional org policy block that is applied only when its [`PolicyCondition`] matches.
///
/// Conditional policies are declared in the `[[conditional_policies]]` array in the org
/// policy TOML file. When a repository's context satisfies the condition, the block's
/// `defaults` and `enforced` [`PolicySet`] values are inserted into the merge chain:
///
/// ```text
/// app_defaults → org_defaults → conditional_defaults* → repo → conditional_enforced* → org_enforced → app_enforced
/// ```
///
/// Multiple matching blocks are merged in declaration order (first declaration wins for
/// non-default conflicting values).
///
/// # Examples
///
/// TOML:
///
/// ```toml
/// [[conditional_policies]]
/// [conditional_policies.condition]
/// has_any_topic = ["payments"]
///
/// [conditional_policies.enforced.policies.title]
/// valid_title_regex = "^(feat|fix|chore)\\(.*\\):"
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConditionalPolicy {
    /// Condition that must match for this block to apply.
    pub condition: PolicyCondition,

    /// Settings that CANNOT be overridden by repo-level config.
    ///
    /// Inserted into the merge chain after the repo tier when the condition matches.
    pub enforced: PolicySet,

    /// Settings that CAN be overridden by repo-level config.
    ///
    /// Inserted into the merge chain before the repo tier when the condition matches.
    pub defaults: PolicySet,
}

/// Parsed and validated org-level policy.
///
/// Contains two [`PolicySet`] values at different precedence levels,
/// populated from the `[enforced]` and `[defaults]` sections of the
/// org policy TOML file.
///
/// When either section is absent from the TOML file, the corresponding
/// [`PolicySet`] is `PolicySet::default()`, which has no effect on the
/// merge chain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrgPolicy {
    /// Settings that CANNOT be overridden by repo-level config.
    ///
    /// Applied as the last merge tier before app-level enforcement flags.
    pub enforced: PolicySet,

    /// Settings that CAN be overridden by repo-level config.
    ///
    /// Applied between `app_defaults_ps` and `repo_ps` in the merge chain.
    pub defaults: PolicySet,

    /// Conditional policy blocks applied only when their condition matches the repository.
    ///
    /// Each block contributes its `defaults` before the repo tier and its `enforced`
    /// after the repo tier. Multiple matching blocks are merged in declaration order.
    pub conditional_policies: Vec<ConditionalPolicy>,
}

/// Internal deserialisation target for the org policy TOML root.
///
/// Uses [`OrgPolicySectionRaw`] for the two sections rather than
/// [`RepositoryProvidedConfig`] directly, to avoid requiring a nested
/// `schemaVersion` key inside each subsection.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct OrgPolicyRaw {
    /// Schema version — must be `1` for the policy to be accepted.
    ///
    /// No `#[serde(default)]` is applied intentionally: if the field is
    /// absent from the TOML file the deserialiser produces a generic
    /// "missing field" error rather than silently treating it as `0`.
    /// This forces policy file authors to declare a schema version
    /// explicitly and surfaces the omission as a clear error.
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,

    /// Enforced policy section (`[enforced]`).
    #[serde(default)]
    pub enforced: OrgPolicySectionRaw,

    /// Default policy section (`[defaults]`).
    #[serde(default)]
    pub defaults: OrgPolicySectionRaw,

    /// Conditional policy blocks (`[[conditional_policies]]`).
    ///
    /// Each block's condition is evaluated at runtime against the target repository's
    /// topics and custom properties. Blocks whose condition matches are merged into the
    /// policy chain for that PR.
    #[serde(default)]
    pub conditional_policies: Vec<ConditionalPolicyRaw>,
}

/// One section (`[enforced]` or `[defaults]`) of the org policy TOML.
///
/// Contains the same policy-relevant fields as [`RepositoryProvidedConfig`]
/// minus `schema_version`, so neither subsection requires a `schemaVersion`
/// key in the TOML.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct OrgPolicySectionRaw {
    /// Pull-request validation policies.
    #[serde(default, rename = "policies")]
    pub policies: PoliciesConfig,

    /// Change-type label configuration.
    #[serde(default)]
    pub change_type_labels: Option<ChangeTypeLabelConfig>,
}

/// Configuration for bypass rules allowing specific users to skip validation
///
/// Bypass rules allow designated users to bypass specific validation rules.
/// This is useful for automated systems, release processes, or emergency fixes
/// where normal validation requirements might need to be temporarily waived.
///
/// # Security Considerations
///
/// - Bypass rules should be used sparingly and only for trusted users
/// - All bypass decisions are logged for audit purposes
/// - Users are identified by their GitHub username (case-sensitive)
/// - Disabled rules will not bypass any validation
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::BypassRule;
///
/// // Allow release automation to bypass title validation
/// let title_bypass = BypassRule::new(
///     true,
///     vec!["release-bot".to_string(), "admin".to_string()]
/// );
///
/// // Disable work item bypass for all users
/// let work_item_bypass = BypassRule::new(false, vec![]);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BypassRule {
    /// Whether this bypass rule is enabled
    enabled: bool,

    /// List of GitHub usernames allowed to bypass this rule
    users: Vec<String>,
}

impl BypassRule {
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
    pub fn can_bypass_validation(&self, user: Option<&User>) -> bool {
        // If bypass is disabled, no one can bypass
        if !self.enabled {
            return false;
        }

        // If no user information is available, cannot bypass
        let user = match user {
            Some(u) => u,
            None => return false,
        };

        // Check if user is in the bypass list
        self.users
            .iter()
            .any(|bypass_user| bypass_user == &user.login)
    }

    /// Returns whether this bypass rule is enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Creates a new bypass rule with the specified settings
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the bypass rule should be active
    /// * `users` - List of GitHub usernames allowed to bypass validation
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::config::BypassRule;
    ///
    /// let rule = BypassRule::new(true, vec!["admin".to_string(), "bot".to_string()]);
    /// assert!(rule.enabled());
    /// assert_eq!(rule.users(), vec!["admin", "bot"]);
    /// ```
    pub fn new(enabled: bool, users: Vec<String>) -> Self {
        Self { enabled, users }
    }

    /// Returns the list of usernames allowed to bypass this rule
    ///
    /// # Returns
    ///
    /// A vector of string references containing the allowed usernames
    pub fn users(&self) -> Vec<&str> {
        self.users.iter().map(|f| f.as_ref()).collect()
    }
}

/// Collection of all bypass rules for different validation types
///
/// This struct groups bypass rules for different validation categories,
/// allowing fine-grained control over which users can bypass which rules.
///
/// # Rule Categories
///
/// - `title_convention` - Bypass for pull request title format validation
/// - `work_items` - Bypass for work item reference validation
/// - `branch_protection` - Reserved for future branch protection bypasses
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::{BypassRules, BypassRule};
///
/// let bypass_rules = BypassRules::new(
///     BypassRule::new(true, vec!["release-bot".to_string()]),
///     BypassRule::new(true, vec!["hotfix-team".to_string()])
/// );
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BypassRules {
    /// Bypass rule for title convention validation
    #[serde(default)]
    title_convention: BypassRule,

    /// Bypass rule for work item validation
    #[serde(default)]
    work_items: BypassRule,

    /// Bypass rule for PR size validation
    #[serde(default)]
    size: BypassRule,
}

impl BypassRules {
    /// Creates a new BypassRules configuration with title and work item rules
    ///
    /// # Arguments
    ///
    /// * `title_convention` - Bypass rule for title validation
    /// * `work_items` - Bypass rule for work item validation
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::config::{BypassRules, BypassRule};
    ///
    /// let title_rule = BypassRule::new(true, vec!["admin".to_string()]);
    /// let work_rule = BypassRule::new(false, vec![]);
    /// let rules = BypassRules::new(title_rule, work_rule);
    /// ```
    pub fn new(title_convention: BypassRule, work_items: BypassRule) -> Self {
        Self {
            title_convention,
            work_items,
            size: BypassRule::default(),
        }
    }

    /// Creates a new BypassRules configuration with all three rule types
    ///
    /// # Arguments
    ///
    /// * `title_convention` - Bypass rule for title validation
    /// * `work_items` - Bypass rule for work item validation
    /// * `size` - Bypass rule for PR size validation
    pub fn new_with_size(
        title_convention: BypassRule,
        work_items: BypassRule,
        size: BypassRule,
    ) -> Self {
        Self {
            title_convention,
            work_items,
            size,
        }
    }

    /// Returns the bypass rule for title convention validation
    pub fn title_convention(&self) -> &BypassRule {
        &self.title_convention
    }

    /// Returns the bypass rule for work item validation
    pub fn work_item_convention(&self) -> &BypassRule {
        &self.work_items
    }

    /// Returns the bypass rule for PR size validation
    pub fn size(&self) -> &BypassRule {
        &self.size
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// For each sub-rule (`title_convention`, `work_items`, `size`):
    /// use the `over` sub-rule if it has been explicitly configured (its user list
    /// is non-empty, or its `enabled` flag differs from the default `false`;
    /// otherwise keep `base`'s sub-rule.
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.8 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        // A sub-rule is "explicitly configured" when its enabled flag is set or
        // it names at least one user.  An unconfigured `over` sub-rule defers to
        // the corresponding `base` sub-rule.
        fn is_configured(rule: &BypassRule) -> bool {
            rule.enabled || !rule.users.is_empty()
        }

        Self {
            title_convention: if is_configured(&over.title_convention) {
                over.title_convention.clone()
            } else {
                base.title_convention.clone()
            },
            work_items: if is_configured(&over.work_items) {
                over.work_items.clone()
            } else {
                base.work_items.clone()
            },
            size: if is_configured(&over.size) {
                over.size.clone()
            } else {
                base.size.clone()
            },
        }
    }
}

/// Bypass-rule configuration used at multiple policy tiers.
///
/// This struct is shared by repository-level (`[policies.bypassRules.*]` in
/// `.github/merge-warden.toml`), org-level defaults (`[defaults.policies.bypassRules.*]`),
/// and org-level enforced overrides (`[enforced.policies.bypassRules.*]`) in the
/// org-policy file.
///
/// Each sub-field is `Option<BypassRule>`.  When a sub-table is absent the field is
/// `None`, and callers fall back to the next tier in the merge chain for that rule.
/// This allows any tier to override individual categories without silently discarding
/// the defaults from lower-priority tiers for the others.
///
/// # Examples
///
/// ```toml
/// # Override title bypass only; work_items and size inherit defaults.
/// [policies.bypassRules.title_convention]
/// enabled = true
/// users = ["release-bot"]
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BypassRulesConfig {
    /// Per-repo override for title-convention bypass.
    /// `None` means the server-level default is used for this rule.
    #[serde(default)]
    title_convention: Option<BypassRule>,

    /// Per-repo override for work-item bypass.
    /// `None` means the server-level default is used for this rule.
    #[serde(default)]
    work_items: Option<BypassRule>,

    /// Per-repo override for PR-size bypass.
    /// `None` means the server-level default is used for this rule.
    #[serde(default)]
    size: Option<BypassRule>,
}

impl BypassRulesConfig {
    /// Creates a [`BypassRulesConfig`] from an already-merged [`BypassRules`].
    ///
    /// All three sub-rules are stored as `Some(...)` so that
    /// [`RepositoryProvidedConfig::to_validation_config`] uses the pre-merged values
    /// and the server-level-default fallback parameter has no effect.
    pub(crate) fn from_merged(rules: &BypassRules) -> Self {
        Self {
            title_convention: Some(rules.title_convention().clone()),
            work_items: Some(rules.work_item_convention().clone()),
            size: Some(rules.size().clone()),
        }
    }

    /// Returns the per-repo title-convention bypass rule, if configured.
    pub fn title_convention(&self) -> Option<&BypassRule> {
        self.title_convention.as_ref()
    }

    /// Returns the per-repo work-item bypass rule, if configured.
    pub fn work_item_convention(&self) -> Option<&BypassRule> {
        self.work_items.as_ref()
    }

    /// Returns the per-repo size bypass rule, if configured.
    pub fn size(&self) -> Option<&BypassRule> {
        self.size.as_ref()
    }

    /// Converts this config into a [`BypassRules`] value.
    ///
    /// Each sub-rule that is present is used directly; absent sub-rules become
    /// [`BypassRule::default()`] (disabled, no users) so they register as
    /// "unconfigured" and let the higher-priority merge tier win.
    ///
    /// Use together with [`Option::map`] and [`unwrap_or_default`] to handle
    /// an absent `bypassRules` section:
    ///
    /// ```ignore
    /// let opt: Option<BypassRulesConfig> = None;
    /// let rules: BypassRules = opt.as_ref().map(BypassRulesConfig::to_bypass_rules).unwrap_or_default();
    /// ```
    ///
    /// [`unwrap_or_default`]: Option::unwrap_or_default
    pub(crate) fn to_bypass_rules(&self) -> BypassRules {
        BypassRules::new_with_size(
            self.title_convention().cloned().unwrap_or_default(),
            self.work_item_convention().cloned().unwrap_or_default(),
            self.size().cloned().unwrap_or_default(),
        )
    }
}

/// Configuration for propagating issue metadata to pull requests.
///
/// Both flags default to `false`. Teams that do not use GitHub Milestones or
/// Projects v2, or that track issues in an external system, should leave both
/// flags disabled (or omit `[policies.pullRequests.issuePropagation]` entirely).
/// When both are `false`, no additional GitHub API calls are made beyond what
/// the existing work-item reference check already performs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssuePropagationConfig {
    /// When `true`, copy the milestone from the first closing-keyword issue
    /// reference in the PR body onto the pull request.
    ///
    /// No-op when the referenced issue has no milestone, or when the PR
    /// already has the same milestone. Overwrites an existing PR milestone
    /// if it differs from the issue's.
    #[serde(default = "IssuePropagationConfig::default_false")]
    pub sync_milestone_from_issue: bool,

    /// When `true`, add the pull request to every Projects v2 project that
    /// the referenced issue belongs to.
    ///
    /// No-op when the referenced issue has no linked projects. This feature
    /// requires github-bot-sdk support for GraphQL project operations;
    /// see the SDK issue filed against pvandervelde/github-bot-sdk.
    #[serde(default = "IssuePropagationConfig::default_false")]
    pub sync_project_from_issue: bool,
}

impl IssuePropagationConfig {
    /// Returns `false` — used as the serde default for both flags.
    fn default_false() -> bool {
        false
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `sync_milestone_from_issue`: `base.sync_milestone_from_issue || over.sync_milestone_from_issue`
    /// - `sync_project_from_issue`: `base.sync_project_from_issue || over.sync_project_from_issue`
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.6 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        Self {
            sync_milestone_from_issue: base.sync_milestone_from_issue
                || over.sync_milestone_from_issue,
            sync_project_from_issue: base.sync_project_from_issue || over.sync_project_from_issue,
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

    /// Configuration for PR size checking
    pub pr_size_check: PrSizeCheckConfig,

    /// Configuration for intelligent change type label detection
    pub change_type_labels: Option<ChangeTypeLabelConfig>,

    /// Configuration for WIP (Work In Progress) detection and blocking
    pub wip_check: WipCheckConfig,

    /// Configuration for state-based PR lifecycle labels
    pub pr_state_labels: PrStateLabelsConfig,

    /// Configuration for Renovate stability-days label management.
    pub renovate_stability: RenovateStabilityConfig,

    /// Rules for bypassing validation checks
    pub bypass_rules: BypassRules,

    /// Configuration for issue metadata propagation.
    pub issue_propagation: IssuePropagationConfig,

    /// Bot mention prefix used to parse label suppression commands from PR comments.
    pub bot_mention: String,
}

impl CurrentPullRequestValidationConfiguration {
    /// Constructs a baseline [`CurrentPullRequestValidationConfiguration`] from
    /// application defaults alone, without any repo or org overrides.
    ///
    /// Used as the fallback when [`resolve_pull_request_config`] fails, replacing
    /// the large inline struct construction previously found in platform handlers.
    ///
    /// The four app-level enforcement flags (`enable_title_validation`,
    /// `enable_work_item_validation`, `pr_size_check.enabled`,
    /// `wip_check.enforce_wip_blocking`) are applied here so that the fallback
    /// path honours operator-configured enforcement even in error scenarios.
    pub fn from_app_defaults(app: &ApplicationDefaults) -> Self {
        Self {
            enforce_title_convention: app.enable_title_validation,
            title_pattern: app.default_title_pattern.clone(),
            invalid_title_label: app.default_invalid_title_label.clone(),
            enforce_work_item_references: app.enable_work_item_validation,
            work_item_reference_pattern: app.default_work_item_pattern.clone(),
            missing_work_item_label: app.default_missing_work_item_label.clone(),
            pr_size_check: app.pr_size_check.clone(),
            change_type_labels: Some(app.change_type_labels.clone()),
            wip_check: app.wip_check.clone(),
            pr_state_labels: app.pr_state_labels.clone(),
            renovate_stability: app.renovate_stability.clone(),
            bypass_rules: app.bypass_rules.clone(),
            issue_propagation: IssuePropagationConfig::default(),
            bot_mention: app.bot_mention.clone(),
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    fn new(
        enforce_title_convention: bool,
        title_pattern: Option<String>,
        invalid_title_label: Option<String>,
        enforce_work_item_references: bool,
        work_item_reference_pattern: Option<String>,
        missing_work_item_label: Option<String>,
        pr_size_check: Option<PrSizeCheckConfig>,
        bypass_rules: Option<BypassRules>,
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
            pr_size_check: pr_size_check.unwrap_or_default(),
            change_type_labels: None, // Use default behavior for tests
            wip_check: WipCheckConfig::default(),
            pr_state_labels: PrStateLabelsConfig::default(),
            renovate_stability: RenovateStabilityConfig::default(),
            bypass_rules: bypass_rules.unwrap_or_default(),
            issue_propagation: IssuePropagationConfig::default(),
            bot_mention: "@merge-warden".to_string(),
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
            pr_size_check: PrSizeCheckConfig::default(),
            change_type_labels: None, // Default to None, will be populated from app defaults
            wip_check: WipCheckConfig::default(),
            pr_state_labels: PrStateLabelsConfig::default(),
            renovate_stability: RenovateStabilityConfig::default(),
            bypass_rules: BypassRules::default(),
            issue_propagation: IssuePropagationConfig::default(),
            bot_mention: "@merge-warden".to_string(),
        }
    }
}

/// Policies configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PoliciesConfig {
    /// Per-repository bypass-rule overrides parsed from `[policies.bypassRules.*]`.
    ///
    /// When `Some`, individual sub-rules that are present override the corresponding
    /// server-level defaults; sub-rules that are absent inherit the server defaults.
    /// When `None` (i.e. the entire `bypassRules` section is missing from the TOML),
    /// all server-level bypass rules are used unchanged.
    #[serde(default, rename = "bypassRules")]
    pub bypass_rules: Option<BypassRulesConfig>,

    /// Configuration for pull request validation policies
    #[serde(default, rename = "pullRequests")]
    pub pull_requests: PullRequestsPoliciesConfig,
}

/// Pull request policies configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PullRequestsPoliciesConfig {
    /// Configuration for pull request title validation policies
    #[serde(default, rename = "prTitle")]
    pub title_policies: PullRequestsTitlePolicyConfig,

    /// Configuration for work item reference validation policies
    #[serde(default, rename = "workItem")]
    pub work_item_policies: WorkItemPolicyConfig,

    /// Configuration for pull request size validation policies
    #[serde(default, rename = "prSize")]
    pub size_policies: PrSizeCheckConfig,

    /// Configuration for WIP detection and blocking policies
    #[serde(default, rename = "wip")]
    pub wip_policies: WipCheckConfig,

    /// Configuration for state-based PR lifecycle labels
    #[serde(default, rename = "prState")]
    pub pr_state_policies: PrStateLabelsConfig,

    /// Configuration for issue metadata propagation.
    #[serde(default, rename = "issuePropagation")]
    pub issue_propagation: IssuePropagationConfig,

    /// Configuration for Renovate stability-days label management.
    #[serde(default, rename = "renovateStability")]
    pub renovate_stability: RenovateStabilityConfig,
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

    /// Label to apply when the title doesn't match the required pattern
    #[serde(default = "PullRequestsTitlePolicyConfig::default_label")]
    pub label_if_missing: Option<String>,
}

impl PullRequestsTitlePolicyConfig {
    /// Default value for label when validation fails (None)
    fn default_label() -> Option<String> {
        None
    }

    /// Default regex pattern for title validation (conventional commits)
    fn default_pattern() -> String {
        CONVENTIONAL_COMMIT_REGEX.to_string()
    }

    /// Default value for title validation requirement (false)
    fn default_required() -> bool {
        false
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `required`: `base.required || over.required` (OR — once required by either tier, stays required)
    /// - `pattern`: `over.pattern` if non-empty and not equal to `CONVENTIONAL_COMMIT_REGEX`;
    ///   otherwise `base.pattern`
    /// - `label_if_missing`: `over.label_if_missing` if `Some`; otherwise `base.label_if_missing`
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.1 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        let pattern = if !over.pattern.is_empty() && over.pattern != CONVENTIONAL_COMMIT_REGEX {
            over.pattern.clone()
        } else {
            base.pattern.clone()
        };
        Self {
            required: base.required || over.required,
            pattern,
            label_if_missing: over
                .label_if_missing
                .clone()
                .or_else(|| base.label_if_missing.clone()),
        }
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
///
/// # Override the labels that are applied automatically based on keywords found in the PR title or body.
/// # All fields are optional; omitted fields fall back to the built-in defaults shown below.
/// [change_type_labels.keyword_labels]
/// breaking_change = "breaking-change"   # PR title contains `!:` or body contains "breaking change"
/// security = "security"                 # PR body contains "security" or "vulnerability"
/// hotfix = "hotfix"                     # PR body contains "hotfix"
/// tech_debt = "tech-debt"               # PR body contains "technical debt" or "tech debt"
/// ---- ----------- ----
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryProvidedConfig {
    /// Schema version for configuration compatibility
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,

    /// Validation policies for pull requests
    #[serde(default)]
    pub policies: PoliciesConfig,

    /// Repository-specific change type label configuration overrides
    #[serde(default)]
    pub change_type_labels: Option<ChangeTypeLabelConfig>,

    /// Bot mention prefix resolved from application defaults; not read from TOML.
    ///
    /// Set by [`load_merge_warden_config`] after deserialisation, from
    /// [`ApplicationDefaults::bot_mention`].
    #[serde(skip)]
    pub bot_mention: String,
}

/// Convert a RepositoryConfig (TOML) to a ValidationConfig (runtime enforcement)
impl RepositoryProvidedConfig {
    /// Converts repository configuration to validation configuration
    ///
    /// This method transforms the repository-level configuration (loaded from TOML)
    /// into the runtime validation configuration used by the merge warden engine.
    ///
    /// # Arguments
    ///
    /// * `bypass_rules` - Bypass rules to apply for validation exceptions
    ///
    /// # Returns
    ///
    /// A `CurrentPullRequestValidationConfiguration` ready for runtime use
    pub fn to_validation_config(
        &self,
        bypass_rules: &BypassRules,
    ) -> CurrentPullRequestValidationConfiguration {
        // For now, only support the main PR policies (title, work item, size)
        let pr_policies = &self.policies.pull_requests;

        let enforce_title_convention = pr_policies.title_policies.required;
        let title_pattern = pr_policies.title_policies.pattern.clone();
        let invalid_title_label = pr_policies.title_policies.label_if_missing.clone();

        let enforce_work_item_references = pr_policies.work_item_policies.required;
        let work_item_reference_pattern = pr_policies.work_item_policies.pattern.clone();
        let missing_work_item_label = pr_policies.work_item_policies.label_if_missing.clone();

        let pr_size_check = pr_policies.size_policies.clone();
        let wip_check = pr_policies.wip_policies.clone();
        let pr_state_labels = pr_policies.pr_state_policies.clone();

        CurrentPullRequestValidationConfiguration {
            enforce_title_convention,
            title_pattern,
            invalid_title_label,
            enforce_work_item_references,
            work_item_reference_pattern,
            missing_work_item_label,
            pr_size_check,
            change_type_labels: self.change_type_labels.clone(),
            wip_check,
            pr_state_labels,
            renovate_stability: pr_policies.renovate_stability.clone(),
            // Merge per-sub-rule: if the repo specified a particular bypass rule,
            // use it; otherwise fall back to the server-level default for that rule.
            // This prevents a repo that overrides only one category from silently
            // discarding the server defaults for the other two categories.
            bypass_rules: {
                let repo = self.policies.bypass_rules.as_ref();
                let effective_title = repo
                    .and_then(|r| r.title_convention().cloned())
                    .unwrap_or_else(|| bypass_rules.title_convention().clone());
                let effective_work_items = repo
                    .and_then(|r| r.work_item_convention().cloned())
                    .unwrap_or_else(|| bypass_rules.work_item_convention().clone());
                let effective_size = repo
                    .and_then(|r| r.size().cloned())
                    .unwrap_or_else(|| bypass_rules.size().clone());
                BypassRules::new_with_size(effective_title, effective_work_items, effective_size)
            },
            issue_propagation: pr_policies.issue_propagation.clone(),
            bot_mention: self.bot_mention.clone(),
        }
    }
}

impl Default for RepositoryProvidedConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            policies: PoliciesConfig::default(),
            change_type_labels: None,
            bot_mention: "@merge-warden".to_string(),
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

    /// Label to apply when work item reference is missing
    #[serde(default = "WorkItemPolicyConfig::default_label")]
    pub label_if_missing: Option<String>,
}

impl WorkItemPolicyConfig {
    /// Default value for label when work item is missing (None)
    fn default_label() -> Option<String> {
        None
    }

    /// Default regex pattern for work item validation
    fn default_pattern() -> String {
        WORK_ITEM_REGEX.to_string()
    }

    /// Default value for work item validation requirement (false)
    fn default_required() -> bool {
        false
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `required`: `base.required || over.required`
    /// - `pattern`: `over.pattern` if non-empty and not equal to `WORK_ITEM_REGEX`;
    ///   otherwise `base.pattern`
    /// - `label_if_missing`: `over.label_if_missing` if `Some`; otherwise `base.label_if_missing`
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.2 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        let pattern = if !over.pattern.is_empty() && over.pattern != WORK_ITEM_REGEX {
            over.pattern.clone()
        } else {
            base.pattern.clone()
        };
        Self {
            required: base.required || over.required,
            pattern,
            label_if_missing: over
                .label_if_missing
                .clone()
                .or_else(|| base.label_if_missing.clone()),
        }
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

/// Configuration for PR size policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrSizeCheckConfig {
    /// Whether PR size checking is enabled
    #[serde(default = "PrSizeCheckConfig::default_enabled")]
    pub enabled: bool,

    /// Custom size thresholds (optional - uses defaults if not specified)
    #[serde(default)]
    pub thresholds: Option<SizeThresholds>,

    /// Whether to fail the check for oversized PRs (XXL category)
    #[serde(default = "PrSizeCheckConfig::default_fail_on_oversized")]
    pub fail_on_oversized: bool,

    /// File patterns to exclude from size calculations (e.g., ["*.md", "*.txt"])
    #[serde(default)]
    pub excluded_file_patterns: Vec<String>,

    /// Label prefix for size labels (defaults to "size/")
    #[serde(default = "PrSizeCheckConfig::default_label_prefix")]
    pub label_prefix: String,

    /// Whether to add educational comments for oversized PRs
    #[serde(default = "PrSizeCheckConfig::default_add_comment")]
    pub add_comment: bool,

    /// Whether to ignore deleted lines when calculating PR size.
    ///
    /// When `true`, only additions are counted towards the PR size category. This
    /// prevents large file deletions (e.g., removing a generated file) from
    /// inflating the size category unfairly.
    ///
    /// Defaults to `false` (additions + deletions counted, preserving historical
    /// behaviour).
    #[serde(default = "PrSizeCheckConfig::default_ignore_deletions")]
    pub ignore_deletions: bool,
}

impl PrSizeCheckConfig {
    /// Default value for size check enablement (false)
    fn default_enabled() -> bool {
        false
    }

    /// Default value for failing on oversized PRs (false)
    fn default_fail_on_oversized() -> bool {
        false
    }

    /// Default prefix for size labels ("size/")
    fn default_label_prefix() -> String {
        "size/".to_string()
    }

    /// Default value for adding educational comments (true)
    fn default_add_comment() -> bool {
        true
    }

    /// Default value for ignore_deletions (false — count both additions and deletions)
    fn default_ignore_deletions() -> bool {
        false
    }

    /// Get the effective size thresholds, using defaults if not configured
    pub fn get_effective_thresholds(&self) -> SizeThresholds {
        self.thresholds.clone().unwrap_or_default()
    }

    /// Check if a file should be excluded from size calculations
    pub fn should_exclude_file(&self, file_path: &str) -> bool {
        if self.excluded_file_patterns.is_empty() {
            return false;
        }

        // Simple glob-like pattern matching
        for pattern in &self.excluded_file_patterns {
            if pattern_matches(pattern, file_path) {
                return true;
            }
        }
        false
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - `fail_on_oversized`: `over` wins unconditionally
    /// - `thresholds`: `over.thresholds` if `Some`; otherwise `base.thresholds`
    /// - `excluded_file_patterns`: `over` if non-empty; otherwise `base`
    /// - `label_prefix`: `over.label_prefix` if not equal to `"size/"`; otherwise `base.label_prefix`
    /// - `add_comment`: `over` wins unconditionally
    /// - `ignore_deletions`: `over` wins unconditionally
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.3 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        let label_prefix = if over.label_prefix != Self::default_label_prefix() {
            over.label_prefix.clone()
        } else {
            base.label_prefix.clone()
        };
        let excluded_file_patterns = if !over.excluded_file_patterns.is_empty() {
            over.excluded_file_patterns.clone()
        } else {
            base.excluded_file_patterns.clone()
        };
        Self {
            enabled: base.enabled || over.enabled,
            fail_on_oversized: over.fail_on_oversized,
            thresholds: over.thresholds.clone().or_else(|| base.thresholds.clone()),
            excluded_file_patterns,
            label_prefix,
            add_comment: over.add_comment,
            ignore_deletions: over.ignore_deletions,
        }
    }
}

impl Default for PrSizeCheckConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            thresholds: None,
            fail_on_oversized: Self::default_fail_on_oversized(),
            excluded_file_patterns: Vec::new(),
            label_prefix: Self::default_label_prefix(),
            add_comment: Self::default_add_comment(),
            ignore_deletions: Self::default_ignore_deletions(),
        }
    }
}

/// Configuration for WIP (Work In Progress) detection and blocking.
///
/// When WIP blocking is enabled, pull requests whose title or description
/// match any of the configured patterns are blocked from merging. WIP blocking
/// cannot be bypassed by any user.
///
/// # Merge heuristic
///
/// `WipCheckConfig::merge` treats a pattern value equal to `WipCheckConfig::default()`
/// as "not configured" and falls back to the base tier. A repository that explicitly
/// sets its patterns to values identical to the defaults will therefore see the
/// app-level defaults applied instead. This is intentional (operator can supply a
/// richer default set), but may be surprising in edge cases. Document any repo-level
/// override explicitly to avoid ambiguity.
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::WipCheckConfig;
///
/// let config = WipCheckConfig::default();
/// assert!(!config.enforce_wip_blocking);
/// assert_eq!(config.wip_label, Some("WIP".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WipCheckConfig {
    /// Whether WIP detection and blocking is enabled
    #[serde(default = "WipCheckConfig::default_enforce")]
    pub enforce_wip_blocking: bool,

    /// The label to apply to WIP pull requests. Set to `None` (or omit the key
    /// in TOML) to disable WIP labeling. An empty string is treated the same as
    /// `None` — no label will be applied.
    #[serde(default = "WipCheckConfig::default_wip_label")]
    pub wip_label: Option<String>,

    /// Substrings to search for in the PR title to detect WIP status (case-sensitive)
    #[serde(default = "WipCheckConfig::default_title_patterns")]
    pub wip_title_patterns: Vec<String>,

    /// Substrings to search for in the PR description to detect WIP status (case-sensitive)
    #[serde(default)]
    pub wip_description_patterns: Vec<String>,
}

impl WipCheckConfig {
    /// Default value for `enforce_wip_blocking` (false — opt-in)
    fn default_enforce() -> bool {
        false
    }

    /// Default WIP label applied to WIP pull requests
    fn default_wip_label() -> Option<String> {
        Some("WIP".to_string())
    }

    /// Default WIP title patterns covering common conventions.
    ///
    /// Pattern matching uses `str::contains` (case-sensitive), so a pattern that
    /// is a substring of another pattern in the list will always match first,
    /// making the longer pattern redundant. For example, `"WIP"` already matches
    /// titles containing `"[WIP]"` or `"WIP:"`. The defaults below list only
    /// patterns that are _not_ subsumed by another entry.
    fn default_title_patterns() -> Vec<String> {
        vec![
            "WIP".to_string(),
            "wip:".to_string(),
            "[wip]".to_string(),
            "draft:".to_string(),
            "Draft:".to_string(),
        ]
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `enforce_wip_blocking`: `base.enforce_wip_blocking || over.enforce_wip_blocking`
    /// - `wip_label`: `over.wip_label` if it differs from `WipCheckConfig::default().wip_label`;
    ///   otherwise `base.wip_label`
    /// - `wip_title_patterns`: `over` if it differs from `WipCheckConfig::default().wip_title_patterns`;
    ///   otherwise `base`
    /// - `wip_description_patterns`: `over` if non-empty; otherwise `base`
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.4 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        let default = Self::default();
        let wip_label = if over.wip_label != default.wip_label {
            over.wip_label.clone()
        } else {
            base.wip_label.clone()
        };
        let wip_title_patterns = if over.wip_title_patterns != default.wip_title_patterns {
            over.wip_title_patterns.clone()
        } else {
            base.wip_title_patterns.clone()
        };
        let wip_description_patterns = if !over.wip_description_patterns.is_empty() {
            over.wip_description_patterns.clone()
        } else {
            base.wip_description_patterns.clone()
        };
        Self {
            enforce_wip_blocking: base.enforce_wip_blocking || over.enforce_wip_blocking,
            wip_label,
            wip_title_patterns,
            wip_description_patterns,
        }
    }
}

impl Default for WipCheckConfig {
    fn default() -> Self {
        Self {
            enforce_wip_blocking: Self::default_enforce(),
            wip_label: Self::default_wip_label(),
            wip_title_patterns: Self::default_title_patterns(),
            wip_description_patterns: Vec::new(),
        }
    }
}

/// Configuration for state-based PR lifecycle labels.
///
/// When enabled, exactly one label from `{draft_label, review_label, approved_label}`
/// is active on the PR at any time. Labels are automatically applied and removed as the
/// PR moves through its lifecycle. Setting any label to `None` disables labeling for
/// that state without affecting the other states.
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::PrStateLabelsConfig;
///
/// let config = PrStateLabelsConfig::default();
/// assert!(!config.enabled);
/// assert!(config.draft_label.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrStateLabelsConfig {
    /// Whether lifecycle label management is enabled for this repository.
    #[serde(default = "PrStateLabelsConfig::default_enabled")]
    pub enabled: bool,

    /// Label applied when the PR is in draft state.
    /// Set to `None` (or omit the key) to disable labeling for this state.
    #[serde(default)]
    pub draft_label: Option<String>,

    /// Label applied when the PR is open and awaiting review (not draft, not approved).
    /// Set to `None` (or omit the key) to disable labeling for this state.
    #[serde(default)]
    pub review_label: Option<String>,

    /// Label applied when the PR has at least one approved review and is not a draft.
    /// Set to `None` (or omit the key) to disable labeling for this state.
    #[serde(default)]
    pub approved_label: Option<String>,
}

impl PrStateLabelsConfig {
    /// Default value for `enabled` — opt-in, disabled by default.
    fn default_enabled() -> bool {
        false
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - `draft_label`: `over.draft_label` if `Some`; otherwise `base.draft_label`
    /// - `review_label`: `over.review_label` if `Some`; otherwise `base.review_label`
    /// - `approved_label`: `over.approved_label` if `Some`; otherwise `base.approved_label`
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.5 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        Self {
            enabled: base.enabled || over.enabled,
            draft_label: over
                .draft_label
                .clone()
                .or_else(|| base.draft_label.clone()),
            review_label: over
                .review_label
                .clone()
                .or_else(|| base.review_label.clone()),
            approved_label: over
                .approved_label
                .clone()
                .or_else(|| base.approved_label.clone()),
        }
    }
}

impl Default for PrStateLabelsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            draft_label: None,
            review_label: None,
            approved_label: None,
        }
    }
}

/// Configuration for Renovate stability-days label management.
///
/// When enabled, [`RENOVATE_STABILITY_LABEL`] (or the configured label name) is applied
/// to the PR while the `renovate/stability-days` commit status is `pending`, `error`, or
/// `failure`, and removed when the status is `success`.  The label is purely informational —
/// it never influences the merge-blocking check conclusion.
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::{RenovateStabilityConfig, RENOVATE_STABILITY_LABEL};
///
/// let config = RenovateStabilityConfig::default();
/// assert!(config.enabled);
/// assert_eq!(config.pending_stability_label, RENOVATE_STABILITY_LABEL);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenovateStabilityConfig {
    /// Whether Renovate stability label management is enabled.
    ///
    /// Defaults to `true`.  The merge rule is `base || over`: once either tier enables
    /// the feature the merged value is always `true`.  A repository can only opt in,
    /// not opt out.
    #[serde(default = "RenovateStabilityConfig::default_enabled")]
    pub enabled: bool,

    /// Label applied while the Renovate stability period has not yet elapsed.
    ///
    /// Defaults to [`RENOVATE_STABILITY_LABEL`].
    #[serde(default = "RenovateStabilityConfig::default_label")]
    pub pending_stability_label: String,
}

impl RenovateStabilityConfig {
    /// Default value for `enabled` — opt-in enabled by default.
    fn default_enabled() -> bool {
        true
    }

    /// Default value for `pending_stability_label`.
    fn default_label() -> String {
        RENOVATE_STABILITY_LABEL.to_string()
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled` — once enabled in either tier, stays enabled.
    ///   This means a repo-level `enabled = false` cannot override an app-level `enabled = true`.
    ///   To disable for all repos, set `enabled = false` at the application defaults level.
    /// - `pending_stability_label`: `over` wins if non-empty and differs from the default label;
    ///   otherwise `base` is used
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        let label = if !over.pending_stability_label.is_empty()
            && over.pending_stability_label != RENOVATE_STABILITY_LABEL
        {
            over.pending_stability_label.clone()
        } else {
            base.pending_stability_label.clone()
        };
        Self {
            enabled: base.enabled || over.enabled,
            pending_stability_label: label,
        }
    }
}

impl Default for RenovateStabilityConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            pending_stability_label: Self::default_label(),
        }
    }
}

/// A resolved, merged set of validation policies ready for enforcement.
///
/// `PolicySet` is the single value passed to the validation engine. It is
/// constructed by merging application-level defaults with any repository-provided
/// overrides and represents the **final, authoritative** configuration for a
/// single pull-request evaluation cycle.
///
/// # Construction
///
/// Callers should not build `PolicySet` by hand. Use:
/// - [`PolicySet::from_application_defaults`] to create a baseline from
///   [`ApplicationDefaults`].
/// - [`PolicySet::from_repository_config`] to create a set from a
///   [`RepositoryProvidedConfig`] alone.
/// - [`PolicySet::merge`] to combine two sets, letting the *over* (higher-priority)
///   set win on a field-by-field basis.
///
/// See `docs/spec/interfaces/policy-engine.md` §1 for the full contract and merge
/// semantics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolicySet {
    /// Title-format validation policy.
    pub title: PullRequestsTitlePolicyConfig,
    /// Work-item reference validation policy.
    pub work_item: WorkItemPolicyConfig,
    /// PR size classification and labelling policy.
    pub size: PrSizeCheckConfig,
    /// WIP detection and blocking policy.
    pub wip: WipCheckConfig,
    /// Lifecycle state labelling policy.
    pub pr_state: PrStateLabelsConfig,
    /// Renovate stability-days label management policy.
    pub renovate_stability: RenovateStabilityConfig,
    /// Issue-to-PR field propagation policy.
    pub issue_propagation: IssuePropagationConfig,
    /// Conventional-commit type → label mapping policy.
    pub change_type_labels: ChangeTypeLabelConfig,
    /// Per-category bypass allow-lists.
    pub bypass_rules: BypassRules,
}

impl PolicySet {
    /// Merges `over` on top of `self` (lower-priority baseline).
    ///
    /// Each constituent config field is merged independently using the field's
    /// own `merge` method.  The result contains `over`'s values wherever it
    /// carries an explicit non-default override, and `self`'s values elsewhere.
    ///
    /// # Arguments
    /// * `over` — Higher-priority policy set (typically repository-provided config)
    ///
    /// # Returns
    /// A new [`PolicySet`] with all fields merged.
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §1.1 for field-level rules.
    pub fn merge(&self, over: &PolicySet) -> PolicySet {
        PolicySet {
            title: PullRequestsTitlePolicyConfig::merge(&self.title, &over.title),
            work_item: WorkItemPolicyConfig::merge(&self.work_item, &over.work_item),
            size: PrSizeCheckConfig::merge(&self.size, &over.size),
            wip: WipCheckConfig::merge(&self.wip, &over.wip),
            pr_state: PrStateLabelsConfig::merge(&self.pr_state, &over.pr_state),
            renovate_stability: RenovateStabilityConfig::merge(
                &self.renovate_stability,
                &over.renovate_stability,
            ),
            issue_propagation: IssuePropagationConfig::merge(
                &self.issue_propagation,
                &over.issue_propagation,
            ),
            change_type_labels: ChangeTypeLabelConfig::merge(
                &self.change_type_labels,
                &over.change_type_labels,
            ),
            bypass_rules: BypassRules::merge(&self.bypass_rules, &over.bypass_rules),
        }
    }

    /// Constructs a [`PolicySet`] from one section of the org policy TOML
    /// (`[enforced]` or `[defaults]`).
    ///
    /// Equivalent to [`from_repository_config`] but operates on
    /// [`OrgPolicySectionRaw`] instead of [`RepositoryProvidedConfig`].
    ///
    /// # Bypass rules
    ///
    /// [`BypassRules`] are supported in org policy sections (`[enforced]` and
    /// `[defaults]`). When a `[*.policies.bypassRules.*]` block is present in the
    /// org policy TOML, it is parsed and reflected in the returned [`PolicySet`].
    ///
    /// Merge semantics (handled by [`BypassRules::merge`] further up the call
    /// stack):
    ///
    /// - **`[defaults.policies.bypassRules.*]`** — org-wide defaults that individual
    ///   repositories *can* override with their own `.github/merge-warden.toml`.
    /// - **`[enforced.policies.bypassRules.*]`** — org-wide bypass rules that
    ///   repositories *cannot* remove. The enforced tier always wins during the
    ///   final merge.
    ///
    /// When `bypass_rules` is absent from the section, [`BypassRules::default()`]
    /// (i.e. no bypass rules, all sub-rules unconfigured) is returned so that
    /// absent fields do not interfere with the merge chain.
    ///
    /// [`from_repository_config`]: PolicySet::from_repository_config
    pub(crate) fn from_org_section(section: &OrgPolicySectionRaw) -> PolicySet {
        let pr = &section.policies.pull_requests;
        PolicySet {
            title: pr.title_policies.clone(),
            work_item: pr.work_item_policies.clone(),
            size: pr.size_policies.clone(),
            wip: pr.wip_policies.clone(),
            pr_state: pr.pr_state_policies.clone(),
            renovate_stability: pr.renovate_stability.clone(),
            issue_propagation: pr.issue_propagation.clone(),
            change_type_labels: section.change_type_labels.clone().unwrap_or_default(),
            bypass_rules: section
                .policies
                .bypass_rules
                .as_ref()
                .map(BypassRulesConfig::to_bypass_rules)
                .unwrap_or_default(),
        }
    }

    /// Constructs a [`PolicySet`] containing only the settings forced by the
    /// four app-level enforcement flags on [`ApplicationDefaults`].
    ///
    /// All other fields are `PolicySet::default()` so they do not override
    /// anything when this [`PolicySet`] is applied as the last merge tier in
    /// [`resolve_pull_request_config`].
    ///
    /// # Fields mapped
    ///
    /// - `enable_title_validation = true`  → `result.title.required = true`
    /// - `enable_work_item_validation = true` → `result.work_item.required = true`
    /// - `pr_size_check.enabled = true`    → `result.size.enabled = true`
    /// - `wip_check.enforce_wip_blocking = true` → `result.wip.enforce_wip_blocking = true`
    pub(crate) fn from_app_enforcement_flags(app: &ApplicationDefaults) -> PolicySet {
        let mut ps = PolicySet::default();
        if app.enable_title_validation {
            ps.title.required = true;
        }
        if app.enable_work_item_validation {
            ps.work_item.required = true;
        }
        if app.pr_size_check.enabled {
            ps.size.enabled = true;
        }
        if app.wip_check.enforce_wip_blocking {
            ps.wip.enforce_wip_blocking = true;
        }
        ps
    }

    /// Converts a fully-merged [`PolicySet`] into a
    /// [`CurrentPullRequestValidationConfiguration`].
    ///
    /// This replaces the write-back-into-[`RepositoryProvidedConfig`] +
    /// `to_validation_config` pattern used in [`load_merge_warden_config`].
    /// Called once per webhook event from [`resolve_pull_request_config`].
    ///
    /// # Arguments
    ///
    /// * `app_defaults` — needed for fields not covered by [`PolicySet`]
    ///   (`bot_mention`, and label fields that may be `None` in the policy).
    pub(crate) fn to_validation_config(
        &self,
        app_defaults: &ApplicationDefaults,
    ) -> CurrentPullRequestValidationConfiguration {
        CurrentPullRequestValidationConfiguration {
            enforce_title_convention: self.title.required,
            title_pattern: self.title.pattern.clone(),
            invalid_title_label: self.title.label_if_missing.clone(),
            enforce_work_item_references: self.work_item.required,
            work_item_reference_pattern: self.work_item.pattern.clone(),
            missing_work_item_label: self.work_item.label_if_missing.clone(),
            pr_size_check: self.size.clone(),
            change_type_labels: Some(self.change_type_labels.clone()),
            wip_check: self.wip.clone(),
            pr_state_labels: self.pr_state.clone(),
            renovate_stability: self.renovate_stability.clone(),
            bypass_rules: self.bypass_rules.clone(),
            issue_propagation: self.issue_propagation.clone(),
            bot_mention: app_defaults.bot_mention.clone(),
        }
    }

    /// Constructs a [`PolicySet`] from application-level defaults.
    ///
    /// # Arguments
    /// * `app` — Application defaults loaded at server start-up
    ///
    /// # Returns
    /// A [`PolicySet`] seeded with every field taken from the application defaults.
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §1.2 for mapping rules.
    pub fn from_application_defaults(app: &ApplicationDefaults) -> PolicySet {
        PolicySet {
            // Note: `app.enable_title_validation` is intentionally NOT applied here.
            // It is a post-merge enforcement override applied via `from_app_enforcement_flags`
            // in `resolve_pull_request_config`.
            title: PullRequestsTitlePolicyConfig {
                required: false,
                pattern: app.default_title_pattern.clone(),
                label_if_missing: app.default_invalid_title_label.clone(),
            },
            // Note: `app.enable_work_item_validation` is intentionally NOT applied here.
            // It is a post-merge enforcement override applied via `from_app_enforcement_flags`
            // in `resolve_pull_request_config`.
            work_item: WorkItemPolicyConfig {
                required: false,
                pattern: app.default_work_item_pattern.clone(),
                label_if_missing: app.default_missing_work_item_label.clone(),
            },
            size: app.pr_size_check.clone(),
            wip: app.wip_check.clone(),
            pr_state: app.pr_state_labels.clone(),
            renovate_stability: app.renovate_stability.clone(),
            // `ApplicationDefaults` carries no issue-propagation settings — issue propagation
            // is a repository-level opt-in feature, so the app tier always contributes
            // `IssuePropagationConfig::default()` (both flags `false`).
            issue_propagation: IssuePropagationConfig::default(),
            change_type_labels: app.change_type_labels.clone(),
            bypass_rules: app.bypass_rules.clone(),
        }
    }

    /// Constructs a [`PolicySet`] from a repository-provided configuration.
    ///
    /// # Arguments
    /// * `repo` — Config parsed from the `.github/merge-warden.toml` file
    ///
    /// # Returns
    /// A [`PolicySet`] seeded with every field taken from the repository config.
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §1.3 for mapping rules.
    pub fn from_repository_config(repo: &RepositoryProvidedConfig) -> PolicySet {
        let pr = &repo.policies.pull_requests;
        // Repo bypass rules live in `BypassRulesConfig` (where each sub-rule is
        // `Option<BypassRule>`). We convert each present sub-rule directly; absent
        // sub-rules become `BypassRule::default()` (disabled, no users) so they
        // register as "unconfigured" and let the app-defaults rule win during merge.
        let bypass_rules = repo
            .policies
            .bypass_rules
            .as_ref()
            .map(BypassRulesConfig::to_bypass_rules)
            .unwrap_or_default();
        PolicySet {
            title: pr.title_policies.clone(),
            work_item: pr.work_item_policies.clone(),
            size: pr.size_policies.clone(),
            wip: pr.wip_policies.clone(),
            pr_state: pr.pr_state_policies.clone(),
            renovate_stability: pr.renovate_stability.clone(),
            issue_propagation: pr.issue_propagation.clone(),
            change_type_labels: repo.change_type_labels.clone().unwrap_or_default(),
            bypass_rules,
        }
    }
}

/// Simple pattern matching for file exclusions
/// Supports basic glob patterns with * wildcards
pub(crate) fn pattern_matches(pattern: &str, file_path: &str) -> bool {
    // Convert glob pattern to regex-like matching
    if pattern.contains('*') {
        // Escape all regex metacharacters in the literal parts of the pattern
        // first, then replace the escaped `\*` with `.*` so that only `*`
        // acts as a wildcard and characters like `.` are treated literally.
        let regex_pattern = regex::escape(pattern).replace(r"\*", ".*");
        if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            return regex.is_match(file_path);
        }
    }

    // Exact match fallback
    pattern == file_path
}

// ---------------------------------------------------------------------------
// FR-009: Repository Scope Filtering
// ---------------------------------------------------------------------------

/// Compiles a `repository_scope` glob pattern into an anchored,
/// case-insensitive [`regex::Regex`].
///
/// # Pattern syntax
///
/// Only ASCII letters, digits, `-`, `_`, `.`, and the two wildcards `*` (any
/// sequence, including the empty sequence) and `?` (exactly one character)
/// are permitted. Any other character is rejected — the caller treats this
/// as "this pattern is invalid" rather than falling back to a partial or
/// substring match.
///
/// A literal `.` in the pattern is translated to an escaped `\.` in the
/// output regex (a literal dot), NOT left as the regex "any character"
/// metacharacter — only `*` and `?` act as wildcards here.
///
/// # Errors
/// Returns `Err(())` when `pattern` contains a disallowed character, or in
/// the (expected to be unreachable given the allow-list) case that the
/// translated regex fails to compile.
fn compile_repository_scope_pattern(pattern: &str) -> Result<regex::Regex, ()> {
    let mut translated = String::with_capacity(pattern.len() * 2);
    for ch in pattern.chars() {
        match ch {
            '*' => translated.push_str(".*"),
            '?' => translated.push('.'),
            '.' => translated.push_str(r"\."),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => translated.push(c),
            _ => return Err(()),
        }
    }

    let anchored = format!("^{}$", translated);
    regex::RegexBuilder::new(&anchored)
        .case_insensitive(true)
        .build()
        .map_err(|_| ())
}

/// Returns `true` if `repo_name` is in scope according to `scope`.
///
/// # Contract
/// - `scope == None` — every repository is in scope (full backward
///   compatibility with pre-FR-009 behaviour).
/// - `scope.include_patterns` empty — no repository is in scope,
///   regardless of `exclude_patterns` (fail-closed).
/// - Otherwise — in scope iff `repo_name` matches at least one
///   `include_patterns` entry AND matches zero `exclude_patterns` entries.
///   `exclude_patterns` always takes precedence over a matching include
///   pattern.
///
/// Matching is case-insensitive and anchored to the full repository name.
/// Any pattern that fails to compile (see
/// [`compile_repository_scope_pattern`]) is treated as a non-match rather
/// than causing a panic — operators are expected to catch invalid patterns
/// at startup via [`validate_repository_scope_patterns`], not at
/// webhook-handling time.
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::{is_repository_in_scope, RepositoryScope};
///
/// let scope = Some(RepositoryScope {
///     include_patterns: vec!["payments-*".to_string()],
///     exclude_patterns: vec!["payments-legacy".to_string()],
/// });
/// assert!(is_repository_in_scope(&scope, "payments-api"));
/// assert!(!is_repository_in_scope(&scope, "payments-legacy"));
/// assert!(!is_repository_in_scope(&scope, "checkout"));
/// ```
pub fn is_repository_in_scope(scope: &Option<RepositoryScope>, repo_name: &str) -> bool {
    let scope = match scope {
        Some(scope) => scope,
        None => return true,
    };

    if scope.include_patterns.is_empty() {
        return false;
    }

    let is_included = scope.include_patterns.iter().any(|pattern| {
        compile_repository_scope_pattern(pattern)
            .map(|regex| regex.is_match(repo_name))
            .unwrap_or(false)
    });

    if !is_included {
        return false;
    }

    let is_excluded = scope.exclude_patterns.iter().any(|pattern| {
        compile_repository_scope_pattern(pattern)
            .map(|regex| regex.is_match(repo_name))
            .unwrap_or(false)
    });

    !is_excluded
}

/// Validates every pattern in `scope`'s `include_patterns` and
/// `exclude_patterns` (in that order) at startup.
///
/// Intended to be called once during configuration loading (see
/// `crates/server/src/config.rs::load_config`) so that a malformed pattern
/// fails fast with a clear error, rather than silently matching nothing (or
/// everything) at webhook-handling time.
///
/// # Errors
/// Returns `Err(ConfigLoadError::InvalidRepositoryScopePattern)` on the
/// FIRST pattern that fails to compile (fail fast) — it does not collect
/// every invalid pattern.
///
/// # Examples
///
/// ```
/// use merge_warden_core::config::{validate_repository_scope_patterns, RepositoryScope};
///
/// let scope = Some(RepositoryScope {
///     include_patterns: vec!["payments-*".to_string()],
///     exclude_patterns: vec![],
/// });
/// assert!(validate_repository_scope_patterns(&scope).is_ok());
/// ```
pub fn validate_repository_scope_patterns(
    scope: &Option<RepositoryScope>,
) -> Result<(), ConfigLoadError> {
    let scope = match scope {
        Some(scope) => scope,
        None => return Ok(()),
    };

    for pattern in scope
        .include_patterns
        .iter()
        .chain(scope.exclude_patterns.iter())
    {
        compile_repository_scope_pattern(pattern)
            .map_err(|_| ConfigLoadError::InvalidRepositoryScopePattern(pattern.clone()))?;
    }

    Ok(())
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
        Err(e) => {
            warn!(
                repository_owner = repo_owner,
                repository = repo_name,
                path = path_relative_to_repository_root,
                "Failed to find configuration file in repository"
            );
            return Err(ConfigLoadError::NotFound(e.to_string()));
        }
    };

    let mut config: RepositoryProvidedConfig = RepositoryProvidedConfig::default();
    let mut is_valid_config = true;
    if let Some(content) = potential_content {
        config = toml::from_str(&content)?;
        if config.schema_version != 1 {
            error!(
                repository_owner = repo_owner,
                repository = repo_name,
                path = path_relative_to_repository_root,
                config_version = config.schema_version,
                "Configuration in repository has an unexpected version. Will not be able to load configuration."
            );

            // If we can't load the configuration we just pretend it's not there
            config = RepositoryProvidedConfig::default();
            is_valid_config = false;
        }
    }

    // Only apply application defaults if we have a valid configuration
    if is_valid_config {
        // Build merged policy set: application defaults → repository overrides
        let app_ps = PolicySet::from_application_defaults(app_defaults);
        let repo_ps = PolicySet::from_repository_config(&config);
        let merged_ps = app_ps.merge(&repo_ps);

        // Write merged policies back into config for conversion to CPVRC
        config.policies.pull_requests.title_policies = merged_ps.title;
        config.policies.pull_requests.work_item_policies = merged_ps.work_item;
        config.policies.pull_requests.size_policies = merged_ps.size;
        config.policies.pull_requests.wip_policies = merged_ps.wip;
        config.policies.pull_requests.pr_state_policies = merged_ps.pr_state;
        config.policies.pull_requests.issue_propagation = merged_ps.issue_propagation;
        config.change_type_labels = Some(merged_ps.change_type_labels);
        // Write bypass_rules back so to_validation_config uses the merged result
        // rather than re-merging from the raw BypassRulesConfig sub-rules.
        config.policies.bypass_rules =
            Some(BypassRulesConfig::from_merged(&merged_ps.bypass_rules));

        // End of valid config processing
    }

    info!(
        enable_title_validation = config.policies.pull_requests.title_policies.required,
        title_validation_pattern = config.policies.pull_requests.title_policies.pattern,
        label_if_title_validation_fails = config
            .policies
            .pull_requests
            .title_policies
            .label_if_missing
            .clone()
            .unwrap_or_default(),
        enable_work_item_validation = config.policies.pull_requests.work_item_policies.required,
        work_item_validation_pattern = config
            .policies
            .pull_requests
            .work_item_policies
            .pattern
            .clone(),
        label_if_work_item_validation_fails = config
            .policies
            .pull_requests
            .work_item_policies
            .label_if_missing
            .clone()
            .unwrap_or_default(),
        enable_pr_size_checking = config.policies.pull_requests.size_policies.enabled,
        pr_size_fail_on_oversized = config
            .policies
            .pull_requests
            .size_policies
            .fail_on_oversized,
        pr_size_label_prefix = config.policies.pull_requests.size_policies.label_prefix,
        "Configuration loaded"
    );

    // Thread the bot_mention from application defaults into the resolved config
    // so that CurrentPullRequestValidationConfiguration carries it without requiring
    // callers to pass it separately.
    config.bot_mention = app_defaults.bot_mention.clone();

    Ok(config)
}

/// Fetches and parses the org-level policy file.
///
/// # Returns
///
/// - `Ok(Some(OrgPolicy))` — file exists, parses successfully, schema version is `1`.
/// - `Ok(None)` — file does not exist or any failure occurred when
///   `source.fail_if_unreachable` is `false`.
/// - `Err(ConfigLoadError::OrgPolicyUnavailable)` — fetch or parse error when
///   `source.fail_if_unreachable` is `true`.
///
/// # Arguments
///
/// * `source` — coordinates of the org policy file.
/// * `fetcher` — [`ConfigFetcher`] implementation (typically `GitHubProvider`).
pub(crate) async fn load_org_policy(
    source: &OrgPolicySource,
    fetcher: &dyn ConfigFetcher,
) -> Result<Option<OrgPolicy>, ConfigLoadError> {
    let content = match fetcher
        .fetch_config(&source.owner, &source.repo, &source.path)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(
                org_owner = source.owner.as_str(),
                org_repo = source.repo.as_str(),
                org_path = source.path.as_str(),
                "Org policy file not found; using three-tier config"
            );
            return Ok(None);
        }
        Err(e) => {
            let msg = format!("Failed to fetch org policy file: {e}");
            warn!(
                org_owner = source.owner.as_str(),
                org_repo = source.repo.as_str(),
                org_path = source.path.as_str(),
                error = %e,
                "Failed to fetch org policy; using three-tier config"
            );
            if source.fail_if_unreachable {
                return Err(ConfigLoadError::OrgPolicyUnavailable(msg));
            }
            return Ok(None);
        }
    };

    let raw: OrgPolicyRaw = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Failed to parse org policy TOML: {e}");
            warn!(
                org_owner = source.owner.as_str(),
                org_repo = source.repo.as_str(),
                org_path = source.path.as_str(),
                error = %e,
                "Org policy file has parse errors; using three-tier config"
            );
            if source.fail_if_unreachable {
                return Err(ConfigLoadError::OrgPolicyUnavailable(msg));
            }
            return Ok(None);
        }
    };

    if raw.schema_version != 1 {
        let msg = format!(
            "Unsupported org policy schema version: {}",
            raw.schema_version
        );
        warn!(
            org_owner = source.owner.as_str(),
            org_repo = source.repo.as_str(),
            org_path = source.path.as_str(),
            schema_version = raw.schema_version,
            "Org policy file has unsupported schema version; using three-tier config"
        );
        if source.fail_if_unreachable {
            return Err(ConfigLoadError::OrgPolicyUnavailable(msg));
        }
        return Ok(None);
    }

    let policy = OrgPolicy {
        enforced: PolicySet::from_org_section(&raw.enforced),
        defaults: PolicySet::from_org_section(&raw.defaults),
        conditional_policies: raw
            .conditional_policies
            .iter()
            .map(|cp| ConditionalPolicy {
                condition: PolicyCondition {
                    has_any_topic: cp.condition.has_any_topic.clone(),
                    has_custom_property: cp.condition.has_custom_property.clone(),
                },
                enforced: PolicySet::from_org_section(&cp.enforced),
                defaults: PolicySet::from_org_section(&cp.defaults),
            })
            .collect(),
    };

    for (idx, cp) in policy.conditional_policies.iter().enumerate() {
        if cp.condition == PolicyCondition::default() {
            warn!(
                org_owner = source.owner.as_str(),
                org_repo = source.repo.as_str(),
                org_path = source.path.as_str(),
                conditional_policy_index = idx,
                "Conditional policy block has an empty condition and will match every repository; add has_any_topic or has_custom_property criteria to restrict its scope"
            );
        }
    }

    info!(
        org_owner = source.owner.as_str(),
        org_repo = source.repo.as_str(),
        org_path = source.path.as_str(),
        "Org policy loaded successfully"
    );

    Ok(Some(policy))
}

/// Fetches and parses the repository `.github/merge-warden.toml` without applying
/// application defaults.
///
/// This is the low-level primitive used by [`resolve_pull_request_config`] to obtain
/// pure repo-configured values for the repo tier of the four-tier merge chain.
/// Unlike [`load_merge_warden_config`], which blends application defaults into the
/// returned struct, this function returns exactly what the repository TOML contains.
///
/// # Returns
///
/// - `Ok(RepositoryProvidedConfig)` — raw parsed config, or
///   [`RepositoryProvidedConfig::default`] if the file is absent or has an
///   unsupported schema version.
/// - `Err(ConfigLoadError::NotFound)` — the config fetcher returned an error.
/// - `Err(ConfigLoadError::...)` — TOML parse error.
async fn parse_repo_config(
    repo_owner: &str,
    repo_name: &str,
    path: &str,
    fetcher: &dyn ConfigFetcher,
) -> Result<RepositoryProvidedConfig, ConfigLoadError> {
    let content = match fetcher.fetch_config(repo_owner, repo_name, path).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            debug!(
                repository_owner = repo_owner,
                repository = repo_name,
                path = path,
                "No repo config file found; using empty defaults for repo tier"
            );
            return Ok(RepositoryProvidedConfig::default());
        }
        Err(e) => {
            warn!(
                repository_owner = repo_owner,
                repository = repo_name,
                path = path,
                "Failed to find configuration file in repository"
            );
            return Err(ConfigLoadError::NotFound(e.to_string()));
        }
    };

    let config: RepositoryProvidedConfig = toml::from_str(&content)?;
    if config.schema_version != 1 {
        error!(
            repository_owner = repo_owner,
            repository = repo_name,
            path = path,
            config_version = config.schema_version,
            "Configuration in repository has an unexpected version. Will not be able to load configuration."
        );
        return Ok(RepositoryProvidedConfig::default());
    }

    Ok(config)
}

/// Orchestrates the four-tier PR configuration resolution chain.
///
/// This is the primary entry point for platform handlers (server, CLI).
/// It replaces direct calls to [`load_merge_warden_config`].
///
/// # Resolution order (highest priority applied last in merge chain)
///
/// 1. Application defaults ([`PolicySet::from_application_defaults`])
/// 2. Org defaults (from [`OrgPolicy::defaults`], if `org_policy_source` is set)
/// 3. Repository config ([`parse_repo_config`] — raw repo values only)
/// 4. Org enforced (from [`OrgPolicy::enforced`], if `org_policy_source` is set)
/// 5. App-level enforcement flags ([`PolicySet::from_app_enforcement_flags`])
///
/// # Arguments
///
/// * `repo_owner` — GitHub repository owner.
/// * `repo_name` — GitHub repository name.
/// * `config_path` — path to the repo policy TOML file
///   (typically `".github/merge-warden.toml"`).
/// * `fetcher` — [`ConfigFetcher`] implementation used for both repo config
///   and org policy fetches.
/// * `app_defaults` — application-level policy defaults and configuration
///   pointers (including `org_policy_source`).
/// * `metadata_provider` — optional provider for repository topics and custom
///   properties. Required for conditional org policy evaluation. When `None`,
///   any `[[conditional_policies]]` blocks in the org policy are skipped and
///   a `warn!` is emitted.
///
/// # Returns
///
/// * `Ok(CurrentPullRequestValidationConfiguration)` — always returned unless
///   `app_defaults.org_policy_source.fail_if_unreachable = true` and the org
///   policy cannot be loaded.
/// * `Err(ConfigLoadError::OrgPolicyUnavailable)` — only when strict mode is
///   enabled and the org policy is unreachable or unparseable.
///
/// # Errors
///
/// Repo config load failures (file missing, parse error) are handled internally
/// by falling back to `PolicySet::default()` for the repo tier, matching the
/// behaviour previously found in platform handler fallback paths.
pub async fn resolve_pull_request_config(
    repo_owner: &str,
    repo_name: &str,
    config_path: &str,
    fetcher: &dyn ConfigFetcher,
    app_defaults: &ApplicationDefaults,
    metadata_provider: Option<&dyn RepositoryMetadataProvider>,
) -> Result<CurrentPullRequestValidationConfiguration, ConfigLoadError> {
    // Fetch repo config and org policy concurrently — both are independent
    // read-only operations against potentially different remote repositories.
    let (repo_config_res, org_policy_res) = tokio::join!(
        parse_repo_config(repo_owner, repo_name, config_path, fetcher),
        async {
            match &app_defaults.org_policy_source {
                None => Ok::<Option<OrgPolicy>, ConfigLoadError>(None),
                Some(source) => load_org_policy(source, fetcher).await,
            }
        }
    );

    let repo_config = match repo_config_res {
        Ok(c) => c,
        Err(e) => {
            warn!(
                repository_owner = repo_owner,
                repository = repo_name,
                error = %e,
                "Failed to load repo config in resolve_pull_request_config; using empty defaults for repo tier"
            );
            // Fall back to an empty repo config; the four-tier chain supplies app and org values.
            RepositoryProvidedConfig::default()
        }
    };

    // Propagate OrgPolicyUnavailable; all other org policy errors are already
    // handled inside load_org_policy (graceful degradation to three-tier).
    let org_policy: Option<OrgPolicy> = org_policy_res?;

    // Build policy sets for each tier.
    let app_defaults_ps = PolicySet::from_application_defaults(app_defaults);
    let (org_defaults_ps, org_enforced_ps) = match org_policy {
        Some(ref p) => (p.defaults.clone(), p.enforced.clone()),
        None => (PolicySet::default(), PolicySet::default()),
    };
    let repo_ps = PolicySet::from_repository_config(&repo_config);
    let app_enforced_ps = PolicySet::from_app_enforcement_flags(app_defaults);

    // Evaluate conditional policies when org policy defines any.
    let (conditional_defaults_policies, conditional_enforced_policies): (
        Vec<PolicySet>,
        Vec<PolicySet>,
    ) = if let Some(ref op) = org_policy {
        if !op.conditional_policies.is_empty() {
            match metadata_provider {
                Some(provider) => {
                    match provider.get_repository_context(repo_owner, repo_name).await {
                        Ok(context) => {
                            let matching: Vec<_> = op
                                .conditional_policies
                                .iter()
                                .filter(|cp| cp.condition.matches(&context))
                                .collect();
                            debug!(
                                repository_owner = repo_owner,
                                repository = repo_name,
                                total_conditional_policies = op.conditional_policies.len(),
                                matching_conditional_policies = matching.len(),
                                "Evaluated conditional org policies"
                            );
                            (
                                matching.iter().map(|cp| cp.defaults.clone()).collect(),
                                matching.iter().map(|cp| cp.enforced.clone()).collect(),
                            )
                        }
                        Err(e) => {
                            warn!(
                                repository_owner = repo_owner,
                                repository = repo_name,
                                error = %e,
                                "Failed to fetch repository context for conditional policies; skipping conditional evaluation"
                            );
                            (vec![], vec![])
                        }
                    }
                }
                None => {
                    warn!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        conditional_policy_count = op.conditional_policies.len(),
                        "Org policy has conditional policies but no metadata_provider supplied; skipping conditional evaluation"
                    );
                    (vec![], vec![])
                }
            }
        } else {
            (vec![], vec![])
        }
    } else {
        (vec![], vec![])
    };

    // Merge chain:
    // app_defaults → org_defaults → conditional_defaults* → repo →
    // conditional_enforced* → org_enforced → app_enforced
    let mut effective_ps = app_defaults_ps.merge(&org_defaults_ps);
    for cd in &conditional_defaults_policies {
        effective_ps = effective_ps.merge(cd);
    }
    effective_ps = effective_ps.merge(&repo_ps);

    // Warn when a repo has opted out of an org-default bypass list by setting
    // `enabled = true, users = []`.  This silently neutralises the org default
    // (the effective bypass list becomes empty) and is otherwise invisible to
    // platform operators.  We emit one structured warning per affected sub-rule
    // so that log-based alerting can detect the situation.
    //
    // We do NOT warn when the org default itself had an empty users list — that
    // is the normal "no default configured" case, not an opt-out.
    {
        let check_opt_out =
            |sub_rule_name: &str, org_rule: &BypassRule, effective_rule: &BypassRule| {
                if !org_rule.users().is_empty()
                    && effective_rule.enabled()
                    && effective_rule.users().is_empty()
                {
                    warn!(
                        repository_owner = repo_owner,
                        repository = repo_name,
                        sub_rule = sub_rule_name,
                        "Repo has opted out of the org-default bypass user list by \
                     setting enabled=true with an empty users list; the intermediate \
                     bypass list after the repo merge is now empty (org-enforced \
                     policies applied later in the chain may still override this)"
                    );
                }
            };

        // Note: conditional_defaults bypass users (merged before repo_ps) are not
        // monitored here — only opt-outs of org_defaults are detected.  Conditional
        // bypass rules are not a documented feature; this gap is intentional scope
        // deferral.  If conditional bypass support is added, this closure should also
        // compare against the conditional-defaults baseline.
        check_opt_out(
            "title_convention",
            org_defaults_ps.bypass_rules.title_convention(),
            effective_ps.bypass_rules.title_convention(),
        );
        check_opt_out(
            "work_items",
            org_defaults_ps.bypass_rules.work_item_convention(),
            effective_ps.bypass_rules.work_item_convention(),
        );
        check_opt_out(
            "size",
            org_defaults_ps.bypass_rules.size(),
            effective_ps.bypass_rules.size(),
        );
    }

    for ce in &conditional_enforced_policies {
        effective_ps = effective_ps.merge(ce);
    }
    effective_ps = effective_ps.merge(&org_enforced_ps).merge(&app_enforced_ps);

    Ok(effective_ps.to_validation_config(app_defaults))
}

/// Configuration for change type label detection and management
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ChangeTypeLabelConfig {
    /// Whether change type label detection is enabled
    #[serde(default = "ChangeTypeLabelConfig::default_enabled")]
    pub enabled: bool,
    /// Mappings from conventional commit types to repository label names
    #[serde(default)]
    pub conventional_commit_mappings: ConventionalCommitMappings,
    /// Configuration for the label detection strategy
    #[serde(default)]
    pub detection_strategy: LabelDetectionStrategy,
    /// Settings for creating fallback labels when none exist
    #[serde(default)]
    pub fallback_label_settings: FallbackLabelSettings,
    /// Configuration for keyword-triggered labels (breaking change, security, hotfix, tech debt).
    /// When absent all keyword labels use their built-in defaults.
    #[serde(default)]
    pub keyword_labels: KeywordLabelsConfig,
}

impl ChangeTypeLabelConfig {
    /// Returns `true` — change-type label checks are enabled by default.
    fn default_enabled() -> bool {
        true
    }

    /// Merges `over` on top of `base` (lower-priority).
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - `conventional_commit_mappings.*` (11 `Vec<String>` fields):
    ///   `over.field` if non-empty; otherwise `base.field`
    /// - `detection_strategy.exact_match`, `.prefix_match`, `.description_match`:
    ///   `over` wins unconditionally
    /// - `detection_strategy.common_prefixes`: `over` if non-empty; otherwise `base`
    /// - `fallback_label_settings.name_format`: `over` if non-empty and not equal to
    ///   `FallbackLabelSettings::default().name_format`; otherwise `base`
    /// - `fallback_label_settings.color_scheme`: per-key, `over` key wins if present
    /// - `fallback_label_settings.create_if_missing`: `over` wins unconditionally
    /// - `keyword_labels.*`: `over.field` if `Some`; otherwise `base.field`
    ///
    /// See `docs/spec/interfaces/policy-engine.md` §2.7 for the full contract.
    pub(crate) fn merge(base: &Self, over: &Self) -> Self {
        let bm = &base.conventional_commit_mappings;
        let om = &over.conventional_commit_mappings;
        let mappings = ConventionalCommitMappings {
            feat: if !om.feat.is_empty() {
                om.feat.clone()
            } else {
                bm.feat.clone()
            },
            fix: if !om.fix.is_empty() {
                om.fix.clone()
            } else {
                bm.fix.clone()
            },
            docs: if !om.docs.is_empty() {
                om.docs.clone()
            } else {
                bm.docs.clone()
            },
            style: if !om.style.is_empty() {
                om.style.clone()
            } else {
                bm.style.clone()
            },
            refactor: if !om.refactor.is_empty() {
                om.refactor.clone()
            } else {
                bm.refactor.clone()
            },
            perf: if !om.perf.is_empty() {
                om.perf.clone()
            } else {
                bm.perf.clone()
            },
            test: if !om.test.is_empty() {
                om.test.clone()
            } else {
                bm.test.clone()
            },
            chore: if !om.chore.is_empty() {
                om.chore.clone()
            } else {
                bm.chore.clone()
            },
            ci: if !om.ci.is_empty() {
                om.ci.clone()
            } else {
                bm.ci.clone()
            },
            build: if !om.build.is_empty() {
                om.build.clone()
            } else {
                bm.build.clone()
            },
            revert: if !om.revert.is_empty() {
                om.revert.clone()
            } else {
                bm.revert.clone()
            },
        };

        let bd = &base.detection_strategy;
        let od = &over.detection_strategy;
        let detection_strategy = LabelDetectionStrategy {
            exact_match: od.exact_match,
            prefix_match: od.prefix_match,
            description_match: od.description_match,
            common_prefixes: if !od.common_prefixes.is_empty() {
                od.common_prefixes.clone()
            } else {
                bd.common_prefixes.clone()
            },
        };

        let bf = &base.fallback_label_settings;
        let of = &over.fallback_label_settings;
        let default_name_format = FallbackLabelSettings::default_name_format();
        let name_format = if !of.name_format.is_empty() && of.name_format != default_name_format {
            of.name_format.clone()
        } else {
            bf.name_format.clone()
        };
        let mut color_scheme = bf.color_scheme.clone();
        for (key, value) in &of.color_scheme {
            color_scheme.insert(key.clone(), value.clone());
        }
        let fallback_label_settings = FallbackLabelSettings {
            name_format,
            color_scheme,
            create_if_missing: of.create_if_missing,
        };

        let bk = &base.keyword_labels;
        let ok = &over.keyword_labels;
        let keyword_labels = KeywordLabelsConfig {
            breaking_change: ok
                .breaking_change
                .clone()
                .or_else(|| bk.breaking_change.clone()),
            security: ok.security.clone().or_else(|| bk.security.clone()),
            hotfix: ok.hotfix.clone().or_else(|| bk.hotfix.clone()),
            tech_debt: ok.tech_debt.clone().or_else(|| bk.tech_debt.clone()),
        };

        Self {
            enabled: base.enabled || over.enabled,
            conventional_commit_mappings: mappings,
            detection_strategy,
            fallback_label_settings,
            keyword_labels,
        }
    }
}

/// Mappings from conventional commit types to possible repository label names
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ConventionalCommitMappings {
    /// Feature-related mappings
    #[serde(default = "ConventionalCommitMappings::default_feat")]
    pub feat: Vec<String>,
    /// Bug fix-related mappings
    #[serde(default = "ConventionalCommitMappings::default_fix")]
    pub fix: Vec<String>,
    /// Documentation-related mappings
    #[serde(default = "ConventionalCommitMappings::default_docs")]
    pub docs: Vec<String>,
    /// Style-related mappings
    #[serde(default = "ConventionalCommitMappings::default_style")]
    pub style: Vec<String>,
    /// Refactoring-related mappings
    #[serde(default = "ConventionalCommitMappings::default_refactor")]
    pub refactor: Vec<String>,
    /// Performance-related mappings
    #[serde(default = "ConventionalCommitMappings::default_perf")]
    pub perf: Vec<String>,
    /// Test-related mappings
    #[serde(default = "ConventionalCommitMappings::default_test")]
    pub test: Vec<String>,
    /// Chore-related mappings
    #[serde(default = "ConventionalCommitMappings::default_chore")]
    pub chore: Vec<String>,
    /// CI-related mappings
    #[serde(default = "ConventionalCommitMappings::default_ci")]
    pub ci: Vec<String>,
    /// Build-related mappings
    #[serde(default = "ConventionalCommitMappings::default_build")]
    pub build: Vec<String>,
    /// Revert-related mappings
    #[serde(default = "ConventionalCommitMappings::default_revert")]
    pub revert: Vec<String>,
}

impl ConventionalCommitMappings {
    /// Default repository label names for the `feat` commit type.
    fn default_feat() -> Vec<String> {
        vec![
            "enhancement".to_string(),
            "feature".to_string(),
            "new feature".to_string(),
        ]
    }
    /// Default repository label names for the `fix` commit type.
    fn default_fix() -> Vec<String> {
        vec!["bug".to_string(), "bugfix".to_string(), "fix".to_string()]
    }
    /// Default repository label names for the `docs` commit type.
    fn default_docs() -> Vec<String> {
        vec!["documentation".to_string(), "docs".to_string()]
    }
    /// Default repository label names for the `style` commit type.
    fn default_style() -> Vec<String> {
        vec!["style".to_string(), "formatting".to_string()]
    }
    /// Default repository label names for the `refactor` commit type.
    fn default_refactor() -> Vec<String> {
        vec![
            "refactor".to_string(),
            "refactoring".to_string(),
            "code quality".to_string(),
        ]
    }
    /// Default repository label names for the `perf` commit type.
    fn default_perf() -> Vec<String> {
        vec!["performance".to_string(), "optimization".to_string()]
    }
    /// Default repository label names for the `test` commit type.
    fn default_test() -> Vec<String> {
        vec![
            "test".to_string(),
            "tests".to_string(),
            "testing".to_string(),
        ]
    }
    /// Default repository label names for the `chore` commit type.
    fn default_chore() -> Vec<String> {
        vec![
            "chore".to_string(),
            "maintenance".to_string(),
            "housekeeping".to_string(),
        ]
    }
    /// Default repository label names for the `ci` commit type.
    fn default_ci() -> Vec<String> {
        vec![
            "ci".to_string(),
            "continuous integration".to_string(),
            "build".to_string(),
        ]
    }
    /// Default repository label names for the `build` commit type.
    fn default_build() -> Vec<String> {
        vec!["build".to_string(), "dependencies".to_string()]
    }
    /// Default repository label names for the `revert` commit type.
    fn default_revert() -> Vec<String> {
        vec!["revert".to_string()]
    }
}

/// Settings for creating fallback labels when repository labels are not found
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FallbackLabelSettings {
    /// Format for creating new label names (e.g., "type: {change_type}")
    #[serde(default = "FallbackLabelSettings::default_name_format")]
    pub name_format: String,
    /// Color scheme for different conventional commit types
    #[serde(default = "FallbackLabelSettings::default_color_scheme")]
    pub color_scheme: HashMap<String, String>,
    /// Whether to create fallback labels if none are found
    #[serde(default = "FallbackLabelSettings::default_create_if_missing")]
    pub create_if_missing: bool,
}

impl FallbackLabelSettings {
    /// Default label name format: `"type: {change_type}"`.
    fn default_name_format() -> String {
        "type: {change_type}".to_string()
    }
    /// Returns `true` — fallback labels are created when none are found by default.
    fn default_create_if_missing() -> bool {
        true
    }
    /// Default colour scheme for fallback labels.
    ///
    /// Inlined here to avoid constructing a full [`FallbackLabelSettings`] just to extract
    /// the colour map. Keep in sync with [`FallbackLabelSettings::default`].
    fn default_color_scheme() -> HashMap<String, String> {
        // Color scheme as specified in issue #107.
        let mut m = HashMap::new();
        m.insert("feat".to_string(), "#0075ca".to_string());
        m.insert("fix".to_string(), "#d73a4a".to_string());
        m.insert("docs".to_string(), "#0052cc".to_string());
        m.insert("style".to_string(), "#f9d0c4".to_string());
        m.insert("refactor".to_string(), "#fef2c0".to_string());
        m.insert("perf".to_string(), "#a2eeef".to_string());
        m.insert("test".to_string(), "#d4edda".to_string());
        m.insert("chore".to_string(), "#e1e4e8".to_string());
        m.insert("ci".to_string(), "#fbca04".to_string());
        m.insert("build".to_string(), "#c5def5".to_string());
        m.insert("revert".to_string(), "#b60205".to_string());
        m
    }
}

/// Configuration for keyword-triggered labels automatically applied based on PR title/body content.
///
/// Each field names the repository label to apply when the corresponding keyword is detected. When
/// a field is absent or set to an empty string the hard-coded default is used so existing configs
/// require no changes.
///
/// # Defaults
///
/// | Field | Default label |
/// |---|---|
/// | `breaking_change` | `"breaking-change"` |
/// | `security` | `"security"` |
/// | `hotfix` | `"hotfix"` |
/// | `tech_debt` | `"tech-debt"` |
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct KeywordLabelsConfig {
    /// Label applied when the PR title contains `!:` or the phrase "breaking change".
    /// Defaults to `"breaking-change"` when absent or empty.
    #[serde(default)]
    pub breaking_change: Option<String>,

    /// Label applied when the PR body contains "security" or "vulnerability".
    /// Defaults to `"security"` when absent or empty.
    #[serde(default)]
    pub security: Option<String>,

    /// Label applied when the PR body contains "hotfix".
    /// Defaults to `"hotfix"` when absent or empty.
    #[serde(default)]
    pub hotfix: Option<String>,

    /// Label applied when the PR body contains "technical debt" or "tech debt".
    /// Defaults to `"tech-debt"` when absent or empty.
    #[serde(default)]
    pub tech_debt: Option<String>,
}

impl KeywordLabelsConfig {
    /// Returns the effective label for breaking-change detection, falling back to the default.
    #[must_use]
    pub fn breaking_change_label(&self) -> &str {
        Self::resolve(self.breaking_change.as_deref(), "breaking-change")
    }

    /// Returns the effective label for hotfix detection, falling back to the default.
    #[must_use]
    pub fn hotfix_label(&self) -> &str {
        Self::resolve(self.hotfix.as_deref(), "hotfix")
    }

    /// Returns the effective label for security/vulnerability detection, falling back to the default.
    #[must_use]
    pub fn security_label(&self) -> &str {
        Self::resolve(self.security.as_deref(), "security")
    }

    /// Returns the effective label for tech-debt detection, falling back to the default.
    #[must_use]
    pub fn tech_debt_label(&self) -> &str {
        Self::resolve(self.tech_debt.as_deref(), "tech-debt")
    }

    /// Resolves a configured label name, falling back to `default` when the value is absent or empty.
    fn resolve<'a>(configured: Option<&'a str>, default: &'a str) -> &'a str {
        configured.filter(|s| !s.is_empty()).unwrap_or(default)
    }
}

/// Configuration for the label detection strategy
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct LabelDetectionStrategy {
    /// Enable exact name matching (e.g., "feat", "fix")
    #[serde(default = "LabelDetectionStrategy::default_true")]
    pub exact_match: bool,
    /// Enable prefix matching (e.g., "type: feat", "kind: fix")
    #[serde(default = "LabelDetectionStrategy::default_true")]
    pub prefix_match: bool,
    /// Enable description matching (e.g., labels with "(type: feat)" in description)
    #[serde(default = "LabelDetectionStrategy::default_true")]
    pub description_match: bool,
    /// Common prefixes to check for prefix matching
    #[serde(default = "LabelDetectionStrategy::default_common_prefixes")]
    pub common_prefixes: Vec<String>,
}

impl LabelDetectionStrategy {
    /// Returns `true` — all label detection modes are enabled by default.
    fn default_true() -> bool {
        true
    }
    /// Default set of label name prefixes used for prefix matching.
    fn default_common_prefixes() -> Vec<String> {
        vec![
            "type:".to_string(),
            "kind:".to_string(),
            "category:".to_string(),
        ]
    }
}

impl Default for ChangeTypeLabelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            conventional_commit_mappings: ConventionalCommitMappings::default(),
            detection_strategy: LabelDetectionStrategy::default(),
            fallback_label_settings: FallbackLabelSettings::default(),
            keyword_labels: KeywordLabelsConfig::default(),
        }
    }
}

impl Default for ConventionalCommitMappings {
    fn default() -> Self {
        Self {
            feat: vec![
                "enhancement".to_string(),
                "feature".to_string(),
                "new feature".to_string(),
            ],
            fix: vec!["bug".to_string(), "bugfix".to_string(), "fix".to_string()],
            docs: vec!["documentation".to_string(), "docs".to_string()],
            style: vec!["style".to_string(), "formatting".to_string()],
            refactor: vec![
                "refactor".to_string(),
                "refactoring".to_string(),
                "code quality".to_string(),
            ],
            perf: vec!["performance".to_string(), "optimization".to_string()],
            test: vec![
                "test".to_string(),
                "tests".to_string(),
                "testing".to_string(),
            ],
            chore: vec![
                "chore".to_string(),
                "maintenance".to_string(),
                "housekeeping".to_string(),
            ],
            ci: vec![
                "ci".to_string(),
                "continuous integration".to_string(),
                "build".to_string(),
            ],
            build: vec!["build".to_string(), "dependencies".to_string()],
            revert: vec!["revert".to_string()],
        }
    }
}

impl Default for FallbackLabelSettings {
    fn default() -> Self {
        let mut color_scheme = HashMap::new();

        // Color scheme as specified in issue #107
        color_scheme.insert("feat".to_string(), "#0075ca".to_string());
        color_scheme.insert("fix".to_string(), "#d73a4a".to_string());
        color_scheme.insert("docs".to_string(), "#0052cc".to_string());
        color_scheme.insert("style".to_string(), "#f9d0c4".to_string());
        color_scheme.insert("refactor".to_string(), "#fef2c0".to_string());
        color_scheme.insert("perf".to_string(), "#a2eeef".to_string());
        color_scheme.insert("test".to_string(), "#d4edda".to_string());
        color_scheme.insert("chore".to_string(), "#e1e4e8".to_string());
        color_scheme.insert("ci".to_string(), "#fbca04".to_string());
        color_scheme.insert("build".to_string(), "#c5def5".to_string());
        color_scheme.insert("revert".to_string(), "#b60205".to_string());

        Self {
            name_format: "type: {change_type}".to_string(),
            color_scheme,
            create_if_missing: true,
        }
    }
}

impl Default for LabelDetectionStrategy {
    fn default() -> Self {
        Self {
            exact_match: true,
            prefix_match: true,
            description_match: true,
            common_prefixes: vec![
                "type:".to_string(),
                "kind:".to_string(),
                "category:".to_string(),
            ],
        }
    }
}
