use crate::{checks::work_item::check_work_item_reference, models::PullRequest};
use anyhow::Result;

#[test]
fn test_valid_work_item_references() {
    // Test various valid work item reference formats
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
        "This PR fixes #123",
        "This PR\nfixes #123",
        "This PR fixes https://github.com/owner/repo/issues/123",
        "This PR fixes GH-123",
    ];

    for reference in valid_references {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            body: Some(reference.to_string()),
        };

        let result = check_work_item_reference(&pr).unwrap();
        assert!(result, "Reference '{}' should be valid", reference);
    }
}

#[test]
fn test_invalid_work_item_references() {
    // Test various invalid work item reference formats
    let invalid_references = vec![
        "",                                 // Empty body
        "No reference here",                // No reference
        "Issue #123",                       // Missing keyword
        "#123",                             // Just the number
        "Fixes issue 123",                  // Missing # symbol
        "Fixes bug",                        // Missing number
        "github.com/owner/repo/issues/123", // Missing keyword
    ];

    for reference in invalid_references {
        let pr = PullRequest {
            number: 1,
            title: "feat: test".to_string(),
            body: Some(reference.to_string()),
        };

        let result = check_work_item_reference(&pr).unwrap();
        assert!(!result, "Reference '{}' should be invalid", reference);
    }
}

#[test]
fn test_missing_body() {
    // Test PR with no body
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        body: None,
    };

    let result = check_work_item_reference(&pr).unwrap();
    assert!(!result, "PR with no body should be invalid");
}

#[test]
fn test_work_item_correction() {
    // First submit with no work item reference
    let pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        body: Some("No reference here".to_string()),
    };

    let result = check_work_item_reference(&pr).unwrap();
    assert!(!result, "PR with no work item reference should be invalid");

    // Now update with valid work item reference
    let updated_pr = PullRequest {
        number: 1,
        title: "feat: test".to_string(),
        body: Some("Fixes #123".to_string()),
    };

    let result = check_work_item_reference(&updated_pr).unwrap();
    assert!(
        result,
        "Updated PR with work item reference should be valid"
    );
}
