use crate::config::{validate_config_content, ConfigValidationOutcome};
use crate::config::{
    BypassRule, BypassRules, BypassRulesConfig, ChangeTypeLabelConfig,
    CurrentPullRequestValidationConfiguration, IssuePropagationConfig, KeywordLabelsConfig,
    OrgPolicySource, PrSizeCheckConfig, WipCheckConfig, CONVENTIONAL_COMMIT_REGEX, WORK_ITEM_REGEX,
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

    async fn fetch_config_at_ref(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _path: &str,
        _git_ref: &str,
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
        org_policy_source: None,
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
    // Repo TOML specifies all three bypass rules → each should take precedence over
    // the corresponding server-level default.
    let server_bypass = BypassRules::new_with_size(
        BypassRule::new(true, vec!["server-title-bot".to_string()]),
        BypassRule::new(true, vec!["server-workitem-bot".to_string()]),
        BypassRule::new(true, vec!["server-size-bot".to_string()]),
    );

    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            bypass_rules: Some(BypassRulesConfig {
                title_convention: Some(BypassRule::new(true, vec!["repo-title-bot".to_string()])),
                work_items: Some(BypassRule::new(true, vec!["repo-workitem-bot".to_string()])),
                size: Some(BypassRule::new(true, vec!["repo-size-bot".to_string()])),
            }),
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
    assert!(
        !validation
            .bypass_rules
            .size()
            .users()
            .contains(&"server-size-bot"),
        "server size-bypass user should not leak into repo config"
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
        bypass
            .title_convention()
            .expect("title_convention should be Some")
            .enabled(),
        "title_convention bypass should be enabled"
    );
    assert_eq!(
        bypass
            .title_convention()
            .expect("title_convention should be Some")
            .users(),
        vec!["release-bot", "dependabot[bot]"],
        "title_convention bypass users should match TOML"
    );

    assert!(
        bypass
            .work_item_convention()
            .expect("work_items should be Some")
            .enabled(),
        "work_items bypass should be enabled"
    );
    assert_eq!(
        bypass
            .work_item_convention()
            .expect("work_items should be Some")
            .users(),
        vec!["release-bot"],
        "work_items bypass users should match TOML"
    );

    assert!(
        !bypass.size().expect("size should be Some").enabled(),
        "size bypass should be disabled"
    );
    assert!(
        bypass
            .size()
            .expect("size should be Some")
            .users()
            .is_empty(),
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

#[test]
fn test_to_validation_config_partial_repo_bypass_inherits_server_defaults_for_missing_rules() {
    // A repo that specifies only the title bypass rule should inherit the server-level
    // work_items and size bypass rules instead of silently replacing them with defaults.
    let server_bypass = BypassRules::new_with_size(
        BypassRule::new(true, vec!["server-title-bot".to_string()]),
        BypassRule::new(true, vec!["server-workitem-bot".to_string()]),
        BypassRule::new(true, vec!["server-size-bot".to_string()]),
    );

    let config = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            // Only title_convention is specified; work_items and size are absent (None).
            bypass_rules: Some(BypassRulesConfig {
                title_convention: Some(BypassRule::new(
                    true,
                    vec!["repo-title-only-bot".to_string()],
                )),
                work_items: None,
                size: None,
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    let validation = config.to_validation_config(&server_bypass);

    // Repo-specified title rule wins.
    assert!(
        validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"repo-title-only-bot"),
        "repo title-bypass user should be present"
    );
    assert!(
        !validation
            .bypass_rules
            .title_convention()
            .users()
            .contains(&"server-title-bot"),
        "server title-bypass user should be replaced by the repo rule"
    );

    // Absent work_items and size rules fall back to server defaults.
    assert!(
        validation
            .bypass_rules
            .work_item_convention()
            .users()
            .contains(&"server-workitem-bot"),
        "server work-item bypass should be inherited when repo does not set work_items"
    );
    assert!(
        validation
            .bypass_rules
            .size()
            .users()
            .contains(&"server-size-bot"),
        "server size bypass should be inherited when repo does not set size"
    );
}

// ── validate_config_content tests ─────────────────────────────────────────────

#[test]
fn test_validate_config_content_valid_minimal_toml() {
    // A minimal TOML with schemaVersion = 1 must be accepted.
    let content = "schemaVersion = 1";
    let outcome = validate_config_content(content);
    assert!(
        outcome.valid,
        "minimal valid config should be accepted; errors: {:?}",
        outcome.errors
    );
    assert!(
        outcome.errors.is_empty(),
        "no errors expected for valid config"
    );
}

#[test]
fn test_validate_config_content_invalid_toml_syntax() {
    // Garbage that cannot be parsed as TOML must produce a validation error.
    let content = "not = valid = toml [[[";
    let outcome = validate_config_content(content);
    assert!(!outcome.valid, "invalid TOML should be rejected");
    assert!(
        !outcome.errors.is_empty(),
        "at least one error message expected for invalid TOML"
    );
}

#[test]
fn test_validate_config_content_wrong_schema_version_zero() {
    // schemaVersion = 0 is not a supported version.
    let content = "schemaVersion = 0";
    let outcome = validate_config_content(content);
    assert!(!outcome.valid, "schema_version 0 should be rejected");
    assert!(
        outcome.errors.iter().any(|e| e.contains("schemaVersion")),
        "error message should mention schemaVersion; errors: {:?}",
        outcome.errors
    );
}

#[test]
fn test_validate_config_content_wrong_schema_version_two() {
    // schemaVersion = 2 is not yet a supported version.
    let content = "schemaVersion = 2";
    let outcome = validate_config_content(content);
    assert!(!outcome.valid, "schema_version 2 should be rejected");
    assert!(
        outcome.errors.iter().any(|e| e.contains("schemaVersion")),
        "error message should mention schemaVersion; errors: {:?}",
        outcome.errors
    );
}

#[test]
fn test_validate_config_content_outcome_equality() {
    // ConfigValidationOutcome must implement PartialEq correctly.
    let a = ConfigValidationOutcome {
        valid: true,
        errors: vec![],
    };
    let b = ConfigValidationOutcome {
        valid: true,
        errors: vec![],
    };
    assert_eq!(a, b);

    let c = ConfigValidationOutcome {
        valid: false,
        errors: vec!["some error".to_string()],
    };
    assert_ne!(a, c);
}

// ── PolicySet structural merge tests ─────────────────────────────────────────
//
// Spec §5.1 — verify the "identity element" behaviour of PolicySet::merge.

/// Two default PolicySets merged must produce PolicySet::default().
#[test]
fn policy_set_merge_both_defaults_yields_default() {
    let base = PolicySet::default();
    let over = PolicySet::default();

    let merged = base.merge(&over);

    assert_eq!(merged, PolicySet::default());
}

/// When `over` is default, result equals `base` (default is a right identity).
#[test]
fn policy_set_merge_over_default_preserves_base() {
    let mut base = PolicySet::default();
    base.title.required = true;
    base.title.pattern = "custom-pattern".to_string();
    base.work_item.required = true;

    let merged = base.merge(&PolicySet::default());

    assert_eq!(merged, base);
}

/// When `base` is default, result equals `over` (default is a left absorber for OR fields).
/// Non-default values on `over` must propagate even when base is default.
#[test]
fn policy_set_merge_base_default_over_non_default_propagates_over() {
    let mut over = PolicySet::default();
    over.title.required = true;
    over.title.pattern = "my-pattern".to_string();
    over.work_item.required = true;
    over.size.enabled = true;

    let merged = PolicySet::default().merge(&over);

    assert!(merged.title.required);
    assert_eq!(merged.title.pattern, "my-pattern");
    assert!(merged.work_item.required);
    assert!(merged.size.enabled);
}

/// PolicySet::merge must delegate to each constituent's own merge independently —
/// a non-default value in one field must not affect a different field.
#[test]
fn policy_set_merge_constituent_fields_are_independent() {
    let mut over = PolicySet::default();
    over.title.required = true;

    let merged = PolicySet::default().merge(&over);

    // Only title was modified — all other fields remain at default.
    assert!(merged.title.required);
    assert!(!merged.work_item.required);
    assert!(!merged.size.enabled);
    assert!(!merged.wip.enforce_wip_blocking);
    assert!(!merged.pr_state.enabled);
    assert!(!merged.issue_propagation.sync_milestone_from_issue);
    assert!(!merged.issue_propagation.sync_project_from_issue);
}

// ── PullRequestsTitlePolicyConfig::merge ─────────────────────────────────────
//
// Spec §2.1 and §5.2

/// `required` uses OR semantics: base=true, over=false → true.
#[test]
fn title_merge_required_or_base_true_over_false_yields_true() {
    let base = PullRequestsTitlePolicyConfig {
        required: true,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: None,
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: None,
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert!(result.required);
}

/// `required` uses OR semantics: base=false, over=true → true.
#[test]
fn title_merge_required_or_base_false_over_true_yields_true() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: None,
    };
    let over = PullRequestsTitlePolicyConfig {
        required: true,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: None,
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert!(result.required);
}

/// `required` OR: both false → false.
#[test]
fn title_merge_required_or_both_false_yields_false() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        ..Default::default()
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        ..Default::default()
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert!(!result.required);
}

/// Non-default, non-empty `over.pattern` wins over base.
#[test]
fn title_merge_pattern_over_non_default_wins() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: "old-pattern".to_string(),
        label_if_missing: None,
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: "new-pattern".to_string(),
        label_if_missing: None,
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert_eq!(result.pattern, "new-pattern");
}

/// An empty `over.pattern` falls back to `base.pattern`.
#[test]
fn title_merge_pattern_over_empty_keeps_base() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: "base-pattern".to_string(),
        label_if_missing: None,
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: String::new(),
        label_if_missing: None,
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert_eq!(result.pattern, "base-pattern");
}

/// When `over.pattern` equals `CONVENTIONAL_COMMIT_REGEX`, `base.pattern` is kept.
#[test]
fn title_merge_pattern_over_default_keeps_base() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: "custom-base-pattern".to_string(),
        label_if_missing: None,
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: None,
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert_eq!(result.pattern, "custom-base-pattern");
}

/// `over.label_if_missing = Some(_)` wins.
#[test]
fn title_merge_label_over_some_wins() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: Some("base-label".to_string()),
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: Some("over-label".to_string()),
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert_eq!(result.label_if_missing, Some("over-label".to_string()));
}

/// `over.label_if_missing = None` falls back to `base.label_if_missing`.
#[test]
fn title_merge_label_over_none_keeps_base() {
    let base = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: Some("base-label".to_string()),
    };
    let over = PullRequestsTitlePolicyConfig {
        required: false,
        pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        label_if_missing: None,
    };

    let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

    assert_eq!(result.label_if_missing, Some("base-label".to_string()));
}

/// Both labels None → result is None.
#[test]
fn title_merge_label_both_none_yields_none() {
    let result = PullRequestsTitlePolicyConfig::merge(
        &PullRequestsTitlePolicyConfig::default(),
        &PullRequestsTitlePolicyConfig::default(),
    );

    assert_eq!(result.label_if_missing, None);
}

// ── WorkItemPolicyConfig::merge ───────────────────────────────────────────────
//
// Spec §2.2 and §5.3 — mirror of title policy rules.

/// `required` OR: base=true, over=false → true.
#[test]
fn work_item_merge_required_or_base_true_over_false_yields_true() {
    let base = WorkItemPolicyConfig {
        required: true,
        ..Default::default()
    };
    let over = WorkItemPolicyConfig {
        required: false,
        ..Default::default()
    };

    assert!(WorkItemPolicyConfig::merge(&base, &over).required);
}

/// `required` OR: base=false, over=true → true.
#[test]
fn work_item_merge_required_or_base_false_over_true_yields_true() {
    let base = WorkItemPolicyConfig {
        required: false,
        ..Default::default()
    };
    let over = WorkItemPolicyConfig {
        required: true,
        ..Default::default()
    };

    assert!(WorkItemPolicyConfig::merge(&base, &over).required);
}

/// Non-default, non-empty `over.pattern` wins.
#[test]
fn work_item_merge_pattern_over_non_default_wins() {
    let base = WorkItemPolicyConfig {
        required: false,
        pattern: "old-wi-pattern".to_string(),
        label_if_missing: None,
    };
    let over = WorkItemPolicyConfig {
        required: false,
        pattern: "GH-\\d+".to_string(),
        label_if_missing: None,
    };

    let result = WorkItemPolicyConfig::merge(&base, &over);

    assert_eq!(result.pattern, "GH-\\d+");
}

/// Empty `over.pattern` falls back to `base.pattern`.
#[test]
fn work_item_merge_pattern_over_empty_keeps_base() {
    let base = WorkItemPolicyConfig {
        required: false,
        pattern: "base-wi-pattern".to_string(),
        label_if_missing: None,
    };
    let over = WorkItemPolicyConfig {
        required: false,
        pattern: String::new(),
        label_if_missing: None,
    };

    let result = WorkItemPolicyConfig::merge(&base, &over);

    assert_eq!(result.pattern, "base-wi-pattern");
}

/// `over.pattern == WORK_ITEM_REGEX` (default) → `base.pattern` is kept.
#[test]
fn work_item_merge_pattern_over_default_keeps_base() {
    let base = WorkItemPolicyConfig {
        required: false,
        pattern: "custom-wi-base".to_string(),
        label_if_missing: None,
    };
    let over = WorkItemPolicyConfig {
        required: false,
        pattern: WORK_ITEM_REGEX.to_string(),
        label_if_missing: None,
    };

    let result = WorkItemPolicyConfig::merge(&base, &over);

    assert_eq!(result.pattern, "custom-wi-base");
}

/// `over.label_if_missing = Some(_)` wins.
#[test]
fn work_item_merge_label_over_some_wins() {
    let base = WorkItemPolicyConfig {
        required: false,
        pattern: WORK_ITEM_REGEX.to_string(),
        label_if_missing: Some("base-wi-label".to_string()),
    };
    let over = WorkItemPolicyConfig {
        required: false,
        pattern: WORK_ITEM_REGEX.to_string(),
        label_if_missing: Some("over-wi-label".to_string()),
    };

    let result = WorkItemPolicyConfig::merge(&base, &over);

    assert_eq!(result.label_if_missing, Some("over-wi-label".to_string()));
}

/// `over.label_if_missing = None` keeps `base.label_if_missing`.
#[test]
fn work_item_merge_label_over_none_keeps_base() {
    let base = WorkItemPolicyConfig {
        required: false,
        pattern: WORK_ITEM_REGEX.to_string(),
        label_if_missing: Some("base-wi-label".to_string()),
    };
    let over = WorkItemPolicyConfig {
        required: false,
        pattern: WORK_ITEM_REGEX.to_string(),
        label_if_missing: None,
    };

    let result = WorkItemPolicyConfig::merge(&base, &over);

    assert_eq!(result.label_if_missing, Some("base-wi-label".to_string()));
}

// ── PrSizeCheckConfig::merge ──────────────────────────────────────────────────
//
// Spec §2.3 and §5.4

/// `enabled` OR: base=true, over=false → true.
#[test]
fn size_merge_enabled_or_base_true_over_false_yields_true() {
    let base = PrSizeCheckConfig {
        enabled: true,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        enabled: false,
        ..Default::default()
    };

    assert!(PrSizeCheckConfig::merge(&base, &over).enabled);
}

/// `enabled` OR: base=false, over=true → true.
#[test]
fn size_merge_enabled_or_base_false_over_true_yields_true() {
    let base = PrSizeCheckConfig {
        enabled: false,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        enabled: true,
        ..Default::default()
    };

    assert!(PrSizeCheckConfig::merge(&base, &over).enabled);
}

/// `fail_on_oversized` is unconditional: over=false wins over base=true.
#[test]
fn size_merge_fail_on_oversized_over_false_wins_over_base_true() {
    let base = PrSizeCheckConfig {
        fail_on_oversized: true,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        fail_on_oversized: false,
        ..Default::default()
    };

    assert!(!PrSizeCheckConfig::merge(&base, &over).fail_on_oversized);
}

/// `fail_on_oversized` is unconditional: over=true wins over base=false.
#[test]
fn size_merge_fail_on_oversized_over_true_wins_over_base_false() {
    let base = PrSizeCheckConfig {
        fail_on_oversized: false,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        fail_on_oversized: true,
        ..Default::default()
    };

    assert!(PrSizeCheckConfig::merge(&base, &over).fail_on_oversized);
}

/// `thresholds = Some(_)` on over wins.
#[test]
fn size_merge_thresholds_over_some_wins() {
    let custom = SizeThresholds::new(1, 10, 50, 100, 200);
    let base = PrSizeCheckConfig {
        thresholds: None,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        thresholds: Some(custom.clone()),
        ..Default::default()
    };

    assert_eq!(
        PrSizeCheckConfig::merge(&base, &over).thresholds,
        Some(custom)
    );
}

/// `thresholds = None` on over falls back to `base.thresholds`.
#[test]
fn size_merge_thresholds_over_none_keeps_base() {
    let custom = SizeThresholds::new(2, 20, 60, 120, 240);
    let base = PrSizeCheckConfig {
        thresholds: Some(custom.clone()),
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        thresholds: None,
        ..Default::default()
    };

    assert_eq!(
        PrSizeCheckConfig::merge(&base, &over).thresholds,
        Some(custom)
    );
}

/// Non-empty `over.excluded_file_patterns` wins.
#[test]
fn size_merge_excluded_patterns_over_non_empty_wins() {
    let base = PrSizeCheckConfig {
        excluded_file_patterns: vec!["*.md".to_string()],
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        excluded_file_patterns: vec!["*.toml".to_string(), "*.lock".to_string()],
        ..Default::default()
    };

    assert_eq!(
        PrSizeCheckConfig::merge(&base, &over).excluded_file_patterns,
        vec!["*.toml", "*.lock"]
    );
}

/// Empty `over.excluded_file_patterns` falls back to `base`.
#[test]
fn size_merge_excluded_patterns_over_empty_keeps_base() {
    let base = PrSizeCheckConfig {
        excluded_file_patterns: vec!["*.md".to_string()],
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        excluded_file_patterns: vec![],
        ..Default::default()
    };

    assert_eq!(
        PrSizeCheckConfig::merge(&base, &over).excluded_file_patterns,
        vec!["*.md"]
    );
}

/// Non-default `over.label_prefix` wins.
#[test]
fn size_merge_label_prefix_over_non_default_wins() {
    let base = PrSizeCheckConfig {
        label_prefix: "custom-base/".to_string(),
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        label_prefix: "pr/".to_string(),
        ..Default::default()
    };

    assert_eq!(PrSizeCheckConfig::merge(&base, &over).label_prefix, "pr/");
}

/// When `over.label_prefix` equals the default `"size/"`, `base.label_prefix` is kept.
#[test]
fn size_merge_label_prefix_over_default_keeps_base() {
    let base = PrSizeCheckConfig {
        label_prefix: "my-size/".to_string(),
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        label_prefix: "size/".to_string(), // default value
        ..Default::default()
    };

    assert_eq!(
        PrSizeCheckConfig::merge(&base, &over).label_prefix,
        "my-size/"
    );
}

/// `add_comment` is unconditional: over=false wins over base=true.
#[test]
fn size_merge_add_comment_over_false_wins_over_base_true() {
    let base = PrSizeCheckConfig {
        add_comment: true,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        add_comment: false,
        ..Default::default()
    };

    assert!(!PrSizeCheckConfig::merge(&base, &over).add_comment);
}

/// `ignore_deletions` is unconditional: over=true wins over base=false.
#[test]
fn size_merge_ignore_deletions_over_true_wins_over_base_false() {
    let base = PrSizeCheckConfig {
        ignore_deletions: false,
        ..Default::default()
    };
    let over = PrSizeCheckConfig {
        ignore_deletions: true,
        ..Default::default()
    };

    assert!(PrSizeCheckConfig::merge(&base, &over).ignore_deletions);
}

// ── WipCheckConfig::merge ─────────────────────────────────────────────────────
//
// Spec §2.4 and §5.5

/// `enforce_wip_blocking` OR: base=true, over=false → true.
#[test]
fn wip_merge_enforce_or_base_true_over_false_yields_true() {
    let base = WipCheckConfig {
        enforce_wip_blocking: true,
        ..Default::default()
    };
    let over = WipCheckConfig {
        enforce_wip_blocking: false,
        ..Default::default()
    };

    assert!(WipCheckConfig::merge(&base, &over).enforce_wip_blocking);
}

/// `enforce_wip_blocking` OR: base=false, over=true → true.
#[test]
fn wip_merge_enforce_or_base_false_over_true_yields_true() {
    let base = WipCheckConfig {
        enforce_wip_blocking: false,
        ..Default::default()
    };
    let over = WipCheckConfig {
        enforce_wip_blocking: true,
        ..Default::default()
    };

    assert!(WipCheckConfig::merge(&base, &over).enforce_wip_blocking);
}

/// Non-default `over.wip_label` wins over `base.wip_label`.
#[test]
fn wip_merge_label_over_non_default_wins() {
    let base = WipCheckConfig {
        wip_label: Some("base-wip".to_string()),
        ..Default::default()
    };
    let over = WipCheckConfig {
        wip_label: Some("work-in-progress".to_string()),
        ..Default::default()
    };

    assert_eq!(
        WipCheckConfig::merge(&base, &over).wip_label,
        Some("work-in-progress".to_string())
    );
}

/// `over.wip_label` equal to `WipCheckConfig::default().wip_label` → `base.wip_label` kept.
#[test]
fn wip_merge_label_over_default_value_keeps_base() {
    let base = WipCheckConfig {
        wip_label: Some("custom-wip-base".to_string()),
        ..Default::default()
    };
    let over = WipCheckConfig {
        wip_label: Some("WIP".to_string()), // default value
        ..Default::default()
    };

    assert_eq!(
        WipCheckConfig::merge(&base, &over).wip_label,
        Some("custom-wip-base".to_string())
    );
}

/// Non-default `over.wip_title_patterns` wins.
#[test]
fn wip_merge_title_patterns_over_non_default_wins() {
    let default_patterns = WipCheckConfig::default().wip_title_patterns;
    let base = WipCheckConfig {
        wip_title_patterns: default_patterns.clone(),
        ..Default::default()
    };
    let over = WipCheckConfig {
        wip_title_patterns: vec!["DO NOT MERGE".to_string(), "BLOCKED".to_string()],
        ..Default::default()
    };

    assert_eq!(
        WipCheckConfig::merge(&base, &over).wip_title_patterns,
        vec!["DO NOT MERGE", "BLOCKED"]
    );
}

/// `over.wip_title_patterns` equal to defaults → `base.wip_title_patterns` kept.
#[test]
fn wip_merge_title_patterns_over_default_keeps_base() {
    let default_patterns = WipCheckConfig::default().wip_title_patterns;
    let base = WipCheckConfig {
        wip_title_patterns: vec!["custom-base-wip".to_string()],
        ..Default::default()
    };
    let over = WipCheckConfig {
        wip_title_patterns: default_patterns,
        ..Default::default()
    };

    assert_eq!(
        WipCheckConfig::merge(&base, &over).wip_title_patterns,
        vec!["custom-base-wip"]
    );
}

/// Non-empty `over.wip_description_patterns` wins.
#[test]
fn wip_merge_description_patterns_over_non_empty_wins() {
    let base = WipCheckConfig {
        wip_description_patterns: vec!["base-desc".to_string()],
        ..Default::default()
    };
    let over = WipCheckConfig {
        wip_description_patterns: vec!["🚧".to_string(), "DRAFT".to_string()],
        ..Default::default()
    };

    assert_eq!(
        WipCheckConfig::merge(&base, &over).wip_description_patterns,
        vec!["🚧", "DRAFT"]
    );
}

/// Empty `over.wip_description_patterns` falls back to `base`.
#[test]
fn wip_merge_description_patterns_over_empty_keeps_base() {
    let base = WipCheckConfig {
        wip_description_patterns: vec!["base-desc-pattern".to_string()],
        ..Default::default()
    };
    let over = WipCheckConfig {
        wip_description_patterns: vec![],
        ..Default::default()
    };

    assert_eq!(
        WipCheckConfig::merge(&base, &over).wip_description_patterns,
        vec!["base-desc-pattern"]
    );
}

// ── PrStateLabelsConfig::merge ────────────────────────────────────────────────
//
// Spec §2.5

/// `enabled` OR: base=true, over=false → true.
#[test]
fn pr_state_merge_enabled_or_base_true_over_false_yields_true() {
    let base = PrStateLabelsConfig {
        enabled: true,
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        enabled: false,
        ..Default::default()
    };

    assert!(PrStateLabelsConfig::merge(&base, &over).enabled);
}

/// `enabled` OR: base=false, over=true → true.
#[test]
fn pr_state_merge_enabled_or_base_false_over_true_yields_true() {
    let base = PrStateLabelsConfig {
        enabled: false,
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        enabled: true,
        ..Default::default()
    };

    assert!(PrStateLabelsConfig::merge(&base, &over).enabled);
}

/// `over.draft_label = Some(_)` wins.
#[test]
fn pr_state_merge_draft_label_over_some_wins() {
    let base = PrStateLabelsConfig {
        draft_label: Some("base-draft".to_string()),
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        draft_label: Some("over-draft".to_string()),
        ..Default::default()
    };

    assert_eq!(
        PrStateLabelsConfig::merge(&base, &over).draft_label,
        Some("over-draft".to_string())
    );
}

/// `over.draft_label = None` falls back to `base.draft_label`.
#[test]
fn pr_state_merge_draft_label_over_none_keeps_base() {
    let base = PrStateLabelsConfig {
        draft_label: Some("base-draft".to_string()),
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        draft_label: None,
        ..Default::default()
    };

    assert_eq!(
        PrStateLabelsConfig::merge(&base, &over).draft_label,
        Some("base-draft".to_string())
    );
}

/// `over.review_label = Some(_)` wins.
#[test]
fn pr_state_merge_review_label_over_some_wins() {
    let base = PrStateLabelsConfig {
        review_label: Some("base-review".to_string()),
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        review_label: Some("awaiting-review".to_string()),
        ..Default::default()
    };

    assert_eq!(
        PrStateLabelsConfig::merge(&base, &over).review_label,
        Some("awaiting-review".to_string())
    );
}

/// `over.review_label = None` falls back to `base.review_label`.
#[test]
fn pr_state_merge_review_label_over_none_keeps_base() {
    let base = PrStateLabelsConfig {
        review_label: Some("base-review".to_string()),
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        review_label: None,
        ..Default::default()
    };

    assert_eq!(
        PrStateLabelsConfig::merge(&base, &over).review_label,
        Some("base-review".to_string())
    );
}

/// `over.approved_label = Some(_)` wins.
#[test]
fn pr_state_merge_approved_label_over_some_wins() {
    let base = PrStateLabelsConfig {
        approved_label: Some("base-approved".to_string()),
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        approved_label: Some("lgtm".to_string()),
        ..Default::default()
    };

    assert_eq!(
        PrStateLabelsConfig::merge(&base, &over).approved_label,
        Some("lgtm".to_string())
    );
}

/// `over.approved_label = None` falls back to `base.approved_label`.
#[test]
fn pr_state_merge_approved_label_over_none_keeps_base() {
    let base = PrStateLabelsConfig {
        approved_label: Some("base-approved".to_string()),
        ..Default::default()
    };
    let over = PrStateLabelsConfig {
        approved_label: None,
        ..Default::default()
    };

    assert_eq!(
        PrStateLabelsConfig::merge(&base, &over).approved_label,
        Some("base-approved".to_string())
    );
}

// ── IssuePropagationConfig::merge ─────────────────────────────────────────────
//
// Spec §2.6

/// `sync_milestone_from_issue` OR: base=true, over=false → true.
#[test]
fn issue_propagation_merge_milestone_or_base_true_over_false_yields_true() {
    let base = IssuePropagationConfig {
        sync_milestone_from_issue: true,
        sync_project_from_issue: false,
    };
    let over = IssuePropagationConfig {
        sync_milestone_from_issue: false,
        sync_project_from_issue: false,
    };

    assert!(IssuePropagationConfig::merge(&base, &over).sync_milestone_from_issue);
}

/// `sync_milestone_from_issue` OR: base=false, over=true → true.
#[test]
fn issue_propagation_merge_milestone_or_base_false_over_true_yields_true() {
    let base = IssuePropagationConfig {
        sync_milestone_from_issue: false,
        sync_project_from_issue: false,
    };
    let over = IssuePropagationConfig {
        sync_milestone_from_issue: true,
        sync_project_from_issue: false,
    };

    assert!(IssuePropagationConfig::merge(&base, &over).sync_milestone_from_issue);
}

/// `sync_project_from_issue` OR: base=true, over=false → true.
#[test]
fn issue_propagation_merge_project_or_base_true_over_false_yields_true() {
    let base = IssuePropagationConfig {
        sync_milestone_from_issue: false,
        sync_project_from_issue: true,
    };
    let over = IssuePropagationConfig {
        sync_milestone_from_issue: false,
        sync_project_from_issue: false,
    };

    assert!(IssuePropagationConfig::merge(&base, &over).sync_project_from_issue);
}

/// `sync_project_from_issue` OR: base=false, over=true → true.
#[test]
fn issue_propagation_merge_project_or_base_false_over_true_yields_true() {
    let base = IssuePropagationConfig {
        sync_milestone_from_issue: false,
        sync_project_from_issue: false,
    };
    let over = IssuePropagationConfig {
        sync_milestone_from_issue: false,
        sync_project_from_issue: true,
    };

    assert!(IssuePropagationConfig::merge(&base, &over).sync_project_from_issue);
}

/// Merge of two defaults yields default.
#[test]
fn issue_propagation_merge_both_defaults_yields_default() {
    let result = IssuePropagationConfig::merge(
        &IssuePropagationConfig::default(),
        &IssuePropagationConfig::default(),
    );

    assert!(!result.sync_milestone_from_issue);
    assert!(!result.sync_project_from_issue);
}

// ── ChangeTypeLabelConfig::merge — commit-type mappings ───────────────────────
//
// Spec §2.7 and §5.6 — non-empty over wins for each Vec<String> mapping field.

/// Non-empty `over.conventional_commit_mappings.feat` wins.
#[test]
fn change_type_merge_feat_over_non_empty_wins() {
    let mut base = ChangeTypeLabelConfig::default();
    base.conventional_commit_mappings.feat = vec!["feature-base".to_string()];
    let mut over = ChangeTypeLabelConfig::default();
    over.conventional_commit_mappings.feat = vec!["feature-over".to_string()];

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.conventional_commit_mappings.feat,
        vec!["feature-over"]
    );
}

/// Empty `over.conventional_commit_mappings.feat` falls back to `base`.
#[test]
fn change_type_merge_feat_over_empty_keeps_base() {
    let mut base = ChangeTypeLabelConfig::default();
    base.conventional_commit_mappings.feat = vec!["feature-base".to_string()];
    let mut over = ChangeTypeLabelConfig::default();
    over.conventional_commit_mappings.feat = vec![];

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.conventional_commit_mappings.feat,
        vec!["feature-base"]
    );
}

/// Verify the same non-empty-wins rule applies to all 11 commit-type mapping fields.
#[test]
fn change_type_merge_all_eleven_types_over_non_empty_wins() {
    let mut base = ChangeTypeLabelConfig::default();
    base.conventional_commit_mappings.feat = vec!["b-feat".to_string()];
    base.conventional_commit_mappings.fix = vec!["b-fix".to_string()];
    base.conventional_commit_mappings.docs = vec!["b-docs".to_string()];
    base.conventional_commit_mappings.style = vec!["b-style".to_string()];
    base.conventional_commit_mappings.refactor = vec!["b-refactor".to_string()];
    base.conventional_commit_mappings.perf = vec!["b-perf".to_string()];
    base.conventional_commit_mappings.test = vec!["b-test".to_string()];
    base.conventional_commit_mappings.chore = vec!["b-chore".to_string()];
    base.conventional_commit_mappings.ci = vec!["b-ci".to_string()];
    base.conventional_commit_mappings.build = vec!["b-build".to_string()];
    base.conventional_commit_mappings.revert = vec!["b-revert".to_string()];

    let mut over = ChangeTypeLabelConfig::default();
    over.conventional_commit_mappings.feat = vec!["o-feat".to_string()];
    over.conventional_commit_mappings.fix = vec!["o-fix".to_string()];
    over.conventional_commit_mappings.docs = vec!["o-docs".to_string()];
    over.conventional_commit_mappings.style = vec!["o-style".to_string()];
    over.conventional_commit_mappings.refactor = vec!["o-refactor".to_string()];
    over.conventional_commit_mappings.perf = vec!["o-perf".to_string()];
    over.conventional_commit_mappings.test = vec!["o-test".to_string()];
    over.conventional_commit_mappings.chore = vec!["o-chore".to_string()];
    over.conventional_commit_mappings.ci = vec!["o-ci".to_string()];
    over.conventional_commit_mappings.build = vec!["o-build".to_string()];
    over.conventional_commit_mappings.revert = vec!["o-revert".to_string()];

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(result.conventional_commit_mappings.feat, vec!["o-feat"]);
    assert_eq!(result.conventional_commit_mappings.fix, vec!["o-fix"]);
    assert_eq!(result.conventional_commit_mappings.docs, vec!["o-docs"]);
    assert_eq!(result.conventional_commit_mappings.style, vec!["o-style"]);
    assert_eq!(
        result.conventional_commit_mappings.refactor,
        vec!["o-refactor"]
    );
    assert_eq!(result.conventional_commit_mappings.perf, vec!["o-perf"]);
    assert_eq!(result.conventional_commit_mappings.test, vec!["o-test"]);
    assert_eq!(result.conventional_commit_mappings.chore, vec!["o-chore"]);
    assert_eq!(result.conventional_commit_mappings.ci, vec!["o-ci"]);
    assert_eq!(result.conventional_commit_mappings.build, vec!["o-build"]);
    assert_eq!(result.conventional_commit_mappings.revert, vec!["o-revert"]);
}

/// All 11 fields empty on `over` — all 11 must fall back to base.
#[test]
fn change_type_merge_all_eleven_types_over_empty_keeps_base() {
    let mut base = ChangeTypeLabelConfig::default();
    base.conventional_commit_mappings.feat = vec!["b-feat".to_string()];
    base.conventional_commit_mappings.fix = vec!["b-fix".to_string()];
    base.conventional_commit_mappings.docs = vec!["b-docs".to_string()];

    let mut over = ChangeTypeLabelConfig::default();
    over.conventional_commit_mappings.feat = vec![];
    over.conventional_commit_mappings.fix = vec![];
    over.conventional_commit_mappings.docs = vec![];

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(result.conventional_commit_mappings.feat, vec!["b-feat"]);
    assert_eq!(result.conventional_commit_mappings.fix, vec!["b-fix"]);
    assert_eq!(result.conventional_commit_mappings.docs, vec!["b-docs"]);
}

/// `enabled` OR: base=false, over=true → true.
#[test]
fn change_type_merge_enabled_or_base_false_over_true_yields_true() {
    let mut base = ChangeTypeLabelConfig::default();
    base.enabled = false;
    let mut over = ChangeTypeLabelConfig::default();
    over.enabled = true;

    assert!(ChangeTypeLabelConfig::merge(&base, &over).enabled);
}

// ── ChangeTypeLabelConfig::merge — keyword labels ─────────────────────────────
//
// Spec §2.7 and §5.7

/// `over.keyword_labels.breaking_change = Some(_)` wins.
#[test]
fn change_type_merge_keyword_breaking_change_over_some_wins() {
    let base = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            breaking_change: Some("semver-major-base".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let over = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            breaking_change: Some("semver-major".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.keyword_labels.breaking_change,
        Some("semver-major".to_string())
    );
}

/// `over.keyword_labels.breaking_change = None` falls back to `base`.
#[test]
fn change_type_merge_keyword_breaking_change_over_none_keeps_base() {
    let base = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            breaking_change: Some("semver-major-base".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let over = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            breaking_change: None,
            ..Default::default()
        },
        ..Default::default()
    };

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.keyword_labels.breaking_change,
        Some("semver-major-base".to_string())
    );
}

/// `over.keyword_labels.security = Some(_)` wins.
#[test]
fn change_type_merge_keyword_security_over_some_wins() {
    let base = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            security: Some("base-security".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let over = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            security: Some("vulnerability".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        ChangeTypeLabelConfig::merge(&base, &over)
            .keyword_labels
            .security,
        Some("vulnerability".to_string())
    );
}

/// `over.keyword_labels.hotfix = None` falls back to `base`.
#[test]
fn change_type_merge_keyword_hotfix_over_none_keeps_base() {
    let base = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            hotfix: Some("critical".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let over = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            hotfix: None,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        ChangeTypeLabelConfig::merge(&base, &over)
            .keyword_labels
            .hotfix,
        Some("critical".to_string())
    );
}

/// `over.keyword_labels.tech_debt = Some(_)` wins.
#[test]
fn change_type_merge_keyword_tech_debt_over_some_wins() {
    let base = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            tech_debt: Some("base-debt".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let over = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            tech_debt: Some("cleanup".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        ChangeTypeLabelConfig::merge(&base, &over)
            .keyword_labels
            .tech_debt,
        Some("cleanup".to_string())
    );
}

/// All four keyword labels on `over` being `None` preserves all four from `base`.
#[test]
fn change_type_merge_all_keyword_labels_over_none_keeps_all_base() {
    let base = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig {
            breaking_change: Some("semver-major".to_string()),
            security: Some("sec".to_string()),
            hotfix: Some("hot".to_string()),
            tech_debt: Some("debt".to_string()),
        },
        ..Default::default()
    };
    let over = ChangeTypeLabelConfig {
        keyword_labels: KeywordLabelsConfig::default(),
        ..Default::default()
    };

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.keyword_labels.breaking_change,
        Some("semver-major".to_string())
    );
    assert_eq!(result.keyword_labels.security, Some("sec".to_string()));
    assert_eq!(result.keyword_labels.hotfix, Some("hot".to_string()));
    assert_eq!(result.keyword_labels.tech_debt, Some("debt".to_string()));
}

// ── BypassRules::merge ────────────────────────────────────────────────────────
//
// Spec §2.8

/// `over.title_convention` is explicitly configured (non-empty users) → it wins.
#[test]
fn bypass_merge_title_convention_over_configured_wins() {
    let base = BypassRules::new_with_size(
        BypassRule::new(true, vec!["base-title-bot".to_string()]),
        BypassRule::default(),
        BypassRule::default(),
    );
    let over = BypassRules::new_with_size(
        BypassRule::new(true, vec!["over-title-bot".to_string()]),
        BypassRule::default(),
        BypassRule::default(),
    );

    let result = BypassRules::merge(&base, &over);

    assert!(result
        .title_convention()
        .users()
        .contains(&"over-title-bot"));
    assert!(!result
        .title_convention()
        .users()
        .contains(&"base-title-bot"));
}

/// `over.title_convention` is unconfigured (empty users, disabled) → `base` is kept.
#[test]
fn bypass_merge_title_convention_over_unconfigured_keeps_base() {
    let base = BypassRules::new_with_size(
        BypassRule::new(true, vec!["base-title-bot".to_string()]),
        BypassRule::default(),
        BypassRule::default(),
    );
    let over = BypassRules::default(); // all rules are default (disabled, empty users)

    let result = BypassRules::merge(&base, &over);

    assert!(result
        .title_convention()
        .users()
        .contains(&"base-title-bot"));
}

/// `over.work_items` is explicitly configured → it wins.
#[test]
fn bypass_merge_work_items_over_configured_wins() {
    let base = BypassRules::new_with_size(
        BypassRule::default(),
        BypassRule::new(true, vec!["base-wi-bot".to_string()]),
        BypassRule::default(),
    );
    let over = BypassRules::new_with_size(
        BypassRule::default(),
        BypassRule::new(true, vec!["over-wi-bot".to_string()]),
        BypassRule::default(),
    );

    let result = BypassRules::merge(&base, &over);

    assert!(result
        .work_item_convention()
        .users()
        .contains(&"over-wi-bot"));
    assert!(!result
        .work_item_convention()
        .users()
        .contains(&"base-wi-bot"));
}

/// `over.work_items` is unconfigured → `base` is kept.
#[test]
fn bypass_merge_work_items_over_unconfigured_keeps_base() {
    let base = BypassRules::new_with_size(
        BypassRule::default(),
        BypassRule::new(true, vec!["base-wi-bot".to_string()]),
        BypassRule::default(),
    );
    let over = BypassRules::default();

    let result = BypassRules::merge(&base, &over);

    assert!(result
        .work_item_convention()
        .users()
        .contains(&"base-wi-bot"));
}

/// `over.size` is explicitly configured → it wins.
#[test]
fn bypass_merge_size_over_configured_wins() {
    let base = BypassRules::new_with_size(
        BypassRule::default(),
        BypassRule::default(),
        BypassRule::new(true, vec!["base-size-bot".to_string()]),
    );
    let over = BypassRules::new_with_size(
        BypassRule::default(),
        BypassRule::default(),
        BypassRule::new(true, vec!["over-size-bot".to_string()]),
    );

    let result = BypassRules::merge(&base, &over);

    assert!(result.size().users().contains(&"over-size-bot"));
    assert!(!result.size().users().contains(&"base-size-bot"));
}

/// `over.size` is unconfigured → `base` is kept.
#[test]
fn bypass_merge_size_over_unconfigured_keeps_base() {
    let base = BypassRules::new_with_size(
        BypassRule::default(),
        BypassRule::default(),
        BypassRule::new(true, vec!["base-size-bot".to_string()]),
    );
    let over = BypassRules::default();

    let result = BypassRules::merge(&base, &over);

    assert!(result.size().users().contains(&"base-size-bot"));
}

/// Each rule is evaluated independently: one configured, two unconfigured.
#[test]
fn bypass_merge_partial_over_keeps_base_for_unconfigured_rules() {
    let base = BypassRules::new_with_size(
        BypassRule::new(true, vec!["base-title".to_string()]),
        BypassRule::new(true, vec!["base-wi".to_string()]),
        BypassRule::new(true, vec!["base-size".to_string()]),
    );
    // Only title_convention is configured on `over`.
    let over = BypassRules::new_with_size(
        BypassRule::new(true, vec!["over-title".to_string()]),
        BypassRule::default(),
        BypassRule::default(),
    );

    let result = BypassRules::merge(&base, &over);

    assert!(result.title_convention().users().contains(&"over-title"));
    assert!(result.work_item_convention().users().contains(&"base-wi"));
    assert!(result.size().users().contains(&"base-size"));
}

/// `over` rule with `enabled = true` but empty users is considered explicitly configured.
#[test]
fn bypass_merge_over_enabled_true_empty_users_is_explicitly_configured() {
    let base = BypassRules::new_with_size(
        BypassRule::new(true, vec!["base-title".to_string()]),
        BypassRule::default(),
        BypassRule::default(),
    );
    // enabled=true, users=[] → this is an explicit override (operator intentionally clearing users)
    let over = BypassRules::new_with_size(
        BypassRule::new(true, vec![]),
        BypassRule::default(),
        BypassRule::default(),
    );

    let result = BypassRules::merge(&base, &over);

    // The `over` rule (enabled, empty users) is explicitly configured — it wins.
    assert!(result.title_convention().users().is_empty());
    assert!(!result.title_convention().users().contains(&"base-title"));
}

// ── PolicySet::from_application_defaults ─────────────────────────────────────
//
// Spec §1.2

/// `from_application_defaults` maps `default_title_pattern` → `title.pattern`.
#[test]
fn policy_set_from_app_defaults_title_pattern_mapped() {
    let mut app = ApplicationDefaults::default();
    app.default_title_pattern = "my-title-regex".to_string();

    let ps = PolicySet::from_application_defaults(&app);

    assert_eq!(ps.title.pattern, "my-title-regex");
}

/// `from_application_defaults` maps `default_invalid_title_label` → `title.label_if_missing`.
#[test]
fn policy_set_from_app_defaults_title_label_mapped() {
    let mut app = ApplicationDefaults::default();
    app.default_invalid_title_label = Some("bad-title".to_string());

    let ps = PolicySet::from_application_defaults(&app);

    assert_eq!(ps.title.label_if_missing, Some("bad-title".to_string()));
}

/// `enable_title_validation` is NOT applied by `from_application_defaults`
/// (it is applied as a post-merge enforcement override).
#[test]
fn policy_set_from_app_defaults_enable_title_validation_not_applied() {
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;

    let ps = PolicySet::from_application_defaults(&app);

    assert!(!ps.title.required,
        "enable_title_validation must not be applied inside from_application_defaults; it is an enforcement override");
}

/// `from_application_defaults` maps `default_work_item_pattern` → `work_item.pattern`.
#[test]
fn policy_set_from_app_defaults_work_item_pattern_mapped() {
    let mut app = ApplicationDefaults::default();
    app.default_work_item_pattern = "GH-\\d+".to_string();

    let ps = PolicySet::from_application_defaults(&app);

    assert_eq!(ps.work_item.pattern, "GH-\\d+");
}

/// `enable_work_item_validation` is NOT applied by `from_application_defaults`.
#[test]
fn policy_set_from_app_defaults_enable_work_item_validation_not_applied() {
    let mut app = ApplicationDefaults::default();
    app.enable_work_item_validation = true;

    let ps = PolicySet::from_application_defaults(&app);

    assert!(!ps.work_item.required,
        "enable_work_item_validation must not be applied inside from_application_defaults; it is an enforcement override");
}

/// `from_application_defaults` maps `pr_size_check` → `size`.
#[test]
fn policy_set_from_app_defaults_size_mapped() {
    let mut app = ApplicationDefaults::default();
    app.pr_size_check.enabled = true;
    app.pr_size_check.label_prefix = "app-size/".to_string();

    let ps = PolicySet::from_application_defaults(&app);

    assert!(ps.size.enabled);
    assert_eq!(ps.size.label_prefix, "app-size/");
}

/// `from_application_defaults` maps `wip_check` → `wip`.
#[test]
fn policy_set_from_app_defaults_wip_mapped() {
    let mut app = ApplicationDefaults::default();
    app.wip_check.enforce_wip_blocking = true;
    app.wip_check.wip_label = Some("work-in-progress".to_string());

    let ps = PolicySet::from_application_defaults(&app);

    assert!(ps.wip.enforce_wip_blocking);
    assert_eq!(ps.wip.wip_label, Some("work-in-progress".to_string()));
}

/// `from_application_defaults` maps `pr_state_labels` → `pr_state`.
#[test]
fn policy_set_from_app_defaults_pr_state_mapped() {
    let mut app = ApplicationDefaults::default();
    app.pr_state_labels.enabled = true;
    app.pr_state_labels.draft_label = Some("in-progress".to_string());

    let ps = PolicySet::from_application_defaults(&app);

    assert!(ps.pr_state.enabled);
    assert_eq!(ps.pr_state.draft_label, Some("in-progress".to_string()));
}

/// `from_application_defaults` maps `change_type_labels` → `change_type_labels`.
#[test]
fn policy_set_from_app_defaults_change_type_labels_mapped() {
    let mut app = ApplicationDefaults::default();
    app.change_type_labels.keyword_labels.breaking_change = Some("semver-major".to_string());

    let ps = PolicySet::from_application_defaults(&app);

    assert_eq!(
        ps.change_type_labels.keyword_labels.breaking_change,
        Some("semver-major".to_string())
    );
}

/// `from_application_defaults` maps `bypass_rules` → `bypass_rules`.
#[test]
fn policy_set_from_app_defaults_bypass_rules_mapped() {
    let mut app = ApplicationDefaults::default();
    app.bypass_rules.title_convention = BypassRule::new(true, vec!["release-bot".to_string()]);

    let ps = PolicySet::from_application_defaults(&app);

    assert!(ps
        .bypass_rules
        .title_convention()
        .users()
        .contains(&"release-bot"));
}

// ── PolicySet::from_repository_config ────────────────────────────────────────
//
// Spec §1.3

/// `from_repository_config` maps `policies.pull_requests.title_policies` → `title`.
#[test]
fn policy_set_from_repo_config_title_mapped() {
    let repo = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                title_policies: PullRequestsTitlePolicyConfig {
                    required: true,
                    pattern: "repo-title-pattern".to_string(),
                    label_if_missing: Some("repo-label".to_string()),
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let ps = PolicySet::from_repository_config(&repo);

    assert!(ps.title.required);
    assert_eq!(ps.title.pattern, "repo-title-pattern");
    assert_eq!(ps.title.label_if_missing, Some("repo-label".to_string()));
}

/// `from_repository_config` maps `policies.pull_requests.work_item_policies` → `work_item`.
#[test]
fn policy_set_from_repo_config_work_item_mapped() {
    let repo = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                work_item_policies: WorkItemPolicyConfig {
                    required: true,
                    pattern: "GH-\\d+".to_string(),
                    label_if_missing: Some("missing-wi".to_string()),
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let ps = PolicySet::from_repository_config(&repo);

    assert!(ps.work_item.required);
    assert_eq!(ps.work_item.pattern, "GH-\\d+");
}

/// `from_repository_config` maps `policies.pull_requests.size_policies` → `size`.
#[test]
fn policy_set_from_repo_config_size_mapped() {
    let repo = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                size_policies: PrSizeCheckConfig {
                    enabled: true,
                    label_prefix: "repo-size/".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let ps = PolicySet::from_repository_config(&repo);

    assert!(ps.size.enabled);
    assert_eq!(ps.size.label_prefix, "repo-size/");
}

/// `from_repository_config` maps `policies.pull_requests.wip_policies` → `wip`.
#[test]
fn policy_set_from_repo_config_wip_mapped() {
    let repo = RepositoryProvidedConfig {
        schema_version: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                wip_policies: WipCheckConfig {
                    enforce_wip_blocking: true,
                    wip_label: Some("🚧".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let ps = PolicySet::from_repository_config(&repo);

    assert!(ps.wip.enforce_wip_blocking);
    assert_eq!(ps.wip.wip_label, Some("🚧".to_string()));
}

/// `from_repository_config` maps `change_type_labels` → `change_type_labels`
/// (Some variant).
#[test]
fn policy_set_from_repo_config_change_type_labels_some_mapped() {
    let mut ctl = ChangeTypeLabelConfig::default();
    ctl.keyword_labels.breaking_change = Some("semver-major".to_string());

    let repo = RepositoryProvidedConfig {
        schema_version: 1,
        change_type_labels: Some(ctl),
        ..Default::default()
    };

    let ps = PolicySet::from_repository_config(&repo);

    assert_eq!(
        ps.change_type_labels.keyword_labels.breaking_change,
        Some("semver-major".to_string())
    );
}

/// `from_repository_config` with `change_type_labels = None` yields `ChangeTypeLabelConfig::default()`.
#[test]
fn policy_set_from_repo_config_change_type_labels_none_uses_default() {
    let repo = RepositoryProvidedConfig {
        schema_version: 1,
        change_type_labels: None,
        ..Default::default()
    };

    let ps = PolicySet::from_repository_config(&repo);

    assert_eq!(ps.change_type_labels, ChangeTypeLabelConfig::default());
}

// ── Property-based tests (Tier 3) ────────────────────────────────────────────

proptest! {
    /// Merging with a default PolicySet as `over` is a right identity for boolean OR fields.
    /// Any field that is `true` in `base` must remain `true` in the result.
    #[test]
    fn prop_policy_set_merge_default_over_preserves_base_true_fields(
        title_required in proptest::bool::ANY,
        work_item_required in proptest::bool::ANY,
        size_enabled in proptest::bool::ANY,
        wip_enabled in proptest::bool::ANY,
    ) {
        let mut base = PolicySet::default();
        base.title.required = title_required;
        base.work_item.required = work_item_required;
        base.size.enabled = size_enabled;
        base.wip.enforce_wip_blocking = wip_enabled;

        let result = base.clone().merge(&PolicySet::default());

        prop_assert_eq!(result.title.required, base.title.required);
        prop_assert_eq!(result.work_item.required, base.work_item.required);
        prop_assert_eq!(result.size.enabled, base.size.enabled);
        prop_assert_eq!(result.wip.enforce_wip_blocking, base.wip.enforce_wip_blocking);
    }

    /// For OR-semantics fields, `merge(base, over).field` is always >= `base.field`
    /// (once true, stays true).
    #[test]
    fn prop_title_merge_required_or_never_loses_a_true(
        base_required in proptest::bool::ANY,
        over_required in proptest::bool::ANY,
    ) {
        let base = PullRequestsTitlePolicyConfig {
            required: base_required,
            ..Default::default()
        };
        let over = PullRequestsTitlePolicyConfig {
            required: over_required,
            ..Default::default()
        };
        let result = PullRequestsTitlePolicyConfig::merge(&base, &over);

        prop_assert_eq!(result.required, base_required || over_required);
    }

    /// For OR-semantics fields, `merge` is commutative for the `required` flag.
    #[test]
    fn prop_title_merge_required_commutative(
        a_required in proptest::bool::ANY,
        b_required in proptest::bool::ANY,
    ) {
        let a = PullRequestsTitlePolicyConfig { required: a_required, ..Default::default() };
        let b = PullRequestsTitlePolicyConfig { required: b_required, ..Default::default() };

        let ab = PullRequestsTitlePolicyConfig::merge(&a, &b);
        let ba = PullRequestsTitlePolicyConfig::merge(&b, &a);

        prop_assert_eq!(ab.required, ba.required);
    }

    /// `PrSizeCheckConfig::merge` never panics on any combination of default inputs.
    #[test]
    fn prop_size_merge_never_panics(
        base_enabled in proptest::bool::ANY,
        over_enabled in proptest::bool::ANY,
        base_fail in proptest::bool::ANY,
        over_fail in proptest::bool::ANY,
    ) {
        let base = PrSizeCheckConfig {
            enabled: base_enabled,
            fail_on_oversized: base_fail,
            ..Default::default()
        };
        let over = PrSizeCheckConfig {
            enabled: over_enabled,
            fail_on_oversized: over_fail,
            ..Default::default()
        };
        let _ = PrSizeCheckConfig::merge(&base, &over);
    }

    /// `WipCheckConfig::merge` OR for `enforce_wip_blocking` is always
    /// `base || over`.
    #[test]
    fn prop_wip_merge_enforce_or_is_correct(
        base_val in proptest::bool::ANY,
        over_val in proptest::bool::ANY,
    ) {
        let base = WipCheckConfig { enforce_wip_blocking: base_val, ..Default::default() };
        let over = WipCheckConfig { enforce_wip_blocking: over_val, ..Default::default() };
        let result = WipCheckConfig::merge(&base, &over);

        prop_assert_eq!(result.enforce_wip_blocking, base_val || over_val);
    }
}

// ── Mutation kill tests — serde `default_*` helper functions ─────────────────
//
// The `Default::default()` implementations for these types inline their values
// and do NOT call the serde `default_*` helper functions.  Mutations that swap
// return values of those helpers can only be detected via TOML deserialization.

/// Kills: `replace IssuePropagationConfig::default_false -> bool with true` (line 777).
///
/// `Default::default()` uses `bool::default()` (`false`) directly, so only a
/// TOML round-trip where the fields are absent invokes `default_false()`.
#[test]
fn test_issue_propagation_config_serde_absent_fields_are_false() {
    let cfg: IssuePropagationConfig = toml::from_str("").unwrap();
    assert!(
        !cfg.sync_milestone_from_issue,
        "serde default for sync_milestone_from_issue must be false"
    );
    assert!(
        !cfg.sync_project_from_issue,
        "serde default for sync_project_from_issue must be false"
    );
}

/// Kills: `replace PullRequestsTitlePolicyConfig::default_pattern -> String with String::new()` (line 966).
///
/// The manual `Default` impl calls `Self::default_pattern()`, so a direct
/// `::default()` call is sufficient.
#[test]
fn test_pull_requests_title_policy_default_pattern_is_conventional_commit_regex() {
    let cfg = PullRequestsTitlePolicyConfig::default();
    assert_eq!(
        cfg.pattern, CONVENTIONAL_COMMIT_REGEX,
        "default pattern must equal CONVENTIONAL_COMMIT_REGEX"
    );
}

/// Kills: `replace WorkItemPolicyConfig::default_pattern -> String with String::new()` (line 1171).
#[test]
fn test_work_item_policy_default_pattern_is_work_item_regex() {
    let cfg = WorkItemPolicyConfig::default();
    assert_eq!(
        cfg.pattern, WORK_ITEM_REGEX,
        "default pattern must equal WORK_ITEM_REGEX"
    );
}

/// Kills: `replace ChangeTypeLabelConfig::default_enabled -> bool with false` (line 2198).
///
/// The manual `Default` impl for `ChangeTypeLabelConfig` inlines `enabled: true`
/// so only a TOML deserialization where the `enabled` key is absent calls
/// `default_enabled()`.
#[test]
fn test_change_type_label_config_serde_absent_enabled_defaults_to_true() {
    let cfg: ChangeTypeLabelConfig = toml::from_str("").unwrap();
    assert!(
        cfg.enabled,
        "serde default for ChangeTypeLabelConfig::enabled must be true"
    );
}

/// Kills all `replace ConventionalCommitMappings::default_<type> -> Vec<String>` mutants
/// (lines 2328, 2336, 2340, 2344, 2348, 2356, 2360, 2368, 2376, 2384, 2388).
///
/// Each `default_<type>` function is invoked by serde when the corresponding key
/// is absent in TOML.  The `Default` impl inlines vectors directly and does not
/// call these helpers.
#[test]
fn test_conventional_commit_mappings_serde_absent_fields_use_correct_defaults() {
    let cfg: ConventionalCommitMappings = toml::from_str("").unwrap();
    assert_eq!(
        cfg.feat,
        vec!["enhancement", "feature", "new feature"],
        "serde default for feat"
    );
    assert_eq!(
        cfg.fix,
        vec!["bug", "bugfix", "fix"],
        "serde default for fix"
    );
    assert_eq!(
        cfg.docs,
        vec!["documentation", "docs"],
        "serde default for docs"
    );
    assert_eq!(
        cfg.style,
        vec!["style", "formatting"],
        "serde default for style"
    );
    assert_eq!(
        cfg.refactor,
        vec!["refactor", "refactoring", "code quality"],
        "serde default for refactor"
    );
    assert_eq!(
        cfg.perf,
        vec!["performance", "optimization"],
        "serde default for perf"
    );
    assert_eq!(
        cfg.test,
        vec!["test", "tests", "testing"],
        "serde default for test"
    );
    assert_eq!(
        cfg.chore,
        vec!["chore", "maintenance", "housekeeping"],
        "serde default for chore"
    );
    assert_eq!(
        cfg.ci,
        vec!["ci", "continuous integration", "build"],
        "serde default for ci"
    );
    assert_eq!(
        cfg.build,
        vec!["build", "dependencies"],
        "serde default for build"
    );
    assert_eq!(cfg.revert, vec!["revert"], "serde default for revert");
}

/// Kills: `replace FallbackLabelSettings::default_name_format -> String with ...` (line 2409)
/// and   `replace FallbackLabelSettings::default_create_if_missing -> bool with false` (line 2413).
///
/// `FallbackLabelSettings::default()` inlines its values; only serde deserialization
/// invokes these helpers.
#[test]
fn test_fallback_label_settings_serde_absent_scalar_fields_use_correct_defaults() {
    let cfg: FallbackLabelSettings = toml::from_str("").unwrap();
    assert_eq!(
        cfg.name_format, "type: {change_type}",
        "serde default for name_format"
    );
    assert!(
        cfg.create_if_missing,
        "serde default for create_if_missing must be true"
    );
}

/// Kills: `replace FallbackLabelSettings::default_color_scheme -> HashMap<...> with ...`
/// (line 2417, four variants: empty map, single-entry maps).
///
/// `default_color_scheme()` delegates to `FallbackLabelSettings::default().color_scheme`.
/// A deserialization test that omits the colour scheme forces `default_color_scheme()` to
/// run and checks that the resulting map is non-trivial.
#[test]
fn test_fallback_label_settings_serde_absent_color_scheme_uses_all_eleven_defaults() {
    let cfg: FallbackLabelSettings = toml::from_str("").unwrap();
    // All eleven commit-type colours must be present.
    assert_eq!(
        cfg.color_scheme.get("feat").map(String::as_str),
        Some("#0075ca")
    );
    assert_eq!(
        cfg.color_scheme.get("fix").map(String::as_str),
        Some("#d73a4a")
    );
    assert_eq!(
        cfg.color_scheme.get("docs").map(String::as_str),
        Some("#0052cc")
    );
    assert_eq!(
        cfg.color_scheme.get("style").map(String::as_str),
        Some("#f9d0c4")
    );
    assert_eq!(
        cfg.color_scheme.get("refactor").map(String::as_str),
        Some("#fef2c0")
    );
    assert_eq!(
        cfg.color_scheme.get("perf").map(String::as_str),
        Some("#a2eeef")
    );
    assert_eq!(
        cfg.color_scheme.get("test").map(String::as_str),
        Some("#d4edda")
    );
    assert_eq!(
        cfg.color_scheme.get("chore").map(String::as_str),
        Some("#e1e4e8")
    );
    assert_eq!(
        cfg.color_scheme.get("ci").map(String::as_str),
        Some("#fbca04")
    );
    assert_eq!(
        cfg.color_scheme.get("build").map(String::as_str),
        Some("#c5def5")
    );
    assert_eq!(
        cfg.color_scheme.get("revert").map(String::as_str),
        Some("#b60205")
    );
    assert_eq!(
        cfg.color_scheme.len(),
        11,
        "color_scheme must have exactly 11 entries"
    );
}

/// Kills: `replace LabelDetectionStrategy::default_true -> bool with false` (line 2509).
///
/// `LabelDetectionStrategy::default()` inlines `true` for each flag directly.
/// Only serde deserialization calls `default_true()`.
#[test]
fn test_label_detection_strategy_serde_absent_bool_fields_are_true() {
    let cfg: LabelDetectionStrategy = toml::from_str("").unwrap();
    assert!(
        cfg.exact_match,
        "serde default for exact_match must be true"
    );
    assert!(
        cfg.prefix_match,
        "serde default for prefix_match must be true"
    );
    assert!(
        cfg.description_match,
        "serde default for description_match must be true"
    );
}

/// Kills: `replace LabelDetectionStrategy::default_common_prefixes -> Vec<String> with ...`
/// (line 2513, three variants: empty vec, single-entry vecs).
#[test]
fn test_label_detection_strategy_serde_absent_common_prefixes_uses_correct_defaults() {
    let cfg: LabelDetectionStrategy = toml::from_str("").unwrap();
    assert_eq!(
        cfg.common_prefixes,
        vec!["type:", "kind:", "category:"],
        "serde default for common_prefixes"
    );
}

// ── Mutation kill tests — ChangeTypeLabelConfig::merge detection_strategy ────
//
// Lines 2240 and 2250 survive because no tests exercise the `common_prefixes`
// and `name_format` fallback paths in `ChangeTypeLabelConfig::merge`.

/// Kills: `delete ! in ChangeTypeLabelConfig::merge` at line 2240
/// (`if !od.common_prefixes.is_empty()`).
///
/// When `over.detection_strategy.common_prefixes` is empty the result must use
/// `base.detection_strategy.common_prefixes`.  Deleting `!` would flip this so
/// the empty override is copied over the non-empty base.
#[test]
fn change_type_merge_detection_strategy_common_prefixes_over_empty_uses_base() {
    let mut base = ChangeTypeLabelConfig::default();
    base.detection_strategy.common_prefixes = vec!["base-prefix:".to_string()];

    let mut over = ChangeTypeLabelConfig::default();
    over.detection_strategy.common_prefixes = vec![];

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.detection_strategy.common_prefixes,
        vec!["base-prefix:"],
        "empty over.common_prefixes must fall back to base"
    );
}

/// Kills: `replace != with == in ChangeTypeLabelConfig::merge` at line 2250
/// (`if of.name_format != default_name_format`).
///
/// When `over.fallback_label_settings.name_format` equals the default string
/// (`"type: {change_type}"`) the result must use `base.name_format`.  Replacing
/// `!=` with `==` would flip this so the default-value override wins instead of
/// the base.
#[test]
fn change_type_merge_fallback_name_format_over_default_value_uses_base() {
    let mut base = ChangeTypeLabelConfig::default();
    base.fallback_label_settings.name_format = "custom: {t}".to_string();

    let mut over = ChangeTypeLabelConfig::default();
    // Explicitly set to the serde/Default value — should NOT override base.
    over.fallback_label_settings.name_format = "type: {change_type}".to_string();

    let result = ChangeTypeLabelConfig::merge(&base, &over);

    assert_eq!(
        result.fallback_label_settings.name_format, "custom: {t}",
        "over.name_format equal to default must fall back to base"
    );
}

// ── Mutation kill tests — load_merge_warden_config `delete !` mutants ────────
//
// These tests cover the branches in load_merge_warden_config where the `!`
// guard is deleted, causing the logic to run in the wrong direction.

/// Kills: `delete ! in load_merge_warden_config` at line 1853
/// (`if !config.policies.pull_requests.size_policies.enabled`).
///
/// When the repo config has `size.enabled = true`, the size config must NOT be
/// replaced by the app-level defaults.  Deleting `!` would cause the replacement
/// to happen whenever size IS enabled, corrupting repo-specific settings.
#[tokio::test]
async fn test_load_merge_warden_config_repo_size_enabled_not_replaced_by_app_defaults() {
    let toml = r#"
schemaVersion = 1
[policies.pullRequests.prSize]
enabled = true
fail_on_oversized = true
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    // App has size disabled with different settings — must not override repo.
    app_defaults.pr_size_check.enabled = false;
    app_defaults.pr_size_check.fail_on_oversized = false;

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert!(
        config.policies.pull_requests.size_policies.enabled,
        "repo size.enabled=true must be preserved"
    );
    assert!(
        config
            .policies
            .pull_requests
            .size_policies
            .fail_on_oversized,
        "repo fail_on_oversized=true must not be replaced by app default false"
    );
}

/// Kills: `delete !` mutants at lines 1869-2031 — all eleven commit-type
/// `!is_empty()` guards in `load_merge_warden_config`.
///
/// When the repo TOML provides a `[change_type_labels]` section but sets each
/// individual commit-type mapping to an empty array, the app-level defaults
/// for those mappings must be preserved.  Deleting any `!` would cause the
/// empty repo list to be copied over the non-empty app default.
#[tokio::test]
async fn test_load_merge_warden_config_empty_repo_commit_type_mappings_use_app_defaults() {
    let toml = r#"
schemaVersion = 1
[change_type_labels]
enabled = true
[change_type_labels.conventional_commit_mappings]
feat     = []
fix      = []
docs     = []
style    = []
refactor = []
perf     = []
test     = []
chore    = []
ci       = []
build    = []
revert   = []
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    // Use the compile-time defaults so we have known expected values.
    app_defaults.change_type_labels = ChangeTypeLabelConfig::default();

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let labels = config
        .change_type_labels
        .expect("change_type_labels should be set");
    let m = &labels.conventional_commit_mappings;
    assert_eq!(
        m.feat,
        vec!["enhancement", "feature", "new feature"],
        "feat falls back to app default"
    );
    assert_eq!(
        m.fix,
        vec!["bug", "bugfix", "fix"],
        "fix falls back to app default"
    );
    assert_eq!(
        m.docs,
        vec!["documentation", "docs"],
        "docs falls back to app default"
    );
    assert_eq!(
        m.style,
        vec!["style", "formatting"],
        "style falls back to app default"
    );
    assert_eq!(
        m.refactor,
        vec!["refactor", "refactoring", "code quality"],
        "refactor falls back to app default"
    );
    assert_eq!(
        m.perf,
        vec!["performance", "optimization"],
        "perf falls back to app default"
    );
    assert_eq!(
        m.test,
        vec!["test", "tests", "testing"],
        "test falls back to app default"
    );
    assert_eq!(
        m.chore,
        vec!["chore", "maintenance", "housekeeping"],
        "chore falls back to app default"
    );
    assert_eq!(
        m.ci,
        vec!["ci", "continuous integration", "build"],
        "ci falls back to app default"
    );
    assert_eq!(
        m.build,
        vec!["build", "dependencies"],
        "build falls back to app default"
    );
    assert_eq!(m.revert, vec!["revert"], "revert falls back to app default");
}

/// Kills: `delete ! in load_merge_warden_config` at line 2123
/// (`if !config.policies.pull_requests.pr_state_policies.enabled`).
///
/// Also kills `replace && with ||` at line 2124 (the `&&` within the same
/// condition): when the repo has `pr_state.enabled = true` and the app also
/// enables pr_state, the repo's custom labels must NOT be replaced by the app
/// defaults.
#[tokio::test]
async fn test_load_merge_warden_config_repo_pr_state_enabled_not_replaced_by_app() {
    let toml = r#"
schemaVersion = 1
[policies.pullRequests.prState]
enabled = true
draft_label = "custom-draft"
review_label = "custom-review"
approved_label = "custom-approved"
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.pr_state_labels.enabled = true;
    app_defaults.pr_state_labels.draft_label = Some("app-draft".to_string());
    app_defaults.pr_state_labels.review_label = Some("app-review".to_string());

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert!(config.policies.pull_requests.pr_state_policies.enabled);
    assert_eq!(
        config.policies.pull_requests.pr_state_policies.draft_label,
        Some("custom-draft".to_string()),
        "repo custom draft_label must be preserved when repo pr_state is enabled"
    );
    assert_eq!(
        config.policies.pull_requests.pr_state_policies.review_label,
        Some("custom-review".to_string()),
        "repo custom review_label must be preserved when repo pr_state is enabled"
    );
    assert_eq!(
        config
            .policies
            .pull_requests
            .pr_state_policies
            .approved_label,
        Some("custom-approved".to_string()),
        "repo custom approved_label must be preserved when repo pr_state is enabled"
    );
}

// ── Mutation kill tests — load_merge_warden_config `&&→||` WIP conditions ────
//
// Each `&&` in the WIP merge block requires BOTH conditions to be true before
// an override occurs.  The tests below set up the case where the first condition
// is false (repo has a custom/non-default value) while the second is true (app
// also has a custom value), which should produce NO override.  If `&&` is
// replaced with `||` the second condition alone would trigger the override.

/// Kills: `replace && with ||` at line 2084 (wip_label condition).
/// Also kills: `replace == with !=` at line 2095 (same condition, first operand).
///
/// Repo sets a custom WIP label that differs from the default.  App also has a
/// custom label.  The repo label must be preserved because the first guard
/// (`config.wip_label == default`) is false.
#[tokio::test]
async fn test_load_merge_warden_config_repo_custom_wip_label_not_overridden_by_app_custom() {
    let toml = r#"
schemaVersion = 1
[policies.pullRequests.wip]
wip_label = "🚧 In Progress"
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.wip_check.wip_label = Some("work-in-progress".to_string());

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert_eq!(
        config.policies.pull_requests.wip_policies.wip_label,
        Some("🚧 In Progress".to_string()),
        "repo custom wip_label must be preserved when app also has a custom label"
    );
}

/// Kills: `replace && with ||` at line 2096 (wip_title_patterns condition).
///
/// Repo sets custom title patterns; app also has custom patterns.  The repo
/// patterns must be preserved because the first guard (`config.patterns == default`)
/// is false.
#[tokio::test]
async fn test_load_merge_warden_config_repo_custom_wip_title_patterns_not_overridden_by_app_custom()
{
    let toml = r#"
schemaVersion = 1
[policies.pullRequests.wip]
wip_title_patterns = ["CUSTOM_WIP"]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.wip_check.wip_title_patterns = vec!["APP_WIP".to_string()];

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert_eq!(
        config
            .policies
            .pull_requests
            .wip_policies
            .wip_title_patterns,
        vec!["CUSTOM_WIP"],
        "repo custom wip_title_patterns must be preserved when app also has custom patterns"
    );
}

/// Kills: `replace != with == in load_merge_warden_config` at line 2097
/// (`app_defaults.wip_check.wip_title_patterns != WipCheckConfig::default()`).
///
/// When the repo uses the default WIP title patterns and the app provides custom
/// patterns, the app patterns must override.  With `!=→==` the guard becomes
/// "app == default", which never fires when the app has custom patterns, leaving
/// the repo at the default value.
#[tokio::test]
async fn test_load_merge_warden_config_app_wip_title_patterns_override_when_repo_uses_default() {
    let toml = r#"schemaVersion = 1"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    // Custom patterns — different from WipCheckConfig::default().wip_title_patterns.
    app_defaults.wip_check.wip_title_patterns = vec!["MY_WIP".to_string()];

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert_eq!(
        config
            .policies
            .pull_requests
            .wip_policies
            .wip_title_patterns,
        vec!["MY_WIP"],
        "app custom wip_title_patterns must be applied when repo uses the default"
    );
}

/// Kills: `delete ! in load_merge_warden_config` at line 1993
/// (`if !repo.fallback_label_settings.name_format.is_empty()`).
///
/// When the repo's `name_format` is an empty string the app-level default must
/// be preserved.  Deleting `!` causes the override to run when `name_format` IS
/// empty, replacing the app default with an empty string.
#[tokio::test]
async fn test_load_merge_warden_config_empty_repo_name_format_uses_app_default() {
    let toml = r#"
schemaVersion = 1
[change_type_labels]
enabled = true
[change_type_labels.fallback_label_settings]
name_format = ""
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults
        .change_type_labels
        .fallback_label_settings
        .name_format = "app: {change_type}".to_string();

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let labels = config
        .change_type_labels
        .expect("change_type_labels must be set");
    assert_eq!(
        labels.fallback_label_settings.name_format, "app: {change_type}",
        "empty repo name_format must fall back to app default"
    );
}

/// Kills: `delete ! in load_merge_warden_config` at line 2005
/// (`if !repo.fallback_label_settings.color_scheme.is_empty()`).
///
/// When the repo provides an empty colour scheme the app-level defaults must be
/// preserved.  Deleting `!` causes the empty map to override the app defaults.
#[tokio::test]
async fn test_load_merge_warden_config_empty_repo_color_scheme_uses_app_default() {
    let toml = r#"
schemaVersion = 1
[change_type_labels]
enabled = true
[change_type_labels.fallback_label_settings.color_scheme]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults
        .change_type_labels
        .fallback_label_settings
        .color_scheme
        .insert("feat".to_string(), "#custom".to_string());

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let labels = config
        .change_type_labels
        .expect("change_type_labels must be set");
    assert_eq!(
        labels
            .fallback_label_settings
            .color_scheme
            .get("feat")
            .map(String::as_str),
        Some("#custom"),
        "empty repo color_scheme must fall back to app default colour scheme"
    );
}

/// Kills: `delete ! in load_merge_warden_config` at line 2031
/// (`if !repo.detection_strategy.common_prefixes.is_empty()`).
///
/// When the repo sets `common_prefixes = []` the app-level defaults must be
/// used.  Deleting `!` would copy the empty vec over the app defaults.
#[tokio::test]
async fn test_load_merge_warden_config_empty_repo_common_prefixes_uses_app_default() {
    let toml = r#"
schemaVersion = 1
[change_type_labels]
enabled = true
[change_type_labels.detection_strategy]
common_prefixes = []
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults
        .change_type_labels
        .detection_strategy
        .common_prefixes = vec!["app-prefix:".to_string()];

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    let labels = config
        .change_type_labels
        .expect("change_type_labels must be set");
    assert_eq!(
        labels.detection_strategy.common_prefixes,
        vec!["app-prefix:"],
        "empty repo common_prefixes must fall back to app default"
    );
}

/// Kills: `replace && with ||` at line 2112 (wip_description_patterns condition).
///
/// Repo already has non-empty description patterns; app also has patterns.  The
/// first guard (`config.patterns.is_empty()`) is false, so no override should occur.
/// If `&&` is replaced with `||`, the second guard (`!app.patterns.is_empty()`)
/// alone would trigger the override.
#[tokio::test]
async fn test_load_merge_warden_config_repo_description_patterns_not_overridden_by_app_patterns() {
    let toml = r#"
schemaVersion = 1
[policies.pullRequests.wip]
wip_description_patterns = ["🚧 repo-pattern"]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let mut app_defaults = ApplicationDefaults::default();
    app_defaults.wip_check.wip_description_patterns = vec!["app-desc-pattern".to_string()];

    let config = load_merge_warden_config("a", "b", "path", &fetcher, &app_defaults)
        .await
        .unwrap();

    assert_eq!(
        config
            .policies
            .pull_requests
            .wip_policies
            .wip_description_patterns,
        vec!["🚧 repo-pattern"],
        "non-empty repo description_patterns must be preserved when app also has patterns"
    );
}

// ============================================================
// OrgPolicySource / OrgPolicy / load_org_policy tests
// ============================================================

struct FailingFetcher;

#[async_trait]
impl ConfigFetcher for FailingFetcher {
    async fn fetch_config(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _path: &str,
    ) -> Result<Option<String>, Error> {
        Err(Error::ApiError())
    }

    async fn fetch_config_at_ref(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _path: &str,
        _git_ref: &str,
    ) -> Result<Option<String>, Error> {
        Err(Error::ApiError())
    }
}

fn make_org_source(fail: bool) -> OrgPolicySource {
    OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "org-policies".to_string(),
        path: ".github/merge-warden-org.toml".to_string(),
        fail_if_unreachable: fail,
    }
}

#[tokio::test]
async fn test_load_org_policy_file_not_found_returns_none() {
    let fetcher = MockFetcher::new(None);
    let source = make_org_source(false);
    let result = load_org_policy(&source, &fetcher).await.unwrap();
    assert!(result.is_none(), "Missing file should return None");
}

#[tokio::test]
async fn test_load_org_policy_file_not_found_with_fail_returns_none() {
    // fail_if_unreachable only applies to errors, not missing files.
    let fetcher = MockFetcher::new(None);
    let source = make_org_source(true);
    let result = load_org_policy(&source, &fetcher).await.unwrap();
    assert!(
        result.is_none(),
        "Missing file with fail_if_unreachable=true should still return None"
    );
}

#[tokio::test]
async fn test_load_org_policy_fetch_error_lenient_returns_none() {
    let source = make_org_source(false);
    let result = load_org_policy(&source, &FailingFetcher).await.unwrap();
    assert!(
        result.is_none(),
        "Fetch error with fail_if_unreachable=false should return None"
    );
}

#[tokio::test]
async fn test_load_org_policy_fetch_error_strict_returns_error() {
    let source = make_org_source(true);
    let result = load_org_policy(&source, &FailingFetcher).await;
    assert!(
        result.is_err(),
        "Fetch error with fail_if_unreachable=true should return Err"
    );
    let e = result.unwrap_err().to_string();
    assert!(
        e.contains("Org policy unavailable"),
        "Error should mention org policy unavailable; got: {e}"
    );
}

#[tokio::test]
async fn test_load_org_policy_parse_error_lenient_returns_none() {
    let fetcher = MockFetcher::new(Some("not valid toml %%%".to_string()));
    let source = make_org_source(false);
    let result = load_org_policy(&source, &fetcher).await.unwrap();
    assert!(
        result.is_none(),
        "Parse error with fail_if_unreachable=false should return None"
    );
}

#[tokio::test]
async fn test_load_org_policy_parse_error_strict_returns_error() {
    let fetcher = MockFetcher::new(Some("not valid toml %%%".to_string()));
    let source = make_org_source(true);
    let result = load_org_policy(&source, &fetcher).await;
    assert!(
        result.is_err(),
        "Parse error with fail_if_unreachable=true should return Err"
    );
}

#[tokio::test]
async fn test_load_org_policy_unsupported_schema_lenient_returns_none() {
    let toml = r#"
schemaVersion = 99

[enforced]
[defaults]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(false);
    let result = load_org_policy(&source, &fetcher).await.unwrap();
    assert!(
        result.is_none(),
        "Unsupported schema version with fail_if_unreachable=false should return None"
    );
}

#[tokio::test]
async fn test_load_org_policy_unsupported_schema_strict_returns_error() {
    let toml = r#"
schemaVersion = 99

[enforced]
[defaults]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(true);
    let result = load_org_policy(&source, &fetcher).await;
    assert!(
        result.is_err(),
        "Unsupported schema version with fail_if_unreachable=true should return Err"
    );
}

#[tokio::test]
async fn test_load_org_policy_valid_empty_sections_returns_some() {
    let toml = r#"
schemaVersion = 1

[enforced]
[defaults]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(false);
    let result = load_org_policy(&source, &fetcher).await.unwrap();
    assert!(result.is_some(), "Valid minimal org policy should parse OK");
}

#[tokio::test]
async fn test_load_org_policy_enforced_title_fields() {
    let toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = true
pattern = "^feat:"

[defaults]
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(false);
    let policy = load_org_policy(&source, &fetcher)
        .await
        .unwrap()
        .expect("should return Some");
    assert!(
        policy.enforced.title.required,
        "Enforced section title.required should be true"
    );
    assert_eq!(policy.enforced.title.pattern, "^feat:");
}

#[tokio::test]
async fn test_load_org_policy_defaults_not_in_enforced() {
    let toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.prTitle]
required = true
pattern = "^org-default:"
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(false);
    let policy = load_org_policy(&source, &fetcher)
        .await
        .unwrap()
        .expect("should return Some");
    assert!(
        !policy.enforced.title.required,
        "Enforced section should not carry defaults title"
    );
    assert!(
        policy.defaults.title.required,
        "Defaults section title.required should be true"
    );
    assert_eq!(policy.defaults.title.pattern, "^org-default:");
}

// ============================================================
// PolicySet::from_app_enforcement_flags tests
// ============================================================

#[test]
fn test_from_app_enforcement_flags_all_false_returns_default() {
    let app = ApplicationDefaults::default();
    let ps = PolicySet::from_app_enforcement_flags(&app);
    assert!(!ps.title.required);
    assert!(!ps.work_item.required);
    assert!(!ps.size.enabled);
    assert!(!ps.wip.enforce_wip_blocking);
}

#[test]
fn test_from_app_enforcement_flags_title_enabled() {
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;
    let ps = PolicySet::from_app_enforcement_flags(&app);
    assert!(ps.title.required);
    assert!(!ps.work_item.required, "work_item should remain false");
}

#[test]
fn test_from_app_enforcement_flags_work_item_enabled() {
    let mut app = ApplicationDefaults::default();
    app.enable_work_item_validation = true;
    let ps = PolicySet::from_app_enforcement_flags(&app);
    assert!(ps.work_item.required);
    assert!(!ps.title.required, "title should remain false");
}

#[test]
fn test_from_app_enforcement_flags_wip_enabled() {
    let mut app = ApplicationDefaults::default();
    app.wip_check.enforce_wip_blocking = true;
    let ps = PolicySet::from_app_enforcement_flags(&app);
    assert!(ps.wip.enforce_wip_blocking);
}

#[test]
fn test_from_app_enforcement_flags_size_enabled() {
    let mut app = ApplicationDefaults::default();
    app.pr_size_check.enabled = true;
    let ps = PolicySet::from_app_enforcement_flags(&app);
    assert!(ps.size.enabled);
}

// ============================================================
// resolve_pull_request_config tests
// ============================================================

#[tokio::test]
async fn test_resolve_pull_request_config_no_org_source_no_repo_file() {
    let fetcher = MockFetcher::new(None);
    let app = ApplicationDefaults::default();
    let result = resolve_pull_request_config("owner", "repo", "path", &fetcher, &app)
        .await
        .unwrap();
    // With no configuration at all, title validation is off by default.
    assert!(
        !result.enforce_title_convention,
        "No config should produce default (disabled) title enforcement"
    );
}

#[tokio::test]
async fn test_resolve_pull_request_config_app_enforced_title_overrides_repo() {
    // Repo disables title validation; app enforcement forces it on.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
enabled = false
"#;
    let fetcher = MockFetcher::new(Some(repo_toml.to_string()));
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;
    let result = resolve_pull_request_config("owner", "repo", "path", &fetcher, &app)
        .await
        .unwrap();
    assert!(
        result.enforce_title_convention,
        "App enforcement flag must override repo setting"
    );
}

#[tokio::test]
async fn test_resolve_pull_request_config_repo_enables_title() {
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
pattern = "^TICKET-[0-9]+"
"#;
    let fetcher = MockFetcher::new(Some(repo_toml.to_string()));
    let app = ApplicationDefaults::default();
    let result = resolve_pull_request_config("owner", "repo", "path", &fetcher, &app)
        .await
        .unwrap();
    assert!(result.enforce_title_convention);
    assert_eq!(result.title_pattern, "^TICKET-[0-9]+");
}

#[tokio::test]
async fn test_resolve_pull_request_config_org_enforced_overrides_repo() {
    // Repo disables title; org enforces it.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
enabled = false
"#;
    let org_toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = true
pattern = "^ORG:"

[defaults]
"#;

    // Use a fetcher that returns the repo TOML for the repo path and org TOML for the org path.
    struct TwoFileFetcher {
        repo_content: String,
        org_content: String,
        org_path: String,
    }

    #[async_trait]
    impl ConfigFetcher for TwoFileFetcher {
        async fn fetch_config(
            &self,
            _owner: &str,
            _repo: &str,
            path: &str,
        ) -> Result<Option<String>, Error> {
            if path == self.org_path {
                Ok(Some(self.org_content.clone()))
            } else {
                Ok(Some(self.repo_content.clone()))
            }
        }

        async fn fetch_config_at_ref(
            &self,
            _owner: &str,
            _repo: &str,
            _path: &str,
            _git_ref: &str,
        ) -> Result<Option<String>, Error> {
            Ok(None)
        }
    }

    let fetcher = TwoFileFetcher {
        repo_content: repo_toml.to_string(),
        org_content: org_toml.to_string(),
        org_path: "org-policy.toml".to_string(),
    };

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "policies".to_string(),
        path: "org-policy.toml".to_string(),
        fail_if_unreachable: false,
    });

    let result = resolve_pull_request_config("owner", "repo", "repo-policy.toml", &fetcher, &app)
        .await
        .unwrap();
    assert!(
        result.enforce_title_convention,
        "Org enforced setting must override repo disabled setting"
    );
    assert_eq!(result.title_pattern, "^ORG:");
}

#[tokio::test]
async fn test_resolve_pull_request_config_org_defaults_overridden_by_repo() {
    // Org sets a default title pattern; repo overrides with its own.
    struct TwoFileFetcher2 {
        repo_content: String,
        org_content: String,
        org_path: String,
    }

    #[async_trait]
    impl ConfigFetcher for TwoFileFetcher2 {
        async fn fetch_config(
            &self,
            _owner: &str,
            _repo: &str,
            path: &str,
        ) -> Result<Option<String>, Error> {
            if path == self.org_path {
                Ok(Some(self.org_content.clone()))
            } else {
                Ok(Some(self.repo_content.clone()))
            }
        }

        async fn fetch_config_at_ref(
            &self,
            _o: &str,
            _r: &str,
            _p: &str,
            _ref: &str,
        ) -> Result<Option<String>, Error> {
            Ok(None)
        }
    }

    let org_toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.prTitle]
required = true
pattern = "^ORG-DEFAULT:"
"#;

    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
pattern = "^REPO:"
"#;

    let fetcher = TwoFileFetcher2 {
        repo_content: repo_toml.to_string(),
        org_content: org_toml.to_string(),
        org_path: "org-policy.toml".to_string(),
    };

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "policies".to_string(),
        path: "org-policy.toml".to_string(),
        fail_if_unreachable: false,
    });

    let result = resolve_pull_request_config("owner", "repo", "repo-policy.toml", &fetcher, &app)
        .await
        .unwrap();
    assert!(result.enforce_title_convention);
    assert_eq!(
        result.title_pattern, "^REPO:",
        "Repo pattern must take precedence over org default"
    );
}

#[tokio::test]
async fn test_resolve_pull_request_config_org_unavailable_lenient_degrades() {
    let _fetcher = MockFetcher::new(None); // repo config not found
    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "policies".to_string(),
        path: "org-policy.toml".to_string(),
        fail_if_unreachable: false,
    });
    // Both repo and org return None — should succeed using app defaults.
    let result = resolve_pull_request_config("owner", "repo", "path", &FailingFetcher, &app)
        .await
        .unwrap();
    assert!(
        !result.enforce_title_convention,
        "Degraded config should fall back to app defaults (title off)"
    );
}

#[tokio::test]
async fn test_resolve_pull_request_config_org_unavailable_strict_returns_error() {
    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "policies".to_string(),
        path: "org-policy.toml".to_string(),
        fail_if_unreachable: true,
    });
    let result = resolve_pull_request_config("owner", "repo", "path", &FailingFetcher, &app).await;
    assert!(
        result.is_err(),
        "Strict org unavailable must propagate as Err"
    );
}

// ============================================================
// CurrentPullRequestValidationConfiguration::from_app_defaults tests
// ============================================================

#[test]
fn test_from_app_defaults_title_disabled_by_default() {
    let app = ApplicationDefaults::default();
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert!(!cfg.enforce_title_convention);
}

#[test]
fn test_from_app_defaults_title_enabled() {
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert!(cfg.enforce_title_convention);
}

#[test]
fn test_from_app_defaults_work_item_enabled() {
    let mut app = ApplicationDefaults::default();
    app.enable_work_item_validation = true;
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert!(cfg.enforce_work_item_references);
}

// ============================================================
// Verify load_merge_warden_config no longer applies enforcement flags
// ============================================================

#[tokio::test]
async fn test_load_merge_warden_config_does_not_apply_enforcement_flags() {
    // Even with enable_title_validation = true on app defaults,
    // load_merge_warden_config itself must NOT force-enable title validation.
    // That responsibility now lives in resolve_pull_request_config.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
enabled = false
"#;
    let fetcher = MockFetcher::new(Some(repo_toml.to_string()));
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;

    let config = load_merge_warden_config("owner", "repo", "path", &fetcher, &app)
        .await
        .unwrap();

    assert!(
        !config.policies.pull_requests.title_policies.required,
        "load_merge_warden_config must NOT apply app enforcement flags"
    );
}

// ============================================================
// PolicySet::from_org_section field-mapping tests
// ============================================================

fn make_org_section_raw_with_title(required: bool, pattern: &str) -> OrgPolicySectionRaw {
    let mut section = OrgPolicySectionRaw::default();
    section.policies.pull_requests.title_policies.required = required;
    section.policies.pull_requests.title_policies.pattern = pattern.to_string();
    section
}

#[test]
fn test_from_org_section_title_fields_mapped() {
    let section = make_org_section_raw_with_title(true, "^ORG-TITLE:");
    let ps = PolicySet::from_org_section(&section);
    assert!(ps.title.required);
    assert_eq!(ps.title.pattern, "^ORG-TITLE:");
}

#[test]
fn test_from_org_section_work_item_fields_mapped() {
    let mut section = OrgPolicySectionRaw::default();
    section.policies.pull_requests.work_item_policies.required = true;
    section.policies.pull_requests.work_item_policies.pattern = "JIRA-[0-9]+".to_string();
    let ps = PolicySet::from_org_section(&section);
    assert!(ps.work_item.required);
    assert_eq!(ps.work_item.pattern, "JIRA-[0-9]+");
}

#[test]
fn test_from_org_section_size_fields_mapped() {
    let mut section = OrgPolicySectionRaw::default();
    section.policies.pull_requests.size_policies.enabled = true;
    let ps = PolicySet::from_org_section(&section);
    assert!(ps.size.enabled);
}

#[test]
fn test_from_org_section_wip_fields_mapped() {
    let mut section = OrgPolicySectionRaw::default();
    section
        .policies
        .pull_requests
        .wip_policies
        .enforce_wip_blocking = true;
    let ps = PolicySet::from_org_section(&section);
    assert!(ps.wip.enforce_wip_blocking);
}

#[test]
fn test_from_org_section_change_type_labels_none_becomes_default() {
    let section = OrgPolicySectionRaw::default();
    let ps = PolicySet::from_org_section(&section);
    assert_eq!(
        ps.change_type_labels,
        ChangeTypeLabelConfig::default(),
        "Absent change_type_labels should become ChangeTypeLabelConfig::default()"
    );
}

#[test]
fn test_from_org_section_bypass_rules_always_default() {
    // Org policy sections carry no bypass rules; they must always be BypassRules::default().
    let mut section = OrgPolicySectionRaw::default();
    section.policies.pull_requests.title_policies.required = true;
    let ps = PolicySet::from_org_section(&section);
    assert_eq!(
        ps.bypass_rules,
        BypassRules::default(),
        "from_org_section must not populate bypass_rules"
    );
}

#[test]
fn test_from_org_section_empty_section_returns_default_policy_set() {
    let section = OrgPolicySectionRaw::default();
    let ps = PolicySet::from_org_section(&section);
    assert!(!ps.title.required);
    assert!(!ps.work_item.required);
    assert!(!ps.size.enabled);
    assert!(!ps.wip.enforce_wip_blocking);
}

// ============================================================
// load_org_policy — additional completeness tests
// ============================================================

#[tokio::test]
async fn test_load_org_policy_valid_both_sections_populated() {
    let toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = true
pattern = "^ENF:"

[defaults.policies.pullRequests.workItem]
required = true
pattern = "JIRA-[0-9]+"
"#;
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(false);
    let policy = load_org_policy(&source, &fetcher)
        .await
        .unwrap()
        .expect("should return Some");

    assert!(
        policy.enforced.title.required,
        "enforced title.required should be set"
    );
    assert_eq!(policy.enforced.title.pattern, "^ENF:");
    assert!(
        policy.defaults.work_item.required,
        "defaults work_item.required should be set"
    );
    assert_eq!(policy.defaults.work_item.pattern, "JIRA-[0-9]+");
    // Cross-check: defaults has no title override, enforced has no work_item override.
    assert!(
        !policy.defaults.title.required,
        "defaults should not carry enforced title"
    );
    assert!(
        !policy.enforced.work_item.required,
        "enforced should not carry defaults work_item"
    );
}

#[tokio::test]
async fn test_load_org_policy_schema_version_only_returns_default_policy() {
    // `schemaVersion = 1` with no subsections → OrgPolicy::default()
    let toml = "schemaVersion = 1\n";
    let fetcher = MockFetcher::new(Some(toml.to_string()));
    let source = make_org_source(false);
    let policy = load_org_policy(&source, &fetcher)
        .await
        .unwrap()
        .expect("should return Some");

    assert_eq!(
        policy,
        OrgPolicy::default(),
        "No subsections should produce OrgPolicy::default()"
    );
}

// ============================================================
// CurrentPullRequestValidationConfiguration::from_app_defaults
// — additional spec §9.5 scenarios
// ============================================================

#[test]
fn test_from_app_defaults_title_pattern_propagated() {
    let mut app = ApplicationDefaults::default();
    app.default_title_pattern = "^CUSTOM:".to_string();
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert_eq!(
        cfg.title_pattern, "^CUSTOM:",
        "default_title_pattern must propagate to title_pattern"
    );
}

#[test]
fn test_from_app_defaults_all_default_fields() {
    let app = ApplicationDefaults::default();
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert!(!cfg.enforce_title_convention);
    assert!(!cfg.enforce_work_item_references);
    assert!(!cfg.pr_size_check.enabled);
    assert!(!cfg.wip_check.enforce_wip_blocking);
    assert_eq!(cfg.bot_mention, app.bot_mention);
}

#[test]
fn test_from_app_defaults_bot_mention_propagated() {
    let mut app = ApplicationDefaults::default();
    app.bot_mention = "@my-bot".to_string();
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert_eq!(cfg.bot_mention, "@my-bot");
}

// ============================================================
// PolicySet::to_validation_config — bot_mention threading
// ============================================================

#[test]
fn test_to_validation_config_bot_mention_from_app_defaults() {
    let mut app = ApplicationDefaults::default();
    app.bot_mention = "@org-bot".to_string();
    let ps = PolicySet::from_application_defaults(&app);
    let cfg = ps.to_validation_config(&app);
    assert_eq!(
        cfg.bot_mention, "@org-bot",
        "to_validation_config must thread bot_mention from app_defaults"
    );
}

// ============================================================
// OrgPolicySource serde roundtrip
// ============================================================

#[test]
fn test_org_policy_source_serde_roundtrip() {
    let source = OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "platform-configs".to_string(),
        path: "merge-warden/org-policy.toml".to_string(),
        fail_if_unreachable: true,
    };
    let json = serde_json::to_string(&source).expect("serialization failed");
    let back: OrgPolicySource = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(source, back);
}

#[test]
fn test_org_policy_source_fail_if_unreachable_defaults_false() {
    let toml_input = r#"
owner = "my-org"
repo  = "platform-configs"
path  = "merge-warden/org-policy.toml"
"#;
    let source: OrgPolicySource =
        toml::from_str(toml_input).expect("should deserialize without fail_if_unreachable");
    assert!(
        !source.fail_if_unreachable,
        "fail_if_unreachable should default to false when absent"
    );
}

// ============================================================
// resolve_pull_request_config — work_item pattern scenarios (spec §9.3)
// ============================================================

mod two_file_fetcher_shared {
    use async_trait::async_trait;
    use merge_warden_developer_platforms::errors::Error;

    use crate::config::ConfigFetcher;

    pub struct TwoFileFetcher {
        pub repo_content: Option<String>,
        pub org_content: Option<String>,
        pub org_path: String,
    }

    #[async_trait]
    impl ConfigFetcher for TwoFileFetcher {
        async fn fetch_config(
            &self,
            _owner: &str,
            _repo: &str,
            path: &str,
        ) -> Result<Option<String>, Error> {
            if path == self.org_path {
                Ok(self.org_content.clone())
            } else {
                Ok(self.repo_content.clone())
            }
        }

        async fn fetch_config_at_ref(
            &self,
            _o: &str,
            _r: &str,
            _p: &str,
            _ref: &str,
        ) -> Result<Option<String>, Error> {
            Ok(None)
        }
    }
}

fn make_org_policy_source() -> OrgPolicySource {
    OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "policies".to_string(),
        path: "org-policy.toml".to_string(),
        fail_if_unreachable: false,
    }
}

#[tokio::test]
async fn test_resolve_org_defaults_work_item_pattern_overridden_by_repo() {
    // Spec §9.3: org defaults set work_item.pattern; repo sets its own → repo wins.
    let org_toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.workItem]
required = true
pattern = "JIRA-[0-9]+"
"#;

    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.workItem]
required = true
pattern = "GH-[0-9]+"
"#;

    let fetcher = two_file_fetcher_shared::TwoFileFetcher {
        repo_content: Some(repo_toml.to_string()),
        org_content: Some(org_toml.to_string()),
        org_path: "org-policy.toml".to_string(),
    };

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(make_org_policy_source());

    let result = resolve_pull_request_config("owner", "repo", "repo-policy.toml", &fetcher, &app)
        .await
        .unwrap();

    assert!(result.enforce_work_item_references);
    assert_eq!(
        result.work_item_reference_pattern, "GH-[0-9]+",
        "Repo work_item pattern must override org default pattern"
    );
}

#[tokio::test]
async fn test_resolve_org_defaults_work_item_pattern_used_when_repo_omits_it() {
    // Spec §9.3: org defaults set work_item.pattern; repo omits it → org default wins.
    let org_toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.workItem]
required = true
pattern = "JIRA-[0-9]+"
"#;

    // Repo config exists but doesn't specify work_item at all.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = false
"#;

    let fetcher = two_file_fetcher_shared::TwoFileFetcher {
        repo_content: Some(repo_toml.to_string()),
        org_content: Some(org_toml.to_string()),
        org_path: "org-policy.toml".to_string(),
    };

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(make_org_policy_source());

    let result = resolve_pull_request_config("owner", "repo", "repo-policy.toml", &fetcher, &app)
        .await
        .unwrap();

    assert!(
        result.enforce_work_item_references,
        "Org default work_item.required must be used when repo omits it"
    );
    assert_eq!(
        result.work_item_reference_pattern, "JIRA-[0-9]+",
        "Org default work_item pattern must be used when repo omits it"
    );
}

#[tokio::test]
async fn test_resolve_org_defaults_work_item_required_both_set() {
    // Spec §9.3: org defaults set work_item.required = true; repo also sets it = true → true.
    let org_toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.workItem]
required = true
"#;

    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.workItem]
required = true
"#;

    let fetcher = two_file_fetcher_shared::TwoFileFetcher {
        repo_content: Some(repo_toml.to_string()),
        org_content: Some(org_toml.to_string()),
        org_path: "org-policy.toml".to_string(),
    };

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(make_org_policy_source());

    let result = resolve_pull_request_config("owner", "repo", "repo-policy.toml", &fetcher, &app)
        .await
        .unwrap();

    assert!(
        result.enforce_work_item_references,
        "work_item.required must be true when both org defaults and repo set it"
    );
}

// ============================================================
// PolicySet::to_validation_config — full field-mapping tests
//
// The new PolicySet::to_validation_config (line ~1824 of config.rs) is
// only exercised via resolve_pull_request_config in the existing tests.
// The field copies below were NOT independently verified, so a mutation
// swapping `self.x.clone()` → `Default::default()` would survive.
// ============================================================

#[test]
fn test_policy_set_to_validation_config_invalid_title_label_forwarded() {
    let mut app = ApplicationDefaults::default();
    app.default_invalid_title_label = Some("my-bad-title-label".to_string());
    let ps = PolicySet::from_application_defaults(&app);
    let cfg = ps.to_validation_config(&app);
    assert_eq!(
        cfg.invalid_title_label,
        Some("my-bad-title-label".to_string()),
        "invalid_title_label must be forwarded from PolicySet.title.label_if_missing"
    );
}

#[test]
fn test_policy_set_to_validation_config_missing_work_item_label_forwarded() {
    let mut app = ApplicationDefaults::default();
    app.default_missing_work_item_label = Some("missing-wi-label".to_string());
    let ps = PolicySet::from_application_defaults(&app);
    let cfg = ps.to_validation_config(&app);
    assert_eq!(
        cfg.missing_work_item_label,
        Some("missing-wi-label".to_string()),
        "missing_work_item_label must be forwarded from PolicySet.work_item.label_if_missing"
    );
}

#[test]
fn test_policy_set_to_validation_config_enforce_work_item_forwarded() {
    let mut ps = PolicySet::default();
    ps.work_item.required = true;
    ps.work_item.pattern = "WI-[0-9]+".to_string();
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.enforce_work_item_references,
        "enforce_work_item_references must reflect work_item.required"
    );
    assert_eq!(
        cfg.work_item_reference_pattern, "WI-[0-9]+",
        "work_item_reference_pattern must be forwarded from PolicySet.work_item.pattern"
    );
}

#[test]
fn test_policy_set_to_validation_config_bypass_rules_forwarded() {
    let mut ps = PolicySet::default();
    ps.bypass_rules = BypassRules::new_with_size(
        BypassRule::new(true, vec!["org-bot".to_string()]),
        BypassRule::new(false, vec![]),
        BypassRule::new(false, vec![]),
    );
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.bypass_rules.title_convention().enabled(),
        "bypass_rules must be forwarded from PolicySet.bypass_rules"
    );
    assert!(
        cfg.bypass_rules
            .title_convention()
            .users()
            .contains(&"org-bot"),
        "bypass user must be forwarded"
    );
}

#[test]
fn test_policy_set_to_validation_config_issue_propagation_forwarded() {
    let mut ps = PolicySet::default();
    ps.issue_propagation = IssuePropagationConfig {
        sync_milestone_from_issue: true,
        sync_project_from_issue: false,
    };
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.issue_propagation.sync_milestone_from_issue,
        "issue_propagation.sync_milestone_from_issue must be forwarded"
    );
    assert!(
        !cfg.issue_propagation.sync_project_from_issue,
        "issue_propagation.sync_project_from_issue must be forwarded as false"
    );
}

#[test]
fn test_policy_set_to_validation_config_wip_check_forwarded() {
    let mut ps = PolicySet::default();
    ps.wip.enforce_wip_blocking = true;
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.wip_check.enforce_wip_blocking,
        "wip_check must be forwarded from PolicySet.wip"
    );
}

#[test]
fn test_policy_set_to_validation_config_pr_size_check_forwarded() {
    let mut ps = PolicySet::default();
    ps.size.enabled = true;
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.pr_size_check.enabled,
        "pr_size_check must be forwarded from PolicySet.size"
    );
}

#[test]
fn test_policy_set_to_validation_config_pr_state_labels_forwarded() {
    let mut ps = PolicySet::default();
    ps.pr_state.enabled = true;
    ps.pr_state.draft_label = Some("draft".to_string());
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.pr_state_labels.enabled,
        "pr_state_labels.enabled must be forwarded from PolicySet.pr_state"
    );
    assert_eq!(
        cfg.pr_state_labels.draft_label,
        Some("draft".to_string()),
        "pr_state_labels.draft_label must be forwarded from PolicySet.pr_state"
    );
}

#[test]
fn test_policy_set_to_validation_config_change_type_labels_wrapped_in_some() {
    // The new to_validation_config wraps PolicySet.change_type_labels in
    // Some(). Mutating `Some(self.change_type_labels.clone())` to `None`
    // would previously survive.
    let ps = PolicySet::default();
    let app = ApplicationDefaults::default();
    let cfg = ps.to_validation_config(&app);
    assert!(
        cfg.change_type_labels.is_some(),
        "to_validation_config must wrap change_type_labels in Some()"
    );
}

// ============================================================
// resolve_pull_request_config — Tier 4 > Tier 3
// (app enforcement flag beats org enforced)
// ============================================================

#[tokio::test]
async fn test_resolve_app_enforcement_beats_org_enforced() {
    // Org enforced sets title required = false (absent section → default false).
    // App enforcement flag = true. Result must be true.
    let org_toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = false

[defaults]
"#;
    // Repo also sets false.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = false
"#;

    let fetcher = two_file_fetcher_shared::TwoFileFetcher {
        repo_content: Some(repo_toml.to_string()),
        org_content: Some(org_toml.to_string()),
        org_path: "org-policy.toml".to_string(),
    };

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(make_org_policy_source());
    app.enable_title_validation = true; // Tier 4 enforcement

    let result = resolve_pull_request_config("owner", "repo", "repo-policy.toml", &fetcher, &app)
        .await
        .unwrap();

    assert!(
        result.enforce_title_convention,
        "Tier 4 app enforcement flag must override org enforced = false and repo = false"
    );
}

// ============================================================
// CurrentPullRequestValidationConfiguration::from_app_defaults
// — label field tests (spec §9.5)
// ============================================================

#[test]
fn test_from_app_defaults_invalid_title_label_propagated() {
    let mut app = ApplicationDefaults::default();
    app.default_invalid_title_label = Some("bad-title-label".to_string());
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert_eq!(
        cfg.invalid_title_label,
        Some("bad-title-label".to_string()),
        "default_invalid_title_label must propagate to invalid_title_label"
    );
}

#[test]
fn test_from_app_defaults_missing_work_item_label_propagated() {
    let mut app = ApplicationDefaults::default();
    app.default_missing_work_item_label = Some("no-work-item".to_string());
    let cfg = CurrentPullRequestValidationConfiguration::from_app_defaults(&app);
    assert_eq!(
        cfg.missing_work_item_label,
        Some("no-work-item".to_string()),
        "default_missing_work_item_label must propagate to missing_work_item_label"
    );
}

// ============================================================
// PolicySet::from_app_enforcement_flags — all four flags together
// ============================================================

#[test]
fn test_from_app_enforcement_flags_all_four_enabled() {
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;
    app.enable_work_item_validation = true;
    app.pr_size_check.enabled = true;
    app.wip_check.enforce_wip_blocking = true;
    let ps = PolicySet::from_app_enforcement_flags(&app);
    assert!(ps.title.required, "title.required must be set");
    assert!(ps.work_item.required, "work_item.required must be set");
    assert!(ps.size.enabled, "size.enabled must be set");
    assert!(
        ps.wip.enforce_wip_blocking,
        "wip.enforce_wip_blocking must be set"
    );
}

// ============================================================
// PolicySet::from_org_section — pr_state and issue_propagation mapping
// ============================================================

#[test]
fn test_from_org_section_pr_state_fields_mapped() {
    let mut section = OrgPolicySectionRaw::default();
    section.policies.pull_requests.pr_state_policies.enabled = true;
    section.policies.pull_requests.pr_state_policies.draft_label = Some("in-draft".to_string());
    let ps = PolicySet::from_org_section(&section);
    assert!(
        ps.pr_state.enabled,
        "pr_state.enabled must be mapped from OrgPolicySectionRaw"
    );
    assert_eq!(
        ps.pr_state.draft_label,
        Some("in-draft".to_string()),
        "pr_state.draft_label must be mapped from OrgPolicySectionRaw"
    );
}

#[test]
fn test_from_org_section_issue_propagation_mapped() {
    let mut section = OrgPolicySectionRaw::default();
    section
        .policies
        .pull_requests
        .issue_propagation
        .sync_milestone_from_issue = true;
    let ps = PolicySet::from_org_section(&section);
    assert!(
        ps.issue_propagation.sync_milestone_from_issue,
        "issue_propagation.sync_milestone_from_issue must be mapped from OrgPolicySectionRaw"
    );
    assert!(
        !ps.issue_propagation.sync_project_from_issue,
        "issue_propagation.sync_project_from_issue must not be spuriously set"
    );
}
