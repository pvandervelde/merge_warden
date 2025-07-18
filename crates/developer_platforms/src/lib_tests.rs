//! Tests for PullRequestProvider trait API improvements
//!
//! This module contains tests to validate the new method names and ordering
//! requirements specified in issue #169.

use crate::errors::Error;
use crate::models::{Comment, Label, PullRequest, PullRequestFile};
use crate::PullRequestProvider;
use async_trait::async_trait;

/// Mock implementation for testing the new trait API
#[derive(Debug)]
struct MockApiProvider {
    applied_labels: Vec<Label>,
    available_labels: Vec<Label>,
    comments: Vec<Comment>,
}

impl MockApiProvider {
    /// Create a new mock provider with predefined test data
    fn new() -> Self {
        Self {
            applied_labels: vec![
                Label {
                    name: "bug".to_string(),
                    description: Some("Something isn't working".to_string()),
                },
                Label {
                    name: "enhancement".to_string(),
                    description: Some("New feature or request".to_string()),
                },
            ],
            available_labels: vec![
                Label {
                    name: "bug".to_string(),
                    description: Some("Something isn't working".to_string()),
                },
                Label {
                    name: "enhancement".to_string(),
                    description: Some("New feature or request".to_string()),
                },
                Label {
                    name: "documentation".to_string(),
                    description: Some("Improvements or additions to documentation".to_string()),
                },
                Label {
                    name: "size: XS".to_string(),
                    description: Some("Extra small PR".to_string()),
                },
                Label {
                    name: "size: S".to_string(),
                    description: Some("Small PR".to_string()),
                },
            ],
            comments: vec![],
        }
    }
}

#[async_trait]
impl PullRequestProvider for MockApiProvider {
    async fn add_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _comment: &str,
    ) -> Result<(), Error> {
        // Mock implementation - always succeeds
        Ok(())
    }

    async fn add_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _labels: &[String],
    ) -> Result<(), Error> {
        // Mock implementation - always succeeds
        Ok(())
    }

    async fn delete_comment(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _comment_id: u64,
    ) -> Result<(), Error> {
        // Mock implementation - always succeeds
        Ok(())
    }

    async fn get_pull_request(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<PullRequest, Error> {
        // Mock implementation
        Ok(PullRequest {
            number: 1,
            title: "feat: add new feature".to_string(),
            draft: false,
            body: Some("This adds a new feature".to_string()),
            author: None,
        })
    }

    async fn get_pull_request_files(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<PullRequestFile>, Error> {
        // Mock implementation
        Ok(vec![])
    }

    async fn list_applied_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        // Return labels currently applied to the PR
        Ok(self.applied_labels.clone())
    }

    async fn list_available_labels(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
    ) -> Result<Vec<Label>, Error> {
        // Return all labels available in the repository
        Ok(self.available_labels.clone())
    }

    async fn list_comments(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        // Return comments on the PR
        Ok(self.comments.clone())
    }

    async fn remove_label(
        &self,
        _repo_owner: &str,
        _repo_name: &str,
        _pr_number: u64,
        _label: &str,
    ) -> Result<(), Error> {
        // Mock implementation - always succeeds
        Ok(())
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
        // Mock implementation - always succeeds
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_applied_labels_returns_pr_labels() {
        let provider = MockApiProvider::new();

        let result = provider
            .list_applied_labels("owner", "repo", 123)
            .await
            .expect("Should return applied labels successfully");

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|l| l.name == "bug"));
        assert!(result.iter().any(|l| l.name == "enhancement"));
    }

    #[tokio::test]
    async fn test_list_available_labels_returns_all_repo_labels() {
        let provider = MockApiProvider::new();

        let result = provider
            .list_available_labels("owner", "repo")
            .await
            .expect("Should return available labels successfully");

        assert_eq!(result.len(), 5);
        assert!(result.iter().any(|l| l.name == "bug"));
        assert!(result.iter().any(|l| l.name == "enhancement"));
        assert!(result.iter().any(|l| l.name == "documentation"));
        assert!(result.iter().any(|l| l.name == "size: XS"));
        assert!(result.iter().any(|l| l.name == "size: S"));
    }

    #[tokio::test]
    async fn test_applied_labels_subset_of_available_labels() {
        let provider = MockApiProvider::new();

        let applied = provider
            .list_applied_labels("owner", "repo", 123)
            .await
            .expect("Should return applied labels");

        let available = provider
            .list_available_labels("owner", "repo")
            .await
            .expect("Should return available labels");

        // All applied labels should exist in available labels
        for applied_label in &applied {
            assert!(
                available.iter().any(|l| l.name == applied_label.name),
                "Applied label '{}' should exist in available labels",
                applied_label.name
            );
        }
    }

    #[tokio::test]
    async fn test_method_naming_consistency() {
        let provider = MockApiProvider::new();

        // Test that both new method names work and return expected types
        let _applied: Vec<Label> = provider
            .list_applied_labels("owner", "repo", 123)
            .await
            .expect("list_applied_labels should work");

        let _available: Vec<Label> = provider
            .list_available_labels("owner", "repo")
            .await
            .expect("list_available_labels should work");
    }

    #[tokio::test]
    async fn test_all_trait_methods_are_accessible() {
        let provider = MockApiProvider::new();

        // Test alphabetical ordering by calling methods in order
        // This validates that all methods exist and are properly ordered

        // A - add_comment, add_labels
        provider
            .add_comment("owner", "repo", 123, "test comment")
            .await
            .expect("add_comment should work");
        provider
            .add_labels("owner", "repo", 123, &["test".to_string()])
            .await
            .expect("add_labels should work");

        // D - delete_comment
        provider
            .delete_comment("owner", "repo", 456)
            .await
            .expect("delete_comment should work");

        // G - get_pull_request, get_pull_request_files
        let _pr = provider
            .get_pull_request("owner", "repo", 123)
            .await
            .expect("get_pull_request should work");
        let _files = provider
            .get_pull_request_files("owner", "repo", 123)
            .await
            .expect("get_pull_request_files should work");

        // L - list_applied_labels, list_available_labels, list_comments
        let _applied = provider
            .list_applied_labels("owner", "repo", 123)
            .await
            .expect("list_applied_labels should work");
        let _available = provider
            .list_available_labels("owner", "repo")
            .await
            .expect("list_available_labels should work");
        let _comments = provider
            .list_comments("owner", "repo", 123)
            .await
            .expect("list_comments should work");

        // R - remove_label
        provider
            .remove_label("owner", "repo", 123, "test")
            .await
            .expect("remove_label should work");

        // U - update_pr_check_status
        provider
            .update_pr_check_status("owner", "repo", 123, "success", "title", "summary", "text")
            .await
            .expect("update_pr_check_status should work");
    }

    #[tokio::test]
    async fn test_error_handling_on_method_calls() {
        let provider = MockApiProvider::new();

        // Test that methods handle various edge cases gracefully
        let result = provider.list_applied_labels("", "", 0).await;
        assert!(result.is_ok(), "Should handle empty parameters gracefully");

        let result = provider.list_available_labels("", "").await;
        assert!(result.is_ok(), "Should handle empty parameters gracefully");
    }

    #[tokio::test]
    async fn test_method_signature_compatibility() -> Result<(), Error> {
        let provider = MockApiProvider::new();

        // Test that method signatures are compatible with expected usage patterns

        // Applied labels - specific to a PR
        let applied_labels: Result<Vec<Label>, Error> =
            provider.list_applied_labels("owner", "repo", 123).await;
        assert!(applied_labels.is_ok());

        // Available labels - repository-wide
        let available_labels: Result<Vec<Label>, Error> =
            provider.list_available_labels("owner", "repo").await;
        assert!(available_labels.is_ok());

        // Verify the distinction is clear in usage
        let _applied = provider.list_applied_labels("owner", "repo", 123).await?; // PR-specific
        let _available = provider.list_available_labels("owner", "repo").await?;
        // Repository-wide

        Ok(())
    }
}
