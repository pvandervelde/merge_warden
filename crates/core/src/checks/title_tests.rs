use crate::{checks::title::check_pr_title, models::PullRequest};
use anyhow::Result;

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
fn test_invalid_title_formats() {
    // Test various invalid title formats
    let invalid_titles = vec![
        "add new feature",             // Missing type
        "Feature: add new feature",    // Capitalized type
        "feat - add new feature",      // Wrong separator
        "feat(AUTH): add new feature", // Uppercase scope
        "feat(): empty scope",         // Empty scope
        "feat: ",                      // Empty description
        "feat",                        // Missing description and separator
        "feat(api) add new feature",   // Missing separator
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
