use super::*;
use crate::{
    models::{
        Comment, Label, PullRequest, MISSING_WORK_ITEM_LABEL, TITLE_COMMENT_MARKER,
        TITLE_INVALID_LABEL, WORK_ITEM_COMMENT_MARKER,
    },
    CheckResult, GitProvider, MergeWarden,
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::test;

// Mock implementation of GitProvider for testing
struct MockGitProvider {
    pull_request: Arc<Mutex<Option<PullRequest>>>,
    labels: Arc<Mutex<Vec<Label>>>,
    comments: Arc<Mutex<Vec<Comment>>>,
    pr_mergeable: Arc<Mutex<bool>>,
}

impl MockGitProvider {
    fn new() -> Self {
        Self {
            pull_request: Arc::new(Mutex::new(None)),
            labels: Arc::new(Mutex::new(Vec::new())),
            comments: Arc::new(Mutex::new(Vec::new())),
            pr_mergeable: Arc::new(Mutex::new(true)),
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

    fn is_mergeable(&self) -> bool {
        *self.pr_mergeable.lock().unwrap()
    }
}

#[async_trait]
impl GitProvider for MockGitProvider {
    async fn get_pull_request(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<PullRequest> {
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
    ) -> Result<()> {
        let mut comments = self.comments.lock().unwrap();
        comments.push(Comment {
            id: comments.len() as u64 + 1,
            body: comment.to_string(),
        });
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        comment_id: u64,
    ) -> Result<()> {
        let mut comments = self.comments.lock().unwrap();
        comments.retain(|c| c.id != comment_id);
        Ok(())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>> {
        let comments = self.comments.lock().unwrap();
        Ok(comments.clone())
    }

    async fn add_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        labels: &[String],
    ) -> Result<()> {
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
    ) -> Result<()> {
        let mut current_labels = self.labels.lock().unwrap();
        current_labels.retain(|l| l.name != label);
        Ok(())
    }

    async fn list_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Label>> {
        let labels = self.labels.lock().unwrap();
        Ok(labels.clone())
    }

    async fn update_pr_mergeable_state(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        mergeable: bool,
    ) -> Result<()> {
        let mut pr_mergeable = self.pr_mergeable.lock().unwrap();
        *pr_mergeable = mergeable;
        Ok(())
    }
}

#[test]
async fn test_process_pull_request_valid() {
    // Create a mock provider
    let provider = MockGitProvider::new();

    // Set up a valid PR
    let pr = PullRequest {
        number: 1,
        title: "feat: add new feature".to_string(),
        body: Some("Fixes #123".to_string()),
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

    // Verify the PR is mergeable
    assert!(warden.provider.is_mergeable(), "PR should be mergeable");

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
        body: Some("Fixes #123".to_string()),
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

    // Verify the PR is not mergeable
    assert!(
        !warden.provider.is_mergeable(),
        "PR should not be mergeable"
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
        body: Some("No work item reference".to_string()),
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

    // Verify the PR is not mergeable
    assert!(
        !warden.provider.is_mergeable(),
        "PR should not be mergeable"
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
        body: Some("No work item reference".to_string()),
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

    // Verify the PR is not mergeable
    assert!(
        !warden.provider.is_mergeable(),
        "PR should not be mergeable"
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
        body: Some("Test body".to_string()),
    };

    // Handle title validation with valid title
    warden
        .handle_title_validation("owner", "repo", &pr, true)
        .await
        .unwrap();

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
        body: Some("Fixes #123".to_string()),
    };

    // Handle work item validation with valid work item reference
    warden
        .handle_work_item_validation("owner", "repo", &pr, true)
        .await
        .unwrap();

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
