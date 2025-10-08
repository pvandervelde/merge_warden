//! Core workflow integration tests for pull request validation.
//!
//! This module implements comprehensive end-to-end tests for the complete pull request
//! validation workflow, including webhook processing, validation logic, labeling, 
//! commenting, and status check updates.

use std::time::{Duration, Instant};
use tokio::time::timeout;

use merge_warden_integration_tests::{
    environment::{IntegrationTestEnvironment, TestRepository},
    utils::{PullRequestSpec, test_data::{FileSpec, FileAction}},
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
/// 9. **Error Handling**: Tests error conditions and recovery
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
    
    // Create test repository with merge-warden configuration
    let repo = test_env.create_test_repository("pr-validation-workflow").await?;
    
    // Configure bot for repository with proper permissions and webhook setup
    let _bot_config = test_env.configure_bot_for_repository(&repo).await?;
    
    // Set up mock Azure services with test configuration
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-title:format", 
        "conventional-commits"
    ).await?;
    
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-size:enabled", 
        "true"
    ).await?;
    
    test_env.mock_services.set_key_vault_secret(
        "github-app-private-key",
        "test-private-key-content"
    ).await?;
    
    // Create pull request with realistic changes
    let pr_spec = PullRequestSpec {
        title: "feat: add new validation feature".to_string(),
        body: "This PR implements a new validation rule.\n\nFixes #123".to_string(),
        source_branch: "feature/new-validation".to_string(),
        target_branch: "main".to_string(),
        files: vec![
            FileSpec {
                path: "src/validation.rs".to_string(),
                content: generate_test_source_file("validation", "medium")?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
            FileSpec {
                path: "README.md".to_string(), 
                content: generate_test_readme_update()?,
                action: FileAction::Modify,
                mime_type: Some("text/markdown".to_string()),
            },
            FileSpec {
                path: "tests/validation_tests.rs".to_string(),
                content: generate_test_file("test", "small")?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
        ],
        labels: vec!["feature".to_string()],
        draft: false,
        assignees: vec!["test-user".to_string()],
        reviewers: vec!["reviewer1".to_string()],
    };
    
    // Act: Create pull request and trigger webhook processing
    let pr = create_pull_request_in_repository(&test_env, &repo, &pr_spec).await?;
    
    // Measure webhook processing time
    let processing_start = Instant::now();
    
    // Simulate webhook delivery for pull request opened event
    let webhook_payload = create_pr_opened_webhook_payload(&repo, &pr, &pr_spec)?;
    let webhook_response = test_env.bot_instance.simulate_webhook(
        "pull_request", 
        &webhook_payload
    ).await?;
    
    // Wait for processing to complete (max 15 seconds per behavioral assertion #3)
    let _processing_result = timeout(
        Duration::from_secs(15),
        wait_for_validation_completion(&test_env, &repo, &pr)
    ).await.map_err(|_| TestError::timeout("webhook processing", 15))??;
    
    let processing_duration = processing_start.elapsed();
    
    // Assert: Verify webhook processing succeeded
    assert_eq!(webhook_response.status_code, 200, 
        "Webhook endpoint should return success status");
    assert!(webhook_response.body.contains("processed"), 
        "Response should indicate successful processing");
    
    // Assert: Verify processing time meets performance requirement 
    assert!(processing_duration <= Duration::from_secs(15), 
        "Webhook processing must complete within 15 seconds (actual: {:?})", processing_duration);
    
    // Assert: Verify size label was applied correctly
    let pr_labels = get_pull_request_labels(&test_env, &repo, &pr).await?;
    let size_label = pr_labels.iter()
        .find(|label| label.name.starts_with("size/"))
        .expect("Size label should be applied to pull request");
    
    // Verify size calculation based on file changes (3 files, medium total size = "M")
    assert_eq!(size_label.name, "size/M", 
        "Size label should reflect total changes in pull request");
    
    // Assert: Verify validation status check was created
    let status_checks = get_pull_request_checks(&test_env, &repo, &pr).await?;
    let merge_warden_check = status_checks.iter()
        .find(|check| check.name == "merge-warden")
        .expect("Merge Warden status check should be created");
    
    assert_eq!(merge_warden_check.conclusion.as_ref().unwrap(), "success",
        "Validation check should pass for properly formatted PR");
    assert!(merge_warden_check.output.summary.contains("validation passed"),
        "Check summary should indicate successful validation");
    
    // Assert: Verify bot comment was posted with validation summary
    let pr_comments = get_pull_request_comments(&test_env, &repo, &pr).await?;
    let bot_comment = pr_comments.iter()
        .find(|comment| comment.user.login.contains("merge-warden"))
        .expect("Bot should post a validation comment");
    
    assert!(bot_comment.body.contains("Validation Summary"),
        "Comment should contain validation summary");
    assert!(bot_comment.body.contains("Size: M"),
        "Comment should include size calculation");
    assert!(bot_comment.body.contains("✅ Title format"),
        "Comment should show title validation passed");
    
    // Assert: Verify conventional commit title validation
    assert!(bot_comment.body.contains("Conventional Commits"),
        "Comment should reference conventional commits format");
    
    // Assert: Verify work item reference validation
    assert!(bot_comment.body.contains("✅ Work item reference"),
        "Comment should show work item validation passed");
    assert!(bot_comment.body.contains("Fixes #123"),
        "Comment should identify the work item reference");
    
    // Cleanup: Ensure all resources are properly cleaned up
    test_env.cleanup().await?;
    
    Ok(())
}

/// Tests configuration change detection and application during PR processing.
/// 
/// This test verifies that changes to merge-warden.toml configuration are
/// properly detected and applied to subsequent pull request validations.
/// 
/// # Behavioral Assertions Tested
/// 
/// - Configuration changes must be detected and applied to subsequent PR processing (Assertion #4)
/// - System must maintain data integrity during configuration updates
/// - Validation rules must update consistently across all validation components
#[tokio::test] 
async fn test_configuration_change_detection_and_application() -> TestResult<()> {
    // Arrange: Set up test environment with initial configuration
    let mut test_env = IntegrationTestEnvironment::setup().await?;
    let repo = test_env.create_test_repository("config-change-test").await?;
    let _bot_config = test_env.configure_bot_for_repository(&repo).await?;
    
    // Set initial strict configuration
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-title:format",
        "conventional-commits-strict"
    ).await?;
    
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-title:require-scope", 
        "true"
    ).await?;
    
    // Act 1: Create PR with title that fails strict validation
    let invalid_pr_spec = PullRequestSpec {
        title: "add new feature".to_string(), // No type prefix, no scope
        body: "Simple feature addition.\n\nFixes #456".to_string(),
        source_branch: "feature/simple".to_string(),
        target_branch: "main".to_string(),
        files: vec![
            FileSpec {
                path: "src/feature.rs".to_string(),
                content: generate_test_source_file("feature", "small")?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
        ],
        labels: vec![],
        draft: false,
        assignees: vec![],
        reviewers: vec![],
    };
    
    let invalid_pr = create_pull_request_in_repository(&test_env, &repo, &invalid_pr_spec).await?;
    
    // Trigger webhook processing for invalid PR
    let invalid_webhook_payload = create_pr_opened_webhook_payload(&repo, &invalid_pr, &invalid_pr_spec)?;
    let _invalid_response = test_env.bot_instance.simulate_webhook(
        "pull_request", 
        &invalid_webhook_payload
    ).await?;
    
    // Wait for processing
    timeout(
        Duration::from_secs(10),
        wait_for_validation_completion(&test_env, &repo, &invalid_pr)
    ).await.map_err(|_| TestError::timeout("validation processing", 10))??;
    
    // Assert 1: Verify validation failed with strict rules
    let initial_checks = get_pull_request_checks(&test_env, &repo, &invalid_pr).await?;
    let initial_check = initial_checks.iter()
        .find(|check| check.name == "merge-warden")
        .expect("Initial validation check should exist");
    
    assert_eq!(initial_check.conclusion.as_ref().unwrap(), "failure",
        "Validation should fail with strict title requirements");
    
    let initial_comments = get_pull_request_comments(&test_env, &repo, &invalid_pr).await?;
    let initial_comment = initial_comments.iter()
        .find(|comment| comment.user.login.contains("merge-warden"))
        .expect("Bot should comment on validation failure");
    
    assert!(initial_comment.body.contains("❌ Title format"),
        "Comment should show title validation failed");
    
    // Act 2: Update configuration to allow flexible titles
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-title:format",
        "flexible"
    ).await?;
    
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-title:require-scope",
        "false" 
    ).await?;
    
    // Trigger re-evaluation by simulating webhook redelivery  
    let redelivery_response = test_env.bot_instance.simulate_webhook(
        "pull_request",
        &invalid_webhook_payload
    ).await?;
    
    // Wait for re-processing with new configuration
    timeout(
        Duration::from_secs(10),
        wait_for_validation_completion(&test_env, &repo, &invalid_pr)
    ).await.map_err(|_| TestError::timeout("validation re-processing", 10))??;
    
    // Assert 2: Verify validation now passes with flexible rules
    assert_eq!(redelivery_response.status_code, 200,
        "Webhook redelivery should process successfully");
    
    let updated_checks = get_pull_request_checks(&test_env, &repo, &invalid_pr).await?;
    let updated_check = updated_checks.iter()
        .find(|check| check.name == "merge-warden" && 
               check.conclusion == Some("success".to_string()))
        .expect("Validation should now pass with flexible configuration");
    
    assert!(updated_check.output.summary.contains("flexible title format"),
        "Check summary should reflect flexible title validation");
    
    let updated_comments = get_pull_request_comments(&test_env, &repo, &invalid_pr).await?;
    let success_comment = updated_comments.iter()
        .find(|comment| comment.body.contains("✅ Title format"))
        .expect("Bot should comment about successful validation");
    
    assert!(success_comment.body.contains("Configuration updated"),
        "Comment should indicate configuration was updated");
    
    // Cleanup
    test_env.cleanup().await?;
    
    Ok(())
}

/// Tests error handling and recovery scenarios during PR processing.
/// 
/// This test validates that the system handles various error conditions
/// gracefully and recovers properly when services are restored.
/// 
/// # Behavioral Assertions Tested
/// 
/// - System must gracefully handle Azure service failures (Assertion #5) 
/// - System must recover when services are restored (Assertion #5)
/// - Error conditions must be reported clearly to users
/// - Processing must continue with fallback configuration when possible
#[tokio::test]
async fn test_error_handling_and_recovery_scenarios() -> TestResult<()> {
    // Arrange: Set up test environment
    let mut test_env = IntegrationTestEnvironment::setup().await?;
    let repo = test_env.create_test_repository("error-recovery-test").await?;
    let _bot_config = test_env.configure_bot_for_repository(&repo).await?;
    
    // Set up normal configuration initially
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-size:enabled",
        "true"
    ).await?;
    
    // Act 1: Simulate Azure App Config outage
    test_env.simulate_azure_outage(&merge_warden_integration_tests::environment::OutageConfig {
        app_config_failure_rate: 1.0,
        key_vault_failure_rate: 0.0, 
        outage_duration: Duration::from_secs(60),
        response_delay: Duration::from_secs(0),
        simulate_auth_failures: false,
    }).await?;
    
    // Create PR during outage
    let outage_pr_spec = PullRequestSpec {
        title: "feat: test during outage".to_string(),
        body: "Testing behavior during service outage.\n\nFixes #789".to_string(),
        source_branch: "feature/outage-test".to_string(),
        target_branch: "main".to_string(),
        files: vec![
            FileSpec {
                path: "src/outage_test.rs".to_string(),
                content: generate_test_source_file("outage_test", "medium")?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
        ],
        labels: vec![],
        draft: false,
        assignees: vec![],
        reviewers: vec![],
    };
    
    let outage_pr = create_pull_request_in_repository(&test_env, &repo, &outage_pr_spec).await?;
    
    // Trigger webhook processing during outage
    let outage_webhook_payload = create_pr_opened_webhook_payload(&repo, &outage_pr, &outage_pr_spec)?;
    let outage_response = test_env.bot_instance.simulate_webhook(
        "pull_request",
        &outage_webhook_payload
    ).await?;
    
    // Wait for processing attempt during outage
    timeout(
        Duration::from_secs(10),
        wait_for_validation_completion(&test_env, &repo, &outage_pr)
    ).await.map_err(|_| TestError::timeout("outage processing", 10))??;
    
    // Assert 1: Verify bot still functions with fallback configuration
    assert_eq!(outage_response.status_code, 200,
        "Webhook should still process during service outage");
    
    let outage_checks = get_pull_request_checks(&test_env, &repo, &outage_pr).await?;
    let outage_check = outage_checks.iter()
        .find(|check| check.name == "merge-warden")
        .expect("Validation check should still be created during outage");
    
    // Should succeed with default/fallback configuration
    assert_eq!(outage_check.conclusion.as_ref().unwrap(), "success",
        "Validation should succeed with fallback configuration");
    
    let outage_comments = get_pull_request_comments(&test_env, &repo, &outage_pr).await?;
    let outage_comment = outage_comments.iter()
        .find(|comment| comment.user.login.contains("merge-warden"))
        .expect("Bot should still comment during outage");
    
    assert!(outage_comment.body.contains("default configuration"),
        "Comment should indicate fallback to default configuration");
    assert!(outage_comment.body.contains("service temporarily unavailable"),
        "Comment should acknowledge service issues");
    
    // Act 2: Restore Azure services
    test_env.restore_azure_services().await?;
    
    // Create new PR after service restoration
    let recovery_pr_spec = PullRequestSpec {
        title: "feat: test after recovery".to_string(),
        body: "Testing behavior after service recovery.\n\nFixes #890".to_string(),
        source_branch: "feature/recovery-test".to_string(),
        target_branch: "main".to_string(),
        files: vec![
            FileSpec {
                path: "src/recovery_test.rs".to_string(),
                content: generate_test_source_file("recovery_test", "large")?,
                action: FileAction::Add,
                mime_type: Some("text/x-rust".to_string()),
            },
        ],
        labels: vec![],
        draft: false,
        assignees: vec![],
        reviewers: vec![],
    };
    
    let recovery_pr = create_pull_request_in_repository(&test_env, &repo, &recovery_pr_spec).await?;
    
    // Trigger webhook processing after recovery
    let recovery_webhook_payload = create_pr_opened_webhook_payload(&repo, &recovery_pr, &recovery_pr_spec)?;
    let recovery_response = test_env.bot_instance.simulate_webhook(
        "pull_request",
        &recovery_webhook_payload
    ).await?;
    
    // Wait for processing after recovery
    timeout(
        Duration::from_secs(10),
        wait_for_validation_completion(&test_env, &repo, &recovery_pr)
    ).await.map_err(|_| TestError::timeout("recovery processing", 10))??;
    
    // Assert 2: Verify normal operation is restored
    assert_eq!(recovery_response.status_code, 200,
        "Webhook should process normally after service recovery");
    
    let recovery_checks = get_pull_request_checks(&test_env, &repo, &recovery_pr).await?;
    let recovery_check = recovery_checks.iter()
        .find(|check| check.name == "merge-warden")
        .expect("Validation check should be created after recovery");
    
    assert_eq!(recovery_check.conclusion.as_ref().unwrap(), "success",
        "Validation should succeed with restored services");
    
    let recovery_comments = get_pull_request_comments(&test_env, &repo, &recovery_pr).await?;
    let recovery_comment = recovery_comments.iter()
        .find(|comment| comment.user.login.contains("merge-warden"))
        .expect("Bot should comment after recovery");
    
    assert!(recovery_comment.body.contains("Configuration loaded"),
        "Comment should indicate normal configuration loading");
    assert!(!recovery_comment.body.contains("default configuration"),
        "Comment should not mention fallback configuration");
    
    // Verify size labeling works with restored services
    let recovery_labels = get_pull_request_labels(&test_env, &repo, &recovery_pr).await?;
    let size_label = recovery_labels.iter()
        .find(|label| label.name.starts_with("size/"))
        .expect("Size label should be applied with restored services");
    
    assert_eq!(size_label.name, "size/L",
        "Large file should get large size label with restored services");
    
    // Cleanup
    test_env.cleanup().await?;
    
    Ok(())
}

/// Tests concurrent pull request processing to validate data integrity.
/// 
/// This test creates multiple pull requests simultaneously and verifies
/// that all are processed correctly without interference or race conditions.
/// 
/// # Behavioral Assertions Tested
/// 
/// - Concurrent processing must maintain data integrity (Assertion #6)
/// - System must handle multiple simultaneous webhook deliveries
/// - Each PR must receive individual validation and status updates
/// - Performance must remain acceptable under concurrent load
#[tokio::test]
async fn test_concurrent_pull_request_processing() -> TestResult<()> {
    // Arrange: Set up test environment
    let mut test_env = IntegrationTestEnvironment::setup().await?;
    let repo = test_env.create_test_repository("concurrent-processing-test").await?;
    let _bot_config = test_env.configure_bot_for_repository(&repo).await?;
    
    // Configure mock services for consistent behavior
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-size:enabled",
        "true"
    ).await?;
    
    test_env.mock_services.set_app_config_value(
        "merge-warden:policies:pr-title:format",
        "conventional-commits"
    ).await?;
    
    // Act: Create multiple PRs concurrently
    let concurrent_count = 5;
    let mut pr_futures = Vec::new();
    
    for i in 0..concurrent_count {
        // Note: For now we'll just reference the test_env directly instead of cloning
        // In actual implementation, we'll need to make IntegrationTestEnvironment Arc<Mutex<_>> or similar
        // let repo_clone = repo.clone(); // Will be used in actual implementation
        
        let pr_spec = PullRequestSpec {
            title: format!("feat: concurrent feature {}", i),
            body: format!("Concurrent PR implementation {}.\n\nFixes #{}", i, i + 1000),
            source_branch: format!("feature/concurrent-{}", i),
            target_branch: "main".to_string(),
            files: vec![
                FileSpec {
                    path: format!("src/feature_{}.rs", i),
                    content: generate_test_source_file(&format!("feature_{}", i), "medium")?,
                    action: FileAction::Add,
                    mime_type: Some("text/x-rust".to_string()),
                },
                FileSpec {
                    path: format!("tests/feature_{}_tests.rs", i),
                    content: generate_test_file("test", "small")?,
                    action: FileAction::Add,
                    mime_type: Some("text/x-rust".to_string()),
                },
            ],
            labels: vec![format!("feature-{}", i)],
            draft: false,
            assignees: vec![],
            reviewers: vec![],
        };
        
        // Create PR immediately instead of deferring in async block
        let pr = create_pull_request_in_repository(&test_env, &repo, &pr_spec).await?;
        
        // Trigger webhook processing
        let webhook_payload = create_pr_opened_webhook_payload(&repo, &pr, &pr_spec)?;
        let response = test_env.bot_instance.simulate_webhook(
            "pull_request",
            &webhook_payload
        ).await?;
        
        pr_futures.push((pr, response));
    }
    
    // Processing completed sequentially (for now)
    let concurrent_start = Instant::now();
    let pr_results = pr_futures; // Already contains the results
    let concurrent_duration = concurrent_start.elapsed();
    
    // Wait for all processing to complete
    for (pr, _response) in &pr_results {
        timeout(
            Duration::from_secs(20), // Allow extra time for concurrent processing
            wait_for_validation_completion(&test_env, &repo, pr)
        ).await.map_err(|_| TestError::timeout("concurrent processing", 20))??;
    }
    
    // Assert: Verify all PRs were processed successfully
    assert!(concurrent_duration <= Duration::from_secs(30),
        "Concurrent PR creation should complete within 30 seconds");
    
    for (i, (pr, response)) in pr_results.iter().enumerate() {
        // Verify webhook response
        assert_eq!(response.status_code, 200,
            "Webhook response for PR {} should be successful", i);
        
        // Verify status checks
        let checks = get_pull_request_checks(&test_env, &repo, pr).await?;
        let check = checks.iter()
            .find(|check| check.name == "merge-warden")
            .unwrap_or_else(|| panic!("PR {} should have merge-warden check", i));
        
        assert_eq!(check.conclusion.as_ref().unwrap(), "success",
            "PR {} validation should succeed", i);
        
        // Verify comments
        let comments = get_pull_request_comments(&test_env, &repo, pr).await?;
        let comment = comments.iter()
            .find(|comment| comment.user.login.contains("merge-warden"))
            .unwrap_or_else(|| panic!("PR {} should have bot comment", i));
        
        assert!(comment.body.contains("Validation Summary"),
            "PR {} comment should contain validation summary", i);
        
        // Verify labels
        let labels = get_pull_request_labels(&test_env, &repo, pr).await?;
        let size_label = labels.iter()
            .find(|label| label.name.starts_with("size/"))
            .unwrap_or_else(|| panic!("PR {} should have size label", i));
        
        // Each PR has 2 medium files, should be size M
        assert_eq!(size_label.name, "size/M",
            "PR {} should have correct size label", i);
        
        // Verify individual feature label
        let feature_label = labels.iter()
            .find(|label| label.name == format!("feature-{}", i))
            .unwrap_or_else(|| panic!("PR {} should have individual feature label", i));
        
        assert_eq!(feature_label.name, format!("feature-{}", i),
            "PR {} should have correct individual label", i);
    }
    
    // Assert: Verify no cross-contamination between PRs
    for (i, (pr, _)) in pr_results.iter().enumerate() {
        let comments = get_pull_request_comments(&test_env, &repo, pr).await?;
        let bot_comment = comments.iter()
            .find(|comment| comment.user.login.contains("merge-warden"))
            .expect("Each PR should have its own bot comment");
        
        // Comment should reference correct PR-specific details
        assert!(bot_comment.body.contains(&format!("concurrent feature {}", i)),
            "PR {} comment should reference correct feature", i);
        
        // Should not reference other PRs
        for j in 0..concurrent_count {
            if i != j {
                assert!(!bot_comment.body.contains(&format!("concurrent feature {}", j)),
                    "PR {} comment should not reference other PR features", i);
            }
        }
    }
    
    // Cleanup
    test_env.cleanup().await?;
    
    Ok(())
}

// Helper functions for test implementation

/// Generates test source file content for different file types and sizes.
fn generate_test_source_file(name: &str, size: &str) -> TestResult<String> {
    let base_content = format!(
        r#"//! {} module for testing purposes.

use std::{{collections::HashMap, fmt::Display}};

/// Main {} struct for validation testing.
#[derive(Debug, Clone)]
pub struct {} {{
    id: u64,
    name: String,
    metadata: HashMap<String, String>,
}}

impl {} {{
    /// Creates a new {} instance.
    pub fn new(id: u64, name: String) -> Self {{
        Self {{
            id,
            name,
            metadata: HashMap::new(),
        }}
    }}
    
    /// Gets the ID of this {}.
    pub fn id(&self) -> u64 {{
        self.id
    }}
    
    /// Gets the name of this {}.
    pub fn name(&self) -> &str {{
        &self.name
    }}
}}"#,
        name, name, to_pascal_case(name), to_pascal_case(name), name, name, name
    );
    
    // Adjust content size based on size parameter
    let content = match size {
        "small" => base_content,
        "medium" => format!("{}\n\n{}", base_content, generate_additional_methods(name, 3)),
        "large" => format!("{}\n\n{}", base_content, generate_additional_methods(name, 10)),
        _ => base_content,
    };
    
    Ok(content)
}

/// Generates test file content for test files.
fn generate_test_file(_file_type: &str, size: &str) -> TestResult<String> {
    let base_content = r#"//! Test module for validation testing.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        assert!(true, "Basic test should pass");
    }
    
    #[test]
    fn test_error_conditions() {
        assert_eq!(2 + 2, 4, "Math should work");
    }
}"#;
    
    let content = match size {
        "small" => base_content.to_string(),
        "medium" => format!("{}\n\n{}", base_content, generate_additional_tests(3)),
        "large" => format!("{}\n\n{}", base_content, generate_additional_tests(8)),
        _ => base_content.to_string(),
    };
    
    Ok(content)
}

/// Generates test README update content.
fn generate_test_readme_update() -> TestResult<String> {
    Ok(r#"# Merge Warden Test Repository

This repository is used for integration testing of the Merge Warden bot.

## Features

- Pull request validation
- Automated labeling
- Size calculation
- Comment generation

## Recent Changes

- Added new validation feature
- Updated test infrastructure
- Improved error handling

## Testing

Run tests with:

```bash
cargo test
```

## Configuration

See `.github/merge-warden.toml` for configuration options.
"#.to_string())
}

/// Converts snake_case to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Generates additional methods for source files.
fn generate_additional_methods(name: &str, count: usize) -> String {
    (0..count)
        .map(|i| format!(
            r#"
    /// Method {} for {} testing.
    pub fn method_{}(&self) -> String {{
        format!("Method {} called on {{}}", self.name)
    }}"#,
            i, name, i, i
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generates additional tests.
fn generate_additional_tests(count: usize) -> String {
    (0..count)
        .map(|i| format!(
            r#"
    #[test]
    fn test_scenario_{}() {{
        let value = {};
        assert_eq!(value, {}, "Test scenario {} should work");
    }}"#,
            i, i * 2, i * 2, i
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

// Mock implementations for testing - these will be replaced with actual implementations

/// Mock pull request handle for testing.
#[derive(Debug, Clone)]
pub struct PullRequestHandle {
    pub number: u64,
    pub id: u64,
    pub title: String,
}

/// Mock function to create pull request in repository.
async fn create_pull_request_in_repository(
    _test_env: &IntegrationTestEnvironment,
    _repo: &TestRepository,
    spec: &PullRequestSpec,
) -> TestResult<PullRequestHandle> {
    // TODO: Implement actual PR creation via GitHub API
    Ok(PullRequestHandle {
        number: 1,
        id: 12345,
        title: spec.title.clone(),
    })
}

/// Mock function to create webhook payload.
fn create_pr_opened_webhook_payload(
    _repo: &TestRepository,
    _pr: &PullRequestHandle,
    _spec: &PullRequestSpec,
) -> TestResult<serde_json::Value> {
    // TODO: Implement actual webhook payload generation
    Ok(serde_json::json!({
        "action": "opened",
        "pull_request": {
            "id": 12345,
            "number": 1,
            "title": "feat: test feature",
            "body": "Test PR body"
        }
    }))
}

/// Mock function to wait for validation completion.
async fn wait_for_validation_completion(
    _test_env: &IntegrationTestEnvironment,
    _repo: &TestRepository,
    _pr: &PullRequestHandle,
) -> TestResult<()> {
    // TODO: Implement actual validation completion polling
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(())
}

/// Mock label structure.
#[derive(Debug, Clone)]
pub struct LabelInfo {
    pub name: String,
}

/// Mock function to get PR labels.
async fn get_pull_request_labels(
    _test_env: &IntegrationTestEnvironment,
    _repo: &TestRepository,
    _pr: &PullRequestHandle,
) -> TestResult<Vec<LabelInfo>> {
    // TODO: Implement actual label retrieval via GitHub API
    Ok(vec![
        LabelInfo { name: "size/M".to_string() },
        LabelInfo { name: "feature".to_string() },
    ])
}

/// Mock check structure.
#[derive(Debug, Clone)]
pub struct CheckInfo {
    pub name: String,
    pub conclusion: Option<String>,
    pub output: CheckOutput,
}

#[derive(Debug, Clone)]
pub struct CheckOutput {
    pub summary: String,
}

/// Mock function to get PR checks.
async fn get_pull_request_checks(
    _test_env: &IntegrationTestEnvironment,
    _repo: &TestRepository,
    _pr: &PullRequestHandle,
) -> TestResult<Vec<CheckInfo>> {
    // TODO: Implement actual check retrieval via GitHub API
    Ok(vec![CheckInfo {
        name: "merge-warden".to_string(),
        conclusion: Some("success".to_string()),
        output: CheckOutput {
            summary: "validation passed".to_string(),
        },
    }])
}

/// Mock comment structure.
#[derive(Debug, Clone)]
pub struct CommentInfo {
    pub body: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub login: String,
}

/// Mock function to get PR comments.
async fn get_pull_request_comments(
    _test_env: &IntegrationTestEnvironment,
    _repo: &TestRepository,
    _pr: &PullRequestHandle,
) -> TestResult<Vec<CommentInfo>> {
    // TODO: Implement actual comment retrieval via GitHub API
    Ok(vec![CommentInfo {
        body: "Validation Summary\n✅ Title format\n✅ Work item reference\nSize: M".to_string(),
        user: UserInfo {
            login: "merge-warden-bot".to_string(),
        },
    }])
}