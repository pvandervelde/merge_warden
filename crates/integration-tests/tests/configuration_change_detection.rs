//! Configuration change detection and application testing.
//!
//! This test validates that configuration changes are properly detected and applied
//! to subsequent pull request processing operations.
//!
//! Covers behavioral assertions #4 from the specification.

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

/// Test that configuration changes are detected and applied to PR processing.
///
/// This test validates that when repository configuration is updated,
/// the changes are properly detected and applied to subsequent pull request
/// validation operations.
///
/// # Test Scenario Coverage
///
/// 1. **Initial Configuration**: Sets up repository with strict validation rules
/// 2. **Invalid PR Creation**: Creates PR that violates initial configuration
/// 3. **Validation Failure**: Verifies that validation fails as expected
/// 4. **Configuration Update**: Updates configuration to allow previously invalid PR
/// 5. **Re-evaluation**: Triggers re-evaluation of the same PR
/// 6. **Validation Success**: Verifies that validation now passes
/// 7. **Timing Validation**: Ensures configuration changes apply within expected timeframe
/// 8. **Behavioral Assertions**: Tests assertion #4 from specification
///
/// # Behavioral Assertions Tested
///
/// - Configuration changes to merge-warden.toml must be detected and applied to subsequent PR processing (Assertion #4)
#[tokio::test]
#[ignore = "requires GitHub App credentials (REPO_CREATION_APP_ID, MERGE_WARDEN_APP_ID)"]
async fn test_configuration_changes_are_applied() -> TestResult<()> {
    // Arrange: Set up test environment with strict initial configuration
    let mut test_env = IntegrationTestEnvironment::setup().await?;

    let repo = test_env
        .create_test_repository("config-change-test")
        .await?;

    // Configure repository with strict validation rules
    setup_strict_configuration(&test_env, &repo).await?;

    // Create PR with title format that violates strict rules
    let pr_spec = PullRequestSpec {
        title: "invalid title format".to_string(), // Violates conventional-commits format
        body: "Test PR for configuration testing".to_string(),
        source_branch: "test-config-changes".to_string(),
        target_branch: "main".to_string(),
        files: vec![FileSpec {
            path: "test-file.txt".to_string(),
            content: "Simple test content".to_string(),
            action: FileAction::Add,
            mime_type: Some("text/plain".to_string()),
        }],
        labels: vec!["test".to_string()],
        draft: false,
        assignees: vec![],
        reviewers: vec![],
    };

    // Act: Create pull request and wait for initial processing
    let pr = create_test_pull_request(&test_env, &repo, &pr_spec).await?;

    // Explicitly trigger initial webhook processing (simulates GitHub delivering
    // a pull_request.opened event to the Merge Warden bot).
    let initial_webhook_payload = serde_json::json!({
        "action": "opened",
        "number": pr.number,
        "pull_request": {
            "id": pr.id,
            "number": pr.number,
            "head": { "sha": &pr.head, "ref": &pr.head }
        },
        "repository": {
            "id": repo.id,
            "name": repo.name,
            "full_name": format!("{}/{}", repo.organization, repo.name)
        },
        "installation": { "id": 0 }
    });
    test_env
        .bot_instance
        .simulate_webhook("pull_request", &initial_webhook_payload)
        .await?;

    // Wait for initial processing with timeout
    let processing_timeout = Duration::from_secs(10);
    let _initial_result = timeout(
        processing_timeout,
        wait_for_validation_completion(&test_env, &repo, pr.number),
    )
    .await
    .map_err(|_| TestError::timeout("initial validation", processing_timeout.as_secs()))??;

    // Assert: Verify initial validation failed due to strict rules
    let initial_checks = test_env.get_pr_checks(&repo, pr.number).await?;
    let merge_warden_check = initial_checks
        .iter()
        .find(|c| c.name == "MergeWarden")
        .ok_or_else(|| TestError::validation_failed("MergeWarden check", "not found"))?;

    assert_eq!(
        merge_warden_check.conclusion.as_ref().unwrap(),
        "failure",
        "Initial validation should fail with strict configuration"
    );

    // Act: Update configuration to allow flexible title format
    let config_update_start = Instant::now();

    update_repository_configuration(&test_env, &repo).await?;

    // Trigger re-evaluation by sending webhook event
    trigger_configuration_reload(&test_env, &repo, &pr).await?;

    // Wait for configuration change to be detected and applied
    let reeval_timeout = Duration::from_secs(15);
    let _reeval_result = timeout(
        reeval_timeout,
        wait_for_configuration_update_completion(&test_env, &repo, pr.number),
    )
    .await
    .map_err(|_| TestError::timeout("configuration update", reeval_timeout.as_secs()))??;

    let config_update_duration = config_update_start.elapsed();

    // Assert: Verify configuration changes were applied within reasonable time
    assert!(
        config_update_duration <= Duration::from_secs(30),
        "Configuration changes should be applied within 30 seconds, took {:?}",
        config_update_duration
    );

    // Assert: Verify validation now passes with updated configuration
    let updated_checks = test_env.get_pr_checks(&repo, pr.number).await?;
    let updated_merge_warden_check = updated_checks
        .iter()
        .find(|c| c.name == "MergeWarden")
        .ok_or_else(|| {
            TestError::validation_failed("MergeWarden check", "not found after update")
        })?;

    assert_eq!(
        updated_merge_warden_check.conclusion.as_ref().unwrap(),
        "success",
        "Validation should pass after configuration update"
    );

    // Assert: Verify check was updated (different from initial check)
    assert_ne!(
        merge_warden_check.id, updated_merge_warden_check.id,
        "Check should be updated, not just modified in place"
    );

    // Assert: Verify new comment reflects configuration change
    let comments = test_env.get_pr_comments(&repo, pr.number).await?;
    assert!(
        comments
            .iter()
            .any(|c| c.body.contains("Configuration updated")
                || c.body.contains("Re-evaluated with new configuration")),
        "Bot should indicate that configuration was updated"
    );

    // Cleanup
    test_env.cleanup().await?;

    Ok(())
}

/// Helper function to set up strict initial configuration
async fn setup_strict_configuration(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
) -> TestResult<()> {
    let strict_config = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
format = "conventional-commits"
requireScope = true

[policies.pullRequests.prBody]
requireWorkItemReference = true
workItemPattern = "(Fixes|Closes|Resolves) #\\d+"

[policies.pullRequests.prSize]
enabled = true
maxLines = 500
"#;

    test_env
        .update_file_in_repository(
            repo,
            ".github/merge-warden.toml",
            strict_config,
            "Configure strict validation rules",
        )
        .await?;

    Ok(())
}

/// Helper function to update repository configuration to be more permissive
async fn update_repository_configuration(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
) -> TestResult<()> {
    let permissive_config = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
format = "freeform"
requireScope = false

[policies.pullRequests.prBody]
requireWorkItemReference = false

[policies.pullRequests.prSize]
enabled = true
maxLines = 2000
"#;

    test_env
        .update_file_in_repository(
            repo,
            ".github/merge-warden.toml",
            permissive_config,
            "Update to permissive validation rules",
        )
        .await?;

    Ok(())
}

/// Helper function to create a test pull request
async fn create_test_pull_request(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr_spec: &PullRequestSpec,
) -> TestResult<merge_warden_integration_tests::utils::TestPullRequest> {
    // Create source branch
    test_env
        .create_branch(repo, &pr_spec.source_branch, "main")
        .await?;

    // Add files to source branch
    for file_spec in &pr_spec.files {
        test_env
            .add_file_to_branch(
                repo,
                &pr_spec.source_branch,
                &file_spec.path,
                &file_spec.content,
                &format!(
                    "{}: {}",
                    file_spec.action.as_commit_message(),
                    file_spec.path
                ),
            )
            .await?;
    }

    // Create pull request
    let pr = test_env.create_pull_request(repo, pr_spec).await?;

    Ok(pr)
}

/// Helper function to trigger configuration reload
async fn trigger_configuration_reload(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr: &merge_warden_integration_tests::utils::TestPullRequest,
) -> TestResult<()> {
    // Simulate a push event to the configuration file to trigger reload
    let webhook_payload = serde_json::json!({
        "action": "synchronize",
        "number": pr.number,
        "pull_request": {
            "id": pr.id,
            "number": pr.number,
            "head": {
                "sha": "updated-commit-sha"
            }
        },
        "repository": {
            "id": repo.id,
            "name": repo.name,
            "full_name": format!("{}/{}", repo.organization, repo.name)
        }
    });

    test_env
        .bot_instance
        .simulate_webhook("pull_request", &webhook_payload)
        .await?;

    Ok(())
}

/// Helper function to wait for validation completion
async fn wait_for_validation_completion(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr_number: u64,
) -> TestResult<()> {
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(30);

    while start_time.elapsed() < timeout_duration {
        let checks = test_env.get_pr_checks(repo, pr_number).await?;

        if let Some(merge_warden_check) = checks.iter().find(|c| c.name == "MergeWarden") {
            if merge_warden_check.conclusion.is_some() {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(TestError::timeout(
        "validation completion",
        timeout_duration.as_secs(),
    ))
}

/// Helper function to wait for configuration update completion
async fn wait_for_configuration_update_completion(
    test_env: &IntegrationTestEnvironment,
    repo: &merge_warden_integration_tests::environment::TestRepository,
    pr_number: u64,
) -> TestResult<()> {
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(45);

    // Store initial check state to detect changes
    let initial_checks = test_env.get_pr_checks(repo, pr_number).await?;
    let initial_check_id = initial_checks
        .iter()
        .find(|c| c.name == "MergeWarden")
        .map(|c| c.id.clone());

    while start_time.elapsed() < timeout_duration {
        let checks = test_env.get_pr_checks(repo, pr_number).await?;

        if let Some(current_check) = checks.iter().find(|c| c.name == "MergeWarden") {
            // Check if this is a new check run (different ID) or conclusion changed
            if let Some(ref initial_id) = initial_check_id {
                if current_check.id != *initial_id && current_check.conclusion.is_some() {
                    return Ok(());
                }
            } else if current_check.conclusion.is_some() {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Err(TestError::timeout(
        "configuration update completion",
        timeout_duration.as_secs(),
    ))
}
