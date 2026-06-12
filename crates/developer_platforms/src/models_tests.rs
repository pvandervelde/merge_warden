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
        milestone_number: None,
        head_sha: String::new(),
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
        milestone_number: None,
        head_sha: String::new(),
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
        milestone_number: None,
        head_sha: String::new(),
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
        milestone_number: None,
        head_sha: String::new(),
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

// ── IssueMetadata tests ──────────────────────────────────────────────────────

#[test]
fn test_issue_metadata_with_milestone_and_projects() {
    let metadata = IssueMetadata {
        milestone: Some(IssueMilestone {
            number: 3,
            title: "v1.2.0".to_string(),
        }),
        projects: vec![
            IssueProject {
                number: 1,
                owner_login: "myorg".to_string(),
                title: "Team Roadmap".to_string(),
            },
            IssueProject {
                number: 2,
                owner_login: "myorg".to_string(),
                title: "Sprint Board".to_string(),
            },
        ],
    };

    let milestone = metadata.milestone.unwrap();
    assert_eq!(milestone.number, 3);
    assert_eq!(milestone.title, "v1.2.0");
    assert_eq!(metadata.projects.len(), 2);
    assert_eq!(metadata.projects[0].title, "Team Roadmap");
    assert_eq!(metadata.projects[1].title, "Sprint Board");
}

#[test]
fn test_issue_metadata_no_milestone() {
    let metadata = IssueMetadata {
        milestone: None,
        projects: vec![IssueProject {
            number: 3,
            owner_login: "myorg".to_string(),
            title: "Roadmap".to_string(),
        }],
    };

    assert!(metadata.milestone.is_none());
    assert_eq!(metadata.projects.len(), 1);
}

#[test]
fn test_issue_metadata_no_projects() {
    let metadata = IssueMetadata {
        milestone: Some(IssueMilestone {
            number: 7,
            title: "v3.0.0".to_string(),
        }),
        projects: vec![],
    };

    assert_eq!(metadata.milestone.unwrap().number, 7);
    assert!(metadata.projects.is_empty());
}

#[test]
fn test_issue_metadata_empty() {
    let metadata = IssueMetadata {
        milestone: None,
        projects: vec![],
    };

    assert!(metadata.milestone.is_none());
    assert!(metadata.projects.is_empty());
}

#[test]
fn test_issue_metadata_clone() {
    let original = IssueMetadata {
        milestone: Some(IssueMilestone {
            number: 1,
            title: "v1.0.0".to_string(),
        }),
        projects: vec![IssueProject {
            number: 10,
            owner_login: "myorg".to_string(),
            title: "My Project".to_string(),
        }],
    };

    let cloned = original.clone();
    assert_eq!(cloned.milestone.unwrap().number, 1);
    assert_eq!(cloned.projects.len(), 1);
    assert_eq!(cloned.projects[0].number, 10);
}

// ── IssueMilestone tests ─────────────────────────────────────────────────────

#[test]
fn test_issue_milestone_fields() {
    let milestone = IssueMilestone {
        number: 12,
        title: "Q2 2025".to_string(),
    };

    assert_eq!(milestone.number, 12);
    assert_eq!(milestone.title, "Q2 2025");
}

#[test]
fn test_issue_milestone_clone() {
    let original = IssueMilestone {
        number: 5,
        title: "v2.0.0".to_string(),
    };
    let cloned = original.clone();

    assert_eq!(cloned.number, original.number);
    assert_eq!(cloned.title, original.title);
}

#[test]
fn test_issue_milestone_zero_number() {
    // milestone number 0 is technically invalid in GitHub but we don't validate here
    let milestone = IssueMilestone {
        number: 0,
        title: "empty".to_string(),
    };
    assert_eq!(milestone.number, 0);
}

// ── IssueProject tests ───────────────────────────────────────────────────────

#[test]
fn test_issue_project_fields() {
    let project = IssueProject {
        number: 5,
        owner_login: "myorg".to_string(),
        title: "Engineering Backlog".to_string(),
    };

    assert_eq!(project.number, 5);
    assert_eq!(project.owner_login, "myorg");
    assert_eq!(project.title, "Engineering Backlog");
}

#[test]
fn test_issue_project_clone() {
    let original = IssueProject {
        number: 3,
        owner_login: "myorg".to_string(),
        title: "Roadmap".to_string(),
    };
    let cloned = original.clone();

    assert_eq!(cloned.number, original.number);
    assert_eq!(cloned.owner_login, original.owner_login);
    assert_eq!(cloned.title, original.title);
}

#[test]
fn test_issue_project_zero_number_allowed() {
    // Struct does not validate — zero project number is accepted at construction time
    let project = IssueProject {
        number: 0,
        owner_login: String::new(),
        title: "Unnamed".to_string(),
    };
    assert_eq!(project.number, 0);
}

// ── CommitStatus tests ────────────────────────────────────────────────────────

// Tier 1 – Specification tests
// Each test maps directly to a behavioural assertion from the interface contract.

#[test]
fn test_commit_status_round_trip_all_fields_populated() {
    // Verify that a CommitStatus with all three fields populated survives a
    // serde_json serialise → deserialise round-trip without data loss.
    let original = CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "success".to_string(),
        description: Some("All stability checks passed".to_string()),
    };

    let json_str = to_string(&original).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert_eq!(restored.context, "renovate/stability-days");
    assert_eq!(restored.state, "success");
    assert_eq!(
        restored.description,
        Some("All stability checks passed".to_string())
    );
}

#[test]
fn test_commit_status_round_trip_description_null() {
    // Verify that description: None serialises to null and deserialises back
    // to None without error.
    let original = CommitStatus {
        context: "ci/build".to_string(),
        state: "pending".to_string(),
        description: None,
    };

    let json_str = to_string(&original).expect("Failed to serialize CommitStatus");

    // The JSON representation must have description as null
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse serialized JSON");
    assert!(
        parsed["description"].is_null(),
        "Expected description to be null in JSON, got: {}",
        parsed["description"]
    );

    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");
    assert_eq!(restored.description, None);
}

#[test]
fn test_commit_status_deserialize_from_realistic_github_api_payload() {
    // Deserialise from a verbatim GitHub Commit Statuses API-style JSON object
    // (the three fields this struct models).
    let json_str = r#"{
        "context": "renovate/stability-days",
        "state": "pending",
        "description": "Waiting for stability period to elapse"
    }"#;

    let status: CommitStatus =
        from_str(json_str).expect("Failed to deserialize CommitStatus from GitHub payload");

    assert_eq!(status.context, "renovate/stability-days");
    assert_eq!(status.state, "pending");
    assert_eq!(
        status.description,
        Some("Waiting for stability period to elapse".to_string())
    );
}

#[test]
fn test_commit_status_state_pending_round_trips() {
    // The GitHub-defined state value "pending" must survive a round-trip.
    let status = CommitStatus {
        context: "ci/test".to_string(),
        state: "pending".to_string(),
        description: None,
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert_eq!(restored.state, "pending");
}

#[test]
fn test_commit_status_state_success_round_trips() {
    // The GitHub-defined state value "success" must survive a round-trip.
    let status = CommitStatus {
        context: "ci/test".to_string(),
        state: "success".to_string(),
        description: None,
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert_eq!(restored.state, "success");
}

#[test]
fn test_commit_status_state_failure_round_trips() {
    // The GitHub-defined state value "failure" must survive a round-trip.
    let status = CommitStatus {
        context: "ci/test".to_string(),
        state: "failure".to_string(),
        description: None,
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert_eq!(restored.state, "failure");
}

#[test]
fn test_commit_status_state_error_round_trips() {
    // The GitHub-defined state value "error" must survive a round-trip.
    let status = CommitStatus {
        context: "ci/test".to_string(),
        state: "error".to_string(),
        description: None,
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert_eq!(restored.state, "error");
}

// Tier 2 – Adversarial / boundary tests
// These tests would fail against stubs or trivially wrong implementations.

#[test]
fn test_commit_status_clone_produces_equal_values() {
    // Clone must produce an independent copy with identical field values.
    // Fails if Clone is not derived or if a field is not copied.
    let original = CommitStatus {
        context: "security/scan".to_string(),
        state: "success".to_string(),
        description: Some("No vulnerabilities found".to_string()),
    };

    let cloned = original.clone();

    assert_eq!(cloned.context, original.context);
    assert_eq!(cloned.state, original.state);
    assert_eq!(cloned.description, original.description);
}

#[test]
fn test_commit_status_clone_with_none_description_produces_equal_values() {
    // Clone must also work correctly when description is None.
    let original = CommitStatus {
        context: "lint/clippy".to_string(),
        state: "pending".to_string(),
        description: None,
    };

    let cloned = original.clone();

    assert_eq!(cloned.context, original.context);
    assert_eq!(cloned.state, original.state);
    assert_eq!(cloned.description, None);
}

#[test]
fn test_commit_status_debug_format_does_not_panic_with_description() {
    // Debug formatting must not panic for a fully-populated instance.
    // Fails if Debug is not derived.
    let status = CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "failure".to_string(),
        description: Some("Package has not reached stability threshold".to_string()),
    };

    let debug_output = format!("{:?}", status);
    assert!(!debug_output.is_empty());
}

#[test]
fn test_commit_status_debug_format_does_not_panic_without_description() {
    // Debug formatting must not panic when description is None.
    let status = CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "pending".to_string(),
        description: None,
    };

    let debug_output = format!("{:?}", status);
    assert!(!debug_output.is_empty());
}

#[test]
fn test_commit_status_json_field_names_match_github_api_contract() {
    // The JSON keys produced by Serialize must be exactly "context", "state",
    // and "description" — matching the GitHub API field names.
    // Fails if serde rename attributes alter the field names.
    let status = CommitStatus {
        context: "ci/build".to_string(),
        state: "success".to_string(),
        description: Some("Build passed".to_string()),
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert!(
        parsed.get("context").is_some(),
        "Expected JSON key 'context' to be present"
    );
    assert!(
        parsed.get("state").is_some(),
        "Expected JSON key 'state' to be present"
    );
    assert!(
        parsed.get("description").is_some(),
        "Expected JSON key 'description' to be present"
    );
}

#[test]
fn test_commit_status_context_field_with_slashes_and_dots_preserved() {
    // A context value containing slashes and dots (the canonical GitHub format,
    // e.g. "renovate/stability-days") must survive serialisation and
    // deserialisation without truncation or escaping changes.
    let original_context = "renovate/stability-days";

    let status = CommitStatus {
        context: original_context.to_string(),
        state: "success".to_string(),
        description: None,
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert_eq!(restored.context, original_context);
}

#[test]
fn test_commit_status_description_some_value_round_trips_correctly() {
    // Confirm that a Some(description) value deserialises to Some with the
    // exact same string — distinct from the None path.
    let description_text = "Stability period of 3 days has elapsed";

    let original = CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "success".to_string(),
        description: Some(description_text.to_string()),
    };

    let json_str = to_string(&original).expect("Failed to serialize CommitStatus");
    let restored: CommitStatus =
        from_str(&json_str).expect("Failed to deserialize CommitStatus");

    assert!(
        restored.description.is_some(),
        "Expected description to be Some after round-trip"
    );
    assert_eq!(restored.description.unwrap(), description_text);
}

#[test]
fn test_commit_status_deserialize_ignores_extra_github_api_fields() {
    // The GitHub Commit Statuses API returns many additional fields (id, url,
    // avatar_url, created_at, updated_at, etc.).  Deserialisation must succeed
    // even when those extra fields are present, because serde's default
    // behaviour (deny_unknown_fields is NOT set) must apply.
    let json_str = r#"{
        "url": "https://api.github.com/repos/owner/repo/statuses/abc123def",
        "id": 99887766,
        "node_id": "SC_abc123",
        "state": "success",
        "description": "Build passed",
        "target_url": "https://ci.example.com/builds/42",
        "context": "ci/build",
        "created_at": "2024-01-15T10:00:00Z",
        "updated_at": "2024-01-15T10:05:00Z",
        "creator": {
            "login": "octocat",
            "id": 1
        }
    }"#;

    let status: CommitStatus =
        from_str(json_str).expect("Failed to deserialize CommitStatus with extra fields");

    assert_eq!(status.context, "ci/build");
    assert_eq!(status.state, "success");
    assert_eq!(status.description, Some("Build passed".to_string()));
}

#[test]
fn test_commit_status_description_none_serializes_as_null_key_present() {
    // Kill test for mutant: adding `#[serde(skip_serializing_if = "Option::is_none")]`
    // to the description field would cause description: None to be omitted from
    // the JSON output entirely, instead of being serialized as `"description": null`.
    //
    // The GitHub Commit Statuses API always emits the description key (even when null),
    // so the serialized wire format must include the key.  This test uses
    // `parsed.get("description")` (returns None when the key is absent) rather than
    // `parsed["description"]` (returns Value::Null for both absent and null), making
    // it sensitive to the difference.
    let status = CommitStatus {
        context: "ci/lint".to_string(),
        state: "pending".to_string(),
        description: None,
    };

    let json_str = to_string(&status).expect("Failed to serialize CommitStatus");
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse serialized JSON");

    assert!(
        parsed.get("description").is_some(),
        "Expected JSON key 'description' to be present even when value is None/null; \
         got JSON: {json_str}"
    );
    assert!(
        parsed["description"].is_null(),
        "Expected JSON value for 'description' to be null when description is None; \
         got: {}",
        parsed["description"]
    );
}
