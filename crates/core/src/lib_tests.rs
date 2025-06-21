use crate::{
    config::{BypassRules, CurrentPullRequestValidationConfiguration},
    config::{
        MISSING_WORK_ITEM_LABEL, TITLE_COMMENT_MARKER, TITLE_INVALID_LABEL,
        WORK_ITEM_COMMENT_MARKER,
    },
    MergeWarden,
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::test;

use merge_warden_developer_platforms::models::{Comment, Label, PullRequest};
use merge_warden_developer_platforms::PullRequestProvider;
use merge_warden_developer_platforms::{errors::Error, models::User};

// Mock implementation of PullRequestProvider for testing
#[derive(Debug)]
struct ErrorMockGitProvider {
    error_on_get_pr: bool,
    error_on_add_labels: bool,
    error_on_add_comment: bool,
    invalid_pr_title: bool,
    invalid_pr_body: bool,
}

impl ErrorMockGitProvider {
    fn new() -> Self {
        Self {
            error_on_get_pr: false,
            error_on_add_labels: false,
            error_on_add_comment: false,
            invalid_pr_body: false,
            invalid_pr_title: false,
        }
    }

    fn with_add_comment_error(&mut self) {
        self.error_on_add_comment = true;
    }

    fn with_add_labels_error(&mut self) {
        self.error_on_add_labels = true;
    }

    fn with_get_pr_error(&mut self) {
        self.error_on_get_pr = true;
    }

    fn with_invalid_pr_body(&mut self) {
        self.invalid_pr_body = true;
    }

    fn with_invalid_pr_title(&mut self) {
        self.invalid_pr_title = true;
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
        if self.error_on_get_pr {
            Err(Error::ApiError())
        } else {
            let title = if self.invalid_pr_title {
                "test"
            } else {
                "feat: test"
            };

            let body = if self.invalid_pr_body {
                "Fixes stuff"
            } else {
                "Fixes #123"
            };

            Ok(PullRequest {
                number: 1,
                title: title.to_string(),
                draft: false,
                body: Some(body.to_string()),
                author: Some(User {
                    id: 456,
                    login: "developer123".to_string(),
                }),
            })
        }
    }

    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        if self.error_on_add_comment {
            Err(Error::FailedToUpdatePullRequest(
                "Failed to add comment".to_string(),
            ))
        } else {
            Ok(())
        }
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
        Ok(Vec::new())
    }

    async fn add_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _labels: &[String],
    ) -> Result<(), Error> {
        if self.error_on_add_labels {
            Err(Error::FailedToUpdatePullRequest(
                "Failed to add labels".to_string(),
            ))
        } else {
            Ok(())
        }
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
        Ok(())
    }
}

// Mock implementation of PullRequestProvider that returns different PRs based on PR number
#[derive(Debug)]
struct DynamicMockGitProvider {
    pull_requests: HashMap<u64, PullRequest>,
    labels: Arc<Mutex<Vec<Label>>>,
    comments: Arc<Mutex<Vec<Comment>>>,
}

impl DynamicMockGitProvider {
    fn new() -> Self {
        Self {
            pull_requests: HashMap::new(),
            labels: Arc::new(Mutex::new(Vec::new())),
            comments: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_pull_request(&mut self, pr: PullRequest) {
        self.pull_requests.insert(pr.number, pr);
    }

    fn get_labels(&self) -> Vec<Label> {
        let labels = self.labels.lock().unwrap().clone();
        labels
    }

    fn get_comments(&self) -> Vec<Comment> {
        let comments = self.comments.lock().unwrap().clone();
        comments
    }
}

#[async_trait]
impl PullRequestProvider for DynamicMockGitProvider {
    async fn get_pull_request(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        pr_number: u64,
    ) -> Result<PullRequest, Error> {
        match self.pull_requests.get(&pr_number) {
            Some(pr) => Ok(pr.clone()),
            None => Err(Error::InvalidResponse),
        }
    }

    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        comment: &str,
    ) -> Result<(), Error> {
        let mut comments = self.comments.lock().unwrap();
        let number_of_comments = comments.len() as u64;
        comments.push(Comment {
            id: number_of_comments + 1,
            body: comment.to_string(),
            user: User {
                id: 10,
                login: "a".to_string(),
            },
        });
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error> {
        let mut comments = self.comments.lock().unwrap();
        comments.retain(|c| c.id != comment_id);
        Ok(())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        let comments = self.comments.lock().unwrap();
        Ok(comments.clone())
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
        label: &str,
    ) -> Result<(), Error> {
        let mut current_labels = self.labels.lock().unwrap();
        current_labels.retain(|l| l.name != label);
        Ok(())
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
        Ok(())
    }
}

// Mock implementation of PullRequestProvider for testing
#[derive(Debug)]
struct MockGitProvider {
    pull_request: Arc<Mutex<Option<PullRequest>>>,
    labels: Arc<Mutex<Vec<Label>>>,
    comments: Arc<Mutex<Vec<Comment>>>,
}

impl MockGitProvider {
    fn new() -> Self {
        Self {
            pull_request: Arc::new(Mutex::new(None)),
            labels: Arc::new(Mutex::new(Vec::new())),
            comments: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_pull_request(&self, pr: PullRequest) {
        let mut pull_request = self.pull_request.lock().unwrap();
        *pull_request = Some(pr);
    }

    fn get_labels(&self) -> Vec<Label> {
        let labels = self.labels.lock().unwrap().clone();
        labels
    }

    fn get_comments(&self) -> Vec<Comment> {
        let comments = self.comments.lock().unwrap().clone();
        comments
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
        let pull_request = self.pull_request.lock().unwrap();
        match &*pull_request {
            Some(pr) => Ok(pr.clone()),
            None => panic!("Pull request not set"),
        }
    }

    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        comment: &str,
    ) -> Result<(), Error> {
        let mut comments = self.comments.lock().unwrap();
        let number_of_comments = comments.len() as u64;
        comments.push(Comment {
            id: number_of_comments + 1,
            body: comment.to_string(),
            user: User {
                id: 10,
                login: "a".to_string(),
            },
        });
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error> {
        let mut comments = self.comments.lock().unwrap();
        comments.retain(|c| c.id != comment_id);
        Ok(())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        let comments = self.comments.lock().unwrap();
        Ok(comments.clone())
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
        label: &str,
    ) -> Result<(), Error> {
        let mut current_labels = self.labels.lock().unwrap();
        current_labels.retain(|l| l.name != label);
        Ok(())
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
        Ok(())
    }
}

#[test]
async fn test_constructor_new() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Create a MergeWarden instance with default config
    let warden = MergeWarden::new(provider);

    // Verify the default configuration
    assert!(
        warden.config.enforce_title_convention,
        "Default config should enforce conventional commits"
    );
    assert!(
        warden.config.enforce_work_item_references,
        "Default config should require work item references"
    );
}

#[test]
async fn test_constructor_with_config() {
    // Create a mock provider
    let provider = MockGitProvider::new(); // Create a custom configuration
    let config = CurrentPullRequestValidationConfiguration {
        enforce_title_convention: false,
        title_pattern: "ab".to_string(),
        invalid_title_label: None,
        enforce_work_item_references: true,
        work_item_reference_pattern: "cd".to_string(),
        missing_work_item_label: None,
        bypass_rules: BypassRules::default(),
    };

    // Create a MergeWarden instance with custom config
    let warden = MergeWarden::with_config(provider, config);

    // Verify the custom configuration
    assert!(
        !warden.config.enforce_title_convention,
        "Custom config should not enforce conventional commits"
    );
    assert!(
        warden.config.enforce_work_item_references,
        "Custom config should require work item references"
    );
}

#[test]
async fn test_process_pull_request_valid() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Set up a valid PR
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("Fixes #123".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };
    provider.set_pull_request(pr);

    // Create a MergeWarden instance with default config
    let warden = MergeWarden::new(provider);

    // Process the PR
    let result = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result
    assert!(result.title_valid, "Title should be valid");
    assert!(
        result.work_item_referenced,
        "Work item should be referenced"
    );

    // Verify no labels were added
    let labels = warden.provider.get_labels();
    assert!(
        !labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should not be added"
    );
    assert!(
        !labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should not be added"
    );

    // Verify no comments were added
    let comments = warden.provider.get_comments();
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(TITLE_COMMENT_MARKER)),
        "Title comment should not be added"
    );
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(WORK_ITEM_COMMENT_MARKER)),
        "Work item comment should not be added"
    );
}

#[test]
async fn test_process_pull_request_invalid_title() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Set up a PR with invalid title but valid work item reference
    let pr = PullRequest {
        number: 1,
        title: "invalid title".to_string(), // Missing conventional commit format
        draft: false,
        body: Some("Fixes #123".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };
    provider.set_pull_request(pr);

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR
    let result = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result
    assert!(!result.title_valid, "Title should be invalid");
    assert!(
        result.work_item_referenced,
        "Work item should be referenced"
    );

    // Verify the invalid title label was added
    let labels = warden.provider.get_labels();
    assert!(
        labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should be added"
    );

    // Verify the title comment was added
    let comments = warden.provider.get_comments();
    assert!(
        comments
            .iter()
            .any(|c| c.body.contains(TITLE_COMMENT_MARKER)),
        "Title comment should be added"
    );
}

#[test]
async fn test_process_pull_request_missing_work_item() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Set up a PR with valid title but missing work item reference
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("No work item reference".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };
    provider.set_pull_request(pr);

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR
    let result = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result
    assert!(result.title_valid, "Title should be valid");
    assert!(
        !result.work_item_referenced,
        "Work item should not be referenced"
    );

    // Verify the missing work item label was added
    let labels = warden.provider.get_labels();
    assert!(
        labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should be added"
    );

    // Verify the work item comment was added
    let comments = warden.provider.get_comments();
    assert!(
        comments
            .iter()
            .any(|c| c.body.contains(WORK_ITEM_COMMENT_MARKER)),
        "Work item comment should be added"
    );
}

#[test]
async fn test_process_pull_request_both_invalid() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Set up a PR with invalid title and missing work item reference
    let pr = PullRequest {
        number: 1,
        title: "invalid title".to_string(), // Missing conventional commit format
        draft: false,
        body: Some("No work item reference".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };
    provider.set_pull_request(pr);

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR
    let result = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result
    assert!(!result.title_valid, "Title should be invalid");
    assert!(
        !result.work_item_referenced,
        "Work item should not be referenced"
    );

    // Verify both labels were added
    let labels = warden.provider.get_labels();
    assert!(
        labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should be added"
    );
    assert!(
        labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should be added"
    );

    // Verify both comments were added
    let comments = warden.provider.get_comments();
    assert!(
        comments
            .iter()
            .any(|c| c.body.contains(TITLE_COMMENT_MARKER)),
        "Title comment should be added"
    );
    assert!(
        comments
            .iter()
            .any(|c| c.body.contains(WORK_ITEM_COMMENT_MARKER)),
        "Work item comment should be added"
    );
}

#[test]
async fn test_handle_title_validation_invalid_to_valid() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Add the invalid title label
    provider
        .add_labels("owner", "repo", 1, &[TITLE_INVALID_LABEL.to_string()])
        .await
        .unwrap();

    // Add a title comment
    let comment = format!(
            "{}\n## Invalid PR Title Format\n\nYour PR title doesn't follow the Conventional Commits format.",
            TITLE_COMMENT_MARKER
        );
    provider
        .add_comment("owner", "repo", 1, &comment)
        .await
        .unwrap();

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Set up a PR
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("Test body".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    // Handle title validation with valid title
    warden
        .communicate_pr_title_validity_status("owner", "repo", &pr, true)
        .await;

    // Verify the label was removed
    let labels = warden.provider.get_labels();
    assert!(
        !labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should be removed"
    );

    // Verify the comment was removed
    let comments = warden.provider.get_comments();
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(TITLE_COMMENT_MARKER)),
        "Title comment should be removed"
    );
}

#[test]
async fn test_process_pull_request_custom_config_disabled_checks() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Set up a PR with invalid title and missing work item reference
    let pr = PullRequest {
        number: 1,
        title: "invalid title".to_string(), // Missing conventional commit format
        draft: false,
        body: Some("No work item reference".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };
    provider.set_pull_request(pr); // Create a custom configuration with disabled checks
    let config = CurrentPullRequestValidationConfiguration {
        enforce_title_convention: false,
        title_pattern: "ab".to_string(),
        invalid_title_label: None,
        enforce_work_item_references: false,
        work_item_reference_pattern: "cd".to_string(),
        missing_work_item_label: None,
        bypass_rules: BypassRules::default(),
    };

    // Create a MergeWarden instance with custom config
    let warden = MergeWarden::with_config(provider, config);

    // Process the PR
    let result = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result - both should be valid since checks are disabled
    assert!(
        result.title_valid,
        "Title should be valid when check is disabled"
    );
    assert!(
        result.work_item_referenced,
        "Work item should be referenced when check is disabled"
    );

    // Verify no labels were added
    let labels = warden.provider.get_labels();
    assert!(
        !labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should not be added when check is disabled"
    );
    assert!(
        !labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should not be added when check is disabled"
    );

    // Verify no comments were added
    let comments = warden.provider.get_comments();
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(TITLE_COMMENT_MARKER)),
        "Title comment should not be added when check is disabled"
    );
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(WORK_ITEM_COMMENT_MARKER)),
        "Work item comment should not be added when check is disabled"
    );
}

#[test]
async fn test_process_pull_request_existing_labels_comments() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Add existing labels and comments
    provider
        .add_labels(
            "owner",
            "repo",
            1,
            &[
                TITLE_INVALID_LABEL.to_string(),
                MISSING_WORK_ITEM_LABEL.to_string(),
                "feature".to_string(),
            ],
        )
        .await
        .unwrap();

    let title_comment = format!(
        "{}\n## Invalid PR Title Format\n\nYour PR title doesn't follow the Conventional Commits format.",
        TITLE_COMMENT_MARKER
    );
    provider
        .add_comment("owner", "repo", 1, &title_comment)
        .await
        .unwrap();

    let work_item_comment = format!(
        "{}\n## Missing Work Item Reference\n\nYour PR description doesn't reference a work item or GitHub issue.",
        WORK_ITEM_COMMENT_MARKER
    );
    provider
        .add_comment("owner", "repo", 1, &work_item_comment)
        .await
        .unwrap();

    // Set up a valid PR
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("Fixes #123".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };
    provider.set_pull_request(pr);

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR
    let result = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result
    assert!(result.title_valid, "Title should be valid");
    assert!(
        result.work_item_referenced,
        "Work item should be referenced"
    );

    // Verify the invalid labels were removed
    let labels = warden.provider.get_labels();
    assert!(
        !labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should be removed"
    );
    assert!(
        !labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should be removed"
    );

    // Verify the feature label remains
    assert!(
        labels.iter().any(|l| l.name == "feature"),
        "Feature label should remain"
    );

    // Verify the comments were removed
    let comments = warden.provider.get_comments();
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(TITLE_COMMENT_MARKER)),
        "Title comment should be removed"
    );
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(WORK_ITEM_COMMENT_MARKER)),
        "Work item comment should be removed"
    );
}

#[test]
async fn test_process_pull_request_dynamic_provider() {
    // Create a dynamic mock provider
    let mut provider = DynamicMockGitProvider::new();

    // Add two different PRs
    let valid_pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("Fixes #123".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    let invalid_pr = PullRequest {
        number: 2,
        title: "invalid title".to_string(),
        draft: false,
        body: Some("No work item reference".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    provider.add_pull_request(valid_pr);
    provider.add_pull_request(invalid_pr);

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the valid PR
    let result1 = warden
        .process_pull_request("owner", "repo", 1)
        .await
        .unwrap();

    // Verify the result for valid PR
    assert!(result1.title_valid, "Title should be valid for PR #1");
    assert!(
        result1.work_item_referenced,
        "Work item should be referenced for PR #1"
    );

    // Process the invalid PR
    let result2 = warden
        .process_pull_request("owner", "repo", 2)
        .await
        .unwrap();

    // Verify the result for invalid PR
    assert!(!result2.title_valid, "Title should be invalid for PR #2");
    assert!(
        !result2.work_item_referenced,
        "Work item should not be referenced for PR #2"
    );

    // Verify the labels were added
    let labels = warden.provider.get_labels();
    assert!(
        labels.iter().any(|l| l.name == TITLE_INVALID_LABEL),
        "Invalid title label should be added for PR #2"
    );
    assert!(
        labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should be added for PR #2"
    );
}

#[test]
async fn test_process_pull_request_error_add_comment() {
    // Create a mock provider that returns an error when adding a comment
    let mut provider = ErrorMockGitProvider::new();
    provider.with_invalid_pr_title();
    provider.with_add_comment_error();

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR - should return Ok even if adding a comment fails
    let result = warden.process_pull_request("owner", "repo", 1).await;

    // Verify the result is Ok (no error should be returned)
    assert!(
        result.is_ok(),
        "Should return Ok even when adding a comment fails"
    );
    // Optionally, check that the output string contains a warning or expected message
    let output = result.unwrap();
    assert!(!output.title_valid, "The title should not be valid");
}

#[test]
async fn test_process_pull_request_error_add_labels() {
    // Create a mock provider that returns an error when adding labels
    let mut provider = ErrorMockGitProvider::new();
    provider.with_add_labels_error();

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR - should return an error
    let result = warden.process_pull_request("owner", "repo", 1).await;

    // Verify the error
    assert!(
        result.is_err(),
        "Should return an error when adding labels fails"
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "Failed to update pull request. Issue was: 'Failed to add label'.",
        "Should return the specific error message"
    );
}

#[test]
async fn test_process_pull_request_error_get_pr() {
    // Create a mock provider that returns an error when getting a PR
    let mut provider = ErrorMockGitProvider::new();
    provider.with_get_pr_error();

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Process the PR - should return an error
    let result = warden.process_pull_request("owner", "repo", 1).await;

    // Verify the error
    assert!(
        result.is_err(),
        "Should return an error when getting a PR fails"
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "Git provider error: Failed to find the PR with number [1] in owner/repo",
        "Should return the specific error message"
    );
}

#[test]
async fn test_handle_work_item_validation_missing_to_present() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Add the missing work item label
    provider
        .add_labels("owner", "repo", 1, &[MISSING_WORK_ITEM_LABEL.to_string()])
        .await
        .unwrap();

    // Add a work item comment
    let comment = format!(
            "{}\n## Missing Work Item Reference\n\nYour PR description doesn't reference a work item or GitHub issue.",
            WORK_ITEM_COMMENT_MARKER
        );
    provider
        .add_comment("owner", "repo", 1, &comment)
        .await
        .unwrap();

    // Create a MergeWarden instance
    let warden = MergeWarden::new(provider);

    // Set up a PR
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        draft: false,
        body: Some("Fixes #123".to_string()),
        author: Some(User {
            id: 456,
            login: "developer123".to_string(),
        }),
    };

    // Handle work item validation with valid work item reference
    warden
        .communicate_pr_work_item_validity_status("owner", "repo", &pr, true)
        .await;

    // Verify the label was removed
    let labels = warden.provider.get_labels();
    assert!(
        !labels.iter().any(|l| l.name == MISSING_WORK_ITEM_LABEL),
        "Missing work item label should be removed"
    );

    // Verify the comment was removed
    let comments = warden.provider.get_comments();
    assert!(
        !comments
            .iter()
            .any(|c| c.body.contains(WORK_ITEM_COMMENT_MARKER)),
        "Work item comment should be removed"
    );
}
