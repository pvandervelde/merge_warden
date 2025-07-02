use crate::config::{
    BypassRule, BypassRules, CurrentPullRequestValidationConfiguration, PrSizeCheckConfig,
    CONVENTIONAL_COMMIT_REGEX, WORK_ITEM_REGEX,
};
use crate::size::SizeThresholds;
use async_trait::async_trait;
use merge_warden_developer_platforms::errors::Error;
use proptest::prelude::*;
use regex::Regex;

use super::*;

struct MockFetcher {
    config_text: Option<String>,
}

impl MockFetcher {
    pub fn new(config_text: Option<String>) -> Self {
        Self { config_text }
    }
}

#[async_trait]
impl ConfigFetcher for MockFetcher {
    async fn fetch_config(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _path: &str,
    ) -> Result<Option<String>, Error> {
        Ok(self.config_text.clone())
    }
}

#[test]
fn test_application_defaults_struct_and_methods() {
    let defaults = ApplicationDefaults::default();
    assert!(!defaults.enable_title_validation);
    assert_eq!(
        defaults.default_title_pattern,
        CONVENTIONAL_COMMIT_REGEX.to_string()
    );
    assert_eq!(defaults.default_invalid_title_label, None);
    assert!(!defaults.enable_work_item_validation);
    assert_eq!(
        defaults.default_work_item_pattern,
        WORK_ITEM_REGEX.to_string()
    );
    assert_eq!(defaults.default_missing_work_item_label, None);
}

#[test]
fn test_conventional_commit_regex_edge_cases() {
    let edge_cases = vec![
        "feat(api-v1): add feature",   // Scope with hyphen
        "feat(api_v1): add feature",   // Scope with underscore
        "feat(api)!: breaking change", // Breaking change indicator
        "feat!: breaking change",      // Breaking change indicator without scope
        "feat(api)!: add feature with special chars !@#$%^&*()", // Special characters in description
    ];

    let regex = Regex::new(CONVENTIONAL_COMMIT_REGEX).unwrap();
    for title in edge_cases {
        assert!(
            regex.is_match(title),
            "CONVENTIONAL_COMMIT_REGEX should match edge case title '{}'",
            title
        );
    }
}

#[test]
fn test_conventional_commit_regex_invalid_formats() {
    let invalid_titles = vec![
        "unknown: add feature",             // Invalid prefix
        "feat-add feature",                 // Incorrect separator
        "feat add feature",                 // Missing separator
        "feat:add feature",                 // No space after colon
        "feat",                             // Missing description
        "feat: ",                           // Empty description
        "feat(AUTH): add feature",          // Uppercase scope
        "feat(api/v1): add feature",        // Scope with slash
        "feat(api)(auth): add new feature", // Multiple scopes
    ];

    let regex = Regex::new(CONVENTIONAL_COMMIT_REGEX).unwrap();
    for title in invalid_titles {
        assert!(
            !regex.is_match(title),
            "CONVENTIONAL_COMMIT_REGEX should not match invalid title '{}'",
            title
        );
    }
}

#[test]
fn test_conventional_commit_regex_valid_formats() {
    let valid_titles = vec![
        "feat: add new feature",
        "fix(auth): correct login issue",
        "docs: update README",
        "style: format code",
        "refactor(api): simplify logic",
        "perf: improve performance",
        "test: add unit tests",
        "build: update dependencies",
        "ci: configure GitHub Actions",
        "chore: update gitignore",
        "revert: remove feature X",
        "feat!: breaking change",
        "feat(api)!: breaking change in API",
    ];

    let regex = Regex::new(CONVENTIONAL_COMMIT_REGEX).unwrap();
    for title in valid_titles {
        assert!(
            regex.is_match(title),
            "CONVENTIONAL_COMMIT_REGEX should match valid title '{}'",
            title
        );
    }
}

proptest! {
    #[test]
    fn test_conventional_commit_regex_random_inputs(input in ".*") {
        let regex = Regex::new(CONVENTIONAL_COMMIT_REGEX).unwrap();
        let _ = regex.is_match(&input); // Ensure no panic occurs
    }

    #[test]
    fn test_work_item_regex_random_inputs(input in ".*") {
        let regex = Regex::new(WORK_ITEM_REGEX).unwrap();
        let _ = regex.is_match(&input); // Ensure no panic occurs
    }
}

#[test]
fn test_current_pr_validation_config_new() {
    let config = CurrentPullRequestValidationConfiguration::new(
        true,
        Some("custom-title".to_string()),
        Some("custom-invalid-label".to_string()),
        false,
        Some("custom-work-item".to_string()),
        Some("custom-missing-label".to_string()),
        Some(PrSizeCheckConfig::default()),
        Some(BypassRules::default()),
    );
    assert!(config.enforce_title_convention);
    assert_eq!(config.title_pattern, "custom-title");
    assert_eq!(
        config.invalid_title_label,
        Some("custom-invalid-label".to_string())
    );
    assert!(!config.enforce_work_item_references);
    assert_eq!(config.work_item_reference_pattern, "custom-work-item");
    assert_eq!(
        config.missing_work_item_label,
        Some("custom-missing-label".to_string())
    );
}

#[test]
fn test_custom_regex_patterns_are_used() {
    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                title_policies: PullRequestsTitlePolicyConfig {
                    required: true,
                    pattern: "^CUSTOM: .+".to_string(),
                    label_if_missing: Some("custom-label".to_string()),
                },
                work_item_policies: WorkItemPolicyConfig {
                    required: true,
                    pattern: String::from(r"CUSTOM-\d+"),
                    label_if_missing: Some("custom-missing".to_string()),
                },
                size_policies: PrSizeCheckConfig::default(),
            },
        },
    };
    let validation = config.to_validation_config(&BypassRules::default());
    let custom_title = "CUSTOM: test title";
    let custom_title_regex = regex::Regex::new(&validation.title_pattern).unwrap();
    assert!(custom_title_regex.is_match(custom_title));
    let custom_work_item = "CUSTOM-123";
    let custom_work_item_regex =
        regex::Regex::new(&validation.work_item_reference_pattern).unwrap();
    assert!(custom_work_item_regex.is_match(custom_work_item));
}

#[tokio::test]
async fn test_load_merge_warden_config_empty_file() {
    let file_path = "merge-warden.toml";
    let fetcher = MockFetcher::new(None);
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", file_path, &fetcher, &app_defaults)
        .await
        .unwrap();

    assert!(!config.policies.pull_requests.title_policies.required,);
    assert_eq!(
        config.policies.pull_requests.title_policies.pattern,
        app_defaults.default_title_pattern
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .title_policies
            .label_if_missing,
        app_defaults.default_invalid_title_label
    );

    assert!(!config.policies.pull_requests.work_item_policies.required,);
    assert_eq!(
        config.policies.pull_requests.work_item_policies.pattern,
        app_defaults.default_work_item_pattern
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .work_item_policies
            .label_if_missing,
        app_defaults.default_missing_work_item_label
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_invalid_schema() {
    let file_path = "merge-warden.toml";
    let toml = r##"schemaVersion = 2
[policies.pullRequests.prTitle]
required = true
pattern = "foo"
[policies.pullRequests.workItem]
required = true
pattern = "bar"
"##;

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let result = load_merge_warden_config("a", "b", file_path, &fetcher, &app_defaults).await;
    // The code returns Ok(default) for unsupported schema version, not an error
    assert!(
        result.is_ok(),
        "Should return Ok(default) for unsupported schema version"
    );
    let config = result.unwrap();
    assert_eq!(config, RepositoryProvidedConfig::default());
}

#[tokio::test]
async fn test_load_merge_warden_config_malformed_toml() {
    let file_path = "merge-warden.toml";
    let toml = r#"schemaVersion = 1
[policies.pullRequests.prTitle
required = true
pattern = "foo"
"#; // missing closing bracket for table

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let result = load_merge_warden_config("a", "b", file_path, &fetcher, &app_defaults).await;
    assert!(matches!(result, Err(ConfigLoadError::Toml(_))));
}

#[tokio::test]
async fn test_load_merge_warden_config_missing_file() {
    let fetcher = MockFetcher::new(None);
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config(
        "a",
        "b",
        "/nonexistent/path/merge-warden.toml",
        &fetcher,
        &app_defaults,
    )
    .await
    .unwrap();

    assert!(!config.policies.pull_requests.title_policies.required);
    assert_eq!(
        config.policies.pull_requests.title_policies.pattern,
        app_defaults.default_title_pattern
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .title_policies
            .label_if_missing,
        app_defaults.default_invalid_title_label
    );

    assert!(!config.policies.pull_requests.work_item_policies.required);
    assert_eq!(
        config.policies.pull_requests.work_item_policies.pattern,
        app_defaults.default_work_item_pattern
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .work_item_policies
            .label_if_missing,
        app_defaults.default_missing_work_item_label
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_missing_optional_fields() {
    let file_path = "merge-warden.toml";
    let toml = r#"schemaVersion = 1
[policies.pullRequests.prTitle]
required = true
pattern = "foo"
# label_if_missing omitted
[policies.pullRequests.workItem]
required = true
pattern = "bar"
# label_if_missing omitted
"#;

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", file_path, &fetcher, &app_defaults)
        .await
        .unwrap();
    assert_eq!(
        config
            .policies
            .pull_requests
            .title_policies
            .label_if_missing,
        app_defaults.default_invalid_title_label
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .work_item_policies
            .label_if_missing,
        app_defaults.default_missing_work_item_label
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_only_schema_version() {
    let file_path = "merge-warden.toml";
    let toml = r#"schemaVersion = 1
"#;

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", file_path, &fetcher, &app_defaults).await;
    // Should succeed, but policies will be defaulted
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.schema_version, 1);
    // Defaults for policies
    assert!(!config.policies.pull_requests.title_policies.required);
    assert!(!config.policies.pull_requests.work_item_policies.required);
}

#[tokio::test]
async fn test_load_merge_warden_config_valid() {
    let file_path = "merge-warden.toml";
    let toml = r##"schemaVersion = 1
[policies.pullRequests.prTitle]
required = true
pattern = "^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\\([a-z0-9_-]+\\))?!?: .+"
label_if_missing = "invalid-title-format"
[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"
label_if_missing = "missing-work-item"
"##;

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", file_path, &fetcher, &app_defaults)
        .await
        .unwrap();
    assert_eq!(config.schema_version, 1);
    assert!(config.policies.pull_requests.title_policies.required);
    assert_eq!(
        config.policies.pull_requests.title_policies.pattern,
        "^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\\([a-z0-9_-]+\\))?!?: .+"
            .to_string()
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .title_policies
            .label_if_missing,
        Some("invalid-title-format".to_string())
    );
    assert!(config.policies.pull_requests.work_item_policies.required);
    assert_eq!(
        config.policies.pull_requests.work_item_policies.pattern,
        "#\\d+".to_string()
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .work_item_policies
            .label_if_missing,
        Some("missing-work-item".to_string())
    );
}

#[test]
fn test_merge_warden_config_to_validation_config_conventional_commits_and_work_item() {
    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                title_policies: PullRequestsTitlePolicyConfig {
                    required: true,
                    pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
                    label_if_missing: Some(TITLE_INVALID_LABEL.to_string()),
                },
                work_item_policies: WorkItemPolicyConfig {
                    required: true,
                    pattern: WORK_ITEM_REGEX.to_string(),
                    label_if_missing: Some(MISSING_WORK_ITEM_LABEL.to_string()),
                },
                size_policies: PrSizeCheckConfig::default(),
            },
        },
    };
    let validation = config.to_validation_config(&BypassRules::default());
    assert!(validation.enforce_title_convention);
    assert!(validation.enforce_work_item_references);
    assert_eq!(
        validation.invalid_title_label,
        Some(TITLE_INVALID_LABEL.to_string())
    );
    assert_eq!(
        validation.missing_work_item_label,
        Some(MISSING_WORK_ITEM_LABEL.to_string())
    );
}

#[test]
fn test_merge_warden_config_to_validation_config_non_conventional_commits() {
    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                title_policies: PullRequestsTitlePolicyConfig {
                    required: false,
                    pattern: "".to_string(),
                    label_if_missing: None,
                },
                work_item_policies: WorkItemPolicyConfig {
                    required: false,
                    pattern: "".to_string(),
                    label_if_missing: None,
                },
                size_policies: PrSizeCheckConfig::default(),
            },
        },
    };
    let validation = config.to_validation_config(&BypassRules::default());
    assert!(!validation.enforce_title_convention);
    assert!(!validation.enforce_work_item_references);
    assert_eq!(validation.invalid_title_label, None);
    assert_eq!(validation.missing_work_item_label, None);
}

#[test]
fn test_missing_work_item_label() {
    assert_eq!(MISSING_WORK_ITEM_LABEL, "missing-work-item");
}

#[test]
fn test_title_comment_marker() {
    assert_eq!(TITLE_COMMENT_MARKER, "<!-- PR_TITLE_CHECK -->");
}

#[test]
fn test_title_invalid_label() {
    assert_eq!(TITLE_INVALID_LABEL, "invalid-title-format");
}

#[test]
fn test_valid_pr_types() {
    assert!(VALID_PR_TYPES.contains(&"feat"));
    assert!(VALID_PR_TYPES.contains(&"fix"));
}

#[test]
fn test_validation_config_default() {
    let config = CurrentPullRequestValidationConfiguration::default();

    assert!(
        config.enforce_title_convention,
        "Default ValidationConfig should enforce conventional commits"
    );
    assert!(
        config.enforce_work_item_references,
        "Default ValidationConfig should require work item references"
    );
}

#[test]
fn test_work_item_regex_edge_cases() {
    let edge_cases = vec![
        "See #123",                                          // Different keyword (not in regex)
        "Related to #456",                                   // Different keyword (not in regex)
        "Fixes GH-123",                                      // Different issue number format
        "Closes org/repo#123",                               // Different issue number format
        "References https://github.com/owner/repo/pull/123", // Pull request URL
        "Relates to https://github.com/owner/repo/issues/123?query=param", // URL with query parameters
    ];

    // Test which ones should match according to the current regex
    let should_match = [
        "Fixes GH-123",
        "Closes org/repo#123",
        "Relates to https://github.com/owner/repo/issues/123?query=param",
    ];

    let regex = Regex::new(WORK_ITEM_REGEX).unwrap();
    for reference in edge_cases {
        let expected_match = should_match.contains(&reference);
        assert_eq!(
            regex.is_match(reference),
            expected_match,
            "WORK_ITEM_REGEX match for '{}' should be {}",
            reference,
            expected_match
        );
    }
}

#[test]
fn test_work_item_regex_invalid_formats() {
    let invalid_references = vec![
        "Fixes 123",                                     // Missing #
        "Fixesissue123",                                 // No space
        "Fixes https://github.com/issues",               // Missing issue number
        "Fixes https://github.com/owner/repo/issues123", // Missing separator
    ];

    let regex = Regex::new(WORK_ITEM_REGEX).unwrap();
    for reference in invalid_references {
        assert!(
            !regex.is_match(reference),
            "WORK_ITEM_REGEX should not match invalid reference '{}'",
            reference
        );
    }
}

#[test]
fn test_work_item_regex_valid_formats() {
    let valid_references = vec![
        "Fixes #123",
        "fixes #123",
        "Closes #456",
        "closes #456",
        "Resolves #789",
        "resolves #789",
        "References #101112",
        "references #101112",
        "Relates to #131415",
        "relates to #131415",
        "Fixes GH-123",
        "Fixes https://github.com/owner/repo/issues/123",
    ];

    let regex = Regex::new(WORK_ITEM_REGEX).unwrap();
    for reference in valid_references {
        assert!(
            regex.is_match(reference),
            "WORK_ITEM_REGEX should match valid reference '{}'",
            reference
        );
    }
}

#[test]
fn test_bypass_rule_default() {
    let rule = BypassRule::default();
    assert!(!rule.enabled);
    assert!(rule.users.is_empty());
}

#[test]
fn test_bypass_rule_serialization() {
    let rule = BypassRule {
        enabled: true,
        users: vec!["user1".to_string(), "user2".to_string()],
    };

    let serialized = serde_json::to_string(&rule).expect("Failed to serialize BypassRule");
    let parsed: serde_json::Value =
        serde_json::from_str(&serialized).expect("Failed to parse JSON");

    assert_eq!(parsed["enabled"], true);
    assert_eq!(parsed["users"][0], "user1");
    assert_eq!(parsed["users"][1], "user2");
}

#[test]
fn test_bypass_rule_deserialization() {
    let json = r#"{"enabled": false, "users": ["admin", "bot"]}"#;
    let rule: BypassRule = serde_json::from_str(json).expect("Failed to deserialize BypassRule");

    assert!(!rule.enabled);
    assert_eq!(rule.users.len(), 2);
    assert_eq!(rule.users[0], "admin");
    assert_eq!(rule.users[1], "bot");
}

#[test]
fn test_bypass_rules_default() {
    let rules = BypassRules::default();
    assert!(!rules.title_convention.enabled);
    assert!(!rules.work_items.enabled);
    assert!(rules.title_convention.users.is_empty());
    assert!(rules.work_items.users.is_empty());
}

#[test]
fn test_bypass_rules_serialization() {
    let rules = BypassRules {
        title_convention: BypassRule {
            enabled: true,
            users: vec!["release-bot".to_string()],
        },
        work_items: BypassRule {
            enabled: false,
            users: vec![],
        },
        size: BypassRule {
            enabled: false,
            users: vec![],
        },
    };

    let serialized = serde_json::to_string(&rules).expect("Failed to serialize BypassRules");
    let parsed: serde_json::Value =
        serde_json::from_str(&serialized).expect("Failed to parse JSON");
    assert_eq!(parsed["title_convention"]["enabled"], true);
    assert_eq!(parsed["title_convention"]["users"][0], "release-bot");
    assert_eq!(parsed["work_items"]["enabled"], false);
}

#[test]
fn test_bypass_rules_deserialization() {
    let json = r#"{
        "title_convention": {"enabled": true, "users": ["admin"]},
        "work_items": {"enabled": true, "users": ["hotfix-team", "admin"]},
        "branch_protection": {"enabled": false, "users": []}
    }"#;

    let rules: BypassRules = serde_json::from_str(json).expect("Failed to deserialize BypassRules");

    assert!(rules.title_convention.enabled);
    assert_eq!(rules.title_convention.users, vec!["admin"]);
    assert!(rules.work_items.enabled);
    assert_eq!(rules.work_items.users, vec!["hotfix-team", "admin"]);
}

#[test]
fn test_bypass_rules_partial_deserialization() {
    // Test that missing fields default properly
    let json = r#"{"title_convention": {"enabled": true, "users": ["admin"]}}"#;

    let rules: BypassRules = serde_json::from_str(json).expect("Failed to deserialize BypassRules");

    assert!(rules.title_convention.enabled);
    assert_eq!(rules.title_convention.users, vec!["admin"]);
    // Other fields should use defaults
    assert!(!rules.work_items.enabled);
    assert!(rules.work_items.users.is_empty());
}

#[test]
fn test_application_defaults_with_bypass_rules() {
    let defaults = ApplicationDefaults::default();

    // Verify bypass rules are included and default correctly
    assert!(!defaults.bypass_rules.title_convention.enabled);
    assert!(!defaults.bypass_rules.work_items.enabled);
}

#[test]
fn test_application_defaults_bypass_rules_serialization() {
    let defaults = ApplicationDefaults {
        enable_title_validation: true,
        default_title_pattern: "test".to_string(),
        default_invalid_title_label: Some("invalid".to_string()),
        enable_work_item_validation: true,
        default_work_item_pattern: "pattern".to_string(),
        default_missing_work_item_label: Some("missing".to_string()),
        pr_size_check: PrSizeCheckConfig::default(),
        bypass_rules: BypassRules {
            title_convention: BypassRule {
                enabled: true,
                users: vec!["admin".to_string()],
            },
            work_items: BypassRule::default(),
            size: BypassRule::default(),
        },
    };

    let serialized =
        serde_json::to_string(&defaults).expect("Failed to serialize ApplicationDefaults");
    let parsed: serde_json::Value =
        serde_json::from_str(&serialized).expect("Failed to parse JSON");

    assert_eq!(parsed["enforceTitleValidation"], true);
    assert_eq!(parsed["bypassRules"]["title_convention"]["enabled"], true);
    assert_eq!(
        parsed["bypassRules"]["title_convention"]["users"][0],
        "admin"
    );
}

#[test]
fn test_pr_size_check_config_defaults() {
    let config = PrSizeCheckConfig::default();
    assert!(!config.enabled);
    assert!(config.thresholds.is_none());
    assert!(!config.fail_on_oversized);
    assert!(config.excluded_file_patterns.is_empty());
    assert_eq!(config.label_prefix, "size/");
    assert!(config.add_comment);
}

#[test]
fn test_pr_size_check_config_effective_thresholds() {
    let config = PrSizeCheckConfig::default();
    let thresholds = config.get_effective_thresholds();
    assert_eq!(thresholds, SizeThresholds::default());

    let custom_thresholds = SizeThresholds::new(5, 25, 75, 150, 300);
    let config_with_custom = PrSizeCheckConfig {
        enabled: true,
        thresholds: Some(custom_thresholds.clone()),
        fail_on_oversized: false,
        excluded_file_patterns: vec![],
        label_prefix: "size/".to_string(),
        add_comment: true,
    };
    assert_eq!(
        config_with_custom.get_effective_thresholds(),
        custom_thresholds
    );
}

#[test]
fn test_pr_size_check_file_exclusion_patterns() {
    let config = PrSizeCheckConfig {
        enabled: true,
        thresholds: None,
        fail_on_oversized: false,
        excluded_file_patterns: vec![
            "*.md".to_string(),
            "*.txt".to_string(),
            "docs/*".to_string(),
        ],
        label_prefix: "size/".to_string(),
        add_comment: true,
    };

    // Test exclusion patterns
    assert!(config.should_exclude_file("README.md"));
    assert!(config.should_exclude_file("CHANGELOG.md"));
    assert!(config.should_exclude_file("notes.txt"));
    assert!(config.should_exclude_file("docs/api.md"));
    assert!(config.should_exclude_file("docs/guide/setup.md"));

    // Test non-excluded files
    assert!(!config.should_exclude_file("src/main.rs"));
    assert!(!config.should_exclude_file("tests/integration.rs"));
    assert!(!config.should_exclude_file("Cargo.toml"));
}

#[test]
fn test_pr_size_check_no_exclusion_patterns() {
    let config = PrSizeCheckConfig::default();

    // No patterns means no files are excluded
    assert!(!config.should_exclude_file("README.md"));
    assert!(!config.should_exclude_file("src/main.rs"));
    assert!(!config.should_exclude_file("docs/api.md"));
}

#[test]
fn test_pattern_matches_function() {
    use crate::config::pattern_matches;

    // Test exact matches
    assert!(pattern_matches("exact.txt", "exact.txt"));
    assert!(!pattern_matches("exact.txt", "other.txt"));

    // Test wildcard patterns
    assert!(pattern_matches("*.md", "README.md"));
    assert!(pattern_matches("*.md", "docs.md"));
    assert!(!pattern_matches("*.md", "main.rs"));

    // Test directory patterns
    assert!(pattern_matches("docs/*", "docs/api.md"));
    assert!(pattern_matches("docs/*", "docs/guide.txt"));
    assert!(!pattern_matches("docs/*", "src/main.rs"));

    // Test complex patterns
    assert!(pattern_matches("test_*.rs", "test_main.rs"));
    assert!(pattern_matches("test_*.rs", "test_helper.rs"));
    assert!(!pattern_matches("test_*.rs", "main.rs"));
}

#[test]
fn test_pr_size_config_serialization() {
    let config = PrSizeCheckConfig {
        enabled: true,
        thresholds: Some(SizeThresholds::new(5, 25, 75, 150, 300)),
        fail_on_oversized: true,
        excluded_file_patterns: vec!["*.md".to_string(), "docs/*".to_string()],
        label_prefix: "pr-size/".to_string(),
        add_comment: false,
    };

    // Test that serialization works (this is important for TOML config)
    let serialized = toml::to_string(&config).expect("Should serialize");
    let deserialized: PrSizeCheckConfig = toml::from_str(&serialized).expect("Should deserialize");

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.fail_on_oversized, deserialized.fail_on_oversized);
    assert_eq!(
        config.excluded_file_patterns,
        deserialized.excluded_file_patterns
    );
    assert_eq!(config.label_prefix, deserialized.label_prefix);
    assert_eq!(config.add_comment, deserialized.add_comment);

    // Note: We can't directly compare thresholds due to Option<SizeThresholds>
    assert_eq!(
        config.get_effective_thresholds(),
        deserialized.get_effective_thresholds()
    );
}

#[test]
fn test_repository_config_with_pr_size() {
    let toml_content = r#"
        schemaVersion = 1

        [policies.pullRequests.prSize]
        enabled = true
        fail_on_oversized = true
        excluded_file_patterns = ["*.md", "*.txt"]
        label_prefix = "pr-size/"
        add_comment = false

        [policies.pullRequests.prSize.thresholds]
        xs = 5
        s = 25
        m = 75
        l = 150
        xl = 300
    "#;

    let config: RepositoryProvidedConfig = toml::from_str(toml_content).expect("Should parse TOML");
    assert_eq!(config.schema_version, 1);

    let size_config = &config.policies.pull_requests.size_policies;
    assert!(size_config.enabled);
    assert!(size_config.fail_on_oversized);
    assert_eq!(size_config.excluded_file_patterns, vec!["*.md", "*.txt"]);
    assert_eq!(size_config.label_prefix, "pr-size/");
    assert!(!size_config.add_comment);

    let thresholds = size_config.get_effective_thresholds();
    assert_eq!(thresholds.xs, 5);
    assert_eq!(thresholds.s, 25);
    assert_eq!(thresholds.m, 75);
    assert_eq!(thresholds.l, 150);
    assert_eq!(thresholds.xl, 300);
}

#[test]
fn test_validation_config_includes_pr_size() {
    let repo_config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                title_policies: PullRequestsTitlePolicyConfig::default(),
                work_item_policies: WorkItemPolicyConfig::default(),
                size_policies: PrSizeCheckConfig {
                    enabled: true,
                    thresholds: None,
                    fail_on_oversized: true,
                    excluded_file_patterns: vec!["*.md".to_string()],
                    label_prefix: "custom/".to_string(),
                    add_comment: false,
                },
            },
        },
    };

    let validation = repo_config.to_validation_config(&BypassRules::default());

    assert!(validation.pr_size_check.enabled);
    assert!(validation.pr_size_check.fail_on_oversized);
    assert_eq!(
        validation.pr_size_check.excluded_file_patterns,
        vec!["*.md"]
    );
    assert_eq!(validation.pr_size_check.label_prefix, "custom/");
    assert!(!validation.pr_size_check.add_comment);
}
