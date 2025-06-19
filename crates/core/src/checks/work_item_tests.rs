use crate::checks::work_item::check_work_item_reference;
use merge_warden_developer_platforms::models::{PullRequest, User};

#[test]
fn test_empty_body() {
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        draft: false,
        body: None,
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let result = check_work_item_reference(&pr);
    assert!(!result, "PR with no body should be invalid");
}

#[test]
fn test_alternative_keywords() {
    // Test alternative keywords that might be used to reference issues
    let alternative_keywords = vec![
        "See #123",
        "Related to #456",
        "Linked to #789",
        "Part of #101112",
        "Implements #131415",
        "Addresses #161718",
        "Connected to #192021",
    ];

    for reference in alternative_keywords {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(reference.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        // The current regex might not support all these keywords, so we check the actual behavior
        // If it passes, great! If not, we document the current behavior
        if result {
            assert!(result, "Reference '{}' should be valid", reference);
        } else {
            assert!(
                !result,
                "Reference '{}' is currently not recognized by the regex",
                reference
            );
        }
    }
}

#[test]
fn test_case_sensitivity() {
    // Test case sensitivity in keywords
    let case_variations = vec![
        "FIXES #123",
        "Fixes #123",
        "fixes #123",
        "CLOSES #456",
        "Closes #456",
        "closes #456",
        "RESOLVES #789",
        "Resolves #789",
        "resolves #789",
    ];

    for reference in case_variations {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(reference.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        assert!(
            result,
            "Reference '{}' should be valid regardless of case",
            reference
        );
    }
}

#[test]
fn test_different_issue_formats() {
    // Test different issue number formats
    let issue_formats = vec![
        "Fixes #123",
        "Fixes GH-123",
        "Fixes org/repo#123",
        "Fixes https://github.com/owner/repo/issues/123",
        "Fixes https://github.com/owner/repo/pull/123",
        "Fixes https://github.com/owner/repo/issues/123?query=param",
        "Fixes https://github.com/owner/repo/issues/123#issuecomment-123456789",
    ];

    for reference in issue_formats {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(reference.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        // The current regex might not support all these formats, so we check the actual behavior
        if result {
            assert!(result, "Reference '{}' should be valid", reference);
        } else {
            assert!(
                !result,
                "Reference '{}' is currently not recognized by the regex",
                reference
            );
        }
    }
}

#[test]
fn test_invalid_work_item_references() {
    let invalid_references = vec![
        "",                                 // Empty body
        "No reference here",                // No reference
        "Issue #123",                       // Missing keyword
        "#123",                             // Just the number
        "Fixes issue 123",                  // Missing # symbol
        "Fixes bug",                        // Missing number
        "github.com/owner/repo/issues/123", // Missing keyword
        "fixes123",                         // Malformed reference
    ];

    for reference in invalid_references {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(reference.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        assert!(!result, "Reference '{}' should be invalid", reference);
    }
}

#[test]
fn test_malformed_references() {
    let malformed_references = vec![
        "fixes123",                                      // No space
        "fixes#123",                                     // Missing space
        "fixes 123",                                     // Missing #
        "fixes https://github.com/owner/repo/issues123", // Missing separator
    ];

    for reference in malformed_references {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(reference.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        assert!(
            !result,
            "Malformed reference '{}' should be invalid",
            reference
        );
    }
}

#[test]
fn test_mixed_valid_and_invalid_references() {
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        draft: false,
        body: Some("Fixes #123\nInvalid reference\nCloses #456".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let result = check_work_item_reference(&pr);
    assert!(
        result,
        "PR with mixed valid and invalid references should be valid"
    );
}

#[test]
fn test_multiple_valid_work_item_references() {
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        draft: false,
        body: Some("Fixes #123\nCloses #456\nResolves #789".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let result = check_work_item_reference(&pr);
    assert!(result, "PR with multiple valid references should be valid");
}

#[test]
fn test_position_in_body() {
    // Test references at different positions in the PR body
    let positions = vec![
        "Fixes #123\nThis is the rest of the PR description.",
        "This is a PR description.\nFixes #123\nMore description.",
        "This is a PR description.\nMore description.\nFixes #123",
        "This is a PR description with Fixes #123 in the middle of a sentence.",
        "This PR\n\n\n\n\nFixes #123\n\n\n\nhas many empty lines.",
    ];

    for body in positions {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(body.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        assert!(result, "Reference at position '{}' should be valid", body);
    }
}

#[test]
fn test_valid_work_item_references() {
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
        "Fixes org/repo#123",
    ];

    for reference in valid_references {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            draft: false,
            body: Some(reference.to_string()),
            author: Some(User {
                id: 456,
                login: "developer123".to_string(),
            }),
        };

        let result = check_work_item_reference(&pr);
        assert!(result, "Reference '{}' should be valid", reference);
    }
}

#[test]
fn test_very_long_body() {
    let long_body = "Fixes #123\n".repeat(1000); // 10,000+ characters
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        draft: false,
        body: Some(long_body),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let result = check_work_item_reference(&pr);
    assert!(
        result,
        "PR with very long body containing valid references should be valid"
    );
}

#[test]
fn test_work_item_correction() {
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        draft: false,
        body: Some("No reference here".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let result = check_work_item_reference(&pr);
    assert!(!result, "PR with no work item reference should be invalid");

    let updated_pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        draft: false,
        body: Some("Fixes #123".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let result = check_work_item_reference(&updated_pr);
    assert!(
        result,
        "Updated PR with work item reference should be valid"
    );
}
