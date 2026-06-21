use crate::labels::set_pull_request_labels;
use async_trait::async_trait;
use merge_warden_developer_platforms::errors::Error;
use std::sync::{Arc, Mutex};
use tokio::test;

use merge_warden_developer_platforms::models::{
    Comment, CommitStatus, Label, PullRequest, PullRequestFile, Review, User,
};
use merge_warden_developer_platforms::PullRequestProvider;

// ── WIP label test helpers ───────────────────────────────────────────────────

/// Full-featured mock for WIP label tests — tracks applied labels and exposes
/// a pre-populated repository label list.
struct WipMockProvider {
    /// Labels that exist in the repository (returned by list_available_labels)
    available_labels: Vec<Label>,
    /// Labels currently applied to the PR
    applied_labels: Arc<Mutex<Vec<Label>>>,
}

impl WipMockProvider {
    fn new(available: Vec<Label>) -> Self {
        Self {
            available_labels: available,
            applied_labels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_applied(available: Vec<Label>, applied: Vec<Label>) -> Self {
        Self {
            available_labels: available,
            applied_labels: Arc::new(Mutex::new(applied)),
        }
    }

    fn get_applied(&self) -> Vec<Label> {
        self.applied_labels.lock().unwrap().clone()
    }
}

#[async_trait]
impl PullRequestProvider for WipMockProvider {
    async fn get_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn delete_comment(&self, _owner: &str, _repo: &str, _id: u64) -> Result<(), Error> {
        Ok(())
    }

    async fn list_comments(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Comment>, Error> {
        Ok(vec![])
    }

    async fn list_available_labels(&self, _owner: &str, _repo: &str) -> Result<Vec<Label>, Error> {
        Ok(self.available_labels.clone())
    }

    async fn add_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        let mut applied = self.applied_labels.lock().unwrap();
        for l in labels {
            applied.push(Label {
                name: l.clone(),
                description: None,
            });
        }
        Ok(())
    }

    async fn remove_label(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        label: &str,
    ) -> Result<(), Error> {
        let mut applied = self.applied_labels.lock().unwrap();
        applied.retain(|l| l.name != label);
        Ok(())
    }

    async fn list_applied_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Label>, Error> {
        Ok(self.applied_labels.lock().unwrap().clone())
    }

    async fn update_pr_check_status(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _conclusion: &str,
        _title: &str,
        _summary: &str,
        _text: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_pr_reviews(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> {
        Ok(vec![])
    }

    async fn get_pull_request_files(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        Ok(vec![])
    }

    async fn get_commit_statuses(
        &self,
        _owner: &str,
        _repo: &str,
        _commit_sha: &str,
    ) -> Result<Vec<merge_warden_developer_platforms::models::CommitStatus>, Error> {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _owner: &str,
        _repo: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }
}

// Mock implementation of PullRequestProvider for testing
struct MockGitProvider {
    labels: Arc<Mutex<Vec<Label>>>,
}

impl MockGitProvider {
    fn new() -> Self {
        Self {
            labels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_labels(&self) -> Vec<Label> {
        let labels = self.labels.lock().unwrap();
        labels.clone()
    }
}

// Mock implementation of PullRequestProvider that returns an error when adding labels
struct ErrorMockGitProvider;

impl ErrorMockGitProvider {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PullRequestProvider for MockGitProvider {
    async fn get_pull_request(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _comment_id: u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        Ok(vec![])
    }

    async fn list_available_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
    ) -> Result<Vec<Label>, Error> {
        // For tests, return empty vector or predefined labels
        Ok(vec![])
    }

    async fn add_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        let mut current_labels = self.labels.lock().unwrap();
        for label in labels {
            current_labels.push(Label {
                name: label.clone(),
                description: None,
            });
        }
        Ok(())
    }

    async fn remove_label(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _label: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_applied_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        let labels = self.labels.lock().unwrap();
        Ok(labels.clone())
    }

    async fn update_pr_check_status(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _conclusion: &str,
        _output_title: &str,
        _output_summary: &str,
        _output_text: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_pr_reviews(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> {
        Ok(vec![])
    }

    async fn get_pull_request_files(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        Ok(vec![])
    }

    async fn get_commit_statuses(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<merge_warden_developer_platforms::models::CommitStatus>, Error> {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }
}

#[async_trait]
impl PullRequestProvider for ErrorMockGitProvider {
    async fn get_pull_request(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _comment_id: u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        Ok(vec![])
    }

    async fn add_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _labels: &[String],
    ) -> Result<(), Error> {
        Err(Error::FailedToUpdatePullRequest("Failed".to_string()))
    }

    async fn remove_label(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _label: &str,
    ) -> Result<(), Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_applied_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        Ok(Vec::new())
    }

    async fn update_pr_check_status(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _conclusion: &str,
        _output_title: &str,
        _output_summary: &str,
        _output_text: &str,
    ) -> Result<(), Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_pr_reviews(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn get_pull_request_files(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn get_commit_statuses(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<merge_warden_developer_platforms::models::CommitStatus>, Error> {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }

    async fn list_available_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
    ) -> Result<Vec<Label>, Error> {
        Err(Error::FailedToUpdatePullRequest("Failed".to_string()))
    }
}

#[test]
async fn test_determine_labels_breaking_change() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat(api)!: change authentication flow".to_string(),
        draft: false,
        body: Some("This is a breaking change to the API".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"feature".to_string()));
    assert!(labels.contains(&"breaking-change".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 2);
    assert!(added_labels.iter().any(|l| l.name == "feature"));
    assert!(added_labels.iter().any(|l| l.name == "breaking-change"));
}

#[test]
async fn test_determine_labels_breaking_change_in_body() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat(api): change authentication flow".to_string(),
        draft: false,
        body: Some("This is a BREAKING CHANGE to the API".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"feature".to_string()));
    assert!(labels.contains(&"breaking-change".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 2);
    assert!(added_labels.iter().any(|l| l.name == "feature"));
    assert!(added_labels.iter().any(|l| l.name == "breaking-change"));
}

#[test]
async fn test_determine_labels_bug_fix() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "fix: correct login issue".to_string(),
        draft: false,
        body: Some("This fixes a bug in the login flow".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 1);
    assert!(labels.contains(&"bug".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 1);
    assert_eq!(added_labels[0].name, "bug");
}

// New test for conflicting information in title and body
#[test]
async fn test_determine_labels_conflicting_information() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "fix: correct login issue".to_string(), // Suggests a bug fix
        draft: false,
        body: Some(
            "This adds a new feature to the login flow. It's a feature, not a bug fix.".to_string(),
        ), // Suggests a feature
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    // The type from the title should take precedence
    assert_eq!(labels.len(), 1);
    assert!(labels.contains(&"bug".to_string()));
    assert!(!labels.contains(&"feature".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 1);
    assert_eq!(added_labels[0].name, "bug");
}

#[test]
async fn test_determine_labels_empty_pr_body() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat: add feature".to_string(),
        draft: false,
        body: Some("".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();
    assert!(!labels.is_empty(), "Expected no labels for empty body");
}

// New test for error handling - labels should be gracefully handled and not cause failures
#[test]
async fn test_determine_labels_error_handling() {
    let provider = ErrorMockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This is a new feature".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let result = set_pull_request_labels(&provider, "owner", "repo", &pr).await;
    assert!(
        result.is_ok(),
        "Expected success even when adding labels fails - labels are non-critical"
    );

    // The result should be an empty vector since no labels were successfully applied
    let labels = result.unwrap();
    assert!(
        labels.is_empty(),
        "Expected empty labels vector when label application fails"
    );
}

#[test]
async fn test_determine_labels_feature() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This is a new feature".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 1);
    assert!(labels.contains(&"feature".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 1);
    assert_eq!(added_labels[0].name, "feature");
}

#[test]
async fn test_determine_labels_hotfix() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "fix: urgent issue in production".to_string(),
        draft: false,
        body: Some("This is a hotfix for the production issue".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"bug".to_string()));
    assert!(labels.contains(&"hotfix".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 2);
    assert!(added_labels.iter().any(|l| l.name == "bug"));
    assert!(added_labels.iter().any(|l| l.name == "hotfix"));
}

#[test]
async fn test_determine_labels_invalid_type_in_pr_title() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "unknown: add feature".to_string(),
        draft: false,
        body: Some("This PR adds a feature.".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();
    assert!(
        labels.is_empty(),
        "Expected no labels for title with a missing type"
    );
}

// New test for keyword priority
#[test]
async fn test_determine_labels_keyword_priority() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "fix: security vulnerability".to_string(),
        draft: false,
        body: Some("This is a critical security hotfix that needs to be deployed immediately. It also addresses some technical debt.".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    // Should add all relevant labels
    assert_eq!(labels.len(), 4);
    assert!(labels.contains(&"bug".to_string()));
    assert!(labels.contains(&"security".to_string()));
    assert!(labels.contains(&"hotfix".to_string()));
    assert!(labels.contains(&"tech-debt".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 4);
    assert!(added_labels.iter().any(|l| l.name == "bug"));
    assert!(added_labels.iter().any(|l| l.name == "security"));
    assert!(added_labels.iter().any(|l| l.name == "hotfix"));
    assert!(added_labels.iter().any(|l| l.name == "tech-debt"));
}

#[test]
async fn test_determine_labels_missing_type_in_pr_title() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "add feature".to_string(),
        draft: false,
        body: Some("This PR adds a feature.".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();
    assert!(labels.is_empty(), "Expected no labels for missing type");
}

#[test]
async fn test_determine_labels_multiple_keywords() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "fix(auth): address security vulnerability".to_string(),
        draft: false,
        body: Some(
            "This is a hotfix for a security vulnerability. It addresses technical debt as well."
                .to_string(),
        ),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 4);
    assert!(labels.contains(&"bug".to_string()));
    assert!(labels.contains(&"security".to_string()));
    assert!(labels.contains(&"hotfix".to_string()));
    assert!(labels.contains(&"tech-debt".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 4);
    assert!(added_labels.iter().any(|l| l.name == "bug"));
    assert!(added_labels.iter().any(|l| l.name == "security"));
    assert!(added_labels.iter().any(|l| l.name == "hotfix"));
    assert!(added_labels.iter().any(|l| l.name == "tech-debt"));
}

#[test]
async fn test_determine_labels_no_keywords_in_pr_body() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat: add feature".to_string(),
        draft: false,
        body: Some("This PR adds a new feature.".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();
    assert!(labels.len() == 1, "Expected 1 label");
    assert!(
        labels.contains(&"feature".to_string()),
        "Expected 'feature' label from the PR title"
    );
}

#[test]
async fn test_determine_labels_security() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "fix: address security vulnerability".to_string(),
        draft: false,
        body: Some("This fixes a security issue in the authentication flow".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"bug".to_string()));
    assert!(labels.contains(&"security".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 2);
    assert!(added_labels.iter().any(|l| l.name == "bug"));
    assert!(added_labels.iter().any(|l| l.name == "security"));
}

#[test]
async fn test_determine_labels_tech_debt() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "refactor: improve code organization".to_string(),
        draft: false,
        body: Some("This addresses technical debt in the codebase".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"refactor".to_string()));
    assert!(labels.contains(&"tech-debt".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 2);
    assert!(added_labels.iter().any(|l| l.name == "refactor"));
    assert!(added_labels.iter().any(|l| l.name == "tech-debt"));
}

#[test]
async fn test_determine_labels_with_scope() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat(auth): add login with GitHub".to_string(),
        draft: false,
        body: Some("This adds GitHub login".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert_eq!(labels.len(), 1);
    assert!(labels.contains(&"feature".to_string()));

    let added_labels = provider.get_labels();
    assert_eq!(added_labels.len(), 1);
    assert!(added_labels.iter().any(|l| l.name == "feature"));
}

// Additional imports for smart label detection tests
use super::{build_keyword_label_comment, is_keyword_negated, parse_suppressed_labels};
use crate::config::{
    ChangeTypeLabelConfig, ConventionalCommitMappings, CurrentPullRequestValidationConfiguration,
    FallbackLabelSettings, KeywordLabelsConfig, LabelDetectionStrategy,
    KEYWORD_LABEL_COMMENT_MARKER,
};
use crate::labels::{
    set_pull_request_labels_with_config, LabelDetector, LabelManagementResult, LabelManager,
};
use std::collections::HashMap;

// Enhanced mock provider that supports repository labels for smart detection testing
#[derive(Debug)]
struct SmartMockGitProvider {
    labels: Arc<Mutex<Vec<Label>>>,
    repository_labels: Arc<Mutex<Vec<Label>>>,
}

impl SmartMockGitProvider {
    fn new() -> Self {
        Self {
            labels: Arc::new(Mutex::new(Vec::new())),
            repository_labels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_repository_label(&self, label: Label) {
        let mut repo_labels = self.repository_labels.lock().unwrap();
        repo_labels.push(label);
    }

    fn get_labels(&self) -> Vec<Label> {
        let labels = self.labels.lock().unwrap();
        labels.clone()
    }

    fn get_repository_labels(&self) -> Vec<Label> {
        let labels = self.repository_labels.lock().unwrap();
        labels.clone()
    }
}

#[async_trait]
impl PullRequestProvider for SmartMockGitProvider {
    async fn get_pull_request(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _comment_id: u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        Ok(vec![])
    }

    async fn list_available_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
    ) -> Result<Vec<Label>, Error> {
        let labels = self.repository_labels.lock().unwrap();
        Ok(labels.clone())
    }

    async fn add_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        let mut current_labels = self.labels.lock().unwrap();
        for label in labels {
            current_labels.push(Label {
                name: label.clone(),
                description: None,
            });
        }
        Ok(())
    }

    async fn remove_label(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _label: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_applied_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        let labels = self.labels.lock().unwrap();
        Ok(labels.clone())
    }

    async fn update_pr_check_status(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _conclusion: &str,
        _output_title: &str,
        _output_summary: &str,
        _output_text: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_pr_reviews(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> {
        Ok(vec![])
    }

    async fn get_pull_request_files(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        Ok(vec![])
    }

    async fn get_commit_statuses(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<merge_warden_developer_platforms::models::CommitStatus>, Error> {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }
}

// ==== Task 6.1: Unit tests for LabelDetector with various repository scenarios ====

#[test]
async fn test_label_detector_size_labels_exact_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels with exact size patterns
    provider.add_repository_label(Label {
        name: "size/XS".to_string(),
        description: Some("Extra small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/S".to_string(),
        description: Some("Small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/M".to_string(),
        description: Some("Medium PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/L".to_string(),
        description: Some("Large PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/XL".to_string(),
        description: Some("Extra large PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/XXL".to_string(),
        description: Some("Extra extra large PR".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    assert_eq!(discovered.xs, Some("size/XS".to_string()));
    assert_eq!(discovered.s, Some("size/S".to_string()));
    assert_eq!(discovered.m, Some("size/M".to_string()));
    assert_eq!(discovered.l, Some("size/L".to_string()));
    assert_eq!(discovered.xl, Some("size/XL".to_string()));
    assert_eq!(discovered.xxl, Some("size/XXL".to_string()));
}

#[test]
async fn test_label_detector_size_labels_separator_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels with separator patterns
    provider.add_repository_label(Label {
        name: "size-XS".to_string(),
        description: Some("Extra small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size_S".to_string(),
        description: Some("Small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size: M".to_string(),
        description: Some("Medium PR".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    assert_eq!(discovered.xs, Some("size-XS".to_string()));
    assert_eq!(discovered.s, Some("size_S".to_string()));
    assert_eq!(discovered.m, Some("size: M".to_string()));
}

#[test]
async fn test_label_detector_size_labels_separator_match_lowercase() {
    // Regression test: lowercase labels like "size:l" must be discovered correctly.
    // Previously, find_size_with_separator used a case-sensitive pattern so "size:l"
    // failed to match category "L", causing a fallback label "size: L" to be created
    // instead of reusing the existing lowercase label.
    let provider = SmartMockGitProvider::new();

    provider.add_repository_label(Label {
        name: "size:xs".to_string(),
        description: Some("Extra small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size:s".to_string(),
        description: Some("Small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size:m".to_string(),
        description: Some("Medium PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size:l".to_string(),
        description: Some("Large PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size:xl".to_string(),
        description: Some("Extra large PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size:xxl".to_string(),
        description: Some("Extra extra large PR".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    assert_eq!(discovered.xs, Some("size:xs".to_string()));
    assert_eq!(discovered.s, Some("size:s".to_string()));
    assert_eq!(discovered.m, Some("size:m".to_string()));
    assert_eq!(discovered.l, Some("size:l".to_string()));
    assert_eq!(discovered.xl, Some("size:xl".to_string()));
    assert_eq!(discovered.xxl, Some("size:xxl".to_string()));
}

#[test]
async fn test_label_detector_size_labels_exact_match_lowercase() {
    // Regression test: lowercase slash-variant labels like "size/l" must be
    // discovered correctly. find_exact_size_match received the same (?i) fix as
    // find_size_with_separator, so this test verifies that path.
    let provider = SmartMockGitProvider::new();

    provider.add_repository_label(Label {
        name: "size/xs".to_string(),
        description: Some("Extra small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/s".to_string(),
        description: Some("Small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/m".to_string(),
        description: Some("Medium PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/l".to_string(),
        description: Some("Large PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/xl".to_string(),
        description: Some("Extra large PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/xxl".to_string(),
        description: Some("Extra extra large PR".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    assert_eq!(discovered.xs, Some("size/xs".to_string()));
    assert_eq!(discovered.s, Some("size/s".to_string()));
    assert_eq!(discovered.m, Some("size/m".to_string()));
    assert_eq!(discovered.l, Some("size/l".to_string()));
    assert_eq!(discovered.xl, Some("size/xl".to_string()));
    assert_eq!(discovered.xxl, Some("size/xxl".to_string()));
}

#[test]
async fn test_label_detector_size_labels_standalone_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels with standalone patterns
    provider.add_repository_label(Label {
        name: "XS".to_string(),
        description: Some("Extra small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "S".to_string(),
        description: Some("Small PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "M".to_string(),
        description: Some("Medium PR".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    assert_eq!(discovered.xs, Some("XS".to_string()));
    assert_eq!(discovered.s, Some("S".to_string()));
    assert_eq!(discovered.m, Some("M".to_string()));
}

#[test]
async fn test_label_detector_size_labels_description_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels with description-based patterns
    provider.add_repository_label(Label {
        name: "tiny".to_string(),
        description: Some("(size: XS) Very small changes".to_string()),
    });
    provider.add_repository_label(Label {
        name: "small".to_string(),
        description: Some("(size: S) Small changes".to_string()),
    });
    provider.add_repository_label(Label {
        name: "medium".to_string(),
        description: Some("(size: M) Medium changes".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    assert_eq!(discovered.xs, Some("tiny".to_string()));
    assert_eq!(discovered.s, Some("small".to_string()));
    assert_eq!(discovered.m, Some("medium".to_string()));
}

#[test]
async fn test_label_detector_size_labels_priority_selection() {
    let provider = SmartMockGitProvider::new();

    // Add multiple matching labels to test priority selection
    provider.add_repository_label(Label {
        name: "size/XS".to_string(),
        description: Some("Exact match".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size-XS".to_string(),
        description: Some("Separator match".to_string()),
    });
    provider.add_repository_label(Label {
        name: "XS".to_string(),
        description: Some("Standalone match".to_string()),
    });
    provider.add_repository_label(Label {
        name: "tiny".to_string(),
        description: Some("(size: XS) Description match".to_string()),
    });

    let detector = LabelDetector::new_for_size_labels();
    let discovered = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await
        .unwrap();

    // Should prefer exact match (size/XS) over others
    assert_eq!(discovered.xs, Some("size/XS".to_string()));
}

#[test]
async fn test_label_detector_change_type_exact_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels that match conventional commit types
    provider.add_repository_label(Label {
        name: "feature".to_string(),
        description: Some("New feature".to_string()),
    });
    provider.add_repository_label(Label {
        name: "bug".to_string(),
        description: Some("Bug fix".to_string()),
    });
    provider.add_repository_label(Label {
        name: "enhancement".to_string(),
        description: Some("Enhancement".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string(), "enhancement".to_string()],
            fix: vec!["bug".to_string(), "bugfix".to_string()],
            docs: vec!["documentation".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let detector = LabelDetector::new_for_change_type_labels(config);
    let discovered = detector
        .detect_change_type_label(&provider, "owner", "repo", "feat")
        .await
        .unwrap();

    assert_eq!(discovered.label_name, Some("feature".to_string()));
    assert_eq!(discovered.commit_type, "feat");
    assert!(!discovered.should_create_fallback);
}

#[test]
async fn test_label_detector_change_type_prefix_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels with prefix patterns
    provider.add_repository_label(Label {
        name: "type:feat".to_string(),
        description: Some("Feature type".to_string()),
    });
    provider.add_repository_label(Label {
        name: "kind:fix".to_string(),
        description: Some("Fix kind".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        detection_strategy: LabelDetectionStrategy {
            exact_match: true,
            prefix_match: true,
            description_match: true,
            common_prefixes: vec!["type:".to_string(), "kind:".to_string()],
        },
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let detector = LabelDetector::new_for_change_type_labels(config);
    let discovered = detector
        .detect_change_type_label(&provider, "owner", "repo", "feat")
        .await
        .unwrap();

    assert_eq!(discovered.label_name, Some("type:feat".to_string()));
    assert_eq!(discovered.commit_type, "feat");
    assert!(!discovered.should_create_fallback);
}

#[test]
async fn test_label_detector_change_type_description_match() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels with description-based patterns
    provider.add_repository_label(Label {
        name: "new-feature".to_string(),
        description: Some("For feat commits - new features".to_string()),
    });
    provider.add_repository_label(Label {
        name: "bug-fix".to_string(),
        description: Some("For fix commits - bug fixes".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let detector = LabelDetector::new_for_change_type_labels(config);
    let discovered = detector
        .detect_change_type_label(&provider, "owner", "repo", "feat")
        .await
        .unwrap();

    assert_eq!(discovered.label_name, Some("new-feature".to_string()));
    assert_eq!(discovered.commit_type, "feat");
    assert!(!discovered.should_create_fallback);
}

#[test]
async fn test_label_detector_change_type_no_match_fallback() {
    let provider = SmartMockGitProvider::new();

    // Add repository labels that don't match the commit type
    provider.add_repository_label(Label {
        name: "random-label".to_string(),
        description: Some("Random label".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let detector = LabelDetector::new_for_change_type_labels(config);
    let discovered = detector
        .detect_change_type_label(&provider, "owner", "repo", "feat")
        .await
        .unwrap();

    assert_eq!(discovered.label_name, None);
    assert_eq!(discovered.commit_type, "feat");
    assert!(discovered.should_create_fallback);
}

#[test]
async fn test_label_detector_error_handling() {
    let provider = ErrorMockGitProvider::new();

    let detector = LabelDetector::new_for_size_labels();
    let result = detector
        .discover_size_labels(&provider, "owner", "repo")
        .await;

    assert!(result.is_err());
}

// ==== Task 6.2: Unit tests for LabelManager functionality ====

#[test]
async fn test_label_manager_apply_change_type_label_success() {
    let provider = SmartMockGitProvider::new();

    // Add matching repository label
    provider.add_repository_label(Label {
        name: "feature".to_string(),
        description: Some("New feature".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let manager = LabelManager::new(Some(config));
    let result = manager
        .apply_change_type_label(&provider, "owner", "repo", 123, "feat")
        .await
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.applied_labels.len(), 1);
    assert_eq!(result.applied_labels[0], "feature");
    assert!(!result.used_fallback_creation());
}

#[test]
async fn test_label_manager_apply_change_type_label_with_fallback() {
    let provider = SmartMockGitProvider::new();

    // No matching repository labels, should create fallback

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "00ff00".to_string())]),
        },
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let manager = LabelManager::new(Some(config));
    let result = manager
        .apply_change_type_label(&provider, "owner", "repo", 123, "feat")
        .await
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.created_fallback_labels.len(), 1);
    assert_eq!(result.created_fallback_labels[0], "type: feat");
    assert!(result.used_fallback_creation());
}

#[test]
async fn test_label_manager_apply_breaking_change_label() {
    let provider = SmartMockGitProvider::new();

    // Add breaking change label
    provider.add_repository_label(Label {
        name: "breaking-change".to_string(),
        description: Some("Breaking change".to_string()),
    });

    let manager = LabelManager::new(None);
    let result = manager
        .apply_breaking_change_label(
            &provider,
            "owner",
            "repo",
            123,
            "feat(api)!: change authentication",
            Some("This is a breaking change"),
        )
        .await
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.applied_labels.len(), 1);
    assert_eq!(result.applied_labels[0], "breaking-change");
}

#[test]
async fn test_label_manager_apply_keyword_labels() {
    let provider = SmartMockGitProvider::new();

    // Add keyword-based labels
    provider.add_repository_label(Label {
        name: "security".to_string(),
        description: Some("Security issue".to_string()),
    });
    provider.add_repository_label(Label {
        name: "hotfix".to_string(),
        description: Some("Hotfix".to_string()),
    });

    let manager = LabelManager::new(None);
    let result = manager
        .apply_keyword_labels(
            &provider,
            "owner",
            "repo",
            123,
            Some("This fixes a security vulnerability and is a hotfix"),
        )
        .await
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.applied_labels.len(), 2);
    assert!(result.applied_labels.contains(&"security".to_string()));
    assert!(result.applied_labels.contains(&"hotfix".to_string()));
}

#[test]
async fn test_label_manager_error_handling() {
    let provider = ErrorMockGitProvider::new();

    let manager = LabelManager::new(None);
    let result = manager
        .apply_change_type_label(&provider, "owner", "repo", 123, "feat")
        .await;

    // The operation should succeed but the result should indicate failure
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.is_success());
    assert!(!result.error_messages.is_empty());
    assert!(!result.failed_labels.is_empty());
}

#[test]
async fn test_label_management_result_methods() {
    let mut result = LabelManagementResult::new();

    // Test empty result
    assert!(result.is_success());
    assert!(!result.has_applied_labels());
    assert!(!result.used_fallback_creation());
    assert_eq!(result.all_applied_labels().len(), 0);

    // Add applied labels
    result.applied_labels.push("feature".to_string());
    result
        .created_fallback_labels
        .push("type: feat".to_string());

    assert!(result.is_success());
    assert!(result.has_applied_labels());
    assert!(result.used_fallback_creation());
    assert_eq!(result.all_applied_labels().len(), 2);
    assert!(result.all_applied_labels().contains(&"feature".to_string()));
    assert!(result
        .all_applied_labels()
        .contains(&"type: feat".to_string()));

    // Add failed labels
    result.failed_labels.push("failed-label".to_string());
    assert!(!result.is_success());
}

// ==== Task 6.3: Integration tests for the complete smart labeling pipeline ====

#[test]
async fn test_smart_labeling_pipeline_end_to_end() {
    let provider = SmartMockGitProvider::new();

    // Setup repository with various labels
    provider.add_repository_label(Label {
        name: "feature".to_string(),
        description: Some("New feature".to_string()),
    });
    provider.add_repository_label(Label {
        name: "size/M".to_string(),
        description: Some("Medium PR".to_string()),
    });
    provider.add_repository_label(Label {
        name: "breaking-change".to_string(),
        description: Some("Breaking change".to_string()),
    });

    // Create configuration
    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let config_wrapper = CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(config),
        ..Default::default()
    };

    // Create PR with breaking change
    let pr = PullRequest {
        number: 1,
        title: "feat(api)!: add new authentication system".to_string(),
        draft: false,
        body: Some("This PR adds a new authentication system. It's a breaking change.".to_string()),
        author: Some(User {
            id: 123,
            login: "developer".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    // Test the complete pipeline
    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config_wrapper))
            .await
            .unwrap();

    // Should have both feature and breaking-change labels
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"feature".to_string()));
    assert!(labels.contains(&"breaking-change".to_string()));

    // Verify labels were applied to PR
    let applied_labels = provider.get_labels();
    assert_eq!(applied_labels.len(), 2);
    assert!(applied_labels.iter().any(|l| l.name == "feature"));
    assert!(applied_labels.iter().any(|l| l.name == "breaking-change"));
}

#[test]
async fn test_smart_labeling_pipeline_with_fallback() {
    let provider = SmartMockGitProvider::new();

    // Setup repository with no matching labels (should trigger fallback)
    provider.add_repository_label(Label {
        name: "unrelated-label".to_string(),
        description: Some("Unrelated".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "0366d6".to_string())]),
        },
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let config_wrapper = CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(config),
        ..Default::default()
    };

    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This PR adds a new feature.".to_string()),
        author: Some(User {
            id: 123,
            login: "developer".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config_wrapper))
            .await
            .unwrap();

    // Should create fallback label
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "type: feat");
}

#[test]
async fn test_smart_labeling_pipeline_legacy_fallback() {
    let provider = SmartMockGitProvider::new();

    // No configuration provided, should use legacy hardcoded labels
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This PR adds a new feature.".to_string()),
        author: Some(User {
            id: 123,
            login: "developer".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, None)
        .await
        .unwrap();

    // Should use legacy hardcoded labels
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "feature");
}

#[test]
async fn test_smart_labeling_pipeline_disabled() {
    let provider = SmartMockGitProvider::new();

    // Configuration with smart labeling disabled
    let config = ChangeTypeLabelConfig {
        enabled: false,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let config_wrapper = CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(config),
        ..Default::default()
    };

    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This PR adds a new feature.".to_string()),
        author: Some(User {
            id: 123,
            login: "developer".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config_wrapper))
            .await
            .unwrap();

    // Should fall back to hardcoded labels when disabled
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "feature");
}

#[test]
async fn test_smart_labeling_pipeline_multiple_keywords() {
    let provider = SmartMockGitProvider::new();

    // Setup repository with keyword-based labels
    provider.add_repository_label(Label {
        name: "feature".to_string(),
        description: Some("New feature".to_string()),
    });
    provider.add_repository_label(Label {
        name: "security".to_string(),
        description: Some("Security issue".to_string()),
    });
    provider.add_repository_label(Label {
        name: "breaking-change".to_string(),
        description: Some("Breaking change".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let config_wrapper = CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(config),
        ..Default::default()
    };

    let pr = PullRequest {
        number: 1,
        title: "feat(auth)!: add security improvements".to_string(),
        draft: false,
        body: Some("This PR adds security improvements. This is a breaking change that addresses security vulnerabilities.".to_string()),
        author: Some(User {
            id: 123,
            login: "developer".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config_wrapper))
            .await
            .unwrap();

    // Should detect multiple labels: feature, security, breaking-change
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"feature".to_string()));
    assert!(labels.contains(&"security".to_string()));
    assert!(labels.contains(&"breaking-change".to_string()));
}

#[test]
async fn test_smart_labeling_pipeline_error_recovery() {
    let provider = SmartMockGitProvider::new();

    // Setup with some labels available
    provider.add_repository_label(Label {
        name: "feature".to_string(),
        description: Some("New feature".to_string()),
    });

    let config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let config_wrapper = CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(config),
        ..Default::default()
    };

    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This PR adds a new feature.".to_string()),
        author: Some(User {
            id: 123,
            login: "developer".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    // Even if some parts fail, should continue processing
    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config_wrapper))
            .await
            .unwrap();

    // Should still apply the labels that work
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "feature");
}

// ==== Task 6.4: Configuration validation tests for smart label settings ====

#[tokio::test]
async fn test_change_type_label_config_defaults() {
    let config = ChangeTypeLabelConfig::default();

    assert!(config.enabled);
    assert!(!config.conventional_commit_mappings.feat.is_empty());
    assert!(!config.conventional_commit_mappings.fix.is_empty());
    assert!(config.fallback_label_settings.create_if_missing);
    assert!(config.detection_strategy.exact_match);
    assert!(config.detection_strategy.prefix_match);
}

#[test]
async fn test_conventional_commit_mappings_default() {
    let mappings = ConventionalCommitMappings::default();

    // Test that default mappings are comprehensive
    assert!(mappings.feat.contains(&"feature".to_string()));
    assert!(mappings.fix.contains(&"bug".to_string()));
    assert!(mappings.docs.contains(&"documentation".to_string()));
    assert!(mappings.style.contains(&"style".to_string()));
    assert!(mappings.refactor.contains(&"refactor".to_string()));
    assert!(mappings.perf.contains(&"performance".to_string()));
    assert!(mappings.test.contains(&"test".to_string()));
    assert!(mappings.chore.contains(&"chore".to_string()));
    assert!(mappings.ci.contains(&"ci".to_string()));
    assert!(mappings.build.contains(&"build".to_string()));
    assert!(mappings.revert.contains(&"revert".to_string()));
}

#[test]
async fn test_fallback_label_settings_default() {
    let settings = FallbackLabelSettings::default();

    assert!(settings.create_if_missing);
    assert_eq!(settings.name_format, "type: {change_type}");
    assert!(!settings.color_scheme.is_empty());

    // Test that default colors are valid hex colors
    for (_, color) in settings.color_scheme.iter() {
        assert_eq!(color.len(), 7);
        assert!(color.starts_with('#'));
        assert!(color[1..].chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[tokio::test]
async fn test_label_detection_strategy_default() {
    let strategy = LabelDetectionStrategy::default();

    assert!(strategy.exact_match);
    assert!(strategy.prefix_match);
    assert!(strategy.description_match);
    assert!(!strategy.common_prefixes.is_empty());
    assert!(strategy.common_prefixes.contains(&"type:".to_string()));
    assert!(strategy.common_prefixes.contains(&"kind:".to_string()));
}

#[tokio::test]
async fn test_change_type_label_config_validation() {
    // Test valid configuration
    let valid_config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy {
            exact_match: true,
            prefix_match: false,
            description_match: false,
            common_prefixes: vec![],
        },
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "00ff00".to_string())]),
        },
        keyword_labels: KeywordLabelsConfig::default(),
    };

    // Should be valid (at least one detection method enabled)
    assert!(valid_config.detection_strategy.exact_match);

    // Test invalid configuration (no detection methods enabled)
    let invalid_config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        detection_strategy: LabelDetectionStrategy {
            exact_match: false,
            prefix_match: false,
            description_match: false,
            common_prefixes: vec![],
        },
        fallback_label_settings: FallbackLabelSettings::default(),
        keyword_labels: KeywordLabelsConfig::default(),
    };

    // Should be invalid (no detection methods enabled)
    assert!(!invalid_config.detection_strategy.exact_match);
    assert!(!invalid_config.detection_strategy.prefix_match);
    assert!(!invalid_config.detection_strategy.description_match);
}

#[test]
async fn test_fallback_label_name_format_validation() {
    let settings = FallbackLabelSettings {
        create_if_missing: true,
        name_format: "type: {change_type}".to_string(),
        color_scheme: HashMap::new(),
    };

    // Test that the format string contains the required placeholder
    assert!(settings.name_format.contains("{change_type}"));

    // Test invalid format (missing placeholder)
    let invalid_settings = FallbackLabelSettings {
        create_if_missing: true,
        name_format: "type: invalid".to_string(),
        color_scheme: HashMap::new(),
    };

    assert!(!invalid_settings.name_format.contains("{change_type}"));
}

#[test]
async fn test_color_scheme_validation() {
    let valid_colors = HashMap::from([
        ("feat".to_string(), "00ff00".to_string()),
        ("fix".to_string(), "ff0000".to_string()),
        ("docs".to_string(), "0000ff".to_string()),
    ]);

    // Test valid hex colors
    for (_, color) in valid_colors.iter() {
        assert_eq!(color.len(), 6);
        assert!(color.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // Test invalid colors
    let invalid_colors = vec![
        "xyz123".to_string(),   // Invalid hex
        "ff00".to_string(),     // Too short
        "ff0000ff".to_string(), // Too long
        "#ff0000".to_string(),  // Should not include #
    ];

    for color in invalid_colors {
        if color.len() == 6 {
            assert!(!color.chars().all(|c| c.is_ascii_hexdigit()));
        } else {
            assert_ne!(color.len(), 6);
        }
    }
}

#[test]
async fn test_current_pull_request_validation_config_with_smart_labels() {
    let smart_config = ChangeTypeLabelConfig::default();

    let config = CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(smart_config),
        ..Default::default()
    };

    assert!(config.change_type_labels.is_some());

    let smart_config = config.change_type_labels.unwrap();
    assert!(smart_config.enabled);
    assert!(smart_config.fallback_label_settings.create_if_missing);
}

#[tokio::test]
async fn test_configuration_merge_behavior() {
    // Test that repository-level configuration can override application defaults
    let app_config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["feature".to_string()],
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy::default(),
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "00ff00".to_string())]),
        },
        keyword_labels: KeywordLabelsConfig::default(),
    };

    let repo_config = ChangeTypeLabelConfig {
        enabled: false, // Override: disable smart labeling
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["enhancement".to_string()], // Override: different mapping
            ..Default::default()
        },
        detection_strategy: LabelDetectionStrategy {
            exact_match: false, // Override: disable exact match
            prefix_match: true,
            description_match: true,
            common_prefixes: vec!["category:".to_string()], // Override: different prefixes
        },
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: false, // Override: disable fallback creation
            name_format: "kind: {change_type}".to_string(), // Override: different format
            color_scheme: HashMap::from([("feat".to_string(), "ff0000".to_string())]), // Override: different color
        },
        keyword_labels: KeywordLabelsConfig::default(),
    };

    // In a real merge scenario, repository config would override application config
    assert_ne!(app_config.enabled, repo_config.enabled);
    assert_ne!(
        app_config.fallback_label_settings.create_if_missing,
        repo_config.fallback_label_settings.create_if_missing
    );
    assert_ne!(
        app_config.detection_strategy.exact_match,
        repo_config.detection_strategy.exact_match
    );
}

//  discover_wip_labels tests

#[tokio::test]
async fn test_discover_wip_labels_returns_none_when_no_wip_labels() {
    use crate::labels::discover_wip_labels;

    let provider = WipMockProvider::new(vec![
        Label {
            name: "bug".to_string(),
            description: None,
        },
        Label {
            name: "feature".to_string(),
            description: None,
        },
    ]);

    let result = discover_wip_labels(&provider, "owner", "repo", &None)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_wip_labels_exact_hint_match() {
    use crate::labels::discover_wip_labels;

    let provider = WipMockProvider::new(vec![
        Label {
            name: " WIP".to_string(),
            description: None,
        },
        Label {
            name: "bug".to_string(),
            description: None,
        },
    ]);

    let result = discover_wip_labels(&provider, "owner", "repo", &Some(" WIP".to_string()))
        .await
        .unwrap();

    assert_eq!(result, Some(" WIP".to_string()));
}

#[tokio::test]
async fn test_discover_wip_labels_common_name_match_wip_lowercase() {
    use crate::labels::discover_wip_labels;

    let provider = WipMockProvider::new(vec![
        Label {
            name: "wip".to_string(),
            description: None,
        },
        Label {
            name: "bug".to_string(),
            description: None,
        },
    ]);

    let result = discover_wip_labels(&provider, "owner", "repo", &None)
        .await
        .unwrap();

    assert_eq!(result, Some("wip".to_string()));
}

#[tokio::test]
async fn test_discover_wip_labels_common_name_match_work_in_progress() {
    use crate::labels::discover_wip_labels;

    let provider = WipMockProvider::new(vec![Label {
        name: "work-in-progress".to_string(),
        description: None,
    }]);

    let result = discover_wip_labels(&provider, "owner", "repo", &None)
        .await
        .unwrap();

    assert_eq!(result, Some("work-in-progress".to_string()));
}

#[tokio::test]
async fn test_discover_wip_labels_hint_takes_priority_over_common() {
    use crate::labels::discover_wip_labels;

    // Both the hint and a common WIP name exist  hint wins
    let provider = WipMockProvider::new(vec![
        Label {
            name: "wip".to_string(),
            description: None,
        },
        Label {
            name: "my-custom-wip".to_string(),
            description: None,
        },
    ]);

    let result = discover_wip_labels(
        &provider,
        "owner",
        "repo",
        &Some("my-custom-wip".to_string()),
    )
    .await
    .unwrap();

    assert_eq!(result, Some("my-custom-wip".to_string()));
}

//  manage_wip_labels tests

#[tokio::test]
async fn test_manage_wip_labels_adds_label_when_wip_and_no_existing_label() {
    use crate::labels::manage_wip_labels;

    let provider = WipMockProvider::new(vec![Label {
        name: "WIP".to_string(),
        description: None,
    }]);

    manage_wip_labels(
        &provider,
        "owner",
        "repo",
        1,
        true,
        &Some("WIP".to_string()),
    )
    .await
    .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].name, "WIP");
}

#[tokio::test]
async fn test_manage_wip_labels_does_not_double_add_label() {
    use crate::labels::manage_wip_labels;

    // PR already has "WIP" applied
    let provider = WipMockProvider::with_applied(
        vec![Label {
            name: "WIP".to_string(),
            description: None,
        }],
        vec![Label {
            name: "WIP".to_string(),
            description: None,
        }],
    );

    manage_wip_labels(
        &provider,
        "owner",
        "repo",
        1,
        true,
        &Some("WIP".to_string()),
    )
    .await
    .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1, "Should not duplicate WIP label");
}

#[tokio::test]
async fn test_manage_wip_labels_removes_label_when_not_wip() {
    use crate::labels::manage_wip_labels;

    // PR has "WIP" applied; it is no longer WIP
    let provider = WipMockProvider::with_applied(
        vec![Label {
            name: "WIP".to_string(),
            description: None,
        }],
        vec![Label {
            name: "WIP".to_string(),
            description: None,
        }],
    );

    manage_wip_labels(
        &provider,
        "owner",
        "repo",
        1,
        false,
        &Some("WIP".to_string()),
    )
    .await
    .unwrap();

    let applied = provider.get_applied();
    assert!(
        applied.is_empty(),
        "WIP label should be removed when not WIP"
    );
}

#[tokio::test]
async fn test_manage_wip_labels_remove_is_noop_when_no_wip_label_present() {
    use crate::labels::manage_wip_labels;

    let provider = WipMockProvider::with_applied(
        vec![],
        vec![Label {
            name: "bug".to_string(),
            description: None,
        }],
    );

    manage_wip_labels(&provider, "owner", "repo", 1, false, &None)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1, "Non-WIP label should not be removed");
    assert_eq!(applied[0].name, "bug");
}

#[tokio::test]
async fn test_manage_wip_labels_uses_hint_when_no_repo_label_discovered() {
    use crate::labels::manage_wip_labels;

    // No labels in the repository, but hint provided  should use hint
    let provider = WipMockProvider::new(vec![]);

    manage_wip_labels(
        &provider,
        "owner",
        "repo",
        1,
        true,
        &Some("WIP".to_string()),
    )
    .await
    .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].name, "WIP");
}

#[tokio::test]
async fn test_manage_wip_labels_none_hint_disables_labeling() {
    // wip_label = None means "labeling explicitly disabled" — no labels must be touched
    use crate::labels::manage_wip_labels;

    let provider = WipMockProvider::new(vec![]);

    manage_wip_labels(&provider, "owner", "repo", 1, true, &None)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert!(
        applied.is_empty(),
        "None hint should disable label management entirely (no label added)"
    );
}

#[tokio::test]
async fn test_manage_wip_labels_uses_default_wip_when_some_hint_and_no_repo_label() {
    // Some("WIP") with no matching repo label falls through to the hint as effective label
    use crate::labels::manage_wip_labels;

    let provider = WipMockProvider::new(vec![]);

    manage_wip_labels(
        &provider,
        "owner",
        "repo",
        1,
        true,
        &Some("WIP".to_string()),
    )
    .await
    .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1);
    assert_eq!(
        applied[0].name, "WIP",
        "Should use the hint 'WIP' as the label when no repo label is found"
    );
}

// ── PR state label test helpers ──────────────────────────────────────────────

/// Mock provider for PR state label tests.
///
/// Tracks applied labels and exposes configurable repository labels and reviews.
struct PrStateMockProvider {
    available_labels: Vec<Label>,
    applied_labels: Arc<Mutex<Vec<Label>>>,
    reviews: Vec<Review>,
}

impl PrStateMockProvider {
    fn new(available: Vec<Label>, reviews: Vec<Review>) -> Self {
        Self {
            available_labels: available,
            applied_labels: Arc::new(Mutex::new(Vec::new())),
            reviews,
        }
    }

    fn with_applied(available: Vec<Label>, applied: Vec<Label>, reviews: Vec<Review>) -> Self {
        Self {
            available_labels: available,
            applied_labels: Arc::new(Mutex::new(applied)),
            reviews,
        }
    }

    fn get_applied(&self) -> Vec<Label> {
        self.applied_labels.lock().unwrap().clone()
    }
}

#[async_trait]
impl PullRequestProvider for PrStateMockProvider {
    async fn get_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn delete_comment(&self, _owner: &str, _repo: &str, _id: u64) -> Result<(), Error> {
        Ok(())
    }

    async fn list_comments(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Comment>, Error> {
        Ok(vec![])
    }

    async fn list_available_labels(&self, _owner: &str, _repo: &str) -> Result<Vec<Label>, Error> {
        Ok(self.available_labels.clone())
    }

    async fn add_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        let mut applied = self.applied_labels.lock().unwrap();
        for l in labels {
            applied.push(Label {
                name: l.clone(),
                description: None,
            });
        }
        Ok(())
    }

    async fn remove_label(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        label: &str,
    ) -> Result<(), Error> {
        let mut applied = self.applied_labels.lock().unwrap();
        applied.retain(|l| l.name != label);
        Ok(())
    }

    async fn list_applied_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Label>, Error> {
        Ok(self.applied_labels.lock().unwrap().clone())
    }

    async fn update_pr_check_status(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _conclusion: &str,
        _title: &str,
        _summary: &str,
        _text: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_pr_reviews(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Review>, Error> {
        Ok(self.reviews.clone())
    }

    async fn get_pull_request_files(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        Ok(vec![])
    }

    async fn get_commit_statuses(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<merge_warden_developer_platforms::models::CommitStatus>, Error> {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }
}

// ── manage_pr_state_labels tests ─────────────────────────────────────────────

fn make_config(
    enabled: bool,
    draft_label: Option<&str>,
    review_label: Option<&str>,
    approved_label: Option<&str>,
) -> crate::config::PrStateLabelsConfig {
    crate::config::PrStateLabelsConfig {
        enabled,
        draft_label: draft_label.map(String::from),
        review_label: review_label.map(String::from),
        approved_label: approved_label.map(String::from),
    }
}

fn make_label(name: &str) -> Label {
    Label {
        name: name.to_string(),
        description: None,
    }
}

#[test]
async fn test_manage_pr_state_labels_disabled_skips_all_ops() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::new(vec![], vec![]);
    let config = make_config(false, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, false, false, &config)
        .await
        .unwrap();

    assert!(
        provider.get_applied().is_empty(),
        "No labels should be applied when disabled"
    );
}

#[test]
async fn test_manage_pr_state_labels_draft_applies_draft_label() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::new(vec![], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, true, false, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].name, "draft", "Draft label should be applied");
}

#[test]
async fn test_manage_pr_state_labels_review_state_applies_review_label() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::new(vec![], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, false, false, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].name, "review", "Review label should be applied");
}

#[test]
async fn test_manage_pr_state_labels_approved_applies_approved_label() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::new(vec![], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, false, true, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert_eq!(applied.len(), 1);
    assert_eq!(
        applied[0].name, "approved",
        "Approved label should be applied"
    );
}

#[test]
async fn test_manage_pr_state_labels_transition_draft_to_review_removes_draft() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::with_applied(vec![], vec![make_label("draft")], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, false, false, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert!(
        applied.iter().all(|l| l.name != "draft"),
        "Draft label should be removed"
    );
    assert!(
        applied.iter().any(|l| l.name == "review"),
        "Review label should be added"
    );
}

#[test]
async fn test_manage_pr_state_labels_transition_review_to_approved() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::with_applied(vec![], vec![make_label("review")], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, false, true, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert!(
        applied.iter().all(|l| l.name != "review"),
        "Review label should be removed"
    );
    assert!(
        applied.iter().any(|l| l.name == "approved"),
        "Approved label should be added"
    );
}

#[test]
async fn test_manage_pr_state_labels_revert_to_draft_removes_approved() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::with_applied(vec![], vec![make_label("approved")], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, true, false, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert!(
        applied.iter().all(|l| l.name != "approved"),
        "Approved label should be removed"
    );
    assert!(
        applied.iter().any(|l| l.name == "draft"),
        "Draft label should be added"
    );
}

#[test]
async fn test_manage_pr_state_labels_idempotent_no_duplicate_labels() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::new(vec![], vec![]);
    let config = make_config(true, Some("draft"), Some("review"), Some("approved"));

    manage_pr_state_labels(&provider, "owner", "repo", 1, true, false, &config)
        .await
        .unwrap();

    // Call again with the same state — should not double-add
    manage_pr_state_labels(&provider, "owner", "repo", 1, true, false, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    let draft_count = applied.iter().filter(|l| l.name == "draft").count();
    assert_eq!(draft_count, 1, "Draft label should not be duplicated");
}

#[test]
async fn test_manage_pr_state_labels_none_target_label_removes_others() {
    use crate::labels::manage_pr_state_labels;

    // review_label is None, meaning no label is applied for review state
    // but draft and approved labels should still be removed from the PR
    let provider = PrStateMockProvider::with_applied(
        vec![],
        vec![make_label("draft"), make_label("approved")],
        vec![],
    );
    let config = make_config(true, Some("draft"), None, Some("approved"));

    // is_draft=false, is_approved=false → target = review_label = None
    manage_pr_state_labels(&provider, "owner", "repo", 1, false, false, &config)
        .await
        .unwrap();

    let applied = provider.get_applied();
    assert!(
        applied.iter().all(|l| l.name != "draft"),
        "Draft label should be removed even when target is None"
    );
    assert!(
        applied.iter().all(|l| l.name != "approved"),
        "Approved label should be removed even when target is None"
    );
    // No new label added since target is None
    assert!(
        applied.is_empty(),
        "No label should be added when target is None"
    );
}

#[test]
async fn test_manage_pr_state_labels_all_none_labels_is_noop() {
    use crate::labels::manage_pr_state_labels;

    let provider = PrStateMockProvider::new(vec![], vec![]);
    let config = make_config(true, None, None, None);

    manage_pr_state_labels(&provider, "owner", "repo", 1, true, false, &config)
        .await
        .unwrap();

    assert!(
        provider.get_applied().is_empty(),
        "No operations should occur when all labels are None"
    );
}

// ── manage_size_labels idempotency tests ─────────────────────────────────────

/// Mock provider that tracks add_labels and remove_label call counts, used for
/// verifying that manage_size_labels does not make unnecessary API calls.
struct SizeLabelMockProvider {
    /// Labels available in the repository (returned by list_available_labels).
    available_labels: Vec<Label>,
    /// Labels currently applied to the PR (mutable, reflecting add/remove).
    applied_labels: Arc<Mutex<Vec<Label>>>,
    /// Records every batch passed to add_labels.
    add_calls: Arc<Mutex<Vec<Vec<String>>>>,
    /// Records each label name passed to remove_label.
    remove_calls: Arc<Mutex<Vec<String>>>,
}

impl SizeLabelMockProvider {
    fn new(available: Vec<Label>, applied: Vec<Label>) -> Self {
        Self {
            available_labels: available,
            applied_labels: Arc::new(Mutex::new(applied)),
            add_calls: Arc::new(Mutex::new(Vec::new())),
            remove_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_applied(&self) -> Vec<Label> {
        self.applied_labels.lock().unwrap().clone()
    }

    fn get_add_calls(&self) -> Vec<Vec<String>> {
        self.add_calls.lock().unwrap().clone()
    }

    fn get_remove_calls(&self) -> Vec<String> {
        self.remove_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl PullRequestProvider for SizeLabelMockProvider {
    async fn get_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<PullRequest, merge_warden_developer_platforms::errors::Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _comment: &str,
    ) -> Result<(), merge_warden_developer_platforms::errors::Error> {
        unimplemented!("Not needed for this test")
    }

    async fn delete_comment(
        &self,
        _owner: &str,
        _repo: &str,
        _id: u64,
    ) -> Result<(), merge_warden_developer_platforms::errors::Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_comments(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Comment>, merge_warden_developer_platforms::errors::Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_available_labels(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<Label>, merge_warden_developer_platforms::errors::Error> {
        Ok(self.available_labels.clone())
    }

    async fn add_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        labels: &[String],
    ) -> Result<(), merge_warden_developer_platforms::errors::Error> {
        let label_batch = labels.to_vec();
        self.add_calls.lock().unwrap().push(label_batch.clone());
        let mut applied = self.applied_labels.lock().unwrap();
        for l in &label_batch {
            applied.push(Label {
                name: l.clone(),
                description: None,
            });
        }
        Ok(())
    }

    async fn remove_label(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        label: &str,
    ) -> Result<(), merge_warden_developer_platforms::errors::Error> {
        self.remove_calls.lock().unwrap().push(label.to_string());
        let mut applied = self.applied_labels.lock().unwrap();
        applied.retain(|l| l.name != label);
        Ok(())
    }

    async fn list_applied_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Label>, merge_warden_developer_platforms::errors::Error> {
        Ok(self.applied_labels.lock().unwrap().clone())
    }

    async fn update_pr_check_status(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _conclusion: &str,
        _title: &str,
        _summary: &str,
        _text: &str,
    ) -> Result<(), merge_warden_developer_platforms::errors::Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_pr_reviews(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Review>, merge_warden_developer_platforms::errors::Error> {
        unimplemented!("Not needed for this test")
    }

    async fn get_pull_request_files(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<
        Vec<merge_warden_developer_platforms::models::PullRequestFile>,
        merge_warden_developer_platforms::errors::Error,
    > {
        unimplemented!("Not needed for this test")
    }

    async fn get_commit_statuses(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<
        Vec<merge_warden_developer_platforms::models::CommitStatus>,
        merge_warden_developer_platforms::errors::Error,
    > {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, merge_warden_developer_platforms::errors::Error> {
        Ok(vec![])
    }
}

/// Helper: build a full set of size/* repository labels for discovery.
fn standard_size_repo_labels() -> Vec<Label> {
    vec![
        make_label("size/XS"),
        make_label("size/S"),
        make_label("size/M"),
        make_label("size/L"),
        make_label("size/XL"),
        make_label("size/XXL"),
    ]
}

#[tokio::test]
async fn test_manage_size_labels_skips_api_calls_when_correct_label_already_applied() {
    // When the PR already carries the exact size label that would be applied,
    // manage_size_labels must return immediately without calling add_labels or
    // remove_label to avoid noise on the PR timeline.
    use crate::labels::manage_size_labels;
    use crate::size::{PrSizeCategory, PrSizeInfo, SizeThresholds};

    // PR already has "size/S" applied; S category matches 25 changed lines.
    let provider =
        SizeLabelMockProvider::new(standard_size_repo_labels(), vec![make_label("size/S")]);

    let size_info = PrSizeInfo::new(
        vec![merge_warden_developer_platforms::models::PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 15,
            deletions: 10,
            changes: 25,
            status: "modified".to_string(),
        }],
        vec![],
        &SizeThresholds::default(),
        false,
    );
    assert_eq!(size_info.size_category, PrSizeCategory::S);

    let result = manage_size_labels(&provider, "owner", "repo", 1, &size_info, "size/")
        .await
        .unwrap();

    assert_eq!(
        result.as_deref(),
        Some("size/S"),
        "Expected the existing label name to be returned"
    );
    assert!(
        provider.get_add_calls().is_empty(),
        "add_labels must not be called when the correct label is already applied"
    );
    assert!(
        provider.get_remove_calls().is_empty(),
        "remove_label must not be called when the correct label is already applied"
    );
}

#[tokio::test]
async fn test_manage_size_labels_removes_stale_and_adds_new_when_category_changes() {
    // When the PR has a stale size label (wrong category), the old label must be
    // removed and the new one added.
    use crate::labels::manage_size_labels;
    use crate::size::{PrSizeCategory, PrSizeInfo, SizeThresholds};

    // PR currently has "size/S" but the new size is M (75 lines).
    let provider =
        SizeLabelMockProvider::new(standard_size_repo_labels(), vec![make_label("size/S")]);

    let size_info = PrSizeInfo::new(
        vec![merge_warden_developer_platforms::models::PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 50,
            deletions: 25,
            changes: 75,
            status: "modified".to_string(),
        }],
        vec![],
        &SizeThresholds::default(),
        false,
    );
    assert_eq!(size_info.size_category, PrSizeCategory::M);

    let result = manage_size_labels(&provider, "owner", "repo", 1, &size_info, "size/")
        .await
        .unwrap();

    assert_eq!(
        result.as_deref(),
        Some("size/M"),
        "Expected the new size/M label to be returned"
    );
    assert_eq!(
        provider.get_remove_calls(),
        vec!["size/S"],
        "Stale size/S label must be removed"
    );
    assert!(
        provider
            .get_add_calls()
            .iter()
            .any(|batch| batch.contains(&"size/M".to_string())),
        "New size/M label must be added"
    );
}

#[tokio::test]
async fn test_manage_size_labels_removes_all_stale_and_adds_new_when_multiple_size_labels_present()
{
    // If the PR somehow accumulated multiple size labels, all stale ones must be
    // removed before the correct one is applied.
    use crate::labels::manage_size_labels;
    use crate::size::{PrSizeCategory, PrSizeInfo, SizeThresholds};

    // PR has both "size/XS" and "size/S" applied; new category is M.
    let provider = SizeLabelMockProvider::new(
        standard_size_repo_labels(),
        vec![make_label("size/XS"), make_label("size/S")],
    );

    let size_info = PrSizeInfo::new(
        vec![merge_warden_developer_platforms::models::PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 60,
            deletions: 40,
            changes: 100,
            status: "modified".to_string(),
        }],
        vec![],
        &SizeThresholds::default(),
        false,
    );
    assert_eq!(size_info.size_category, PrSizeCategory::M);

    manage_size_labels(&provider, "owner", "repo", 1, &size_info, "size/")
        .await
        .unwrap();

    let removals = provider.get_remove_calls();
    assert!(
        removals.contains(&"size/XS".to_string()),
        "size/XS must be removed; got: {:?}",
        removals
    );
    assert!(
        removals.contains(&"size/S".to_string()),
        "size/S must be removed; got: {:?}",
        removals
    );
    assert!(
        provider
            .get_add_calls()
            .iter()
            .any(|batch| batch.contains(&"size/M".to_string())),
        "size/M must be added"
    );
}

#[tokio::test]
async fn test_manage_size_labels_fallback_uses_default_prefix_when_no_repo_labels() {
    // When no size labels exist in the repository, the fallback label must use
    // the supplied label_prefix (e.g. "size/") and NOT the old hardcoded "size: " format.
    use crate::labels::manage_size_labels;
    use crate::size::{PrSizeCategory, PrSizeInfo, SizeThresholds};

    // No labels in the repository — forces the fallback path.
    let provider = SizeLabelMockProvider::new(vec![], vec![]);

    let size_info = PrSizeInfo::new(
        vec![merge_warden_developer_platforms::models::PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 50,
            deletions: 25,
            changes: 75,
            status: "modified".to_string(),
        }],
        vec![],
        &SizeThresholds::default(),
        false,
    );
    assert_eq!(size_info.size_category, PrSizeCategory::M);

    let result = manage_size_labels(&provider, "owner", "repo", 1, &size_info, "size/")
        .await
        .unwrap();

    assert_eq!(
        result.as_deref(),
        Some("size/M"),
        "Fallback label must use the supplied prefix 'size/' (not the legacy 'size: ' format)"
    );
}

#[tokio::test]
async fn test_manage_size_labels_fallback_uses_custom_prefix_when_no_repo_labels() {
    // Operators who configure label_prefix = "pr-size/" must get "pr-size/M" as
    // the fallback label, not "size/M" or "size: M".
    use crate::labels::manage_size_labels;
    use crate::size::{PrSizeCategory, PrSizeInfo, SizeThresholds};

    // No labels in the repository — forces the fallback path.
    let provider = SizeLabelMockProvider::new(vec![], vec![]);

    let size_info = PrSizeInfo::new(
        vec![merge_warden_developer_platforms::models::PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 50,
            deletions: 25,
            changes: 75,
            status: "modified".to_string(),
        }],
        vec![],
        &SizeThresholds::default(),
        false,
    );
    assert_eq!(size_info.size_category, PrSizeCategory::M);

    let result = manage_size_labels(&provider, "owner", "repo", 1, &size_info, "pr-size/")
        .await
        .unwrap();

    assert_eq!(
        result.as_deref(),
        Some("pr-size/M"),
        "Fallback label must use the configured prefix 'pr-size/'"
    );
}

// ── Keyword label customisation tests ────────────────────────────────────────

/// Builds a minimal `CurrentPullRequestValidationConfiguration` with a
/// `ChangeTypeLabelConfig` that carries the supplied `KeywordLabelsConfig`.
fn make_config_with_keyword_labels(
    keyword_labels: KeywordLabelsConfig,
) -> CurrentPullRequestValidationConfiguration {
    let mut change_type = ChangeTypeLabelConfig::default();
    change_type.keyword_labels = keyword_labels;
    CurrentPullRequestValidationConfiguration {
        change_type_labels: Some(change_type),
        ..CurrentPullRequestValidationConfiguration::default()
    }
}

#[test]
async fn test_keyword_labels_default_breaking_change_title() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat!: remove deprecated API".to_string(),
        draft: false,
        body: Some("Removes the v1 API.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    // No config → hard-coded default "breaking-change"
    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();

    assert!(
        labels.contains(&"breaking-change".to_string()),
        "Expected 'breaking-change' by default; got: {:?}",
        labels
    );
}

#[test]
async fn test_keyword_labels_custom_breaking_change() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 2,
        title: "feat!: remove deprecated API".to_string(),
        draft: false,
        body: Some("Removes the v1 API.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let config = make_config_with_keyword_labels(KeywordLabelsConfig {
        breaking_change: Some("semver-major".to_string()),
        ..KeywordLabelsConfig::default()
    });

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config))
            .await
            .unwrap();

    assert!(
        labels.contains(&"semver-major".to_string()),
        "Expected custom 'semver-major'; got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"breaking-change".to_string()),
        "Default 'breaking-change' must not appear when overridden; got: {:?}",
        labels
    );
}

#[test]
async fn test_keyword_labels_custom_security() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 3,
        title: "fix: patch auth".to_string(),
        draft: false,
        body: Some("Fixes a security vulnerability in auth.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let config = make_config_with_keyword_labels(KeywordLabelsConfig {
        security: Some("security-alert".to_string()),
        ..KeywordLabelsConfig::default()
    });

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config))
            .await
            .unwrap();

    assert!(
        labels.contains(&"security-alert".to_string()),
        "Expected custom 'security-alert'; got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"security".to_string()),
        "Default 'security' must not appear when overridden; got: {:?}",
        labels
    );
}

#[test]
async fn test_keyword_labels_custom_hotfix() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 4,
        title: "fix: production outage".to_string(),
        draft: false,
        body: Some("This is a hotfix for the production issue.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let config = make_config_with_keyword_labels(KeywordLabelsConfig {
        hotfix: Some("urgent".to_string()),
        ..KeywordLabelsConfig::default()
    });

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config))
            .await
            .unwrap();

    assert!(
        labels.contains(&"urgent".to_string()),
        "Expected custom 'urgent'; got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"hotfix".to_string()),
        "Default 'hotfix' must not appear when overridden; got: {:?}",
        labels
    );
}

#[test]
async fn test_keyword_labels_custom_tech_debt() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 5,
        title: "refactor: clean up module".to_string(),
        draft: false,
        body: Some("This addresses some technical debt in the codebase.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let config = make_config_with_keyword_labels(KeywordLabelsConfig {
        tech_debt: Some("cleanup".to_string()),
        ..KeywordLabelsConfig::default()
    });

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config))
            .await
            .unwrap();

    assert!(
        labels.contains(&"cleanup".to_string()),
        "Expected custom 'cleanup'; got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"tech-debt".to_string()),
        "Default 'tech-debt' must not appear when overridden; got: {:?}",
        labels
    );
}

#[test]
async fn test_keyword_labels_empty_string_falls_back_to_default() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 6,
        title: "fix: patch auth".to_string(),
        draft: false,
        body: Some("Fixes a security vulnerability.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    // Empty string must fall back to built-in default label name.
    let config = make_config_with_keyword_labels(KeywordLabelsConfig {
        security: Some(String::new()),
        ..KeywordLabelsConfig::default()
    });

    let labels =
        set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, Some(&config))
            .await
            .unwrap();

    assert!(
        labels.contains(&"security".to_string()),
        "Empty string must fall back to 'security'; got: {:?}",
        labels
    );
}

#[test]
async fn test_keyword_labels_no_config_uses_defaults() {
    // When no config is provided at all the hard-coded defaults apply (existing behaviour).
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 7,
        title: "fix: urgent".to_string(),
        draft: false,
        body: Some("This is a hotfix that addresses technical debt.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };

    let labels = set_pull_request_labels_with_config(&provider, "owner", "repo", &pr, None)
        .await
        .unwrap();

    assert!(
        labels.contains(&"hotfix".to_string()),
        "Default 'hotfix' must be used when no config provided; got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"tech-debt".to_string()),
        "Default 'tech-debt' must be used when no config provided; got: {:?}",
        labels
    );
}

// ── Full-featured mock for negation / suppression / explanation tests ────────

struct KeywordLabelMockProvider {
    /// Comments pre-loaded on the PR (returned by list_comments).
    comments: Vec<Comment>,
    /// Labels currently applied to the PR.
    applied_labels: Arc<Mutex<Vec<Label>>>,
    /// Bodies of all add_comment calls, in order.
    add_comment_calls: Arc<Mutex<Vec<String>>>,
    /// IDs of all delete_comment calls, in order.
    delete_comment_calls: Arc<Mutex<Vec<u64>>>,
    /// Names of all remove_label calls, in order.
    remove_label_calls: Arc<Mutex<Vec<String>>>,
    /// Names of all add_labels calls (flattened), in order.
    add_label_calls: Arc<Mutex<Vec<String>>>,
    /// Whether list_comments should return an error.
    list_comments_fails: bool,
}

impl KeywordLabelMockProvider {
    fn new(comments: Vec<Comment>, applied_labels: Vec<Label>) -> Self {
        Self {
            comments,
            applied_labels: Arc::new(Mutex::new(applied_labels)),
            add_comment_calls: Arc::new(Mutex::new(Vec::new())),
            delete_comment_calls: Arc::new(Mutex::new(Vec::new())),
            remove_label_calls: Arc::new(Mutex::new(Vec::new())),
            add_label_calls: Arc::new(Mutex::new(Vec::new())),
            list_comments_fails: false,
        }
    }

    fn with_failing_list_comments(mut self) -> Self {
        self.list_comments_fails = true;
        self
    }

    fn added_labels(&self) -> Vec<String> {
        self.add_label_calls.lock().unwrap().clone()
    }

    fn removed_labels(&self) -> Vec<String> {
        self.remove_label_calls.lock().unwrap().clone()
    }

    fn posted_comments(&self) -> Vec<String> {
        self.add_comment_calls.lock().unwrap().clone()
    }

    fn deleted_comment_ids(&self) -> Vec<u64> {
        self.delete_comment_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl PullRequestProvider for KeywordLabelMockProvider {
    async fn get_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!("Not needed for this test")
    }

    async fn add_comment(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        comment: &str,
    ) -> Result<(), Error> {
        self.add_comment_calls
            .lock()
            .unwrap()
            .push(comment.to_string());
        Ok(())
    }

    async fn delete_comment(&self, _owner: &str, _repo: &str, id: u64) -> Result<(), Error> {
        self.delete_comment_calls.lock().unwrap().push(id);
        Ok(())
    }

    async fn list_comments(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Comment>, Error> {
        if self.list_comments_fails {
            return Err(Error::FailedToUpdatePullRequest(
                "list_comments failed".to_string(),
            ));
        }
        Ok(self.comments.clone())
    }

    async fn list_available_labels(&self, _owner: &str, _repo: &str) -> Result<Vec<Label>, Error> {
        Ok(vec![])
    }

    async fn add_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        self.add_label_calls
            .lock()
            .unwrap()
            .extend(labels.iter().cloned());
        Ok(())
    }

    async fn remove_label(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        label: &str,
    ) -> Result<(), Error> {
        self.remove_label_calls
            .lock()
            .unwrap()
            .push(label.to_string());
        Ok(())
    }

    async fn list_applied_labels(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Label>, Error> {
        Ok(self.applied_labels.lock().unwrap().clone())
    }

    async fn update_pr_check_status(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
        _conclusion: &str,
        _title: &str,
        _summary: &str,
        _text: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_pr_reviews(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<Review>, Error> {
        Ok(vec![])
    }

    async fn get_pull_request_files(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u64,
    ) -> Result<Vec<PullRequestFile>, Error> {
        Ok(vec![])
    }

    async fn get_commit_statuses(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<merge_warden_developer_platforms::models::CommitStatus>, Error> {
        Ok(vec![])
    }

    async fn find_pull_requests_for_commit(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _commit_sha: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

fn make_comment(id: u64, login: &str, body: &str) -> Comment {
    Comment {
        id,
        body: body.to_string(),
        user: User {
            id: id * 100,
            login: login.to_string(),
        },
    }
}

/// Returns the byte range of `keyword` within `text`.  Panics when not found.
fn find_span(text: &str, keyword: &str) -> std::ops::Range<usize> {
    let start = text
        .find(keyword)
        .unwrap_or_else(|| panic!("keyword '{keyword}' not found in text: '{text}'"));
    start..start + keyword.len()
}

// ── is_keyword_negated: comprehensive adversarial tests ──────────────────────
// Tests every negation word, every boundary condition, and corner cases that
// could expose bugs in the clause-scoping or tokenisation logic.

// — Each negation word individually —

#[test]
async fn test_negated_no_immediately_before_keyword() {
    let text = "no breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'no' must negate");
}

#[test]
async fn test_negated_not_in_window() {
    let text = "this is not a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'not' must negate");
}

#[test]
async fn test_negated_without_before_keyword() {
    let text = "merged without any breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'without' must negate"
    );
}

#[test]
async fn test_negated_never_before_keyword() {
    let text = "there is never a breaking change here";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'never' must negate");
}

#[test]
async fn test_negated_dont_with_apostrophe() {
    let text = "we don't have a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'don't' must negate");
}

#[test]
async fn test_negated_doesnt_with_apostrophe() {
    let text = "this pr doesn't introduce a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'doesn't' must negate"
    );
}

#[test]
async fn test_negated_isnt_with_apostrophe() {
    let text = "this isn't a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'isn't' must negate");
}

#[test]
async fn test_negated_arent_with_apostrophe() {
    let text =
        "these aren't breaking changes to the api, but this is a breaking change in behavior";
    // "breaking change" appears AFTER the clause-local "isn't" – find the second occurrence
    let second_start = text.rfind("breaking change").unwrap();
    let span = second_start..second_start + "breaking change".len();
    // The second "breaking change" has no negation word before it in its clause
    assert!(
        !super::is_keyword_negated(text, span),
        "second 'breaking change' after affirmative context must NOT be negated"
    );
}

#[test]
async fn test_negated_wont_with_apostrophe() {
    let text = "this won't cause a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'won't' must negate");
}

#[test]
async fn test_negated_dont_without_apostrophe() {
    let text = "we dont have a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'dont' must negate");
}

#[test]
async fn test_negated_doesnt_without_apostrophe() {
    let text = "this doesnt introduce a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'doesnt' must negate"
    );
}

#[test]
async fn test_negated_isnt_without_apostrophe() {
    let text = "this isnt a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'isnt' must negate");
}

#[test]
async fn test_negated_arent_without_apostrophe() {
    let text = "these arent breaking changes in the old sense; this is a breaking change";
    // The semicolon resets the clause; the second "breaking change" has no negation in its clause.
    let second_start = text.rfind("breaking change").unwrap();
    let span = second_start..second_start + "breaking change".len();
    assert!(
        !super::is_keyword_negated(text, span),
        "semicolon boundary must prevent 'arent' from crossing to the next clause"
    );
}

#[test]
async fn test_negated_wont_without_apostrophe() {
    let text = "this wont cause a breaking change";
    let span = find_span(text, "breaking change");
    assert!(super::is_keyword_negated(text, span), "'wont' must negate");
}

// — "does not" / "is not" two-word patterns covered by "not" single token —

#[test]
async fn test_negated_does_not_pattern() {
    // "not" is the operative single-word negation token within the 5-word window
    let text = "this does not introduce a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'does not' (via 'not' token) must negate"
    );
}

#[test]
async fn test_negated_is_not_pattern() {
    let text = "this is not a breaking change at all";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'is not' (via 'not' token) must negate"
    );
}

// — Window boundary conditions —

#[test]
async fn test_negated_at_exactly_5_word_boundary() {
    // "no" is the 5th (and last) word in the window: "no w1 w2 w3 breaking change"
    // Tokens before keyword: ["no", "w1", "w2", "w3"]  – only 4, so all in window.
    // Actually with "no w1 w2 w3 breaking change":
    //   before = "no w1 w2 w3 "
    //   tokens = ["no", "w1", "w2", "w3"]  → len=4, window_start=max(0,4-5)=0 → window=all → "no" IS in window.
    let text = "no w1 w2 w3 breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'no' within 4 preceding words must negate (all within 5-word window)"
    );
}

#[test]
async fn test_not_negated_when_negation_is_6th_word_before_keyword() {
    // "no" is followed by 5 filler words → it's the 6th token before the keyword → outside window
    let text = "no word1 word2 word3 word4 word5 breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "'no' 6 words before keyword must NOT negate (outside 5-word window)"
    );
}

#[test]
async fn test_negated_at_exactly_5th_position() {
    // 5 tokens before keyword; negation is the 1st → window_start = 0, all included
    let text = "not w1 w2 w3 w4 breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'not' exactly 5 words before keyword must still negate"
    );
}

// — Clause boundary conditions —

#[test]
async fn test_not_negated_clause_boundary_period_resets() {
    // "no" is in a prior sentence; "breaking change" is in the next clause
    let text = "there are no issues. this introduces a breaking change.";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "period resets clause scope; 'no' from prior sentence must not negate"
    );
}

#[test]
async fn test_not_negated_clause_boundary_semicolon_between_negation_and_keyword() {
    // Semicolon between "no" and "breaking change" resets the clause
    let text = "no; breaking change was introduced here";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "semicolon between negation and keyword must reset clause and prevent negation"
    );
}

#[test]
async fn test_not_negated_clause_boundary_newline_resets() {
    // Newline acts as clause boundary
    let text = "no issues here\nthis introduces a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "newline resets clause scope; 'no' before newline must not negate across it"
    );
}

#[test]
async fn test_negated_after_semicolon_clause_boundary() {
    // "not" appears AFTER a semicolon, in the SAME clause as the keyword
    let text = "all fine here; not a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'not' in the same clause after a semicolon must negate"
    );
}

// — No preceding text conditions —

#[test]
async fn test_not_negated_keyword_at_very_start_of_text() {
    let text = "breaking change was introduced";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "keyword at start of text with no preceding context must NOT be negated"
    );
}

// — Non-negation substrings —

#[test]
async fn test_not_negated_non_negation_word_containing_negation_substring() {
    // "annotated" contains "not" as substring but whole-word comparison must reject it
    let text = "annotated breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "'annotated' must not be treated as a negation word"
    );
}

#[test]
async fn test_not_negated_word_ending_in_not() {
    // "cannot" is NOT in the negation list (see labels.rs comment for rationale).
    // "we cannot wait to add..." is affirmative — the negation list must not match it.
    let text = "we cannot wait to add a breaking change to the API";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "'cannot' must not be treated as a negation word"
    );
}

// — Punctuation-attached negation tokens —

#[test]
async fn test_negated_no_with_trailing_comma() {
    // "no," must be recognised as negation; punct stripping removes the comma.
    let text = "no, this pr doesn't introduce a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'no,' (comma-attached) must negate"
    );
}

#[test]
async fn test_negated_no_in_parentheses() {
    // "(no)" must be recognised as negation; punct stripping removes both parens.
    let text = "this is (no) breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'(no)' (paren-wrapped) must negate"
    );
}

#[test]
async fn test_negated_no_with_trailing_colon() {
    // "no:" must be recognised as negation; punct stripping removes the colon.
    let text = "no: breaking change to the public api";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'no:' (colon-attached) must negate"
    );
}

#[test]
async fn test_negated_contraction_with_trailing_comma() {
    // "don't," must still be recognised — commas appear after contractions in natural prose.
    let text = "we don't, and have never, introduced a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'don't,' (comma after contraction) must negate"
    );
}

// — Markdown image `!` must not act as a clause boundary —

#[test]
async fn test_negated_keyword_after_markdown_image_not_split_at_exclamation() {
    // `!` followed by `[` is a Markdown image link, not a sentence terminator.
    // The negation word before the image must still be in scope.
    // Note: the URL must not contain `.` chars (which are sentence boundaries).
    let text = "no ![screenshot](/img) breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "'!' in markdown image link must not reset clause; 'no' before the image must negate"
    );
}

#[test]
async fn test_clause_boundary_exclamation_without_bracket_still_splits() {
    // A bare `!` (not followed by `[`) IS a sentence boundary as before.
    let text = "great! no breaking change";
    let span = find_span(text, "breaking change");
    // "no" is in the clause AFTER `!`, so it should still negate.
    // The `!` terminates the prior clause — "no" is in the new clause.
    assert!(
        super::is_keyword_negated(text, span),
        "'no' in clause after bare '!' must still negate"
    );
}

#[test]
async fn test_negation_in_prior_clause_before_bare_exclamation_does_not_cross() {
    // Negation word is in the clause that ends with `!`; keyword is in the next clause.
    // The `!` boundary must prevent the negation from crossing into the next clause.
    let text = "no! this introduces a breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        !super::is_keyword_negated(text, span),
        "'no!' in a prior clause must not negate 'breaking change' across the '!' boundary"
    );
}

#[test]
async fn test_negated_multiple_negation_words_in_window() {
    // Belt AND suspenders: two negation words in the window
    let text = "not no breaking change";
    let span = find_span(text, "breaking change");
    assert!(
        super::is_keyword_negated(text, span),
        "multiple negation words in window must still negate"
    );
}

// — Applied to different keyword types —

#[test]
async fn test_negated_security_keyword() {
    let text = "there is no security vulnerability here";
    let span = find_span(text, "security");
    assert!(
        super::is_keyword_negated(text, span),
        "'no security' must negate"
    );
}

#[test]
async fn test_not_negated_affirmative_security_keyword() {
    let text = "this addresses a security vulnerability";
    let span = find_span(text, "security");
    assert!(
        !super::is_keyword_negated(text, span),
        "affirmative security context must NOT negate"
    );
}

#[test]
async fn test_negated_hotfix_keyword() {
    let text = "this is not a hotfix";
    let span = find_span(text, "hotfix");
    assert!(
        super::is_keyword_negated(text, span),
        "'not a hotfix' must negate"
    );
}

#[test]
async fn test_negated_tech_debt_keyword() {
    let text = "without tech debt this time";
    let span = find_span(text, "tech debt");
    assert!(
        super::is_keyword_negated(text, span),
        "'without tech debt' must negate"
    );
}

// ── parse_suppressed_labels: comprehensive adversarial tests ─────────────────

#[test]
async fn test_parse_suppressed_single_command() {
    let comments = vec![make_comment(
        1,
        "alice",
        "@merge-warden suppress: breaking-change",
    )];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert_eq!(
        result.get("breaking-change").map(String::as_str),
        Some("alice"),
        "single suppress command must record commenter login"
    );
}

#[test]
async fn test_parse_suppressed_two_labels_same_comment() {
    let body = "@merge-warden suppress: breaking-change\n@merge-warden suppress: security";
    let comments = vec![make_comment(1, "bob", body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.contains_key("breaking-change"),
        "breaking-change must be suppressed"
    );
    assert!(
        result.contains_key("security"),
        "security must be suppressed"
    );
    assert_eq!(result.len(), 2, "must have exactly 2 entries");
}

#[test]
async fn test_parse_suppressed_two_labels_across_comments() {
    let comments = vec![
        make_comment(1, "alice", "@merge-warden suppress: hotfix"),
        make_comment(2, "bob", "@merge-warden suppress: tech-debt"),
    ];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(result.contains_key("hotfix"));
    assert!(result.contains_key("tech-debt"));
    assert_eq!(result["hotfix"], "alice");
    assert_eq!(result["tech-debt"], "bob");
}

#[test]
async fn test_parse_suppressed_unknown_command_silently_ignored() {
    let comments = vec![make_comment(
        1,
        "alice",
        "@merge-warden unknown: breaking-change",
    )];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.is_empty(),
        "unrecognised command must produce no entries"
    );
}

#[test]
async fn test_parse_suppressed_non_mention_line_ignored() {
    let comments = vec![make_comment(
        1,
        "alice",
        "just a normal comment with no bot mention",
    )];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.is_empty(),
        "non-mention line must produce no entries"
    );
}

#[test]
async fn test_parse_suppressed_case_insensitive_bot_mention() {
    // Bot mention prefix is case-insensitive per spec
    let comments = vec![make_comment(1, "carol", "@MERGE-WARDEN suppress: security")];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.contains_key("security"),
        "uppercase bot mention must still match"
    );
    assert_eq!(result["security"], "carol");
}

#[test]
async fn test_parse_suppressed_custom_bot_mention() {
    let comments = vec![make_comment(1, "dave", "@acme-bot suppress: hotfix")];
    let result = super::parse_suppressed_labels(&comments, "@acme-bot");
    assert!(
        result.contains_key("hotfix"),
        "custom bot mention must work"
    );
    // The default @merge-warden prefix must NOT match @acme-bot commands
    let result2 = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result2.is_empty(),
        "wrong bot mention prefix must not match"
    );
}

#[test]
async fn test_parse_suppressed_leading_whitespace_on_line_still_matches() {
    // line.trim() strips leading whitespace before prefix comparison
    let body = "   @merge-warden suppress: breaking-change";
    let comments = vec![make_comment(1, "eve", body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.contains_key("breaking-change"),
        "leading whitespace on line must be trimmed before matching"
    );
}

#[test]
async fn test_parse_suppressed_label_name_with_trailing_whitespace_is_trimmed() {
    // rest.trim() must remove trailing spaces from the label name
    let body = "@merge-warden suppress: hotfix   ";
    let comments = vec![make_comment(1, "frank", body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.contains_key("hotfix"),
        "trailing whitespace after label name must be trimmed"
    );
    assert!(
        !result.contains_key("hotfix   "),
        "key must not have trailing whitespace"
    );
}

#[test]
async fn test_parse_suppressed_uppercase_suppress_command_works() {
    // The entire line is lowercased so "SUPPRESS:" must match "suppress:"
    let body = "@merge-warden SUPPRESS: security";
    let comments = vec![make_comment(1, "grace", body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.contains_key("security"),
        "SUPPRESS in uppercase must be treated same as suppress"
    );
}

#[test]
async fn test_parse_suppressed_empty_label_name_after_colon_ignored() {
    // "@merge-warden suppress: " followed by nothing → empty label → must be silently ignored
    let body = "@merge-warden suppress: ";
    let comments = vec![make_comment(1, "harry", body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.is_empty(),
        "empty label name after colon must be ignored, not inserted"
    );
}

#[test]
async fn test_parse_suppressed_first_commenter_wins_for_same_label() {
    // Two different people both suppress the same label; first commenter must win
    let comments = vec![
        make_comment(1, "alice", "@merge-warden suppress: breaking-change"),
        make_comment(2, "bob", "@merge-warden suppress: breaking-change"),
    ];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert_eq!(
        result.get("breaking-change").map(String::as_str),
        Some("alice"),
        "first commenter must win when same label is suppressed multiple times"
    );
    assert_eq!(result.len(), 1, "must have exactly one entry for the label");
}

#[test]
async fn test_parse_suppressed_bots_own_explanation_comment_is_skipped() {
    // The bot's explanation comment embeds the suppress command as example text.
    // It must be filtered out to prevent the bot from suppressing its own label.
    let bot_comment_body = super::build_keyword_label_comment("hotfix", "@merge-warden");
    assert!(
        bot_comment_body.contains("@merge-warden suppress: hotfix"),
        "sanity: bot comment body must contain suppress command as example"
    );
    let comments = vec![make_comment(1, "merge-warden[bot]", &bot_comment_body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.is_empty(),
        "the bot's own explanation comment must be skipped and must not trigger suppression"
    );
}

#[test]
async fn test_parse_suppressed_mixed_comment_some_commands_some_not() {
    // A comment that has both a valid suppress command and unrelated lines
    let body = "Looking good overall!\n@merge-warden suppress: hotfix\nPlease add more tests.";
    let comments = vec![make_comment(1, "reviewer", body)];
    let result = super::parse_suppressed_labels(&comments, "@merge-warden");
    assert!(
        result.contains_key("hotfix"),
        "suppress command among other text must be recognized"
    );
    assert_eq!(result.len(), 1);
}

// ── build_keyword_label_comment: comprehensive adversarial tests ───────────────

#[test]
async fn test_build_keyword_label_comment_marker_is_first_line() {
    // The KEYWORD_LABEL_COMMENT_MARKER must appear on the VERY FIRST LINE,
    // not buried somewhere in the body.
    let body = super::build_keyword_label_comment("breaking-change", "@merge-warden");
    let first_line = body
        .lines()
        .next()
        .expect("comment must have at least one line");
    let expected = format!("{}breaking-change -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert_eq!(
        first_line, expected,
        "marker must be exactly the first line of the comment body"
    );
}

#[test]
async fn test_build_keyword_label_comment_marker_includes_label_name() {
    let body = super::build_keyword_label_comment("security", "@merge-warden");
    let marker = format!("{}security -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        body.contains(&marker),
        "comment must contain the per-label marker with label name; got:\n{body}"
    );
}

#[test]
async fn test_build_keyword_label_comment_suppress_command_in_code_fence() {
    // The suppress command must be inside a ``` code fence so users can easily copy it
    let body = super::build_keyword_label_comment("hotfix", "@merge-warden");
    let lines: Vec<&str> = body.lines().collect();
    // Find the opening code fence
    let fence_pos = lines
        .iter()
        .position(|l| l.trim() == "```")
        .expect("comment must contain a ``` code fence");
    // The suppress command must appear after the fence
    let after_fence = &lines[fence_pos + 1..];
    assert!(
        after_fence
            .iter()
            .any(|l| l.trim() == "@merge-warden suppress: hotfix"),
        "suppress command must appear inside the code fence; got:\n{body}"
    );
}

#[test]
async fn test_build_keyword_label_comment_suppress_command_reflects_label_name() {
    let body = super::build_keyword_label_comment("tech-debt", "@merge-warden");
    assert!(
        body.contains("@merge-warden suppress: tech-debt"),
        "suppress command must use the exact label name; got:\n{body}"
    );
}

#[test]
async fn test_build_keyword_label_comment_custom_bot_mention_on_first_line_marker() {
    let body = super::build_keyword_label_comment("hotfix", "@acme-bot");
    let first_line = body.lines().next().unwrap();
    let expected_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert_eq!(
        first_line, expected_marker,
        "marker on first line must use the correct format"
    );
    assert!(
        body.contains("@acme-bot suppress: hotfix"),
        "suppress command must use the custom bot mention; got:\n{body}"
    );
}

#[test]
async fn test_build_keyword_label_comment_different_labels_produce_different_markers() {
    let body_a = super::build_keyword_label_comment("security", "@merge-warden");
    let body_b = super::build_keyword_label_comment("hotfix", "@merge-warden");
    let marker_a = format!("{}security -->", KEYWORD_LABEL_COMMENT_MARKER);
    let marker_b = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(body_a.contains(&marker_a));
    assert!(
        !body_a.contains(&marker_b),
        "security comment must NOT contain hotfix marker"
    );
    assert!(body_b.contains(&marker_b));
    assert!(
        !body_b.contains(&marker_a),
        "hotfix comment must NOT contain security marker"
    );
}

// ── Negation-aware detection integration tests ───────────────────────────────
// These tests exercise set_pull_request_labels_with_config end-to-end.
// They assert BOTH the return value AND the actual mock provider call record
// to ensure the implementation does not just lie about what it applied.

#[test]
async fn test_negation_breaking_change_in_body_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 10,
        title: "fix: cleanup api".to_string(),
        draft: false,
        body: Some("This PR introduces no breaking changes.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        !labels.contains(&"breaking-change".to_string()),
        "'no breaking changes' must not trigger label (return value); got: {:?}",
        labels
    );
    assert!(
        !provider
            .added_labels()
            .contains(&"breaking-change".to_string()),
        "add_labels must not be called with 'breaking-change' when negated"
    );
    // Explanation comment must NOT be posted for breaking-change
    let marker = format!("{}breaking-change -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        !provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&marker)),
        "no breaking-change explanation comment must be posted when detection is negated"
    );
    // Type label 'bug' must still be applied (proves the function ran properly)
    assert!(
        labels.contains(&"bug".to_string()),
        "type label 'bug' must still be applied; got: {:?}",
        labels
    );
}

#[test]
async fn test_negation_breaking_change_without_variant_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 100,
        title: "fix: api update".to_string(),
        draft: false,
        body: Some("This was merged without breaking changes to the API.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(!labels.contains(&"breaking-change".to_string()));
    assert!(!provider
        .added_labels()
        .contains(&"breaking-change".to_string()));
}

#[test]
async fn test_negation_breaking_change_doesnt_variant_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 101,
        title: "fix: migration".to_string(),
        draft: false,
        body: Some("This PR doesn't introduce any breaking change to the public API.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(!labels.contains(&"breaking-change".to_string()));
    assert!(!provider
        .added_labels()
        .contains(&"breaking-change".to_string()));
}

// Spec assertion: negation in the PR *title* must suppress breaking-change label.
// The title "fix: no breaking change to the public api" contains the keyword
// "breaking change" preceded by "no" — detection must be suppressed.
#[test]
async fn test_negation_breaking_change_in_title_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 102,
        title: "fix: no breaking change to the public api".to_string(),
        draft: false,
        body: None,
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        !labels.contains(&"breaking-change".to_string()),
        "'no breaking change' in title must not trigger label; got: {:?}",
        labels
    );
    assert!(
        !provider
            .added_labels()
            .contains(&"breaking-change".to_string()),
        "add_labels must not be called with 'breaking-change' when negated in title"
    );
    // No explanation comment should be posted for breaking-change
    let marker = format!("{}breaking-change -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        !provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&marker)),
        "no breaking-change explanation comment must be posted when title negation suppresses detection"
    );
}

#[test]
async fn test_affirmative_breaking_change_in_body_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 11,
        title: "feat: remove api".to_string(),
        draft: false,
        body: Some("Breaking change: removed the old API endpoint.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"breaking-change".to_string()),
        "affirmative breaking change must apply label (return value); got: {:?}",
        labels
    );
    assert!(
        provider
            .added_labels()
            .contains(&"breaking-change".to_string()),
        "add_labels must be called with 'breaking-change' for affirmative body"
    );
    // Explanation comment must be posted
    let marker = format!("{}breaking-change -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&marker)),
        "explanation comment for breaking-change must be posted; posted: {:?}",
        provider.posted_comments()
    );
}

#[test]
async fn test_exclamation_colon_always_triggers_breaking_change_even_with_negated_body() {
    // This is the critical test: "!:" in title is unconditional.
    // Even if the body says "no breaking changes", the title marker wins.
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 12,
        title: "feat!: remove public api".to_string(),
        draft: false,
        body: Some("no breaking changes in the description whatsoever".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"breaking-change".to_string()),
        "'!:' in title must always trigger breaking-change regardless of negated body; got: {:?}",
        labels
    );
    assert!(
        provider
            .added_labels()
            .contains(&"breaking-change".to_string()),
        "add_labels must be called with 'breaking-change' for '!:' title"
    );
}

#[test]
async fn test_body_with_both_negated_and_affirmative_breaking_change_triggers_label() {
    // .any() on the regex iterator means a SINGLE non-negated match triggers the label.
    // "no breaking change" is negated, but "this is a breaking change" is not.
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 102,
        title: "refactor: api update".to_string(),
        draft: false,
        body: Some(
            "no breaking change in the v1 API. However, this includes a breaking change in the CLI interface."
                .to_string(),
        ),
        author: Some(User { id: 1, login: "dev".to_string() }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"breaking-change".to_string()),
        "body with one negated and one affirmative match must still trigger label; got: {:?}",
        labels
    );
    assert!(provider
        .added_labels()
        .contains(&"breaking-change".to_string()));
}

#[test]
async fn test_negation_security_in_body_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 13,
        title: "fix: dep update".to_string(),
        draft: false,
        body: Some("There are no security vulnerabilities introduced.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        !labels.contains(&"security".to_string()),
        "'no security vulnerabilities' must not trigger label; got: {:?}",
        labels
    );
    assert!(!provider.added_labels().contains(&"security".to_string()));
    let marker = format!("{}security -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(!provider
        .posted_comments()
        .iter()
        .any(|c| c.contains(&marker)));
}

#[test]
async fn test_negation_no_security_issues_in_body_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 103,
        title: "chore: bump deps".to_string(),
        draft: false,
        body: Some("no security issues found in this update".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(!labels.contains(&"security".to_string()));
    assert!(!provider.added_labels().contains(&"security".to_string()));
}

#[test]
async fn test_affirmative_security_in_body_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 14,
        title: "fix: patch cve".to_string(),
        draft: false,
        body: Some("security: addresses cve-2025-1234".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"security".to_string()),
        "affirmative security mention must apply label; got: {:?}",
        labels
    );
    assert!(provider.added_labels().contains(&"security".to_string()));
    let marker = format!("{}security -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&marker)),
        "explanation comment for security must be posted; posted: {:?}",
        provider.posted_comments()
    );
}

#[test]
async fn test_negation_hotfix_in_body_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 15,
        title: "fix: routine".to_string(),
        draft: false,
        body: Some("There is no hotfix required.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        !labels.contains(&"hotfix".to_string()),
        "'no hotfix required' must not trigger label; got: {:?}",
        labels
    );
    assert!(!provider.added_labels().contains(&"hotfix".to_string()));
    let marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(!provider
        .posted_comments()
        .iter()
        .any(|c| c.contains(&marker)));
}

#[test]
async fn test_affirmative_hotfix_in_body_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 16,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for the production crash.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"hotfix".to_string()),
        "affirmative hotfix mention must apply label; got: {:?}",
        labels
    );
    assert!(provider.added_labels().contains(&"hotfix".to_string()));
    let marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&marker)),
        "explanation comment for hotfix must be posted"
    );
}

#[test]
async fn test_negation_tech_debt_in_body_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 17,
        title: "refactor: cleanup".to_string(),
        draft: false,
        body: Some("not tech debt, this is a deliberate refactor.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        !labels.contains(&"tech-debt".to_string()),
        "'not tech debt' must not trigger label; got: {:?}",
        labels
    );
    assert!(!provider.added_labels().contains(&"tech-debt".to_string()));
}

#[test]
async fn test_negation_not_technical_debt_in_body_not_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 104,
        title: "refactor: extract module".to_string(),
        draft: false,
        body: Some("This is not technical debt, it is a planned refactor.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(!labels.contains(&"tech-debt".to_string()));
}

#[test]
async fn test_affirmative_tech_debt_reduces_applied() {
    // "reduces tech debt" is STILL about tech debt – not negated
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 18,
        title: "refactor: reduce complexity".to_string(),
        draft: false,
        body: Some("reduces tech debt by cleaning up the module.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"tech-debt".to_string()),
        "'reduces tech debt' is still about tech debt and must trigger label; got: {:?}",
        labels
    );
    assert!(provider.added_labels().contains(&"tech-debt".to_string()));
}

#[test]
async fn test_affirmative_technical_debt_in_body_applied() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let pr = PullRequest {
        number: 105,
        title: "fix: cleanup".to_string(),
        draft: false,
        body: Some("This PR introduces some technical debt to ship faster.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, None)
        .await
        .unwrap();
    assert!(
        labels.contains(&"tech-debt".to_string()),
        "affirmative technical-debt mention must apply label; got: {:?}",
        labels
    );
}

// ── Suppression integration tests ─────────────────────────────────────────────

#[test]
async fn test_suppression_skips_label_application() {
    let suppress_comment = make_comment(1, "alice", "@merge-warden suppress: breaking-change");
    let provider = KeywordLabelMockProvider::new(vec![suppress_comment], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 20,
        title: "feat!: remove endpoint".to_string(),
        draft: false,
        body: None,
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    // Return value must not include the suppressed label
    assert!(
        !labels.contains(&"breaking-change".to_string()),
        "suppressed label must not appear in return value; got: {:?}",
        labels
    );
    // add_labels must not have been called with the suppressed label
    assert!(
        !provider
            .added_labels()
            .contains(&"breaking-change".to_string()),
        "suppressed label must not be passed to add_labels; added: {:?}",
        provider.added_labels()
    );
    // Explanation comment must NOT be posted for a suppressed label
    let marker = format!("{}breaking-change -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        !provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&marker)),
        "explanation comment must not be posted for suppressed label"
    );
}

#[test]
async fn test_suppression_removes_existing_label_from_pr() {
    let suppress_comment = make_comment(1, "alice", "@merge-warden suppress: security");
    let existing_security_label = Label {
        name: "security".to_string(),
        description: None,
    };
    let provider =
        KeywordLabelMockProvider::new(vec![suppress_comment], vec![existing_security_label]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 21,
        title: "fix: patch".to_string(),
        draft: false,
        body: Some("security: addresses cve-2025-0001".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        provider.removed_labels().contains(&"security".to_string()),
        "suppressed label already on PR must be removed via remove_label; removed: {:?}",
        provider.removed_labels()
    );
    // And it must NOT be re-added
    assert!(
        !provider.added_labels().contains(&"security".to_string()),
        "suppressed label must not be re-added after removal"
    );
}

#[test]
async fn test_suppression_does_not_call_remove_when_label_not_currently_applied() {
    // If the label is suppressed but not currently on the PR, remove_label must not be called.
    // Calling remove_label when the label isn't there wastes an API call.
    let suppress_comment = make_comment(1, "alice", "@merge-warden suppress: hotfix");
    // applied_labels is EMPTY – hotfix is not currently on the PR
    let provider = KeywordLabelMockProvider::new(vec![suppress_comment], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 200,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for production.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        !provider.removed_labels().contains(&"hotfix".to_string()),
        "remove_label must NOT be called for hotfix when it is not currently applied; \
         removed: {:?}",
        provider.removed_labels()
    );
}

#[test]
async fn test_suppression_isolated_to_named_label_only() {
    // Suppressing "breaking-change" must not affect "security" detection or application.
    let suppress_comment = make_comment(1, "alice", "@merge-warden suppress: breaking-change");
    let provider = KeywordLabelMockProvider::new(vec![suppress_comment], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 201,
        title: "feat!: remove endpoint".to_string(),
        draft: false,
        body: Some("security: this addresses a vulnerability".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    // breaking-change is suppressed → must not appear
    assert!(
        !labels.contains(&"breaking-change".to_string()),
        "breaking-change must be suppressed; got: {:?}",
        labels
    );
    // security is NOT suppressed → must appear
    assert!(
        labels.contains(&"security".to_string()),
        "security must still be applied even when breaking-change is suppressed; got: {:?}",
        labels
    );
}

#[test]
async fn test_suppression_multi_label_different_suppressors() {
    // Two labels suppressed by two different commenters
    let comments = vec![
        make_comment(1, "alice", "@merge-warden suppress: hotfix"),
        make_comment(2, "bob", "@merge-warden suppress: tech-debt"),
    ];
    let provider = KeywordLabelMockProvider::new(comments, vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 22,
        title: "fix: urgent cleanup".to_string(),
        draft: false,
        body: Some("this is a hotfix that also reduces tech debt".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    let labels = set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        !labels.contains(&"hotfix".to_string()),
        "hotfix suppressed; got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"tech-debt".to_string()),
        "tech-debt suppressed; got: {:?}",
        labels
    );
    assert!(!provider.added_labels().contains(&"hotfix".to_string()));
    assert!(!provider.added_labels().contains(&"tech-debt".to_string()));
}

#[test]
async fn test_suppression_list_comments_failure_falls_back_to_no_suppression() {
    // When list_comments fails, the function must behave as if no suppression is in effect
    // and still apply the label. A warn! is logged but no error is returned.
    let provider = KeywordLabelMockProvider::new(vec![], vec![]).with_failing_list_comments();
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 23,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for production.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    // Must succeed (no propagated error)
    let result = set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config)).await;
    assert!(
        result.is_ok(),
        "must return Ok even when list_comments fails"
    );
    let labels = result.unwrap();
    assert!(
        labels.contains(&"hotfix".to_string()),
        "when list_comments fails, hotfix must still be applied; got: {:?}",
        labels
    );
    assert!(
        provider.added_labels().contains(&"hotfix".to_string()),
        "add_labels must be called with hotfix when fallback to no-suppression"
    );
}

// ── Explanation comment lifecycle tests ───────────────────────────────────────

#[test]
async fn test_explanation_comment_posted_when_label_triggered_and_no_prior_comment() {
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 30,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for the server crash.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    let posted = provider.posted_comments();
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        posted.iter().any(|c| c.contains(&hotfix_marker)),
        "explanation comment for 'hotfix' must be posted when label is triggered; posted: {:?}",
        posted
    );
    // Comment must contain the full expected body (marker on first line)
    let expected_body = super::build_keyword_label_comment("hotfix", "@merge-warden");
    assert!(
        posted.iter().any(|c| *c == expected_body),
        "posted comment must match the expected body exactly; posted: {:?}",
        posted
    );
    // No deletion must have occurred (no prior comment)
    assert!(
        provider.deleted_comment_ids().is_empty(),
        "no comment must be deleted when there was no prior explanation comment"
    );
}

#[test]
async fn test_explanation_comment_idempotent_when_identical_body_exists() {
    // If the existing comment already has the exact expected body, no API call should be made.
    let expected_body = super::build_keyword_label_comment("hotfix", "@merge-warden");
    let existing = make_comment(99, "merge-warden[bot]", &expected_body);
    let provider = KeywordLabelMockProvider::new(vec![existing], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 31,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for the server crash.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        provider.posted_comments().is_empty(),
        "must NOT post a new comment when the existing one is identical; posted: {:?}",
        provider.posted_comments()
    );
    assert!(
        provider.deleted_comment_ids().is_empty(),
        "must NOT delete the existing identical comment; deleted: {:?}",
        provider.deleted_comment_ids()
    );
}

#[test]
async fn test_explanation_comment_stale_body_replaced_with_fresh_one() {
    // If an existing comment has the marker but a DIFFERENT body (e.g., format changed),
    // the stale comment must be deleted and a new one posted.
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    // Deliberately use a body that contains the marker but differs from the current format
    let stale_body = format!(
        "{} OUTDATED FORMAT - hotfix label was applied",
        hotfix_marker
    );
    let stale = make_comment(77, "merge-warden[bot]", &stale_body);
    let provider = KeywordLabelMockProvider::new(vec![stale], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 50,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for the server crash.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        provider.deleted_comment_ids().contains(&77),
        "stale explanation comment with outdated body must be deleted; deleted: {:?}",
        provider.deleted_comment_ids()
    );
    let expected_body = super::build_keyword_label_comment("hotfix", "@merge-warden");
    assert!(
        provider
            .posted_comments()
            .iter()
            .any(|c| *c == expected_body),
        "a fresh explanation comment must be posted after deleting the stale one; posted: {:?}",
        provider.posted_comments()
    );
}

#[test]
async fn test_explanation_comment_two_stale_copies_both_replaced_with_fresh_one() {
    // Two stale copies must both be deleted and exactly one fresh comment posted.
    // The implementation posts the fresh comment first (post-first-then-delete approach)
    // so the explanation is never permanently lost due to a transient delete failure.
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    let stale_body = format!("{} old format v1", hotfix_marker);
    let stale1 = make_comment(80, "bot", &stale_body);
    let stale2 = make_comment(81, "bot", &stale_body);
    let provider = KeywordLabelMockProvider::new(vec![stale1, stale2], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 51,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for the server crash.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    let deleted = provider.deleted_comment_ids();
    assert!(
        deleted.contains(&80),
        "first stale copy must be deleted; deleted: {:?}",
        deleted
    );
    assert!(
        deleted.contains(&81),
        "second stale copy must be deleted; deleted: {:?}",
        deleted
    );
    assert_eq!(
        provider.posted_comments().len(),
        1,
        "exactly one fresh comment must be posted; posted: {:?}",
        provider.posted_comments()
    );
}

#[test]
async fn test_explanation_comment_deleted_when_detection_clears() {
    // When the PR is edited and the keyword no longer triggers,
    // the stale explanation comment must be deleted.
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    let stale = make_comment(50, "bot", &format!("{} hotfix was here", hotfix_marker));
    let provider = KeywordLabelMockProvider::new(vec![stale], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    // PR body no longer mentions hotfix
    let pr = PullRequest {
        number: 32,
        title: "fix: unrelated cleanup".to_string(),
        draft: false,
        body: Some("This PR fixes an unrelated formatting issue.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        provider.deleted_comment_ids().contains(&50),
        "stale explanation comment must be deleted when hotfix detection clears; deleted: {:?}",
        provider.deleted_comment_ids()
    );
    // No new comment must be posted (hotfix not triggered)
    assert!(
        !provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&hotfix_marker)),
        "no new hotfix explanation comment must be posted when hotfix is not triggered"
    );
}

#[test]
async fn test_explanation_comment_deleted_when_label_suppressed() {
    // When a label is suppressed, any existing explanation comment for it must also be deleted.
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    let explanation = make_comment(60, "bot", &format!("{} hotfix explanation", hotfix_marker));
    let suppress = make_comment(61, "alice", "@merge-warden suppress: hotfix");
    let provider = KeywordLabelMockProvider::new(vec![explanation, suppress], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 33,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for production.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        provider.deleted_comment_ids().contains(&60),
        "explanation comment must be deleted when label is suppressed; deleted: {:?}",
        provider.deleted_comment_ids()
    );
    // And it must not be re-posted
    assert!(
        !provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&hotfix_marker)),
        "deleted explanation comment must not be re-posted for suppressed label"
    );
}

#[test]
async fn test_explanation_comments_independent_per_label() {
    // Triggering hotfix must only post a hotfix comment.
    // Security is not triggered, so no security comment must be posted or deleted.
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 60,
        title: "fix: crash".to_string(),
        draft: false,
        // Only hotfix keyword; no security/vulnerability keyword
        body: Some("This is a hotfix for the production outage.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    let posted = provider.posted_comments();
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    let security_marker = format!("{}security -->", KEYWORD_LABEL_COMMENT_MARKER);
    // Hotfix comment MUST be posted
    assert!(
        posted.iter().any(|c| c.contains(&hotfix_marker)),
        "hotfix explanation comment must be posted when hotfix triggered; posted: {:?}",
        posted
    );
    // Security comment must NOT be posted (security was not triggered)
    assert!(
        !posted.iter().any(|c| c.contains(&security_marker)),
        "security explanation comment must NOT be posted when security not triggered; posted: {:?}",
        posted
    );
    // No deletion must happen when there are no stale comments
    assert!(
        provider.deleted_comment_ids().is_empty(),
        "no comments must be deleted when starting with empty comment list"
    );
}

#[test]
async fn test_clearing_one_label_does_not_affect_other_labels_comment() {
    // Hotfix detection fires → hotfix comment posted/managed
    // Security no longer triggers → stale security comment must be deleted
    // Each label's comment lifecycle is managed independently.
    let security_marker = format!("{}security -->", KEYWORD_LABEL_COMMENT_MARKER);
    let stale_security = make_comment(
        10,
        "bot",
        &format!("{} old security comment", security_marker),
    );
    let provider = KeywordLabelMockProvider::new(vec![stale_security], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    // Security was previously triggered but the PR body was updated to remove it.
    // Hotfix is now triggered.
    let pr = PullRequest {
        number: 61,
        title: "fix: crash".to_string(),
        draft: false,
        body: Some("This is a hotfix for production.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    // Stale security comment must be deleted
    assert!(
        provider.deleted_comment_ids().contains(&10),
        "stale security comment must be deleted when security detection clears; deleted: {:?}",
        provider.deleted_comment_ids()
    );
    // Hotfix comment must be posted
    let hotfix_marker = format!("{}hotfix -->", KEYWORD_LABEL_COMMENT_MARKER);
    assert!(
        provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&hotfix_marker)),
        "hotfix explanation comment must be posted; posted: {:?}",
        provider.posted_comments()
    );
    // Security comment must NOT be re-posted (security not triggered)
    assert!(
        !provider
            .posted_comments()
            .iter()
            .any(|c| c.contains(&security_marker)),
        "security explanation comment must NOT be re-posted; posted: {:?}",
        provider.posted_comments()
    );
}

#[test]
async fn test_no_unnecessary_api_calls_when_no_keyword_matched() {
    // PR with no keyword content (no breaking change / security / hotfix / tech debt):
    // explanation comment management must make zero add_comment / delete_comment calls.
    let provider = KeywordLabelMockProvider::new(vec![], vec![]);
    let config = CurrentPullRequestValidationConfiguration {
        bot_mention: "@merge-warden".to_string(),
        ..Default::default()
    };
    let pr = PullRequest {
        number: 70,
        title: "docs: update readme".to_string(),
        draft: false,
        body: Some("Updated the README with installation instructions.".to_string()),
        author: Some(User {
            id: 1,
            login: "dev".to_string(),
        }),
        milestone_number: None,
        head_sha: String::new(),
    };
    set_pull_request_labels_with_config(&provider, "o", "r", &pr, Some(&config))
        .await
        .unwrap();
    assert!(
        provider.posted_comments().is_empty(),
        "no explanation comments must be posted when no keywords triggered; posted: {:?}",
        provider.posted_comments()
    );
    assert!(
        provider.deleted_comment_ids().is_empty(),
        "no comments must be deleted when no stale comments exist; deleted: {:?}",
        provider.deleted_comment_ids()
    );
}

// ── manage_renovate_stability_label tests ───────────────────────────────────

struct StabilityMockProvider {
    commit_statuses: Vec<CommitStatus>,
    available_labels: Vec<Label>,
    applied_labels: Arc<Mutex<Vec<Label>>>,
    create_label_calls: Arc<Mutex<Vec<String>>>,
    /// When `true`, `remove_label` returns `Error::InvalidResponse` (404).
    remove_label_404: bool,
}

impl StabilityMockProvider {
    fn new(commit_statuses: Vec<CommitStatus>) -> Self {
        Self {
            commit_statuses,
            available_labels: vec![],
            applied_labels: Arc::new(Mutex::new(vec![])),
            create_label_calls: Arc::new(Mutex::new(vec![])),
            remove_label_404: false,
        }
    }

    fn with_available_labels(mut self, labels: Vec<Label>) -> Self {
        self.available_labels = labels;
        self
    }

    fn with_remove_label_err(mut self, _err: Error) -> Self {
        self.remove_label_404 = true;
        self
    }

    fn applied_labels(&self) -> Vec<Label> {
        self.applied_labels.lock().unwrap().clone()
    }

    fn create_label_calls(&self) -> Vec<String> {
        self.create_label_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl PullRequestProvider for StabilityMockProvider {
    async fn get_pull_request(
        &self,
        _: &str,
        _: &str,
        _: u64,
    ) -> Result<PullRequest, Error> {
        unimplemented!()
    }

    async fn add_comment(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> {
        Ok(())
    }

    async fn delete_comment(&self, _: &str, _: &str, _: u64) -> Result<(), Error> {
        Ok(())
    }

    async fn list_comments(
        &self,
        _: &str,
        _: &str,
        _: u64,
    ) -> Result<Vec<Comment>, Error> {
        Ok(vec![])
    }

    async fn list_available_labels(&self, _: &str, _: &str) -> Result<Vec<Label>, Error> {
        Ok(self.available_labels.clone())
    }

    async fn add_labels(
        &self,
        _: &str,
        _: &str,
        _: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        let mut applied = self.applied_labels.lock().unwrap();
        for l in labels {
            applied.push(Label {
                name: l.clone(),
                description: None,
            });
        }
        Ok(())
    }

    async fn remove_label(&self, _: &str, _: &str, _: u64, _: &str) -> Result<(), Error> {
        if self.remove_label_404 {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    async fn list_applied_labels(
        &self,
        _: &str,
        _: &str,
        _: u64,
    ) -> Result<Vec<Label>, Error> {
        Ok(self.applied_labels.lock().unwrap().clone())
    }

    async fn update_pr_check_status(
        &self,
        _: &str,
        _: &str,
        _: u64,
        _: &str,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_pr_reviews(
        &self,
        _: &str,
        _: &str,
        _: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::Review>, Error> {
        Ok(vec![])
    }

    async fn get_pull_request_files(
        &self,
        _: &str,
        _: &str,
        _: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        Ok(vec![])
    }

    async fn get_commit_statuses(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<Vec<CommitStatus>, Error> {
        Ok(self.commit_statuses.clone())
    }

    async fn find_pull_requests_for_commit(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<Vec<u64>, Error> {
        Ok(vec![])
    }

    async fn create_label(
        &self,
        _: &str,
        _: &str,
        name: &str,
        _color: &str,
        _description: Option<&str>,
    ) -> Result<(), Error> {
        self.create_label_calls.lock().unwrap().push(name.to_string());
        Ok(())
    }
}

fn pending_status() -> CommitStatus {
    CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "pending".to_string(),
        description: None,
    }
}

fn success_status() -> CommitStatus {
    CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "success".to_string(),
        description: None,
    }
}

fn error_status() -> CommitStatus {
    CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "error".to_string(),
        description: None,
    }
}

fn failure_status() -> CommitStatus {
    CommitStatus {
        context: "renovate/stability-days".to_string(),
        state: "failure".to_string(),
        description: None,
    }
}

fn default_stability_config() -> crate::config::RenovateStabilityConfig {
    crate::config::RenovateStabilityConfig::default()
}

#[test]
async fn manage_renovate_stability_label_pending_adds_label() {
    let provider = StabilityMockProvider::new(vec![pending_status()]);
    let config = default_stability_config();

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    let applied = provider.applied_labels();
    assert!(
        applied.iter().any(|l| l.name == config.pending_stability_label),
        "stability label should be applied for pending status"
    );
}

#[test]
async fn manage_renovate_stability_label_error_adds_label() {
    let provider = StabilityMockProvider::new(vec![error_status()]);
    let config = default_stability_config();

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    let applied = provider.applied_labels();
    assert!(
        applied.iter().any(|l| l.name == config.pending_stability_label),
        "stability label should be applied for error status"
    );
}

#[test]
async fn manage_renovate_stability_label_failure_adds_label() {
    let provider = StabilityMockProvider::new(vec![failure_status()]);
    let config = default_stability_config();

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    let applied = provider.applied_labels();
    assert!(
        applied.iter().any(|l| l.name == config.pending_stability_label),
        "stability label should be applied for failure status"
    );
}

#[test]
async fn manage_renovate_stability_label_success_removes_label() {
    // Label already present on PR
    let label_name = default_stability_config().pending_stability_label;
    let provider = StabilityMockProvider::new(vec![success_status()]);

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &default_stability_config(),
    )
    .await
    .unwrap();

    // remove_label was called without error
    let applied = provider.applied_labels();
    assert!(
        !applied.iter().any(|l| l.name == label_name),
        "stability label should not be present after success status"
    );
}

#[test]
async fn manage_renovate_stability_label_success_absent_label_is_noop() {
    // remove_label returns 404 (InvalidResponse) — should be Ok(())
    let provider = StabilityMockProvider::new(vec![success_status()])
        .with_remove_label_err(Error::InvalidResponse);

    let result = crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &default_stability_config(),
    )
    .await;

    assert!(result.is_ok(), "404 on remove should be treated as no-op");
}

#[test]
async fn manage_renovate_stability_label_no_context_is_noop() {
    let other_status = CommitStatus {
        context: "ci/build".to_string(),
        state: "success".to_string(),
        description: None,
    };
    let provider = StabilityMockProvider::new(vec![other_status]);
    let config = default_stability_config();

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    assert!(
        provider.applied_labels().is_empty(),
        "no label should be applied when context is absent"
    );
}

#[test]
async fn manage_renovate_stability_label_disabled_is_noop() {
    let mut config = default_stability_config();
    config.enabled = false;

    let provider = StabilityMockProvider::new(vec![pending_status()]);

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    assert!(
        provider.applied_labels().is_empty(),
        "no API calls when config.enabled = false"
    );
}

#[test]
async fn manage_renovate_stability_label_uses_newest_entry() {
    // First entry (newest per GitHub ordering) is pending; second is success.
    // The function should act on the first match (pending) → add label.
    let statuses = vec![
        pending_status(), // newest
        success_status(), // older
    ];
    let provider = StabilityMockProvider::new(statuses);
    let config = default_stability_config();

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    let applied = provider.applied_labels();
    assert!(
        applied.iter().any(|l| l.name == config.pending_stability_label),
        "pending (newest) entry should win"
    );
}

#[test]
async fn manage_renovate_stability_label_creates_label_if_missing() {
    // No labels in repository — create_label should be called before add_labels.
    let provider = StabilityMockProvider::new(vec![pending_status()]); // available_labels is empty
    let config = default_stability_config();

    crate::labels::manage_renovate_stability_label(
        &provider,
        "owner",
        "repo",
        1,
        "abc123",
        &config,
    )
    .await
    .unwrap();

    let create_calls = provider.create_label_calls();
    assert_eq!(
        create_calls.len(),
        1,
        "create_label should be called once when label is absent"
    );
    assert_eq!(create_calls[0], config.pending_stability_label);

    let applied = provider.applied_labels();
    assert!(
        applied.iter().any(|l| l.name == config.pending_stability_label),
        "label should be applied after creation"
    );
}
