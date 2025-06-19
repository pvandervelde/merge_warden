use crate::labels::set_pull_request_labels;
use async_trait::async_trait;
use merge_warden_developer_platforms::errors::Error;
use std::sync::{Arc, Mutex};
use tokio::test;

use merge_warden_developer_platforms::models::{Comment, Label, PullRequest};
use merge_warden_developer_platforms::PullRequestProvider;

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
        unimplemented!("Not needed for this test")
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _comment_id: u64,
    ) -> Result<(), Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        unimplemented!("Not needed for this test")
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
        unimplemented!("Not needed for this test")
    }

    async fn list_labels(
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
        unimplemented!("Not needed for this test")
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
        unimplemented!("Not needed for this test")
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _comment_id: u64,
    ) -> Result<(), Error> {
        unimplemented!("Not needed for this test")
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        unimplemented!("Not needed for this test")
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

    async fn list_labels(
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
}

#[test]
async fn test_determine_labels_breaking_change() {
    let provider = MockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat(api)!: change authentication flow".to_string(),
        draft: false,
        body: Some("This is a breaking change to the API".to_string()),
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
    };

    let labels = set_pull_request_labels(&provider, "owner", "repo", &pr)
        .await
        .unwrap();
    assert!(!labels.is_empty(), "Expected no labels for empty body");
}

// New test for error handling
#[test]
async fn test_determine_labels_error_handling() {
    let provider = ErrorMockGitProvider::new();
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("This is a new feature".to_string()),
    };

    let result = set_pull_request_labels(&provider, "owner", "repo", &pr).await;
    assert!(
        result.is_err(),
        "Expected an error when adding labels fails"
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "Failed to update pull request. Issue was: 'Failed to add label'.",
        "Expected specific error message"
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
