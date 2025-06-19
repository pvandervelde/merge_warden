use super::*;
use serde_json::{from_str, to_string};

#[test]
fn test_comment_serialization() {
    // Create a comment
    let comment = Comment {
        id: 123,
        body: "This is a test comment".to_string(),
        user: User {
            id: 10,
            login: "a".to_string(),
        },
    };

    // Serialize to JSON
    let json_str = to_string(&comment).expect("Failed to serialize Comment");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["id"], 123);
    assert_eq!(parsed["body"], "This is a test comment");
}

#[test]
fn test_comment_deserialization() {
    // Create JSON
    let json_str =
        r#"{"id": 456, "body": "Deserialized comment", "user": { "id": 10, "login": "a" }}"#;

    // Deserialize from JSON
    let comment: Comment = from_str(json_str).expect("Failed to deserialize Comment");

    // Verify fields
    assert_eq!(comment.id, 456);
    assert_eq!(comment.body, "Deserialized comment");
}

#[test]
fn test_label_serialization() {
    // Create a label
    let label = Label {
        name: "bug".to_string(),
    };

    // Serialize to JSON
    let json_str = to_string(&label).expect("Failed to serialize Label");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["name"], "bug");
}

#[test]
fn test_label_deserialization() {
    // Create JSON
    let json_str = r#"{"name": "feature"}"#;

    // Deserialize from JSON
    let label: Label = from_str(json_str).expect("Failed to deserialize Label");

    // Verify fields
    assert_eq!(label.name, "feature");
}

#[test]
fn test_pull_request_serialization() {
    // Create a pull request
    let pr = PullRequest {
        number: 42,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This PR adds a new feature.\n\nFixes #123".to_string()),
        author: None,
    };

    // Serialize to JSON
    let json_str = to_string(&pr).expect("Failed to serialize PullRequest");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["number"], 42);
    assert_eq!(parsed["title"], "feat: add new feature");
    assert_eq!(parsed["body"], "This PR adds a new feature.\n\nFixes #123");
}

#[test]
fn test_pull_request_deserialization() {
    // Create JSON
    let json_str = r#"{
        "number": 99,
        "title": "fix: resolve bug",
        "draft": false,
        "body": "This PR fixes a critical bug.\n\nCloses #456"
    }"#;

    // Deserialize from JSON
    let pr: PullRequest = from_str(json_str).expect("Failed to deserialize PullRequest");

    // Verify fields
    assert_eq!(pr.number, 99);
    assert_eq!(pr.title, "fix: resolve bug");
    assert_eq!(
        pr.body,
        Some("This PR fixes a critical bug.\n\nCloses #456".to_string())
    );
}

#[test]
fn test_pull_request_without_body() {
    // Create a pull request without a body
    let pr = PullRequest {
        number: 42,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: None,
        author: None,
    };

    // Serialize to JSON
    let json_str = to_string(&pr).expect("Failed to serialize PullRequest");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["number"], 42);
    assert_eq!(parsed["title"], "feat: add new feature");
    assert!(parsed["body"].is_null());

    // Deserialize back
    let deserialized_pr: PullRequest =
        from_str(&json_str).expect("Failed to deserialize PullRequest");
    assert_eq!(deserialized_pr.number, 42);
    assert_eq!(deserialized_pr.title, "feat: add new feature");
    assert_eq!(deserialized_pr.body, None);
}

#[test]
fn test_review_serialization() {
    // Create a review
    let review = Review {
        id: 789,
        state: "CHANGES_REQUESTED".to_string(),
        user: User {
            id: 101,
            login: "testuser".to_string(),
        },
    };

    // Serialize to JSON
    let json_str = to_string(&review).expect("Failed to serialize Review");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["id"], 789);
    assert_eq!(parsed["state"], "CHANGES_REQUESTED");
    assert_eq!(parsed["user"]["id"], 101);
    assert_eq!(parsed["user"]["login"], "testuser");
}

#[test]
fn test_review_deserialization() {
    // Create JSON
    let json_str = r#"{
        "id": 555,
        "state": "APPROVED",
        "user": {
            "id": 202,
            "login": "reviewer"
        }
    }"#;

    // Deserialize from JSON
    let review: Review = from_str(json_str).expect("Failed to deserialize Review");

    // Verify fields
    assert_eq!(review.id, 555);
    assert_eq!(review.state, "APPROVED");
    assert_eq!(review.user.id, 202);
    assert_eq!(review.user.login, "reviewer");
}

#[test]
fn test_user_serialization() {
    // Create a user
    let user = User {
        id: 303,
        login: "developer".to_string(),
    };

    // Serialize to JSON
    let json_str = to_string(&user).expect("Failed to serialize User");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["id"], 303);
    assert_eq!(parsed["login"], "developer");
}

#[test]
fn test_user_deserialization() {
    // Create JSON
    let json_str = r#"{
        "id": 404,
        "login": "contributor"
    }"#;

    // Deserialize from JSON
    let user: User = from_str(json_str).expect("Failed to deserialize User");

    // Verify fields
    assert_eq!(user.id, 404);
    assert_eq!(user.login, "contributor");
}

#[test]
fn test_pull_request_with_author_serialization() {
    // Create a pull request with author
    let pr = PullRequest {
        number: 123,
        title: "feat(auth): add GitHub login".to_string(),
        draft: false,
        body: Some("This PR adds GitHub login functionality.\n\nFixes #42".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    // Serialize to JSON
    let json_str = to_string(&pr).expect("Failed to serialize PullRequest");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["number"], 123);
    assert_eq!(parsed["title"], "feat(auth): add GitHub login");
    assert_eq!(parsed["draft"], false);
    assert_eq!(parsed["author"]["id"], 456);
    assert_eq!(parsed["author"]["login"], "developer123");
}

#[test]
fn test_pull_request_with_author_deserialization() {
    // Create JSON with author
    let json_str = r#"{"number": 999, "title": "fix: critical bug", "draft": true, "body": null, "author": {"id": 789, "login": "bugfixer"}}"#;

    // Deserialize from JSON
    let pr: PullRequest = from_str(json_str).expect("Failed to deserialize PullRequest");

    // Verify fields
    assert_eq!(pr.number, 999);
    assert_eq!(pr.title, "fix: critical bug");
    assert_eq!(pr.draft, true);
    assert_eq!(pr.body, None);
    assert!(pr.author.is_some());
    let author = pr.author.unwrap();
    assert_eq!(author.id, 789);
    assert_eq!(author.login, "bugfixer");
}

#[test]
fn test_pull_request_without_author() {
    // Create a pull request without author (None)
    let pr = PullRequest {
        number: 42,
        title: "docs: update README".to_string(),
        draft: false,
        body: Some("Updated documentation".to_string()),
        author: None,
    };

    // Serialize to JSON
    let json_str = to_string(&pr).expect("Failed to serialize PullRequest");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["number"], 42);
    assert_eq!(parsed["title"], "docs: update README");
    assert!(parsed["author"].is_null());

    // Deserialize back
    let deserialized_pr: PullRequest =
        from_str(&json_str).expect("Failed to deserialize PullRequest");
    assert_eq!(deserialized_pr.number, 42);
    assert_eq!(deserialized_pr.author, None);
}

#[test]
fn test_pull_request_missing_author_field_backwards_compatibility() {
    // Test deserialization of JSON without author field (backwards compatibility)
    let json_str = r#"{"number": 100, "title": "feat: new feature", "draft": false, "body": "New feature description"}"#;

    // Deserialize from JSON - should work with missing author field
    let pr: PullRequest = from_str(json_str).expect("Failed to deserialize PullRequest");

    // Verify fields
    assert_eq!(pr.number, 100);
    assert_eq!(pr.title, "feat: new feature");
    assert!(!pr.draft);
    assert_eq!(pr.body, Some("New feature description".to_string()));
    assert_eq!(pr.author, None); // Should default to None
}
