//! Complete pull request validation workflow integration test.
//!
//! This test validates the entire end-to-end workflow from webhook reception
//! through validation processing to final status updates and notifications.
//!
//! Covers behavioral assertions #1, #2, #3, #7, and #10 from the specification.

use std::time::{Duration, Instant};
use tokio::time::timeout;

use merge_warden_integration_tests::{
    environment::IntegrationTestEnvironment,
    utils::{
        test_data::{FileAction, FileSpec},
        PullRequestSpec,
    },
    TestError, TestResult,
};

/// Complete pull request validation workflow integration test.
///
/// This test validates the entire end-to-end workflow from webhook reception
/// through validation processing to final status updates and notifications.
///
/// # Test Scenario Coverage
///
/// 1. **Repository Setup**: Creates test repository with proper configuration
/// 2. **Bot Configuration**: Installs GitHub App and configures webhooks  
/// 3. **Pull Request Creation**: Creates realistic PR with file changes
/// 4. **Webhook Processing**: Simulates webhook delivery and processing
/// 5. **Validation Execution**: Validates PR against configured policies
/// 6. **Labeling System**: Verifies size and validation labels are applied
/// 7. **Comment Generation**: Checks for appropriate bot comments
/// 8. **Status Checks**: Validates GitHub status check updates
/// 9. **Performance Validation**: Ensures processing within 15 seconds
/// 10. **Resource Cleanup**: Ensures proper cleanup of all resources
///
/// # Behavioral Assertions Tested
///
/// - Integration test framework must successfully create test repositories (Assertion #1)
/// - Bot instance configuration must complete webhook setup within 30 seconds (Assertion #2) 
/// - Pull request validation workflow must process webhooks within 15 seconds (Assertion #3)
/// - All test resources must be properly cleaned up (Assertion #7)
/// - Mock Azure services must accurately simulate real behavior (Assertion #10)
#[tokio::test]
async fn test_complete_pull_request_validation_workflow() -> TestResult<()> {
    // Arrange: Set up complete test environment
    let mut test_env = IntegrationTestEnvironment::setup().await?;

    // Create test repository with merge-warden configuration - Assertion #1
    let repo = test_env
        .create_test_repository("pr-validation-workflow")
        .await?;

    test_env.setup_repository_configuration(&repo).await?;

    // Configure bot instance with webhook endpoints - Assertion #2
    let bot_setup_start = Instant::now();
    
    test_env.bot_instance.setup_local_tunnel().await?;
    test_env.bot_instance.configure_for_repository(&repo).await?;
    
    let bot_setup_duration = bot_setup_start.elapsed();
    assert!(
        bot_setup_duration <= Duration::from_secs(30),
        "Bot setup took {:?}, must complete within 30 seconds",
        bot_setup_duration
    );

    // Add initial repository content for realistic testing
    test_env.add_default_repository_content(&repo).await?;

    // Create pull request specification with realistic changes
    let pr_spec = PullRequestSpec {
        title: "feat: add new validation feature".to_string(),
        body: "This PR implements a new validation rule for enhanced security.\n\nFixes #123".to_string(),
        source_branch: "feature/new-validation".to_string(),
        target_branch: "main".to_string(),
        files: vec![
            FileSpec {
                path: "src/validation.rs".to_string(),
                content: generate_rust_validation_file()?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
            FileSpec {
                path: "README.md".to_string(),
                content: generate_updated_readme()?,
                action: FileAction::Modify,
                mime_type: Some("text/markdown".to_string()),
            },
            FileSpec {
                path: "tests/validation_tests.rs".to_string(),
                content: generate_test_file()?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
        ],
        labels: vec!["enhancement".to_string(), "validation".to_string()],
        draft: false,
        assignees: vec!["test-assignee".to_string()],
        reviewers: vec!["test-reviewer".to_string()],
    };

    // Act: Create pull request and trigger webhook processing
    let pr = create_pull_request_in_repository(&test_env, &repo, &pr_spec).await?;

    // Measure webhook processing time - Assertion #3
    let processing_start = Instant::now();

    // Simulate webhook delivery for pull request opened event
    let webhook_payload = create_pr_opened_webhook_payload(&repo, &pr, &pr_spec)?;
    
    let webhook_response = timeout(
        Duration::from_secs(15),
        test_env.bot_instance.simulate_webhook("pull_request", &webhook_payload)
    ).await.map_err(|_| TestError::timeout("webhook processing", Duration::from_secs(15)))??;

    let processing_duration = processing_start.elapsed();

    // Assert: Verify webhook processing performance
    assert!(
        processing_duration <= Duration::from_secs(15),
        "Webhook processing took {:?}, must complete within 15 seconds",
        processing_duration
    );

    assert!(
        webhook_response.status_code >= 200 && webhook_response.status_code < 300,
        "Webhook should return success status, got {}",
        webhook_response.status_code
    );

    // Wait for async processing to complete
    wait_for_processing_completion(&test_env, &repo, pr.number).await?;

    // Assert: Verify bot comments were posted
    let comments = test_env.get_pr_comments(&repo, pr.number).await?;
    
    assert!(
        !comments.is_empty(),
        "Bot should have posted at least one comment"
    );

    // Verify validation summary comment exists
    assert!(
        comments.iter().any(|c| c.body.contains("Pull Request Validation Summary")),
        "Bot should post validation summary comment"
    );

    // Assert: Verify appropriate labels were applied  
    let labels = test_env.get_pr_labels(&repo, pr.number).await?;
    
    assert!(
        labels.iter().any(|l| l.name.starts_with("size/")),
        "Size label should be applied based on file changes"
    );

    // Assert: Verify status checks were updated
    let checks = test_env.get_pr_checks(&repo, pr.number).await?;
    
    let merge_warden_check = checks.iter()
        .find(|c| c.name == "merge-warden")
        .expect("merge-warden status check should exist");

    assert!(
        merge_warden_check.conclusion.is_some(),
        "merge-warden check should have a conclusion"
    );

    assert_eq!(
        merge_warden_check.conclusion.as_ref().unwrap(),
        "success",
        "merge-warden check should pass for valid PR"
    );

    // Assert: Verify check details contain expected information
    assert!(
        merge_warden_check.details_url.is_some(),
        "Check should have details URL"
    );

    // Assert: Verify mock Azure service integration
    let app_config_calls = test_env.mock_app_config.get_call_count().await?;
    assert!(
        app_config_calls > 0,
        "Configuration should be loaded from mock Azure App Config"
    );

    // Cleanup: Ensure all test resources are properly cleaned up - Assertion #7
    test_env.cleanup().await?;

    Ok(())
}

/// Helper function to create a realistic pull request in the test repository
async fn create_pull_request_in_repository(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr_spec: &PullRequestSpec,
) -> TestResult<merge_warden_integration_tests::utils::TestPullRequest> {
    // Create source branch with changes
    test_env.create_branch(&repo, &pr_spec.source_branch, "main").await?;

    // Add files to source branch
    for file_spec in &pr_spec.files {
        test_env.add_file_to_branch(
            &repo,
            &pr_spec.source_branch,
            &file_spec.path,
            &file_spec.content,
            &format!("{}: {}", file_spec.action.as_commit_message(), file_spec.path)
        ).await?;
    }

    // Create pull request
    let pr = test_env.create_pull_request(&repo, pr_spec).await?;

    Ok(pr)
}

/// Helper function to create webhook payload for pull request opened event
fn create_pr_opened_webhook_payload(
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr: &merge_warden_integration_tests::utils::TestPullRequest,
    pr_spec: &PullRequestSpec,
) -> TestResult<serde_json::Value> {
    let payload = serde_json::json!({
        "action": "opened",
        "number": pr.number,
        "pull_request": {
            "id": pr.id,
            "number": pr.number,
            "title": pr_spec.title,
            "body": pr_spec.body,
            "head": {
                "ref": pr_spec.source_branch,
                "sha": "test-commit-sha"
            },
            "base": {
                "ref": pr_spec.target_branch,
                "sha": "main-commit-sha"
            },
            "user": {
                "login": "test-user",
                "id": 12345
            },
            "draft": pr_spec.draft,
            "mergeable": true,
            "merged": false
        },
        "repository": {
            "id": repo.id,
            "name": repo.name,
            "full_name": format!("{}/{}", repo.owner, repo.name),
            "owner": {
                "login": repo.owner,
                "id": 67890
            }
        },
        "sender": {
            "login": "test-user",
            "id": 12345
        }
    });

    Ok(payload)
}

/// Helper function to wait for async processing completion
async fn wait_for_processing_completion(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr_number: u64,
) -> TestResult<()> {
    // Poll for completion with timeout
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(30);

    while start_time.elapsed() < timeout_duration {
        let checks = test_env.get_pr_checks(repo, pr_number).await?;
        
        if let Some(merge_warden_check) = checks.iter().find(|c| c.name == "merge-warden") {
            if merge_warden_check.conclusion.is_some() {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(TestError::timeout("processing completion", timeout_duration))
}

/// Generates realistic Rust validation file content
fn generate_rust_validation_file() -> TestResult<String> {
    let content = "//! Enhanced validation rules for pull requests.\n\
\n\
use std::collections::HashMap;\n\
\n\
/// Enhanced validation rule for code quality checks.\n\
pub struct CodeQualityValidator {\n\
    patterns: HashMap<String, String>,\n\
}\n\
\n\
impl CodeQualityValidator {\n\
    /// Creates a new code quality validator.\n\
    pub fn new() -> Self {\n\
        let mut patterns = HashMap::new();\n\
        patterns.insert(\"security\".to_string(), \"password\".to_string());\n\
        Self { patterns }\n\
    }\n\
\n\
    /// Validates code content.\n\
    pub fn validate(&self, content: &str) -> bool {\n\
        !content.contains(\"password\")\n\
    }\n\
}\n\
\n\
#[cfg(test)]\n\
mod tests {\n\
    use super::*;\n\
\n\
    #[test]\n\
    fn test_validator() {\n\
        let validator = CodeQualityValidator::new();\n\
        assert!(validator.validate(\"safe code\"));\n\
    }\n\
}";
    Ok(content.to_string())
}

/// Generates updated README content
fn generate_updated_readme() -> TestResult<String> {
    Ok("# Merge Warden\n\nAdvanced pull request validation and automation.\n\n## Features\n\n- Enhanced validation\n- Smart sizing\n- Flexible configuration\n".to_string())
}

/// Generates test file content  
fn generate_test_file() -> TestResult<String> {
    Ok("//! Test file for validation\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn test_basic() {\n        assert!(true);\n    }\n}".to_string())
}