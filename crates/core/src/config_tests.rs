use crate::config::{
    CurrentPullRequestValidationConfiguration, CONVENTIONAL_COMMIT_REGEX, WORK_ITEM_REGEX,
};
use proptest::prelude::*;

use super::*;

#[test]
fn test_application_defaults_struct_and_methods() {
    use crate::config::*;
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

    for title in edge_cases {
        assert!(
            CONVENTIONAL_COMMIT_REGEX.is_match(title),
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

    for title in invalid_titles {
        assert!(
            !CONVENTIONAL_COMMIT_REGEX.is_match(title),
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

    for title in valid_titles {
        assert!(
            CONVENTIONAL_COMMIT_REGEX.is_match(title),
            "CONVENTIONAL_COMMIT_REGEX should match valid title '{}'",
            title
        );
    }
}

proptest! {
    #[test]
    fn test_conventional_commit_regex_random_inputs(input in ".*") {
        let _ = CONVENTIONAL_COMMIT_REGEX.is_match(&input); // Ensure no panic occurs
    }

    #[test]
    fn test_work_item_regex_random_inputs(input in ".*") {
        let _ = WORK_ITEM_REGEX.is_match(&input); // Ensure no panic occurs
    }
}

#[test]
fn test_current_pr_validation_config_new() {
    use crate::config::*;
    let config = CurrentPullRequestValidationConfiguration::new(
        true,
        Some("custom-title".to_string()),
        Some("custom-invalid-label".to_string()),
        false,
        Some("custom-work-item".to_string()),
        Some("custom-missing-label".to_string()),
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
    use crate::config::*;
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
            },
        },
    };
    let validation = config.to_validation_config();
    let custom_title = "CUSTOM: test title";
    let custom_title_regex = regex::Regex::new(&validation.title_pattern).unwrap();
    assert!(custom_title_regex.is_match(custom_title));
    let custom_work_item = "CUSTOM-123";
    let custom_work_item_regex =
        regex::Regex::new(&validation.work_item_reference_pattern).unwrap();
    assert!(custom_work_item_regex.is_match(custom_work_item));
}

#[test]
fn test_load_merge_warden_config_empty_file() {
    use crate::config::*;
    use std::fs::File;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
    File::create(&file_path).unwrap();
    let app_defaults = ApplicationDefaults::default();
    let result = load_merge_warden_config(&file_path, &app_defaults);
    assert!(matches!(result, Err(ConfigLoadError::Toml(_))));
}

#[test]
fn test_load_merge_warden_config_invalid_schema() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
    let toml = r##"schemaVersion = 2
[policies.pullRequests.prTitle]
required = true
pattern = "foo"
[policies.pullRequests.workItem]
required = true
pattern = "bar"
"##;
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let app_defaults = ApplicationDefaults::default();
    let result = load_merge_warden_config(&file_path, &app_defaults);
    assert!(matches!(
        result,
        Err(ConfigLoadError::UnsupportedSchemaVersion(2))
    ));
}

#[test]
fn test_load_merge_warden_config_malformed_toml() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
    let toml = r#"schemaVersion = 1
[policies.pullRequests.prTitle
required = true
pattern = "foo"
"#; // missing closing bracket for table
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let app_defaults = ApplicationDefaults::default();
    let result = load_merge_warden_config(&file_path, &app_defaults);
    assert!(matches!(result, Err(ConfigLoadError::Toml(_))));
}

#[test]
fn test_load_merge_warden_config_missing_file() {
    use crate::config::*;
    let app_defaults = ApplicationDefaults::default();
    let result = load_merge_warden_config("/nonexistent/path/merge-warden.toml", &app_defaults);
    assert!(matches!(result, Err(ConfigLoadError::NotFound(_))));
}

#[test]
fn test_load_merge_warden_config_missing_optional_fields() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
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
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config(&file_path, &app_defaults).unwrap();
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

#[test]
fn test_load_merge_warden_config_only_schema_version() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
    let toml = r#"schemaVersion = 1
"#;
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config(&file_path, &app_defaults);
    // Should succeed, but policies will be defaulted
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.schema_version, 1);
    // Defaults for policies
    assert!(config.policies.pull_requests.title_policies.required);
    assert!(config.policies.pull_requests.work_item_policies.required);
}

#[test]
fn test_load_merge_warden_config_valid() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
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
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let app_defaults = ApplicationDefaults::default();
    let config = load_merge_warden_config(&file_path, &app_defaults).unwrap();
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
    use crate::config::*;
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
            },
        },
    };
    let validation = config.to_validation_config();
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
    use crate::config::*;
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
            },
        },
    };
    let validation = config.to_validation_config();
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

    for reference in edge_cases {
        let expected_match = should_match.contains(&reference);
        assert_eq!(
            WORK_ITEM_REGEX.is_match(reference),
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

    for reference in invalid_references {
        assert!(
            !WORK_ITEM_REGEX.is_match(reference),
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

    for reference in valid_references {
        assert!(
            WORK_ITEM_REGEX.is_match(reference),
            "WORK_ITEM_REGEX should match valid reference '{}'",
            reference
        );
    }
}
