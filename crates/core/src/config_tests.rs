use crate::config::{ValidationConfig, CONVENTIONAL_COMMIT_REGEX, WORK_ITEM_REGEX};
use proptest::prelude::*;

// New corner case tests for CONVENTIONAL_COMMIT_REGEX
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

// Test for ValidationConfig::default()
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

// New corner case tests for WORK_ITEM_REGEX
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
