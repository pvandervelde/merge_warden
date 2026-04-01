//! Tests for the `GitHubProvider` implementation using WireMock to mock GitHub API calls.
//!
//! Each test spins up a WireMock server, wires a `GitHubProvider` pointing at it,
//! and verifies the correct HTTP path, method, and request/response mapping.

use async_trait::async_trait;
use github_bot_sdk::{
    auth::{
        AuthenticationProvider, GitHubAppId, Installation, InstallationId, InstallationPermissions,
        InstallationToken, JsonWebToken,
    },
    client::{ClientConfig, GitHubClient},
    error::AuthError,
};
use serde_json::json;
use wiremock::{
    matchers::{body_string_contains, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

use super::GitHubProvider;
use crate::errors::Error;
use crate::{ConfigFetcher, IssueMetadataProvider, PullRequestProvider};

// ---------------------------------------------------------------------------
// Test helper: mock authentication provider
// ---------------------------------------------------------------------------

/// Minimal `AuthenticationProvider` suitable for unit tests.
///
/// Returns a pre-configured installation token for any installation ID.
#[derive(Clone)]
struct MockAuth {
    token: String,
}

impl MockAuth {
    fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

#[async_trait]
impl AuthenticationProvider for MockAuth {
    async fn app_token(&self) -> Result<JsonWebToken, AuthError> {
        let app_id = GitHubAppId::new(1);
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);
        Ok(JsonWebToken::new(
            "test.jwt.token".to_string(),
            app_id,
            expires_at,
        ))
    }

    async fn installation_token(
        &self,
        _installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        let id = InstallationId::new(12345);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        Ok(InstallationToken::new(
            self.token.clone(),
            id,
            expires_at,
            InstallationPermissions::default(),
            vec![],
        ))
    }

    async fn refresh_installation_token(
        &self,
        installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        self.installation_token(installation_id).await
    }

    async fn list_installations(&self) -> Result<Vec<Installation>, AuthError> {
        Ok(vec![])
    }

    async fn get_installation_repositories(
        &self,
        _installation_id: InstallationId,
    ) -> Result<Vec<github_bot_sdk::auth::Repository>, AuthError> {
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// Test helper: create provider pointing at WireMock
// ---------------------------------------------------------------------------

/// Constructs a `GitHubProvider` pointing at the WireMock server URI.
async fn make_provider(server_uri: &str) -> GitHubProvider {
    let auth = MockAuth::new("ghs_test_token");
    let github_client = GitHubClient::builder(auth)
        .config(
            ClientConfig::default()
                .with_github_api_url(server_uri.to_string())
                .with_max_retries(0),
        )
        .build()
        .expect("Failed to build GitHubClient");

    let installation_id = InstallationId::new(12345);
    let installation_client = github_client
        .installation_by_id(installation_id)
        .await
        .expect("Failed to create InstallationClient");

    GitHubProvider::new(installation_client)
}

// ---------------------------------------------------------------------------
// add_comment
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_add_comment_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/owner/repo/issues/42/comments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 1,
            "node_id": "IC_1",
            "body": "Hello from the bot",
            "user": { "login": "bot", "id": 1, "node_id": "U_1", "type": "Bot" },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "html_url": "https://github.com/owner/repo/issues/42#issuecomment-1"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .add_comment("owner", "repo", 42, "Hello from the bot")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_comment_not_found_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/owner/repo/issues/99/comments"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.add_comment("owner", "repo", 99, "text").await;

    assert!(matches!(result, Err(Error::FailedToUpdatePullRequest(_))));
}

// ---------------------------------------------------------------------------
// add_labels
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_add_labels_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/owner/repo/issues/5/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "node_id": "L_1", "name": "bug", "color": "ff0000",
              "description": null, "default": false }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .add_labels("owner", "repo", 5, &["bug".to_string()])
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_labels_api_error_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/owner/repo/issues/5/labels"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .add_labels("owner", "repo", 5, &["bug".to_string()])
        .await;

    assert!(matches!(result, Err(Error::FailedToUpdatePullRequest(_))));
}

// ---------------------------------------------------------------------------
// delete_comment
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_delete_comment_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/owner/repo/issues/comments/777"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.delete_comment("owner", "repo", 777).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_comment_not_found_is_error() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/owner/repo/issues/comments/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.delete_comment("owner", "repo", 999).await;

    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// get_pull_request
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_pull_request_returns_pr_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1001,
            "node_id": "PR_1",
            "number": 1,
            "title": "feat: add new feature",
            "body": "This adds a great feature",
            "state": "open",
            "user": { "login": "alice", "id": 42, "node_id": "U_42", "type": "User" },
            "head": {
                "ref": "feature-branch",
                "sha": "abc123",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "base": {
                "ref": "main",
                "sha": "def456",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "draft": false,
            "merged": false,
            "mergeable": null,
            "merge_commit_sha": null,
            "assignees": [],
            "requested_reviewers": [],
            "labels": [],
            "milestone": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "closed_at": null,
            "merged_at": null,
            "html_url": "https://github.com/owner/repo/pull/1"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let pr = provider.get_pull_request("owner", "repo", 1).await.unwrap();

    assert_eq!(pr.number, 1);
    assert_eq!(pr.title, "feat: add new feature");
    assert_eq!(pr.body, Some("This adds a great feature".to_string()));
    assert!(!pr.draft);
    assert_eq!(pr.author.as_ref().unwrap().login, "alice");
}

#[tokio::test]
async fn test_get_pull_request_draft_flag() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1002,
            "node_id": "PR_2",
            "number": 2,
            "title": "WIP: draft pr",
            "body": null,
            "state": "open",
            "user": { "login": "bob", "id": 43, "node_id": "U_43", "type": "User" },
            "head": {
                "ref": "wip-branch",
                "sha": "aaa111",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "base": {
                "ref": "main",
                "sha": "bbb222",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "draft": true,
            "merged": false,
            "mergeable": null,
            "merge_commit_sha": null,
            "assignees": [],
            "requested_reviewers": [],
            "labels": [],
            "milestone": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "closed_at": null,
            "merged_at": null,
            "html_url": "https://github.com/owner/repo/pull/2"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let pr = provider.get_pull_request("owner", "repo", 2).await.unwrap();

    assert!(pr.draft);
    assert_eq!(pr.body, None);
}

#[tokio::test]
async fn test_get_pull_request_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.get_pull_request("owner", "repo", 999).await;

    assert!(matches!(result, Err(Error::InvalidResponse)));
}

// ---------------------------------------------------------------------------
// get_pull_request_files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_pull_request_files_returns_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "filename": "src/main.rs",
                "status": "modified",
                "additions": 10,
                "deletions": 3,
                "changes": 13
            },
            {
                "filename": "tests/lib.rs",
                "status": "added",
                "additions": 50,
                "deletions": 0,
                "changes": 50
            }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let files = provider
        .get_pull_request_files("owner", "repo", 1)
        .await
        .unwrap();

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].filename, "src/main.rs");
    assert_eq!(files[0].additions, 10);
    assert_eq!(files[0].deletions, 3);
    assert_eq!(files[0].status, "modified");
    assert_eq!(files[1].filename, "tests/lib.rs");
    assert_eq!(files[1].status, "added");
}

#[tokio::test]
async fn test_get_pull_request_files_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/3/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let files = provider
        .get_pull_request_files("owner", "repo", 3)
        .await
        .unwrap();

    assert!(files.is_empty());
}

// ---------------------------------------------------------------------------
// list_applied_labels
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_applied_labels_returns_pr_labels() {
    let server = MockServer::start().await;

    // list_applied_labels calls get_pull_request to read the labels field
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1001,
            "node_id": "PR_1",
            "number": 1,
            "title": "feat: labelled pr",
            "body": null,
            "state": "open",
            "user": { "login": "alice", "id": 42, "node_id": "U_42", "type": "User" },
            "head": {
                "ref": "feature-branch",
                "sha": "abc123",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "base": {
                "ref": "main",
                "sha": "def456",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "draft": false,
            "merged": false,
            "mergeable": null,
            "merge_commit_sha": null,
            "assignees": [],
            "requested_reviewers": [],
            "labels": [
                { "id": 1, "node_id": "L_1", "name": "enhancement",
                  "color": "84b6eb", "description": "An enhancement", "default": false },
                { "id": 2, "node_id": "L_2", "name": "good first issue",
                  "color": "0075ca", "description": null, "default": true }
            ],
            "milestone": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "closed_at": null,
            "merged_at": null,
            "html_url": "https://github.com/owner/repo/pull/1"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let labels = provider
        .list_applied_labels("owner", "repo", 1)
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0].name, "enhancement");
    assert_eq!(labels[0].description, Some("An enhancement".to_string()));
    assert_eq!(labels[1].name, "good first issue");
    assert_eq!(labels[1].description, None);
}

#[tokio::test]
async fn test_list_applied_labels_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1007,
            "node_id": "PR_7",
            "number": 7,
            "title": "feat: unlabelled pr",
            "body": null,
            "state": "open",
            "user": { "login": "carol", "id": 50, "node_id": "U_50", "type": "User" },
            "head": {
                "ref": "fix-branch",
                "sha": "fff000",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "base": {
                "ref": "main",
                "sha": "eee999",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "draft": false,
            "merged": false,
            "mergeable": null,
            "merge_commit_sha": null,
            "assignees": [],
            "requested_reviewers": [],
            "labels": [],
            "milestone": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "closed_at": null,
            "merged_at": null,
            "html_url": "https://github.com/owner/repo/pull/7"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let labels = provider
        .list_applied_labels("owner", "repo", 7)
        .await
        .unwrap();

    assert!(labels.is_empty());
}

// ---------------------------------------------------------------------------
// list_available_labels
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_available_labels_returns_repo_labels() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "node_id": "L_1", "name": "bug",
              "color": "d73a4a", "description": "Something wrong", "default": true },
            { "id": 2, "node_id": "L_2", "name": "enhancement",
              "color": "84b6eb", "description": null, "default": false }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let labels = provider
        .list_available_labels("owner", "repo")
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0].name, "bug");
    assert_eq!(labels[0].description, Some("Something wrong".to_string()));
    assert_eq!(labels[1].name, "enhancement");
}

// ---------------------------------------------------------------------------
// list_comments
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_comments_returns_all_comments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/3/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 100,
                "node_id": "IC_100",
                "body": "First comment",
                "user": { "login": "alice", "id": 42, "node_id": "U_42", "type": "User" },
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "html_url": "https://github.com/owner/repo/issues/3#issuecomment-100"
            },
            {
                "id": 101,
                "node_id": "IC_101",
                "body": "Second comment",
                "user": { "login": "bob", "id": 43, "node_id": "U_43", "type": "User" },
                "created_at": "2024-01-02T00:00:00Z",
                "updated_at": "2024-01-02T00:00:00Z",
                "html_url": "https://github.com/owner/repo/issues/3#issuecomment-101"
            }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let comments = provider.list_comments("owner", "repo", 3).await.unwrap();

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].id, 100);
    assert_eq!(comments[0].body, "First comment");
    assert_eq!(comments[0].user.login, "alice");
    assert_eq!(comments[1].id, 101);
    assert_eq!(comments[1].body, "Second comment");
}

#[tokio::test]
async fn test_list_comments_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/4/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let comments = provider.list_comments("owner", "repo", 4).await.unwrap();

    assert!(comments.is_empty());
}

// ---------------------------------------------------------------------------
// remove_label
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_remove_label_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/owner/repo/issues/5/labels/bug"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.remove_label("owner", "repo", 5, "bug").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_label_not_found_is_error() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/owner/repo/issues/5/labels/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .remove_label("owner", "repo", 5, "nonexistent")
        .await;

    assert!(matches!(result, Err(Error::FailedToUpdatePullRequest(_))));
}

// ---------------------------------------------------------------------------
// update_pr_check_status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_update_pr_check_status_success() {
    let server = MockServer::start().await;

    // First: GET pull request to retrieve head SHA
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1010,
            "node_id": "PR_10",
            "number": 10,
            "title": "feat: check test",
            "body": null,
            "state": "open",
            "user": { "login": "dave", "id": 55, "node_id": "U_55", "type": "User" },
            "head": {
                "ref": "check-branch",
                "sha": "deadbeef",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "base": {
                "ref": "main",
                "sha": "cafebabe",
                "repo": { "id": 9, "name": "repo", "full_name": "owner/repo" }
            },
            "draft": false,
            "merged": false,
            "mergeable": null,
            "merge_commit_sha": null,
            "assignees": [],
            "requested_reviewers": [],
            "labels": [],
            "milestone": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "closed_at": null,
            "merged_at": null,
            "html_url": "https://github.com/owner/repo/pull/10"
        })))
        .mount(&server)
        .await;

    // Second: POST check run
    Mock::given(method("POST"))
        .and(path("/repos/owner/repo/check-runs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 5001,
            "name": "MergeWarden",
            "status": "completed",
            "conclusion": "success"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .update_pr_check_status(
            "owner",
            "repo",
            10,
            "success",
            "All checks passed",
            "PR meets all requirements",
            "Everything looks good",
        )
        .await;

    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// fetch_config (ConfigFetcher)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_config_returns_file_content() {
    use base64::Engine;
    let server = MockServer::start().await;
    let content_b64 = base64::engine::general_purpose::STANDARD.encode(b"key = \"value\"\n");

    // First: GET repository to get default branch
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "name": "repo",
            "full_name": "owner/repo",
            "default_branch": "main",
            "private": false,
            "html_url": "https://github.com/owner/repo"
        })))
        .mount(&server)
        .await;

    // Second: GET file contents
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/contents/.merge-warden.toml"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "file",
            "encoding": "base64",
            "name": ".merge-warden.toml",
            "path": ".merge-warden.toml",
            "content": format!("{}\n", content_b64)
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .fetch_config("owner", "repo", ".merge-warden.toml")
        .await
        .unwrap();

    assert_eq!(result, Some("key = \"value\"\n".to_string()));
}

#[tokio::test]
async fn test_fetch_config_missing_file_returns_none() {
    let server = MockServer::start().await;

    // GET repository to get default branch
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "name": "repo",
            "full_name": "owner/repo",
            "default_branch": "main",
            "private": false,
            "html_url": "https://github.com/owner/repo"
        })))
        .mount(&server)
        .await;

    // GET file: 404 (does not exist)
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/contents/.merge-warden.toml"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .fetch_config("owner", "repo", ".merge-warden.toml")
        .await
        .unwrap();

    assert_eq!(result, None);
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_rate_limit_error_maps_correctly() {
    let server = MockServer::start().await;

    // Simulate GitHub rate limiting on a PR fetch
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/1"))
        .respond_with(
            ResponseTemplate::new(429)
                .append_header("x-ratelimit-reset", "9999999999")
                .set_body_string("rate limit exceeded"),
        )
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.get_pull_request("owner", "repo", 1).await;

    // 429 maps to RateLimitExceeded
    assert!(matches!(result, Err(Error::RateLimitExceeded)));
}

#[tokio::test]
async fn test_auth_error_maps_correctly() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/1"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.get_pull_request("owner", "repo", 1).await;

    assert!(matches!(result, Err(Error::AuthError(_))));
}

// ---------------------------------------------------------------------------
// list_pr_reviews
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_pr_reviews_empty_returns_vec() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/7/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 7).await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_list_pr_reviews_single_page_returns_all_reviews() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/10/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "state": "APPROVED",           "user": { "id": 100, "login": "alice" } },
            { "id": 2, "state": "CHANGES_REQUESTED",  "user": { "id": 101, "login": "bob"   } }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 10).await.unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, 1);
    assert_eq!(result[0].state, "approved");
    assert_eq!(result[0].user.login, "alice");
    assert_eq!(result[1].id, 2);
    assert_eq!(result[1].state, "changes_requested");
    assert_eq!(result[1].user.login, "bob");
}

#[tokio::test]
async fn test_list_pr_reviews_state_is_lowercased() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/11/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 99, "state": "COMMENTED", "user": { "id": 200, "login": "carol" } }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 11).await.unwrap();

    assert_eq!(
        result[0].state, "commented",
        "GitHub returns uppercase state strings; provider must lowercase them"
    );
}

#[tokio::test]
async fn test_list_pr_reviews_paginated_fetches_all_pages() {
    let server = MockServer::start().await;

    // Page 1: return 2 reviews and a Link header signalling page 2 exists.
    // The `next` URL in the Link header is absolute but we only use has_next();
    // the implementation drives pagination itself via the page counter.
    let link_header = format!(
        "<{uri}/repos/owner/repo/pulls/20/reviews?per_page=100&page=2>; rel=\"next\", \
         <{uri}/repos/owner/repo/pulls/20/reviews?per_page=100&page=2>; rel=\"last\"",
        uri = server.uri()
    );

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/20/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("Link", link_header.as_str())
                .set_body_json(json!([
                    { "id": 1, "state": "APPROVED",          "user": { "id": 1, "login": "u1" } },
                    { "id": 2, "state": "CHANGES_REQUESTED", "user": { "id": 2, "login": "u2" } }
                ])),
        )
        .mount(&server)
        .await;

    // Page 2: no Link header → last page.
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/20/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 3, "state": "APPROVED", "user": { "id": 3, "login": "u3" } }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 20).await.unwrap();

    assert_eq!(result.len(), 3, "Both pages combined must yield 3 reviews");
    assert_eq!(result[0].id, 1);
    assert_eq!(result[1].id, 2);
    assert_eq!(result[2].id, 3);
}

#[tokio::test]
async fn test_list_pr_reviews_three_pages_fetches_all() {
    let server = MockServer::start().await;

    let make_link = |page: u32, server_uri: &str| -> String {
        format!(
            "<{uri}/repos/owner/repo/pulls/21/reviews?per_page=100&page={next}>; rel=\"next\"",
            uri = server_uri,
            next = page
        )
    };

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/21/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("Link", make_link(2, &server.uri()).as_str())
                .set_body_json(json!([
                    { "id": 10, "state": "APPROVED", "user": { "id": 10, "login": "a" } }
                ])),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/21/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("Link", make_link(3, &server.uri()).as_str())
                .set_body_json(json!([
                    { "id": 11, "state": "COMMENTED", "user": { "id": 11, "login": "b" } }
                ])),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/21/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 12, "state": "CHANGES_REQUESTED", "user": { "id": 12, "login": "c" } }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 21).await.unwrap();

    assert_eq!(result.len(), 3);
    let ids: Vec<u64> = result.iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![10, 11, 12]);
}

#[tokio::test]
async fn test_list_pr_reviews_api_error_returns_err() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/99/reviews"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 99).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_pr_reviews_null_user_id_review_is_skipped() {
    // Reviews whose user object is null or whose user.id is missing cannot be
    // attributed to a specific reviewer and must be silently dropped rather than
    // colliding at key 0 in the per-reviewer deduplication HashMap.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/30/reviews"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            // Valid review — should be included.
            { "id": 1, "state": "APPROVED", "user": { "id": 100, "login": "alice" } },
            // Null user object — should be skipped.
            { "id": 2, "state": "APPROVED", "user": null },
            // Missing user.id — should be skipped.
            { "id": 3, "state": "CHANGES_REQUESTED", "user": { "login": "bot" } }
        ])))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.list_pr_reviews("owner", "repo", 30).await.unwrap();

    // Only the review with a valid, non-null user.id survives.
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 1);
    assert_eq!(result[0].user.id, 100);
}

// ---------------------------------------------------------------------------
// IssueMetadataProvider — get_issue_metadata
// ---------------------------------------------------------------------------

fn minimal_issue_json(issue_number: u64, with_milestone: bool) -> serde_json::Value {
    let milestone = if with_milestone {
        json!({
            "id": 100,
            "node_id": "MI_100",
            "number": 5,
            "title": "v1.0",
            "description": null,
            "state": "open",
            "open_issues": 3,
            "closed_issues": 7,
            "due_on": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-02-01T00:00:00Z",
            "closed_at": null
        })
    } else {
        json!(null)
    };

    json!({
        "id": 1,
        "node_id": "I_1",
        "number": issue_number,
        "title": "Test Issue",
        "body": null,
        "state": "open",
        "user": { "id": 1, "login": "user", "node_id": "U_1", "type": "User" },
        "assignees": [],
        "labels": [],
        "milestone": milestone,
        "comments": 0,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "closed_at": null,
        "html_url": "https://github.com/owner/repo/issues/1"
    })
}

/// Helper: GraphQL response JSON for a single linked project.
fn graphql_linked_projects_response(
    project_number: u64,
    title: &str,
    owner_login: &str,
) -> serde_json::Value {
    json!({
        "data": {
            "repository": {
                "issue": {
                    "projectsV2": {
                        "pageInfo": { "hasNextPage": false, "endCursor": null },
                        "nodes": [{
                            "id": format!("PVT_node{}", project_number),
                            "databaseId": project_number,
                            "number": project_number,
                            "title": title,
                            "description": null,
                            "public": true,
                            "url": format!("https://github.com/orgs/{}/projects/{}", owner_login, project_number),
                            "createdAt": "2024-01-01T00:00:00Z",
                            "updatedAt": "2024-01-01T00:00:00Z",
                            "owner": {
                                "id": "O_org1",
                                "databaseId": 100,
                                "login": owner_login,
                                "type": "Organization"
                            }
                        }]
                    }
                }
            }
        }
    })
}

/// Helper: GraphQL response JSON for empty linked projects.
fn graphql_no_linked_projects_response() -> serde_json::Value {
    json!({
        "data": {
            "repository": {
                "issue": {
                    "projectsV2": {
                        "pageInfo": { "hasNextPage": false, "endCursor": null },
                        "nodes": []
                    }
                }
            }
        }
    })
}

/// Helper: returns a GraphQL response with the org project node ID.
fn graphql_project_node_id_org_response(node_id: &str) -> serde_json::Value {
    json!({
        "data": {
            "organization": {
                "projectV2": {
                    "id": node_id
                }
            }
        }
    })
}

/// Helper: GraphQL response for AddProjectV2ItemById mutation.
fn graphql_add_item_response(item_id: &str) -> serde_json::Value {
    json!({
        "data": {
            "addProjectV2ItemById": {
                "item": {
                    "id": item_id,
                    "type": "PullRequest",
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z",
                    "content": {
                        "id": "PR_1"
                    }
                }
            }
        }
    })
}

fn minimal_pr_json(pr_number: u64) -> serde_json::Value {
    let repo = json!({
        "id": 1,
        "name": "repo",
        "full_name": "owner/repo",
        "owner": { "id": 1, "login": "owner", "node_id": "O_1", "type": "Organization" },
        "private": false,
        "html_url": "https://github.com/owner/repo",
        "clone_url": "https://github.com/owner/repo.git"
    });
    json!({
        "id": 1,
        "node_id": "PR_1",
        "number": pr_number,
        "title": "Test PR",
        "body": null,
        "state": "open",
        "user": { "id": 1, "login": "user", "node_id": "U_1", "type": "User" },
        "head": { "ref": "feature", "sha": "abc123", "repo": repo.clone() },
        "base": { "ref": "main", "sha": "def456", "repo": repo },
        "draft": false,
        "merged": false,
        "mergeable": null,
        "merge_commit_sha": null,
        "assignees": [],
        "requested_reviewers": [],
        "labels": [],
        "milestone": null,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "closed_at": null,
        "merged_at": null,
        "html_url": "https://github.com/owner/repo/pull/1"
    })
}

#[tokio::test]
async fn test_get_issue_metadata_with_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_issue_json(42, true)))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .get_issue_metadata("owner", "repo", 42)
        .await
        .unwrap();

    let metadata = result.expect("Expected Some(IssueMetadata)");
    let milestone = metadata.milestone.expect("Expected milestone");
    assert_eq!(milestone.number, 5);
    assert_eq!(milestone.title, "v1.0");
    assert!(
        metadata.projects.is_empty(),
        "Projects should be empty when GraphQL returns no linked projects"
    );
}

#[tokio::test]
async fn test_get_issue_metadata_without_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_issue_json(10, false)))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .get_issue_metadata("owner", "repo", 10)
        .await
        .unwrap();

    let metadata = result.expect("Expected Some(IssueMetadata)");
    assert!(metadata.milestone.is_none());
    assert!(metadata.projects.is_empty());
}

#[tokio::test]
async fn test_get_issue_metadata_not_found_returns_none() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .get_issue_metadata("owner", "repo", 999)
        .await
        .unwrap();

    assert!(result.is_none(), "404 should yield Ok(None)");
}

#[tokio::test]
async fn test_get_issue_metadata_api_error_returns_err() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/7"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider.get_issue_metadata("owner", "repo", 7).await;

    assert!(result.is_err(), "500 should yield Err");
}

/// Verifies that `get_issue_metadata` populates the `projects` field when the
/// GraphQL endpoint returns linked projects for the issue.
#[tokio::test]
async fn test_get_issue_metadata_with_linked_projects() {
    let server = MockServer::start().await;

    // REST: issue exists with no milestone.
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_issue_json(10, false)))
        .mount(&server)
        .await;

    // GraphQL: issue is linked to one project.
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(graphql_linked_projects_response(5, "Roadmap", "myorg")),
        )
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let metadata = provider
        .get_issue_metadata("owner", "repo", 10)
        .await
        .expect("should succeed")
        .expect("should be Some");

    assert!(metadata.milestone.is_none());
    assert_eq!(metadata.projects.len(), 1);
    assert_eq!(metadata.projects[0].number, 5);
    assert_eq!(metadata.projects[0].owner_login, "myorg");
    assert_eq!(metadata.projects[0].title, "Roadmap");
}

// ---------------------------------------------------------------------------
// IssueMetadataProvider — set_pull_request_milestone
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_pull_request_milestone_success() {
    let server = MockServer::start().await;

    // SDK calls PATCH /repos/{owner}/{repo}/pulls/{number} to set the milestone.
    Mock::given(method("PATCH"))
        .and(path("/repos/owner/repo/pulls/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_pr_json(42)))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .set_pull_request_milestone("owner", "repo", 42, Some(5))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_pull_request_milestone_clear() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/owner/repo/pulls/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_pr_json(42)))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .set_pull_request_milestone("owner", "repo", 42, None)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_pull_request_milestone_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/owner/repo/pulls/99"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found"
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .set_pull_request_milestone("owner", "repo", 99, Some(3))
        .await;

    assert!(matches!(result, Err(Error::FailedToUpdatePullRequest(_))));
}

// ---------------------------------------------------------------------------
// IssueMetadataProvider — add_pull_request_to_project
// ---------------------------------------------------------------------------

/// Verifies that `add_pull_request_to_project` fetches the PR node ID and then
/// calls the GraphQL mutation to add the PR to the project.
#[tokio::test]
async fn test_add_pull_request_to_project_adds_pr_to_project() {
    let server = MockServer::start().await;

    // Fetch PR to get node_id: GET /repos/owner/repo/pulls/42
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_pr_json(42)))
        .mount(&server)
        .await;

    // Resolve project node ID: POST /graphql (GetProjectNodeIdOrg)
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("GetProjectNodeIdOrg"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(graphql_project_node_id_org_response("PVT_orgnode5")),
        )
        .mount(&server)
        .await;

    // Add item to project: POST /graphql (AddProjectV2Item mutation)
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("AddProjectV2Item"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(graphql_add_item_response("PVTI_item1")),
        )
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .add_pull_request_to_project("owner", "repo", 42, 5, "myorg")
        .await;

    assert!(
        result.is_ok(),
        "add_pull_request_to_project must succeed: {result:?}"
    );
}

/// Verifies that `add_pull_request_to_project` returns an error when the project
/// is not found (GraphQL returns NOT_FOUND).
#[tokio::test]
async fn test_add_pull_request_to_project_project_not_found() {
    let server = MockServer::start().await;

    // Fetch PR: GET /repos/owner/repo/pulls/42
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_pr_json(42)))
        .mount(&server)
        .await;

    // Project not found: POST /graphql → GraphQL NOT_FOUND error
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errors": [{ "type": "NOT_FOUND", "message": "project not found" }]
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;
    let result = provider
        .add_pull_request_to_project("owner", "repo", 42, 999, "myorg")
        .await;

    assert!(
        matches!(result, Err(Error::FailedToUpdatePullRequest(_))),
        "Expected FailedToUpdatePullRequest for not-found project"
    );
}

// ---------------------------------------------------------------------------
// End-to-end: milestone propagation flow (GET issue → PATCH pull request)
// ---------------------------------------------------------------------------

/// Verifies the complete milestone propagation flow via the GitHub API:
/// 1. Fetch issue metadata (includes milestone number 5).
/// 2. Apply that milestone number to the pull request.
/// Both HTTP legs are asserted through independent WireMock mocks.
#[tokio::test]
async fn test_milestone_propagation_end_to_end_get_then_set() {
    let server = MockServer::start().await;

    // Step 1 mock: GET /repos/owner/repo/issues/42 → returns milestone 5.
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/issues/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_issue_json(42, true)))
        .mount(&server)
        .await;

    // Step 2 mock: PATCH /repos/owner/repo/pulls/10 → success.
    Mock::given(method("PATCH"))
        .and(path("/repos/owner/repo/pulls/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_pr_json(10)))
        .mount(&server)
        .await;

    let provider = make_provider(&server.uri()).await;

    // Step 1: retrieve milestone from issue.
    let metadata = provider
        .get_issue_metadata("owner", "repo", 42)
        .await
        .expect("get_issue_metadata must succeed")
        .expect("Expected Some(IssueMetadata)");

    let milestone_number = metadata
        .milestone
        .expect("Expected milestone on issue")
        .number;
    assert_eq!(milestone_number, 5, "Milestone number from issue must be 5");

    // Step 2: apply the milestone to the pull request.
    let set_result = provider
        .set_pull_request_milestone("owner", "repo", 10, Some(milestone_number))
        .await;

    assert!(
        set_result.is_ok(),
        "set_pull_request_milestone must succeed: {set_result:?}"
    );
}
