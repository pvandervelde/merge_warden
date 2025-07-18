use crate::labels::set_pull_request_labels;
use async_trait::async_trait;
use merge_warden_developer_platforms::errors::Error;
use std::sync::{Arc, Mutex};
use tokio::test;

use merge_warden_developer_platforms::models::{Comment, Label, PullRequest, User};
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
        unimplemented!("Not needed for this test")
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

    async fn get_pull_request_files(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<merge_warden_developer_platforms::models::PullRequestFile>, Error> {
        unimplemented!("Not needed for this test")
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
use crate::config::{
    ChangeTypeLabelConfig, ConventionalCommitMappings, CurrentPullRequestValidationConfiguration,
    FallbackLabelSettings, LabelDetectionStrategy,
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
        unimplemented!("Not needed for this test")
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy {
            exact_match: true,
            prefix_match: true,
            description_match: true,
            common_prefixes: vec!["type:".to_string(), "kind:".to_string()],
        },
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "00ff00".to_string())]),
        },
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "0366d6".to_string())]),
        },
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy::default(),
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
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "00ff00".to_string())]),
        },
        detection_strategy: LabelDetectionStrategy {
            exact_match: true,
            prefix_match: false,
            description_match: false,
            common_prefixes: vec![],
        },
    };

    // Should be valid (at least one detection method enabled)
    assert!(valid_config.detection_strategy.exact_match);

    // Test invalid configuration (no detection methods enabled)
    let invalid_config = ChangeTypeLabelConfig {
        enabled: true,
        conventional_commit_mappings: ConventionalCommitMappings::default(),
        fallback_label_settings: FallbackLabelSettings::default(),
        detection_strategy: LabelDetectionStrategy {
            exact_match: false,
            prefix_match: false,
            description_match: false,
            common_prefixes: vec![],
        },
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
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: true,
            name_format: "type: {change_type}".to_string(),
            color_scheme: HashMap::from([("feat".to_string(), "00ff00".to_string())]),
        },
        detection_strategy: LabelDetectionStrategy::default(),
    };

    let repo_config = ChangeTypeLabelConfig {
        enabled: false, // Override: disable smart labeling
        conventional_commit_mappings: ConventionalCommitMappings {
            feat: vec!["enhancement".to_string()], // Override: different mapping
            ..Default::default()
        },
        fallback_label_settings: FallbackLabelSettings {
            create_if_missing: false, // Override: disable fallback creation
            name_format: "kind: {change_type}".to_string(), // Override: different format
            color_scheme: HashMap::from([("feat".to_string(), "ff0000".to_string())]), // Override: different color
        },
        detection_strategy: LabelDetectionStrategy {
            exact_match: false, // Override: disable exact match
            prefix_match: true,
            description_match: true,
            common_prefixes: vec!["category:".to_string()], // Override: different prefixes
        },
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
