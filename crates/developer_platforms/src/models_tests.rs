use super::*;
use serde_json::{from_str, to_string};

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
fn test_installation_deserialization() {
    // Create JSON with all fields
    let json_str = r#"{
        "id": 12345,
        "slug": "my-app-installation",
        "client_id": "Iv1.1234567890abcdef",
        "node_id": "MDIzOkluc3RhbGxhdGlvbjEyMzQ1",
        "name": "My App Installation"
    }"#;

    // Deserialize from JSON
    let installation: Installation =
        from_str(json_str).expect("Failed to deserialize Installation");

    // Verify fields
    assert_eq!(installation.id, 12345);
    assert_eq!(installation.slug, Some("my-app-installation".to_string()));
    assert_eq!(
        installation.client_id,
        Some("Iv1.1234567890abcdef".to_string())
    );
    assert_eq!(installation.node_id, "MDIzOkluc3RhbGxhdGlvbjEyMzQ1");
    assert_eq!(installation.name, Some("My App Installation".to_string()));
}

#[test]
fn test_installation_serialization() {
    // Create an installation
    let installation = Installation {
        id: 12345,
        slug: Some("my-app-installation".to_string()),
        client_id: Some("Iv1.1234567890abcdef".to_string()),
        node_id: "MDIzOkluc3RhbGxhdGlvbjEyMzQ1".to_string(),
        name: Some("My App Installation".to_string()),
    };

    // Serialize to JSON
    let json_str = to_string(&installation).expect("Failed to serialize Installation");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["id"], 12345);
    assert_eq!(parsed["slug"], "my-app-installation");
    assert_eq!(parsed["client_id"], "Iv1.1234567890abcdef");
    assert_eq!(parsed["node_id"], "MDIzOkluc3RhbGxhdGlvbjEyMzQ1");
    assert_eq!(parsed["name"], "My App Installation");
}

#[test]
fn test_installation_with_optional_fields_null() {
    // Create JSON with null optional fields
    let json_str = r#"{
        "id": 12345,
        "slug": null,
        "client_id": null,
        "node_id": "MDIzOkluc3RhbGxhdGlvbjEyMzQ1",
        "name": null
    }"#;

    // Deserialize from JSON
    let installation: Installation =
        from_str(json_str).expect("Failed to deserialize Installation");

    // Verify optional fields are None
    assert_eq!(installation.id, 12345);
    assert_eq!(installation.slug, None);
    assert_eq!(installation.client_id, None);
    assert_eq!(installation.node_id, "MDIzOkluc3RhbGxhdGlvbjEyMzQ1");
    assert_eq!(installation.name, None);
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
fn test_label_serialization() {
    // Create a label
    let label = Label {
        name: "bug".to_string(),
        description: None,
    };

    // Serialize to JSON
    let json_str = to_string(&label).expect("Failed to serialize Label");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["name"], "bug");
}

#[test]
fn test_organization_deserialization() {
    // Create JSON
    let json_str = r#"{"name": "my-company"}"#;

    // Deserialize from JSON
    let org: Organization = from_str(json_str).expect("Failed to deserialize Organization");

    // Verify fields
    assert_eq!(org.name, "my-company");
}

#[test]
fn test_organization_serialization() {
    // Create an organization
    let org = Organization {
        name: "my-company".to_string(),
    };

    // Serialize to JSON
    let json_str = to_string(&org).expect("Failed to serialize Organization");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["name"], "my-company");
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
fn test_pull_request_file_deserialization() {
    // Create JSON
    let json_str = r#"{"filename": "tests/test.rs", "additions": 25, "deletions": 10, "changes": 35, "status": "added"}"#;

    // Deserialize from JSON
    let file: PullRequestFile = from_str(json_str).expect("Failed to deserialize PullRequestFile");

    // Verify fields
    assert_eq!(file.filename, "tests/test.rs");
    assert_eq!(file.additions, 25);
    assert_eq!(file.deletions, 10);
    assert_eq!(file.changes, 35);
    assert_eq!(file.status, "added");
}

#[test]
fn test_pull_request_file_different_statuses() {
    // Test different file statuses
    let statuses = vec!["added", "modified", "deleted", "renamed", "copied"];

    for status in statuses {
        let file = PullRequestFile {
            filename: format!("file_{}.rs", status),
            additions: 10,
            deletions: 5,
            changes: 15,
            status: status.to_string(),
        };

        // Serialize and deserialize
        let json_str = to_string(&file).expect("Failed to serialize PullRequestFile");
        let deserialized: PullRequestFile =
            from_str(&json_str).expect("Failed to deserialize PullRequestFile");

        // Verify status is preserved
        assert_eq!(deserialized.status, status);
        assert_eq!(deserialized.filename, format!("file_{}.rs", status));
    }
}

#[test]
fn test_pull_request_file_large_changes() {
    // Test file with large number of changes
    let file = PullRequestFile {
        filename: "src/generated/large_file.rs".to_string(),
        additions: 1000,
        deletions: 500,
        changes: 1500,
        status: "modified".to_string(),
    };

    // Serialize and deserialize
    let json_str = to_string(&file).expect("Failed to serialize PullRequestFile");
    let deserialized: PullRequestFile =
        from_str(&json_str).expect("Failed to deserialize PullRequestFile");

    // Verify fields remain correct
    assert_eq!(deserialized.filename, "src/generated/large_file.rs");
    assert_eq!(deserialized.additions, 1000);
    assert_eq!(deserialized.deletions, 500);
    assert_eq!(deserialized.changes, 1500);
    assert_eq!(deserialized.status, "modified");
}

#[test]
fn test_pull_request_file_serialization() {
    // Create a pull request file
    let file = PullRequestFile {
        filename: "src/main.rs".to_string(),
        additions: 15,
        deletions: 5,
        changes: 20,
        status: "modified".to_string(),
    };

    // Serialize to JSON
    let json_str = to_string(&file).expect("Failed to serialize PullRequestFile");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["filename"], "src/main.rs");
    assert_eq!(parsed["additions"], 15);
    assert_eq!(parsed["deletions"], 5);
    assert_eq!(parsed["changes"], 20);
    assert_eq!(parsed["status"], "modified");
}

#[test]
fn test_pull_request_file_special_characters_in_filename() {
    // Test filename with special characters, spaces, and Unicode
    let file = PullRequestFile {
        filename: "src/files with spaces/特殊文字/file-name_with.dots.rs".to_string(),
        additions: 5,
        deletions: 2,
        changes: 7,
        status: "modified".to_string(),
    };

    // Serialize and deserialize
    let json_str = to_string(&file).expect("Failed to serialize PullRequestFile");
    let deserialized: PullRequestFile =
        from_str(&json_str).expect("Failed to deserialize PullRequestFile");

    // Verify filename is preserved correctly
    assert_eq!(
        deserialized.filename,
        "src/files with spaces/特殊文字/file-name_with.dots.rs"
    );
}

#[test]
fn test_pull_request_file_zero_changes() {
    // Test file with no changes (edge case)
    let file = PullRequestFile {
        filename: "README.md".to_string(),
        additions: 0,
        deletions: 0,
        changes: 0,
        status: "unchanged".to_string(),
    };

    // Serialize and deserialize
    let json_str = to_string(&file).expect("Failed to serialize PullRequestFile");
    let deserialized: PullRequestFile =
        from_str(&json_str).expect("Failed to deserialize PullRequestFile");

    // Verify fields remain correct
    assert_eq!(deserialized.filename, "README.md");
    assert_eq!(deserialized.additions, 0);
    assert_eq!(deserialized.deletions, 0);
    assert_eq!(deserialized.changes, 0);
    assert_eq!(deserialized.status, "unchanged");
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
fn test_pull_request_with_author_deserialization() {
    // Create JSON with author
    let json_str = r#"{"number": 999, "title": "fix: critical bug", "draft": true, "body": null, "author": {"id": 789, "login": "bugfixer"}}"#;

    // Deserialize from JSON
    let pr: PullRequest = from_str(json_str).expect("Failed to deserialize PullRequest");

    // Verify fields
    assert_eq!(pr.number, 999);
    assert_eq!(pr.title, "fix: critical bug");
    assert!(pr.draft);
    assert_eq!(pr.body, None);
    assert!(pr.author.is_some());
    let author = pr.author.unwrap();
    assert_eq!(author.id, 789);
    assert_eq!(author.login, "bugfixer");
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
fn test_repository_deserialization() {
    // Create JSON
    let json_str = r#"{
        "full_name": "octocat/Hello-World",
        "name": "Hello-World",
        "node_id": "MDEwOlJlcG9zaXRvcnkxMjk2MjY5",
        "private": false
    }"#;

    // Deserialize from JSON
    let repo: Repository = from_str(json_str).expect("Failed to deserialize Repository");

    // Verify fields
    assert_eq!(repo.full_name, "octocat/Hello-World");
    assert_eq!(repo.name, "Hello-World");
    assert_eq!(repo.node_id, "MDEwOlJlcG9zaXRvcnkxMjk2MjY5");
    assert!(!repo.private);
}

#[test]
fn test_repository_private_true() {
    // Create JSON for private repository
    let json_str = r#"{
        "full_name": "private-user/secret-repo",
        "name": "secret-repo",
        "node_id": "MDEwOlJlcG9zaXRvcnkxMjM0NTY3",
        "private": true
    }"#;

    // Deserialize from JSON
    let repo: Repository = from_str(json_str).expect("Failed to deserialize Repository");

    // Verify private field is true
    assert!(repo.private);
    assert_eq!(repo.full_name, "private-user/secret-repo");
}

#[test]
fn test_repository_serialization() {
    // Create a repository
    let repo = Repository {
        full_name: "octocat/Hello-World".to_string(),
        name: "Hello-World".to_string(),
        node_id: "MDEwOlJlcG9zaXRvcnkxMjk2MjY5".to_string(),
        private: false,
    };

    // Serialize to JSON
    let json_str = to_string(&repo).expect("Failed to serialize Repository");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["full_name"], "octocat/Hello-World");
    assert_eq!(parsed["name"], "Hello-World");
    assert_eq!(parsed["node_id"], "MDEwOlJlcG9zaXRvcnkxMjk2MjY5");
    assert_eq!(parsed["private"], false);
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
fn test_review_deserialization_changes_requested() {
    // Create JSON for changes requested review
    let json_str = r#"{
        "id": 789,
        "state": "changes_requested",
        "user": {
            "id": 456,
            "login": "strict-reviewer"
        }
    }"#;

    // Deserialize from JSON
    let review: Review = from_str(json_str).expect("Failed to deserialize Review");

    // Verify fields
    assert_eq!(review.id, 789);
    assert_eq!(review.state, "changes_requested");
    assert_eq!(review.user.id, 456);
    assert_eq!(review.user.login, "strict-reviewer");
}

#[test]
fn test_review_deserialization_commented() {
    // Create JSON for commented review
    let json_str = r#"{
        "id": 999,
        "state": "commented",
        "user": {
            "id": 789,
            "login": "helpful-reviewer"
        }
    }"#;

    // Deserialize from JSON
    let review: Review = from_str(json_str).expect("Failed to deserialize Review");

    // Verify fields
    assert_eq!(review.id, 999);
    assert_eq!(review.state, "commented");
    assert_eq!(review.user.id, 789);
    assert_eq!(review.user.login, "helpful-reviewer");
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
fn test_review_serialization_approved() {
    // Create an approved review
    let review = Review {
        id: 789,
        state: "approved".to_string(),
        user: User {
            id: 123,
            login: "reviewer123".to_string(),
        },
    };

    // Serialize to JSON
    let json_str = to_string(&review).expect("Failed to serialize Review");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed["id"], 789);
    assert_eq!(parsed["state"], "approved");
    assert_eq!(parsed["user"]["id"], 123);
    assert_eq!(parsed["user"]["login"], "reviewer123");
}

#[test]
fn test_user_default() {
    // Test default implementation
    let user: User = Default::default();

    // Verify default values
    assert_eq!(user.id, 0);
    assert_eq!(user.login, "");
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
fn test_user_equality() {
    // Create two identical users
    let user1 = User {
        id: 456,
        login: "octocat".to_string(),
    };
    let user2 = User {
        id: 456,
        login: "octocat".to_string(),
    };

    // Test equality
    assert_eq!(user1, user2);
}

#[test]
fn test_user_inequality() {
    // Create two different users
    let user1 = User {
        id: 456,
        login: "octocat".to_string(),
    };
    let user2 = User {
        id: 789,
        login: "other-user".to_string(),
    };

    // Test inequality
    assert_ne!(user1, user2);
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
