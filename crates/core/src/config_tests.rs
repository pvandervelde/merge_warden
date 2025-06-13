use crate::config::{ValidationConfig, CONVENTIONAL_COMMIT_REGEX, WORK_ITEM_REGEX};
use proptest::prelude::*;

use super::*;

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
fn test_default_auth_method() {
    assert_eq!(default_auth_method(), "token");
}

#[test]
fn test_default_provider() {
    assert_eq!(default_provider(), "github");
}

#[test]
fn test_missing_work_item_label() {
    assert_eq!(MISSING_WORK_ITEM_LABEL, "missing-work-item");
}

#[test]
fn test_rules_config_new() {
    let config = RulesConfig::new();
    assert!(!config.require_work_items);
    assert_eq!(config.enforce_title_convention, Some(false));
    assert_eq!(config.min_approvals, Some(1));
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
    let config = ValidationConfig::default();

    assert!(
        config.enforce_conventional_commits,
        "Default ValidationConfig should enforce conventional commits"
    );
    assert!(
        config.require_work_item_references,
        "Default ValidationConfig should require work item references"
    );
    assert!(
        config.auto_label,
        "Default ValidationConfig should enable auto-labeling"
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

#[test]
fn test_merge_warden_config_to_validation_config_conventional_commits_and_work_item() {
    use crate::config::*;
    let config = MergeWardenConfig {
        schemaVersion: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                prTitle: PullRequestsTitlePolicyConfig {
                    format: "conventional-commits".to_string(),
                },
                workItem: WorkItemPolicyConfig {
                    required: true,
                    pattern: "#\\d+".to_string(),
                },
            },
        },
    };
    let validation = config.to_validation_config();
    assert!(validation.enforce_conventional_commits);
    assert!(validation.require_work_item_references);
    assert!(validation.auto_label);
}

#[test]
fn test_merge_warden_config_to_validation_config_non_conventional_commits() {
    use crate::config::*;
    let config = MergeWardenConfig {
        schemaVersion: 1,
        policies: PoliciesConfig {
            pull_requests: PullRequestsPoliciesConfig {
                prTitle: PullRequestsTitlePolicyConfig {
                    format: "none".to_string(),
                },
                workItem: WorkItemPolicyConfig {
                    required: false,
                    pattern: "#\\d+".to_string(),
                },
            },
        },
    };
    let validation = config.to_validation_config();
    assert!(!validation.enforce_conventional_commits);
    assert!(!validation.require_work_item_references);
    assert!(validation.auto_label);
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
format = "conventional-commits"
[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"
"##;
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = load_merge_warden_config(&file_path).unwrap();
    assert_eq!(config.schemaVersion, 1);
    assert_eq!(
        config.policies.pull_requests.prTitle.format,
        "conventional-commits"
    );
    assert!(config.policies.pull_requests.workItem.required);
    assert_eq!(config.policies.pull_requests.workItem.pattern, "#\\d+");
}

#[test]
fn test_load_merge_warden_config_not_found() {
    use crate::config::*;
    let result = load_merge_warden_config("/nonexistent/path/merge-warden.toml");
    assert!(matches!(result, Err(ConfigLoadError::NotFound(_))));
}

#[test]
fn test_load_merge_warden_config_invalid_toml() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
    let toml = "not a valid toml";
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let result = load_merge_warden_config(&file_path);
    assert!(matches!(result, Err(ConfigLoadError::Toml(_))));
}

#[test]
fn test_load_merge_warden_config_unsupported_version() {
    use crate::config::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("merge-warden.toml");
    let toml = r##"schemaVersion = 999
[policies.pullRequests.prTitle]
format = "conventional-commits"
[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"
"##;
    let mut file = File::create(&file_path).unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let result = load_merge_warden_config(&file_path);
    assert!(matches!(
        result,
        Err(ConfigLoadError::UnsupportedSchemaVersion(999))
    ));
}
