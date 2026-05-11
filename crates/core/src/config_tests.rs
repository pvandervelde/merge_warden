use crate::config::{
    BypassRule, BypassRules, ChangeTypeLabelConfig, CurrentPullRequestValidationConfiguration,
    IssuePropagationConfig, KeywordLabelsConfig, PrSizeCheckConfig, WipCheckConfig,
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
                ..Default::default()
            },
            ..Default::default()
        },
        change_type_labels: None,
        ..Default::default()
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
                ..Default::default()
            },
            ..Default::default()
        },
        change_type_labels: None,
        ..Default::default()
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
                ..Default::default()
            },
            ..Default::default()
        },
        change_type_labels: None,
        ..Default::default()
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
        change_type_labels: ChangeTypeLabelConfig::default(),
        wip_check: WipCheckConfig::default(),
        pr_state_labels: crate::config::PrStateLabelsConfig::default(),
        bot_mention: "@merge-warden".to_string(),
    };

    let serialized =
        serde_json::to_string(&defaults).expect("Failed to serialize ApplicationDefaults");
    let parsed: serde_json::Value =
        serde_json::from_str(&serialized).expect("Failed to parse JSON");

    assert_eq!(parsed["enable_title_validation"], true);
    assert_eq!(parsed["bypass_rules"]["title_convention"]["enabled"], true);
    assert_eq!(
        parsed["bypass_rules"]["title_convention"]["users"][0],
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
        ignore_deletions: false,
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
        ignore_deletions: false,
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
        ignore_deletions: false,
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
fn test_pr_size_check_config_ignore_deletions_round_trip() {
    // Verify that ignore_deletions = true survives a TOML serialize → deserialize cycle.
    let config = PrSizeCheckConfig {
        enabled: true,
        thresholds: None,
        fail_on_oversized: false,
        excluded_file_patterns: vec![],
        label_prefix: "size/".to_string(),
        add_comment: true,
        ignore_deletions: true,
    };

    let serialized = toml::to_string(&config).expect("Should serialize");

    // The serialized TOML must contain the snake_case key name.
    assert!(
        serialized.contains("ignore_deletions"),
        "Expected 'ignore_deletions' in serialized TOML, got: {serialized}"
    );

    let deserialized: PrSizeCheckConfig = toml::from_str(&serialized).expect("Should deserialize");

    assert!(deserialized.ignore_deletions);

    // Also verify that omitting the field from TOML yields the default (false).
    let minimal_toml = "[pr_size_check]\nenabled = true\n";
    let minimal: PrSizeCheckConfig =
        toml::from_str(minimal_toml).expect("Should deserialize minimal TOML");

    assert!(
        !minimal.ignore_deletions,
        "ignore_deletions should default to false when absent from TOML"
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
                    ignore_deletions: false,
                },
                ..Default::default()
            },
            ..Default::default()
        },
        change_type_labels: None,
        ..Default::default()
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

// ── WipCheckConfig tests ─────────────────────────────────────────────────────

#[test]
fn test_wip_check_config_default_enforce_is_false() {
    let config = WipCheckConfig::default();
    assert!(
        !config.enforce_wip_blocking,
        "WIP blocking should be opt-in (disabled by default)"
    );
}

#[test]
fn test_wip_check_config_default_label_is_wip() {
    let config = WipCheckConfig::default();
    assert_eq!(
        config.wip_label,
        Some("WIP".to_string()),
        "Default WIP label should be 'WIP'"
    );
}

#[test]
fn test_wip_check_config_default_title_patterns_non_empty() {
    let config = WipCheckConfig::default();
    assert!(
        !config.wip_title_patterns.is_empty(),
        "Default title patterns should not be empty"
    );
    // Check that core WIP patterns are present
    let titles = &config.wip_title_patterns;
    assert!(titles.contains(&"WIP".to_string()), "Should contain 'WIP'");
    assert!(
        titles.contains(&"wip:".to_string()),
        "Should contain 'wip:'"
    );
    assert!(
        titles.contains(&"[wip]".to_string()),
        "Should contain '[wip]'"
    );
    // "[WIP]" and "WIP:" are subsumed by "WIP" via str::contains — not in defaults
    assert!(
        !titles.contains(&"[WIP]".to_string()),
        "'[WIP]' is subsumed by 'WIP' and should not be a separate default"
    );
    assert!(
        !titles.contains(&"WIP:".to_string()),
        "'WIP:' is subsumed by 'WIP' and should not be a separate default"
    );
}

#[test]
fn test_wip_check_config_default_description_patterns_empty() {
    let config = WipCheckConfig::default();
    assert!(
        config.wip_description_patterns.is_empty(),
        "Default description patterns should be empty — WIP in body is opt-in"
    );
}

#[test]
fn test_wip_check_config_serialization_roundtrip() {
    let config = WipCheckConfig {
        enforce_wip_blocking: true,
        wip_label: Some("work-in-progress".to_string()),
        wip_title_patterns: vec!["WIP".to_string(), "DO NOT MERGE".to_string()],
        wip_description_patterns: vec!["🚧".to_string()],
    };

    let toml = toml::to_string(&config).expect("Failed to serialize WipCheckConfig");
    let roundtripped: WipCheckConfig =
        toml::from_str(&toml).expect("Failed to deserialize WipCheckConfig");

    assert_eq!(config, roundtripped);
}

#[test]
fn test_wip_check_config_deserialization_defaults_on_missing_fields() {
    let toml = r#"enforce_wip_blocking = true"#;
    let config: WipCheckConfig = toml::from_str(toml).expect("Failed to deserialize");
    assert!(config.enforce_wip_blocking);
    // Other fields should fall back to defaults
    assert_eq!(config.wip_label, Some("WIP".to_string()));
    assert!(!config.wip_title_patterns.is_empty());
    assert!(config.wip_description_patterns.is_empty());
}

#[test]
fn test_current_pr_validation_config_has_wip_check_field() {
    let config = CurrentPullRequestValidationConfiguration::default();
    // Access wip_check to verify the field exists and has correct defaults
    assert!(!config.wip_check.enforce_wip_blocking);
    assert_eq!(config.wip_check.wip_label, Some("WIP".to_string()));
}

#[test]
fn test_to_validation_config_preserves_wip_policies() {
    use crate::config::{PoliciesConfig, PullRequestsPoliciesConfig, RepositoryProvidedConfig};

    let repo_config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                wip_policies: WipCheckConfig {
                    enforce_wip_blocking: true,
                    wip_label: Some("🚧 WIP".to_string()),
                    wip_title_patterns: vec!["WIP".to_string(), "DO NOT MERGE".to_string()],
                    wip_description_patterns: vec!["draft".to_string()],
                },
                ..Default::default()
            },
            ..Default::default()
        },
        change_type_labels: None,
        ..Default::default()
    };

    let validation = repo_config.to_validation_config(&BypassRules::default());

    assert!(validation.wip_check.enforce_wip_blocking);
    assert_eq!(validation.wip_check.wip_label, Some("🚧 WIP".to_string()));
    assert_eq!(
        validation.wip_check.wip_title_patterns,
        vec!["WIP", "DO NOT MERGE"]
    );
    assert_eq!(validation.wip_check.wip_description_patterns, vec!["draft"]);
}

// ── ApplicationDefaults WIP field tests ──────────────────────────────────────

#[test]
fn test_application_defaults_has_wip_check_field() {
    let defaults = ApplicationDefaults::default();
    assert!(!defaults.wip_check.enforce_wip_blocking);
    assert_eq!(defaults.wip_check.wip_label, Some("WIP".to_string()));
}

#[tokio::test]
async fn test_load_merge_warden_config_app_defaults_enforce_wip_blocks_merge() {
    // When app defaults enable WIP blocking, that takes precedence even if the
    // repo config doesn't set it.
    let toml = r#"schemaVersion = 1"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.wip_check.enforce_wip_blocking = true;

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert!(
        config
            .policies
            .pull_requests
            .wip_policies
            .enforce_wip_blocking,
        "App-level enforce_wip_blocking should propagate into repo config"
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_repo_wip_config_preserved() {
    // Repo explicitly opts into WIP blocking and sets a custom label.
    let toml = r#"
schemaVersion = 1
[policies.pullRequests.wip]
enforce_wip_blocking = true
wip_label = "🚧 WIP"
wip_title_patterns = ["WIP", "DO NOT MERGE"]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert!(
        config
            .policies
            .pull_requests
            .wip_policies
            .enforce_wip_blocking
    );
    assert_eq!(
        config.policies.pull_requests.wip_policies.wip_label,
        Some("🚧 WIP".to_string())
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .wip_policies
            .wip_title_patterns,
        vec!["WIP", "DO NOT MERGE"]
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_app_wip_label_overrides_when_repo_uses_default() {
    // If the app sets a non-default label and the repo doesn't specify one,
    // the app label should be used.
    let toml = r#"schemaVersion = 1"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.wip_check.wip_label = Some("work-in-progress".to_string());

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert_eq!(
        config.policies.pull_requests.wip_policies.wip_label,
        Some("work-in-progress".to_string()),
        "App-level wip_label should be applied when repo uses the default"
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_app_description_patterns_propagate() {
    // App defaults can add description-based WIP patterns (empty by default).
    let toml = r#"schemaVersion = 1"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.wip_check.wip_description_patterns = vec!["🚧".to_string()];

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert_eq!(
        config
            .policies
            .pull_requests
            .wip_policies
            .wip_description_patterns,
        vec!["🚧"],
        "App-level description patterns should propagate when repo has none"
    );
}

// ---------------------------------------------------------------------------
// IssuePropagationConfig — spec assertions 6.7-1, 6.7-2, 6.7-3
// ---------------------------------------------------------------------------

/// Spec assertion 6.7-1: IssuePropagationConfig::default() has both flags false.
#[test]
fn test_issue_propagation_config_default_has_both_flags_false() {
    let config = IssuePropagationConfig::default();
    assert!(
        !config.sync_milestone_from_issue,
        "sync_milestone_from_issue should default to false"
    );
    assert!(
        !config.sync_project_from_issue,
        "sync_project_from_issue should default to false"
    );
}

/// Spec assertion 6.7-2: omitting [policies.pullRequests.issuePropagation] from TOML
/// produces IssuePropagationConfig::default() (both flags false).
#[tokio::test]
async fn test_issue_propagation_config_absent_from_toml_yields_default() {
    let toml = r#"schemaVersion = 1
[policies.pullRequests.prTitle]
required = true
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let propagation = &config.policies.pull_requests.issue_propagation;
    assert!(
        !propagation.sync_milestone_from_issue,
        "sync_milestone_from_issue should default to false when section is absent"
    );
    assert!(
        !propagation.sync_project_from_issue,
        "sync_project_from_issue should default to false when section is absent"
    );
}

/// Spec assertion 6.7-3: setting sync_milestone_from_issue = true in TOML is reflected
/// in the parsed config.
#[tokio::test]
async fn test_issue_propagation_config_sync_milestone_flag_parsed_from_toml() {
    let toml = r#"schemaVersion = 1
[policies.pullRequests.issuePropagation]
sync_milestone_from_issue = true
sync_project_from_issue = false
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let propagation = &config.policies.pull_requests.issue_propagation;
    assert!(
        propagation.sync_milestone_from_issue,
        "sync_milestone_from_issue should be true after parsing TOML"
    );
    assert!(
        !propagation.sync_project_from_issue,
        "sync_project_from_issue should be false"
    );
}

/// Spec assertion 6.7-3 (project flag variant): setting sync_project_from_issue = true
/// in TOML is reflected in the parsed config.
#[tokio::test]
async fn test_issue_propagation_config_sync_project_flag_parsed_from_toml() {
    let toml = r#"schemaVersion = 1
[policies.pullRequests.issuePropagation]
sync_milestone_from_issue = false
sync_project_from_issue = true
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let propagation = &config.policies.pull_requests.issue_propagation;
    assert!(
        !propagation.sync_milestone_from_issue,
        "sync_milestone_from_issue should be false"
    );
    assert!(
        propagation.sync_project_from_issue,
        "sync_project_from_issue should be true after parsing TOML"
    );
}

/// Verify that to_validation_config() forwards issue_propagation into
/// CurrentPullRequestValidationConfiguration.
#[tokio::test]
async fn test_to_validation_config_forwards_issue_propagation() {
    let toml = r#"schemaVersion = 1
[policies.pullRequests.issuePropagation]
sync_milestone_from_issue = true
sync_project_from_issue = true
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();
    let merge_warden_config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let validation = merge_warden_config.to_validation_config(&BypassRules::default());
    assert!(
        validation.issue_propagation.sync_milestone_from_issue,
        "to_validation_config should forward sync_milestone_from_issue = true"
    );
    assert!(
        validation.issue_propagation.sync_project_from_issue,
        "to_validation_config should forward sync_project_from_issue = true"
    );
}

// ── ApplicationDefaults snake_case TOML key tests ────────────────────────────

#[test]
fn test_application_defaults_new_field_names_round_trip_toml() {
    // Round-trip using the new snake_case TOML keys (post-rename).
    let toml_input = r#"
enable_title_validation = true
default_title_pattern = "my-title-pattern"
default_invalid_title_label = "bad-title"
enable_work_item_validation = true
default_work_item_pattern = "my-work-item-pattern"
default_missing_work_item_label = "missing-work-item"
"#;

    let defaults: ApplicationDefaults =
        toml::from_str(toml_input).expect("Should deserialize from snake_case keys");

    assert!(defaults.enable_title_validation);
    assert_eq!(defaults.default_title_pattern, "my-title-pattern");
    assert_eq!(
        defaults.default_invalid_title_label,
        Some("bad-title".to_string())
    );
    assert!(defaults.enable_work_item_validation);
    assert_eq!(defaults.default_work_item_pattern, "my-work-item-pattern");
    assert_eq!(
        defaults.default_missing_work_item_label,
        Some("missing-work-item".to_string())
    );

    // Serialize back to TOML and verify snake_case keys are present.
    let serialized = toml::to_string(&defaults).expect("Should serialize");
    assert!(
        serialized.contains("enable_title_validation"),
        "Serialized TOML must use 'enable_title_validation', got:\n{serialized}"
    );
    assert!(
        !serialized.contains("enforceTitleValidation"),
        "Serialized TOML must NOT contain legacy key 'enforceTitleValidation'"
    );
    assert!(
        serialized.contains("enable_work_item_validation"),
        "Serialized TOML must use 'enable_work_item_validation'"
    );
    assert!(
        !serialized.contains("enforceWorkItemValidation"),
        "Serialized TOML must NOT contain legacy key 'enforceWorkItemValidation'"
    );

    // Deserialize again to confirm full round-trip correctness.
    let roundtripped: ApplicationDefaults =
        toml::from_str(&serialized).expect("Should deserialize after serialization");
    assert_eq!(
        roundtripped.enable_title_validation,
        defaults.enable_title_validation
    );
    assert_eq!(
        roundtripped.default_title_pattern,
        defaults.default_title_pattern
    );
    assert_eq!(
        roundtripped.default_invalid_title_label,
        defaults.default_invalid_title_label
    );
    assert_eq!(
        roundtripped.enable_work_item_validation,
        defaults.enable_work_item_validation
    );
    assert_eq!(
        roundtripped.default_work_item_pattern,
        defaults.default_work_item_pattern
    );
    assert_eq!(
        roundtripped.default_missing_work_item_label,
        defaults.default_missing_work_item_label
    );
}

#[test]
fn test_application_defaults_camelcase_keys_no_longer_deserialize() {
    // After the rename, camelCase keys in TOML should be ignored (serde skips unknown fields),
    // so the fields fall back to their #[serde(default)] values.
    let toml_camel = r#"
enforceTitleValidation = true
titlePattern = "some-pattern"
"#;

    let defaults: ApplicationDefaults =
        toml::from_str(toml_camel).expect("Should succeed (unknown keys are ignored)");

    // Because the keys are now unknown to serde, defaults apply.
    assert!(
        !defaults.enable_title_validation,
        "enforceTitleValidation is no longer a recognised key; default (false) must be used"
    );
    assert_eq!(
        defaults.default_title_pattern,
        CONVENTIONAL_COMMIT_REGEX.to_string(),
        "titlePattern is no longer a recognised key; default pattern must be used"
    );
}

// ── KeywordLabelsConfig tests ────────────────────────────────────────────────

#[test]
fn test_keyword_labels_config_defaults() {
    let cfg = KeywordLabelsConfig::default();
    assert_eq!(cfg.breaking_change_label(), "breaking-change");
    assert_eq!(cfg.security_label(), "security");
    assert_eq!(cfg.hotfix_label(), "hotfix");
    assert_eq!(cfg.tech_debt_label(), "tech-debt");
}

#[test]
fn test_keyword_labels_config_custom_values() {
    let cfg = KeywordLabelsConfig {
        breaking_change: Some("semver-major".to_string()),
        security: Some("security-alert".to_string()),
        hotfix: Some("urgent".to_string()),
        tech_debt: Some("cleanup".to_string()),
    };
    assert_eq!(cfg.breaking_change_label(), "semver-major");
    assert_eq!(cfg.security_label(), "security-alert");
    assert_eq!(cfg.hotfix_label(), "urgent");
    assert_eq!(cfg.tech_debt_label(), "cleanup");
}

#[test]
fn test_keyword_labels_config_empty_string_falls_back_to_default() {
    let cfg = KeywordLabelsConfig {
        breaking_change: Some(String::new()),
        security: Some(String::new()),
        hotfix: Some(String::new()),
        tech_debt: Some(String::new()),
    };
    assert_eq!(cfg.breaking_change_label(), "breaking-change");
    assert_eq!(cfg.security_label(), "security");
    assert_eq!(cfg.hotfix_label(), "hotfix");
    assert_eq!(cfg.tech_debt_label(), "tech-debt");
}

#[test]
fn test_keyword_labels_config_toml_round_trip_with_custom_values() {
    let toml = r#"
[keyword_labels]
breaking_change = "semver-major"
security = "security-alert"
hotfix = "urgent"
tech_debt = "cleanup"
"#;

    let cfg: ChangeTypeLabelConfig = toml::from_str(toml).expect("Should deserialise successfully");
    assert_eq!(cfg.keyword_labels.breaking_change_label(), "semver-major");
    assert_eq!(cfg.keyword_labels.security_label(), "security-alert");
    assert_eq!(cfg.keyword_labels.hotfix_label(), "urgent");
    assert_eq!(cfg.keyword_labels.tech_debt_label(), "cleanup");

    // Round-trip: serialise then deserialise
    let serialised = toml::to_string(&cfg).expect("Should serialise successfully");
    let round_tripped: ChangeTypeLabelConfig =
        toml::from_str(&serialised).expect("Round-trip should succeed");
    assert_eq!(
        round_tripped.keyword_labels.breaking_change_label(),
        "semver-major"
    );
    assert_eq!(
        round_tripped.keyword_labels.security_label(),
        "security-alert"
    );
    assert_eq!(round_tripped.keyword_labels.hotfix_label(), "urgent");
    assert_eq!(round_tripped.keyword_labels.tech_debt_label(), "cleanup");
}

#[test]
fn test_keyword_labels_config_toml_absent_section_uses_defaults() {
    // A ChangeTypeLabelConfig with no keyword_labels section must use hard-coded defaults.
    let toml = r#"
enabled = true
"#;
    let cfg: ChangeTypeLabelConfig = toml::from_str(toml).expect("Should deserialise successfully");
    assert_eq!(
        cfg.keyword_labels.breaking_change_label(),
        "breaking-change"
    );
    assert_eq!(cfg.keyword_labels.security_label(), "security");
    assert_eq!(cfg.keyword_labels.hotfix_label(), "hotfix");
    assert_eq!(cfg.keyword_labels.tech_debt_label(), "tech-debt");
}

#[test]
fn test_change_type_label_config_default_includes_keyword_labels() {
    let cfg = ChangeTypeLabelConfig::default();
    assert_eq!(
        cfg.keyword_labels.breaking_change_label(),
        "breaking-change"
    );
    assert_eq!(cfg.keyword_labels.security_label(), "security");
    assert_eq!(cfg.keyword_labels.hotfix_label(), "hotfix");
    assert_eq!(cfg.keyword_labels.tech_debt_label(), "tech-debt");
}

// ── Config merge: keyword_labels propagated from repo config ─────────────────

/// Regression test: repo-level keyword_labels were silently dropped during the
/// change_type_labels merge step.  Every non-None field set by a repository
/// must survive the merge and override the app-default value.
#[tokio::test]
async fn test_load_merge_warden_config_keyword_labels_merged_from_repo() {
    let toml = r#"
schemaVersion = 1

[change_type_labels]
enabled = true

[change_type_labels.keyword_labels]
breaking_change = "semver-major"
security = "vulnerability"
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let kw = &config
        .change_type_labels
        .as_ref()
        .expect("change_type_labels should be populated after merge")
        .keyword_labels;

    assert_eq!(
        kw.breaking_change_label(),
        "semver-major",
        "Repo-level breaking_change label should override app default"
    );
    assert_eq!(
        kw.security_label(),
        "vulnerability",
        "Repo-level security label should override app default"
    );
    // Fields not supplied by the repo should retain the app-default values.
    assert_eq!(
        kw.hotfix_label(),
        "hotfix",
        "hotfix should keep the app default when not set by repo"
    );
    assert_eq!(
        kw.tech_debt_label(),
        "tech-debt",
        "tech_debt should keep the app default when not set by repo"
    );
}

/// When the app defaults configure non-standard keyword labels and the repo
/// does not override them, the app defaults should be preserved.
#[tokio::test]
async fn test_load_merge_warden_config_keyword_labels_app_defaults_preserved() {
    let toml = r#"
schemaVersion = 1

[change_type_labels]
enabled = true
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults
        .change_type_labels
        .keyword_labels
        .breaking_change = Some("breaking".to_string());

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let kw = &config
        .change_type_labels
        .as_ref()
        .expect("change_type_labels should be populated after merge")
        .keyword_labels;

    assert_eq!(
        kw.breaking_change_label(),
        "breaking",
        "App-default breaking_change label should be used when repo does not override it"
    );
}

// ── Bypass-rule precedence tests ──────────────────────────────────────────────

#[test]
fn test_to_validation_config_uses_repo_bypass_rules_over_server_defaults() {
    // Repo TOML specifies bypass rules → they should take precedence over whatever
    // the caller passes in as the server-level defaults.
    let repo_bypass = BypassRules::new_with_size(
        BypassRule::new(true, vec!["repo-title-bot".to_string()]),
        BypassRule::new(true, vec!["repo-workitem-bot".to_string()]),
        BypassRule::new(true, vec!["repo-size-bot".to_string()]),
    );
    let server_bypass = BypassRules::new_with_size(
        BypassRule::new(true, vec!["server-title-bot".to_string()]),
        BypassRule::new(true, vec!["server-workitem-bot".to_string()]),
        BypassRule::new(true, vec!["server-size-bot".to_string()]),
    );

    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            bypass_rules: Some(repo_bypass.clone()),
            ..Default::default()
        },
        ..Default::default()
    };

    let validation = config.to_validation_config(&server_bypass);

    // Repo bypass users are present.
    assert!(
        validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"repo-title-bot"),
        "repo title-bypass user should be present"
    );
    assert!(
        validation
            .bypass_rules
            .work_item_convention()
            .users()
            .contains(&"repo-workitem-bot"),
        "repo work-item bypass user should be present"
    );
    assert!(
        validation
            .bypass_rules
            .size()
            .users()
            .contains(&"repo-size-bot"),
        "repo size-bypass user should be present"
    );

    // Server bypass users are NOT used.
    assert!(
        !validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"server-title-bot"),
        "server title-bypass user should not leak into repo config"
    );
    assert!(
        !validation
            .bypass_rules
            .work_item_convention()
            .users()
            .contains(&"server-workitem-bot"),
        "server work-item bypass user should not leak into repo config"
    );
}

#[test]
fn test_to_validation_config_falls_back_to_server_bypass_rules_when_repo_has_none() {
    // Repo TOML does not specify bypass rules (bypass_rules is None) → fall back to
    // the server-level BypassRules that are passed into to_validation_config.
    let server_bypass = BypassRules::new_with_size(
        BypassRule::new(true, vec!["server-title-bot".to_string()]),
        BypassRule::new(true, vec!["server-workitem-bot".to_string()]),
        BypassRule::new(true, vec!["server-size-bot".to_string()]),
    );

    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            bypass_rules: None,
            ..Default::default()
        },
        ..Default::default()
    };

    let validation = config.to_validation_config(&server_bypass);

    assert!(
        validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"server-title-bot"),
        "server title-bypass user should be used when repo has no bypass rules"
    );
    assert!(
        validation
            .bypass_rules
            .work_item_convention()
            .users()
            .contains(&"server-workitem-bot"),
        "server work-item bypass user should be used when repo has no bypass rules"
    );
    assert!(
        validation
            .bypass_rules
            .size()
            .users()
            .contains(&"server-size-bot"),
        "server size-bypass user should be used when repo has no bypass rules"
    );
}

#[tokio::test]
async fn test_load_merge_warden_config_parses_bypass_rules_from_toml() {
    // Verify that all three bypass-rule sections are parsed from the TOML and
    // stored on PoliciesConfig.bypass_rules.
    let toml = r#"schemaVersion = 1

[policies.bypassRules.title_convention]
enabled = true
users = ["release-bot", "dependabot[bot]"]

[policies.bypassRules.work_items]
enabled = true
users = ["release-bot"]

[policies.bypassRules.size]
enabled = false
users = []
"#;

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();

    let config = load_merge_warden_config(
        "owner",
        "repo",
        "merge-warden.toml",
        &fetcher,
        &app_defaults,
    )
    .await
    .expect("config should load");

    let bypass = config
        .policies
        .bypass_rules
        .as_ref()
        .expect("bypass_rules should be Some after parsing the TOML");

    assert!(
        bypass.title_convention().enabled(),
        "title_convention bypass should be enabled"
    );
    assert_eq!(
        bypass.title_convention().users(),
        vec!["release-bot", "dependabot[bot]"],
        "title_convention bypass users should match TOML"
    );

    assert!(
        bypass.work_item_convention().enabled(),
        "work_items bypass should be enabled"
    );
    assert_eq!(
        bypass.work_item_convention().users(),
        vec!["release-bot"],
        "work_items bypass users should match TOML"
    );

    assert!(!bypass.size().enabled(), "size bypass should be disabled");
    assert!(
        bypass.size().users().is_empty(),
        "size bypass users should be empty"
    );
}

#[tokio::test]
async fn test_to_validation_config_repo_bypass_rules_override_server_level() {
    // End-to-end: load a TOML that has bypass rules, convert to validation config
    // using a different set of server-level bypass rules, and verify the TOML rules win.
    let toml = r#"schemaVersion = 1

[policies.bypassRules.title_convention]
enabled = true
users = ["pv-release-regent[bot]"]

[policies.bypassRules.work_items]
enabled = true
users = ["pv-release-regent[bot]"]

[policies.bypassRules.size]
enabled = true
users = ["pv-release-regent[bot]"]
"#;

    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let app_defaults = ApplicationDefaults::default();

    let repo_config = load_merge_warden_config(
        "owner",
        "repo",
        "merge-warden.toml",
        &fetcher,
        &app_defaults,
    )
    .await
    .expect("config should load");

    let server_bypass = BypassRules::new_with_size(
        BypassRule::new(true, vec!["server-only-bot".to_string()]),
        BypassRule::new(true, vec!["server-only-bot".to_string()]),
        BypassRule::new(true, vec!["server-only-bot".to_string()]),
    );

    let validation = repo_config.to_validation_config(&server_bypass);

    assert!(
        validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"pv-release-regent[bot]"),
        "TOML title bypass user should be active"
    );
    assert!(
        !validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"server-only-bot"),
        "server-only bypass user must not appear when repo TOML overrides"
    );

    assert!(
        validation
            .bypass_rules
            .work_item_convention()
            .users()
            .contains(&"pv-release-regent[bot]"),
        "TOML work-item bypass user should be active"
    );
    assert!(
        validation
            .bypass_rules
            .size()
            .users()
            .contains(&"pv-release-regent[bot]"),
        "TOML size bypass user should be active"
    );
}
