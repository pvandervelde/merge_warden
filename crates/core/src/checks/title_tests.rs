use crate::{checks::title::check_pr_title, models::PullRequest};
use anyhow::Result;

#[test]
fn test_empty_title() {
    let pr = PullRequest {
        number: 1,
        title: "".to_string(),
        body: Some("Test body".to_string()),
    };

    let result = check_pr_title(&pr).unwrap();
    assert!(!result, "Empty title should be invalid");
}

#[test]
fn test_invalid_prefixes() {
    let invalid_titles = vec![
        "unknown: add feature", // Invalid prefix
        "feature: add feature", // Non-standard prefix
        "bug: fix issue",       // Non-standard prefix
    ];

    for title in invalid_titles {
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(!result, "Title '{}' should be invalid", title);
    }
}

#[test]
fn test_invalid_separators() {
    let invalid_titles = vec![
        "feat-add feature",  // Dash instead of colon
        "feat; add feature", // Semicolon instead of colon
        "feat add feature",  // Missing separator
        "feat:add feature",  // No space after colon
    ];

    for title in invalid_titles {
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(!result, "Title '{}' should be invalid", title);
    }
}

#[test]
fn test_invalid_title_formats() {
    // Test various invalid title formats
    let invalid_titles = vec![
        "add new feature",                              // Missing type
        "Feature: add new feature",                     // Capitalized type
        "feat - add new feature",                       // Wrong separator
        "feat(AUTH): add new feature",                  // Uppercase scope
        "feat(): empty scope",                          // Empty scope
        "feat: ",                                       // Empty description
        "feat",                                         // Missing description and separator
        "feat(api) add new feature",                    // Missing separator
        "refactor(auth/flow): simplify authentication", // slash in the scope
        "docs(readme.md): update documentation",        // file name in the scope
    ];

    for title in invalid_titles {
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(!result, "Title '{}' should be invalid", title);
    }
}

#[test]
fn test_missing_prefix() {
    let pr = PullRequest {
        number: 1,
        title: "add feature".to_string(),
        body: Some("Test body".to_string()),
    };

    let result = check_pr_title(&pr).unwrap();
    assert!(!result, "Title with missing prefix should be invalid");
}

#[test]
fn test_multiple_scopes() {
    // Test titles with multiple scopes
    let titles_with_multiple_scopes = vec![
        "feat(api)(auth): add new feature",
        "fix(ui)(core)(validation): fix multiple issues",
        "refactor(api)(db)(auth): simplify authentication flow",
    ];

    for title in titles_with_multiple_scopes {
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(
            !result,
            "Title '{}' with multiple scopes is not a valid scope",
            title
        );
    }
}

#[test]
fn test_scope_with_special_characters() {
    // Test titles with special characters in the scope
    let titles_with_special_scopes = vec![
        "feat(api-v1): add feature",
        "fix(ui_component): fix styling",
        "chore(deps-dev): update development dependencies",
    ];

    for title in titles_with_special_scopes {
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(
            result,
            "Title '{}' with special characters in scope should be valid",
            title
        );
    }
}

#[test]
fn test_special_characters_in_title() {
    let pr = PullRequest {
        number: 1,
        title: "feat: add feature with special chars !@#$%^&*()".to_string(),
        body: Some("Test body".to_string()),
    };

    let result = check_pr_title(&pr).unwrap();
    assert!(result, "Title with special characters should be valid");
}

#[test]
fn test_title_correction() {
    // First submit with invalid title
    let pr = PullRequest {
        number: 1,
        title: "invalid title".to_string(),
        body: Some("Test body".to_string()),
    };

    let result = check_pr_title(&pr).unwrap();
    assert!(!result, "Title should be invalid");

    // Now update with valid title
    let updated_pr = PullRequest {
        number: 1,
        title: "feat: valid title".to_string(),
        body: Some("Test body".to_string()),
    };

    let result = check_pr_title(&updated_pr).unwrap();
    assert!(result, "Updated title should be valid");
}

#[test]
fn test_title_with_breaking_change_indicators() {
    // Test titles with breaking change indicators in different positions
    let titles_with_breaking_change = vec![
        "feat!: breaking change",
        "feat(api)!: breaking change in API",
        "fix!: critical bug fix with breaking changes",
        "refactor(auth)!: completely redesign authentication flow",
    ];

    for title in titles_with_breaking_change {
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(
            result,
            "Title '{}' with breaking change indicator should be valid",
            title
        );
    }
}

#[test]
fn test_valid_title_formats() {
    // Test various valid title formats
    let valid_titles = vec![
        "feat: add new feature",
        "fix(auth): correct login issue",
        "docs: update README",
        "style: format code",
        "refactor: simplify logic",
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
        let pr = PullRequest {
            number: 1,
            title: title.to_string(),
            body: Some("Test body".to_string()),
        };

        let result = check_pr_title(&pr).unwrap();
        assert!(result, "Title '{}' should be valid", title);
    }
}

#[test]
fn test_very_long_title() {
    let long_title = "feat: ".to_string() + &"a".repeat(300);
    let pr = PullRequest {
        number: 1,
        title: long_title,
        body: Some("Test body".to_string()),
    };

    let result = check_pr_title(&pr).unwrap();
    assert!(result, "Very long title should be valid");
}
