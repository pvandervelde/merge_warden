# Integration Testing

Comprehensive integration testing strategy for Merge Warden to ensure end-to-end functionality works correctly across all components and external dependencies.

## Overview

This document defines the integration testing framework, test scenarios, and infrastructure requirements for Merge Warden. Integration testing validates that components work together correctly and that the system properly integrates with external services like GitHub, Azure App Config, and Azure Key Vault.

## Integration Testing Strategy

### Testing Levels

**Component Integration Testing:**

- Validation engine with configuration system
- GitHub API client with webhook processing
- Mock Azure services for core logic testing
- Configuration management with simulated external services

**Service Integration Testing:**

- End-to-end webhook processing workflow with local/development instances
- Complete pull request validation pipeline using test GitHub organization
- External service connectivity simulation and authentication testing
- Error handling and retry mechanisms with controlled failure injection

**System Integration Testing:**

- GitHub-focused application testing with real repositories
- Cross-service communication validation using development environment
- Performance under realistic GitHub conditions
- Local development environment monitoring and logging

### Testing Environment Constraints

**Available Resources:**

- **GitHub Organization**: `glitchgrove` - dedicated for testing GitHub Apps
- **Test Repositories**: Can create/delete repositories in `glitchgrove` org
- **GitHub App Testing**: Full GitHub App installation and webhook testing capabilities
- **Local/Development Environment**: Azure Functions Core Tools for local testing

**Limited Resources:**

- **Azure Test Environment**: Limited access to dedicated Azure test resources
- **Production-like Infrastructure**: Testing must focus on local development and GitHub integration
- **External Service Dependencies**: Must use mocking/simulation for Azure services when not available

## Test Infrastructure

### Test Repository Management

**Automated Repository Setup with GlitchGrove Organization:**

```rust
pub struct TestRepositoryManager {
    github_client: GitHubClient,
    organization: String, // "glitchgrove"
    cleanup_repos: Vec<String>,
}

impl TestRepositoryManager {
    pub fn new(github_token: String) -> Self {
        Self {
            github_client: GitHubClient::new(github_token),
            organization: "glitchgrove".to_string(),
            cleanup_repos: Vec::new(),
        }
    }

    pub async fn create_test_repository(&mut self, name: &str) -> Result<Repository, TestError> {
        let repo_name = format!("merge-warden-test-{}-{}", name, Uuid::new_v4());

        let repository = self.github_client
            .create_repository(&CreateRepositoryRequest {
                name: repo_name.clone(),
                description: Some("Test repository for Merge Warden integration tests".to_string()),
                private: true,
                auto_init: true,
                organization: Some(self.organization.clone()),
            })
            .await?;

        self.cleanup_repos.push(repo_name);
        Ok(repository)
    }

    pub async fn setup_repository_configuration(&self, repo: &Repository) -> Result<(), TestError> {
        // Add merge-warden.toml configuration
        let config_content = include_str!("../test-data/test-merge-warden.toml");

        self.github_client
            .create_file(&repo, ".github/merge-warden.toml", config_content)
            .await?;

        // Set up branch protection rules
        self.github_client
            .update_branch_protection(&repo, "main", &BranchProtectionRequest {
                required_status_checks: Some(vec!["merge-warden".to_string()]),
                enforce_admins: false,
                required_pull_request_reviews: Some(PullRequestReviewsConfig {
                    dismiss_stale_reviews: true,
                    require_code_owner_reviews: false,
                    required_approving_review_count: 1,
                }),
                restrictions: None,
            })
            .await?;

        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<(), TestError> {
        for repo_name in &self.cleanup_repos {
            if let Err(e) = self.github_client.delete_repository(&self.organization, repo_name).await {
                eprintln!("Failed to cleanup repository {}: {}", repo_name, e);
            }
        }
        self.cleanup_repos.clear();
        Ok(())
    }
}
```

### Bot Instance Configuration

**Local Development and Test Bot Setup:**

```rust
pub struct TestBotInstance {
    app_id: String,
    private_key: String,
    webhook_secret: String,
    installation_id: String,
    local_endpoint: String, // For local development testing
    ngrok_tunnel: Option<String>, // For webhook forwarding
}

impl TestBotInstance {
    pub fn new_for_local_testing() -> Self {
        Self {
            app_id: env::var("GITHUB_TEST_APP_ID").expect("GITHUB_TEST_APP_ID required"),
            private_key: env::var("GITHUB_TEST_PRIVATE_KEY").expect("GITHUB_TEST_PRIVATE_KEY required"),
            webhook_secret: env::var("GITHUB_TEST_WEBHOOK_SECRET").expect("GITHUB_TEST_WEBHOOK_SECRET required"),
            installation_id: String::new(), // Will be set when installing
            local_endpoint: "http://localhost:7071/api/webhook".to_string(),
            ngrok_tunnel: None,
        }
    }

    pub async fn setup_local_tunnel(&mut self) -> Result<(), TestError> {
        // Start ngrok tunnel for local development testing
        let tunnel_url = self.start_ngrok_tunnel().await?;
        self.ngrok_tunnel = Some(tunnel_url);
        Ok(())
    }

    pub async fn configure_for_repository(&self, repo: &Repository) -> Result<(), TestError> {
        // Install the GitHub App on the test repository in glitchgrove org
        let installation = self.install_app_on_repository(repo).await?;

        // Configure webhook endpoint (either ngrok tunnel or mock endpoint)
        self.setup_webhook_endpoint(repo).await?;

        // Verify bot has required permissions
        self.verify_bot_permissions(repo).await?;

        Ok(())
    }

    async fn install_app_on_repository(&self, repo: &Repository) -> Result<Installation, TestError> {
        let jwt_token = self.generate_jwt_token()?;

        let installation = self.github_client
            .create_installation(&repo.owner.login, &repo.name, &InstallationRequest {
                app_id: self.app_id.clone(),
                permissions: self.get_required_permissions(),
            })
            .await?;

        Ok(installation)
    }

    async fn setup_webhook_endpoint(&self, repo: &Repository) -> Result<(), TestError> {
        let webhook_url = format!("{}/api/webhook", self.base_url);

        self.github_client
            .create_webhook(&repo, &WebhookRequest {
                url: webhook_url,
                content_type: "json".to_string(),
                secret: Some(self.webhook_secret.clone()),
                events: vec![
                    "pull_request".to_string(),
                    "pull_request_review".to_string(),
                    "issue_comment".to_string(),
                ],
                active: true,
            })
            .await?;

        Ok(())
    }

    fn get_required_permissions(&self) -> HashMap<String, String> {
        [
            ("issues", "write"),
            ("pull_requests", "write"),
            ("contents", "read"),
            ("metadata", "read"),
            ("checks", "write"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }
}
```

## Test Scenarios

### Core Workflow Testing

**Pull Request Validation Scenario with GlitchGrove:**

```rust
#[tokio::test]
async fn test_complete_pull_request_validation_workflow() {
    let test_env = IntegrationTestEnvironment::setup_for_glitchgrove().await.unwrap();

    // Arrange: Create test repository in glitchgrove org and configure bot
    let repo = test_env.repo_manager
        .create_test_repository("pr-validation-test")
        .await
        .unwrap();

    test_env.repo_manager
        .setup_repository_configuration(&repo)
        .await
        .unwrap();

    // Setup local development environment or use test deployment
    test_env.bot_instance
        .setup_local_tunnel()
        .await
        .unwrap();

    test_env.bot_instance
        .configure_for_repository(&repo)
        .await
        .unwrap();    // Add initial content to repository
    test_env.add_default_repository_content(&repo).await.unwrap();

    // Act: Create a pull request with test changes
    let pr = test_env.create_test_pull_request(&repo, &PullRequestSpec {
        title: "feat: add new validation feature",
        body: "This PR implements a new validation rule.\n\nFixes #123",
        source_branch: "feature/new-validation",
        target_branch: "main",
        files: vec![
            FileChange {
                path: "src/validation.rs",
                content: include_str!("../test-data/validation.rs"),
                action: FileAction::Add,
            },
            FileChange {
                path: "README.md",
                content: include_str!("../test-data/updated-readme.md"),
                action: FileAction::Modify,
            },
        ],
    }).await.unwrap();

    // Wait for webhook processing
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Assert: Verify bot responded correctly
    let comments = test_env.github_client
        .get_pull_request_comments(&repo.owner.login, &repo.name, pr.number)
        .await
        .unwrap();

    assert!(!comments.is_empty(), "Bot should have posted a comment");

    let labels = test_env.github_client
        .get_pull_request_labels(&repo.owner.login, &repo.name, pr.number)
        .await
        .unwrap();

    assert!(
        labels.iter().any(|l| l.name.starts_with("size/")),
        "Size label should be applied"
    );

    let checks = test_env.github_client
        .get_pull_request_checks(&repo.owner.login, &repo.name, pr.number)
        .await
        .unwrap();

    assert!(
        checks.iter().any(|c| c.name == "merge-warden" && c.conclusion == Some("success".to_string())),
        "Merge Warden check should pass"
    );

    // Cleanup
    test_env.cleanup().await.unwrap();
}
```

**Configuration Validation Scenario:**

```rust
#[tokio::test]
async fn test_configuration_changes_are_applied() {
    let test_env = IntegrationTestEnvironment::setup().await.unwrap();
    let repo = test_env.create_configured_repository("config-test").await.unwrap();

    // Create PR with invalid title format
    let pr = test_env.create_test_pull_request(&repo, &PullRequestSpec {
        title: "invalid title format",
        body: "Test PR",
        source_branch: "test-branch",
        target_branch: "main",
        files: vec![],
    }).await.unwrap();

    // Wait for processing
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Verify validation failed
    let checks = test_env.get_pr_checks(&repo, pr.number).await.unwrap();
    assert!(checks.iter().any(|c| c.conclusion == Some("failure".to_string())));

    // Update configuration to allow flexible titles
    test_env.update_repository_config(&repo, &ConfigUpdate {
        path: ".github/merge-warden.toml",
        content: r#"
            schemaVersion = 1

            [policies.pullRequests.prTitle]
            format = "freeform"
        "#,
    }).await.unwrap();

    // Trigger re-evaluation
    test_env.trigger_webhook_redelivery(&repo, &pr).await.unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Verify validation now passes
    let updated_checks = test_env.get_pr_checks(&repo, pr.number).await.unwrap();
    assert!(updated_checks.iter().any(|c| c.conclusion == Some("success".to_string())));

    test_env.cleanup().await.unwrap();
}
```

### Error Handling and Recovery

**Network Failure Recovery:**

```rust
#[tokio::test]
async fn test_recovery_from_github_api_failures() {
    let test_env = IntegrationTestEnvironment::setup().await.unwrap();
    let repo = test_env.create_configured_repository("recovery-test").await.unwrap();

    // Simulate GitHub API failure
    test_env.simulate_github_api_failure().await;

    let pr = test_env.create_test_pull_request(&repo, &PullRequestSpec::default()).await.unwrap();

    // Wait for initial processing attempt
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify no successful processing occurred
    let initial_comments = test_env.get_pr_comments(&repo, pr.number).await.unwrap();
    assert!(initial_comments.is_empty());

    // Restore GitHub API availability
    test_env.restore_github_api().await;

    // Trigger retry mechanism
    test_env.trigger_webhook_redelivery(&repo, &pr).await.unwrap();
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify successful processing after recovery
    let final_comments = test_env.get_pr_comments(&repo, pr.number).await.unwrap();
    assert!(!final_comments.is_empty());

    test_env.cleanup().await.unwrap();
}
```

**Configuration Service Outage:**

```rust
#[tokio::test]
async fn test_fallback_to_default_config_during_outage() {
    let test_env = IntegrationTestEnvironment::setup().await.unwrap();
    let repo = test_env.create_configured_repository("outage-test").await.unwrap();

    // Simulate Azure App Config outage
    test_env.simulate_app_config_outage().await;

    let pr = test_env.create_test_pull_request(&repo, &PullRequestSpec {
        title: "feat: test feature",
        body: "Test PR during outage",
        source_branch: "test-branch",
        target_branch: "main",
        files: vec![],
    }).await.unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify bot still functions with default configuration
    let checks = test_env.get_pr_checks(&repo, pr.number).await.unwrap();
    assert!(checks.iter().any(|c| c.name == "merge-warden"));

    let comments = test_env.get_pr_comments(&repo, pr.number).await.unwrap();
    assert!(comments.iter().any(|c| c.body.contains("Using default configuration")));

    test_env.cleanup().await.unwrap();
}
```

### Performance and Load Testing

**Concurrent Pull Request Processing:**

```rust
#[tokio::test]
async fn test_concurrent_pull_request_processing() {
    let test_env = IntegrationTestEnvironment::setup().await.unwrap();
    let repo = test_env.create_configured_repository("load-test").await.unwrap();

    // Create multiple PRs concurrently
    let pr_futures: Vec<_> = (0..10)
        .map(|i| {
            let repo = repo.clone();
            let test_env = test_env.clone();
            async move {
                test_env.create_test_pull_request(&repo, &PullRequestSpec {
                    title: format!("feat: concurrent feature {}", i),
                    body: format!("Test PR {}", i),
                    source_branch: format!("feature-{}", i),
                    target_branch: "main".to_string(),
                    files: vec![FileChange {
                        path: format!("feature-{}.rs", i),
                        content: format!("// Feature {} implementation", i),
                        action: FileAction::Add,
                    }],
                }).await
            }
        })
        .collect();

    let prs = futures::future::try_join_all(pr_futures).await.unwrap();

    // Wait for all processing to complete
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Verify all PRs were processed successfully
    for pr in &prs {
        let checks = test_env.get_pr_checks(&repo, pr.number).await.unwrap();
        assert!(
            checks.iter().any(|c| c.name == "merge-warden" && c.conclusion.is_some()),
            "PR {} should have completed check", pr.number
        );
    }

    test_env.cleanup().await.unwrap();
}
```

## Azure Service Mocking and Simulation

### Mock Azure Services for Local Testing

Since direct access to Azure test environments is limited, integration tests use mocking and simulation for Azure service dependencies:

**Mock Azure App Configuration:**

```rust
pub struct MockAppConfigService {
    config_store: HashMap<String, String>,
    response_delay: Duration,
    failure_rate: f32,
}

impl MockAppConfigService {
    pub fn new() -> Self {
        Self {
            config_store: Self::default_test_config(),
            response_delay: Duration::from_millis(50),
            failure_rate: 0.0,
        }
    }

    fn default_test_config() -> HashMap<String, String> {
        [
            ("merge-warden:policies:pr-title:format", "conventional-commits"),
            ("merge-warden:policies:pr-body:require-work-item", "true"),
            ("merge-warden:policies:pr-size:enabled", "true"),
            ("merge-warden:labels:size:enabled", "true"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    pub async fn get_configuration(&self, key: &str) -> Result<String, MockError> {
        // Simulate network delay
        tokio::time::sleep(self.response_delay).await;

        // Simulate occasional failures
        if rand::random::<f32>() < self.failure_rate {
            return Err(MockError::ServiceUnavailable);
        }

        self.config_store
            .get(key)
            .cloned()
            .ok_or(MockError::KeyNotFound)
    }

    pub fn set_failure_rate(&mut self, rate: f32) {
        self.failure_rate = rate;
    }

    pub fn update_config(&mut self, key: String, value: String) {
        self.config_store.insert(key, value);
    }
}

#[async_trait]
impl AppConfigClient for MockAppConfigService {
    async fn get_configuration_value(&self, key: &str) -> Result<String, AppConfigError> {
        self.get_configuration(key)
            .await
            .map_err(|e| AppConfigError::ServiceError(e.to_string()))
    }
}
```

**Mock Azure Key Vault:**

```rust
pub struct MockKeyVaultService {
    secrets: HashMap<String, String>,
    response_delay: Duration,
    failure_rate: f32,
}

impl MockKeyVaultService {
    pub fn new() -> Self {
        Self {
            secrets: Self::default_test_secrets(),
            response_delay: Duration::from_millis(100),
            failure_rate: 0.0,
        }
    }

    fn default_test_secrets() -> HashMap<String, String> {
        [
            ("github-app-private-key", "test-private-key-content"),
            ("github-webhook-secret", "test-webhook-secret"),
            ("github-app-id", "123456"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    pub async fn get_secret(&self, name: &str) -> Result<String, MockError> {
        tokio::time::sleep(self.response_delay).await;

        if rand::random::<f32>() < self.failure_rate {
            return Err(MockError::ServiceUnavailable);
        }

        self.secrets
            .get(name)
            .cloned()
            .ok_or(MockError::SecretNotFound)
    }

    pub fn simulate_outage(&mut self) {
        self.failure_rate = 1.0;
    }

    pub fn restore_service(&mut self) {
        self.failure_rate = 0.0;
    }
}

#[async_trait]
impl KeyVaultClient for MockKeyVaultService {
    async fn get_secret(&self, name: &str) -> Result<String, KeyVaultError> {
        self.get_secret(name)
            .await
            .map_err(|e| KeyVaultError::ServiceError(e.to_string()))
    }
}
```

**Integration Test Environment with Mocks:**

```rust
pub struct IntegrationTestEnvironment {
    pub repo_manager: TestRepositoryManager,
    pub bot_instance: TestBotInstance,
    pub mock_app_config: MockAppConfigService,
    pub mock_key_vault: MockKeyVaultService,
    pub github_client: GitHubClient,
}

impl IntegrationTestEnvironment {
    pub async fn setup_for_glitchgrove() -> Result<Self, TestError> {
        let github_token = env::var("GITHUB_TEST_TOKEN")
            .expect("GITHUB_TEST_TOKEN required for integration tests");

        Ok(Self {
            repo_manager: TestRepositoryManager::new(github_token.clone()),
            bot_instance: TestBotInstance::new_for_local_testing(),
            mock_app_config: MockAppConfigService::new(),
            mock_key_vault: MockKeyVaultService::new(),
            github_client: GitHubClient::new(github_token),
        })
    }

    pub async fn simulate_app_config_outage(&mut self) {
        self.mock_app_config.set_failure_rate(1.0);
    }

    pub async fn restore_app_config(&mut self) {
        self.mock_app_config.set_failure_rate(0.0);
    }

    pub async fn update_app_config(&mut self, key: &str, value: &str) {
        self.mock_app_config.update_config(key.to_string(), value.to_string());
    }
}
```

### End-to-End Validation Scenarios

**Complete Merge Workflow:**

```rust
#[tokio::test]
async fn test_complete_merge_workflow() {
    let test_env = IntegrationTestEnvironment::setup().await.unwrap();
    let repo = test_env.create_configured_repository("merge-workflow-test").await.unwrap();

    // Create and validate PR
    let pr = test_env.create_test_pull_request(&repo, &PullRequestSpec {
        title: "feat: implement merge workflow",
        body: "This PR implements the complete merge workflow.\n\nCloses #456",
        source_branch: "feature/merge-workflow",
        target_branch: "main",
        files: vec![
            FileChange {
                path: "src/merge_workflow.rs",
                content: include_str!("../test-data/merge_workflow.rs"),
                action: FileAction::Add,
            },
        ],
    }).await.unwrap();

    // Wait for initial validation
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify initial validation passed
    let initial_checks = test_env.get_pr_checks(&repo, pr.number).await.unwrap();
    assert!(initial_checks.iter().any(|c| c.conclusion == Some("success".to_string())));

    // Add approval
    test_env.create_pr_review(&repo, pr.number, &ReviewRequest {
        event: "APPROVE".to_string(),
        body: Some("LGTM!".to_string()),
    }).await.unwrap();

    // Wait for review processing
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Verify merge requirements are met
    let merge_status = test_env.get_pr_merge_status(&repo, pr.number).await.unwrap();
    assert!(merge_status.mergeable);

    // Perform merge
    test_env.merge_pull_request(&repo, pr.number, &MergeRequest {
        commit_title: Some("feat: implement merge workflow (#1)".to_string()),
        commit_message: Some("Implements complete merge workflow functionality".to_string()),
        merge_method: "squash".to_string(),
    }).await.unwrap();

    // Verify post-merge state
    let merged_pr = test_env.get_pull_request(&repo, pr.number).await.unwrap();
    assert_eq!(merged_pr.state, "closed");
    assert!(merged_pr.merged);

    test_env.cleanup().await.unwrap();
}
```

## Test Environment Configuration

### Environment Variables

**Required Configuration for GlitchGrove Testing:**

```bash
# GitHub Configuration for glitchgrove organization
GITHUB_TEST_TOKEN=github_pat_... # Personal access token with repo permissions in glitchgrove
GITHUB_TEST_APP_ID=123456 # Test GitHub App ID
GITHUB_TEST_PRIVATE_KEY=... # Test GitHub App private key content
GITHUB_TEST_WEBHOOK_SECRET=test-webhook-secret
GITHUB_TEST_ORGANIZATION=glitchgrove

# Local Development Configuration
LOCAL_WEBHOOK_ENDPOINT=http://localhost:7071/api/webhook
NGROK_AUTH_TOKEN=... # For webhook forwarding (optional)

# Mock Service Configuration (replaces Azure services for testing)
USE_MOCK_SERVICES=true
MOCK_APP_CONFIG_ENABLED=true
MOCK_KEY_VAULT_ENABLED=true

# Test Infrastructure
TEST_TIMEOUT_SECONDS=30
TEST_CLEANUP_ENABLED=true
TEST_PARALLEL_EXECUTION=false
TEST_REPOSITORY_PREFIX=merge-warden-test
```

**Optional Azure Configuration (when available):**

```bash
# Azure Configuration (only when Azure test environment is available)
AZURE_TENANT_ID=test-tenant-id
AZURE_CLIENT_ID=test-client-id
AZURE_CLIENT_SECRET=test-client-secret
AZURE_APP_CONFIG_ENDPOINT=https://test-appconfig.azconfig.io
AZURE_KEY_VAULT_URL=https://test-keyvault.vault.azure.net/
USE_REAL_AZURE_SERVICES=false # Set to true when Azure is available
```

### Test Data Management

**Test Configuration Templates:**

```toml
# test-data/test-merge-warden.toml
schemaVersion = 1

[policies.pullRequests.prTitle]
format = "conventional-commits"
requireScope = false

[policies.pullRequests.prBody]
requireWorkItemReference = true
workItemPattern = "(Fixes|Closes|Resolves) #\\d+"

[policies.pullRequests.prSize]
enabled = true
maxLines = 1000

[labels.size]
enabled = true
thresholds = { XS = 10, S = 100, M = 300, L = 1000 }
```

### CI/CD Integration

**GitHub Actions Integration Test Workflow for GlitchGrove:**

```yaml
name: Integration Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push' || (github.event_name == 'pull_request' && contains(github.event.pull_request.labels.*.name, 'run-integration-tests'))

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup Azure Functions Core Tools
        run: |
          npm install -g azure-functions-core-tools@4

      - name: Setup test environment
        run: |
          # Install test dependencies
          cargo install --path crates/cli

      - name: Start local Functions host
        run: |
          # Start Azure Functions host for webhook endpoint
          cd crates/azure-functions
          func start --port 7071 &
          sleep 10 # Wait for startup

      - name: Run integration tests
        env:
          GITHUB_TEST_TOKEN: ${{ secrets.GLITCHGROVE_TEST_TOKEN }}
          GITHUB_TEST_APP_ID: ${{ secrets.GLITCHGROVE_APP_ID }}
          GITHUB_TEST_PRIVATE_KEY: ${{ secrets.GLITCHGROVE_APP_PRIVATE_KEY }}
          GITHUB_TEST_WEBHOOK_SECRET: ${{ secrets.GLITCHGROVE_WEBHOOK_SECRET }}
          GITHUB_TEST_ORGANIZATION: glitchgrove
          USE_MOCK_SERVICES: true
          LOCAL_WEBHOOK_ENDPOINT: http://localhost:7071/api/webhook
        run: cargo test --test integration_tests -- --nocapture      - name: Upload test artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: integration-test-logs
          path: test-logs/
```

## Quality Gates and Acceptance Criteria

### Success Criteria

**Integration Test Coverage:**

- [ ] Repository setup and cleanup automation works correctly
- [ ] Bot instance configuration and webhook setup succeeds
- [ ] Pull request validation workflow processes correctly
- [ ] All expected labels and comments are applied
- [ ] Check status updates reflect validation results
- [ ] Configuration changes are detected and applied
- [ ] Error conditions are handled gracefully
- [ ] Recovery from service outages works properly
- [ ] Concurrent processing handles load appropriately

**Performance Requirements:**

- [ ] Initial webhook response within 5 seconds
- [ ] Complete validation processing within 15 seconds
- [ ] Concurrent PR processing handles 10+ simultaneous requests
- [ ] Recovery from failures completes within 30 seconds
- [ ] Memory usage remains stable during load testing

**Reliability Requirements:**

- [ ] 99% test success rate in CI environment
- [ ] Proper cleanup of all test resources
- [ ] No test interference or race conditions
- [ ] Consistent results across multiple test runs
- [ ] Graceful handling of external service limitations

## Monitoring and Observability

### Test Metrics Collection

**Test Performance Tracking:**

```rust
pub struct TestMetrics {
    webhook_response_times: Vec<Duration>,
    validation_processing_times: Vec<Duration>,
    api_call_counts: HashMap<String, u32>,
    error_counts: HashMap<String, u32>,
}

impl TestMetrics {
    pub fn record_webhook_response_time(&mut self, duration: Duration) {
        self.webhook_response_times.push(duration);
    }

    pub fn record_api_call(&mut self, endpoint: &str) {
        *self.api_call_counts.entry(endpoint.to_string()).or_insert(0) += 1;
    }

    pub fn record_error(&mut self, error_type: &str) {
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
    }

    pub fn generate_report(&self) -> TestReport {
        TestReport {
            avg_webhook_response_time: self.webhook_response_times.iter().sum::<Duration>()
                / self.webhook_response_times.len() as u32,
            max_webhook_response_time: self.webhook_response_times.iter().max().cloned(),
            total_api_calls: self.api_call_counts.values().sum(),
            error_rate: self.error_counts.values().sum::<u32>() as f64
                / self.api_call_counts.values().sum::<u32>() as f64,
        }
    }
}
```

### Integration Test Logging

**Structured Test Logging:**

```rust
use tracing::{info, warn, error, instrument};

#[instrument(fields(repo_name = %repo.name, pr_number = pr.number))]
async fn process_test_pull_request(repo: &Repository, pr: &PullRequest) -> Result<(), TestError> {
    info!("Starting pull request processing test");

    let start_time = Instant::now();

    // Test webhook processing
    let webhook_result = simulate_webhook_delivery(repo, pr).await;
    match webhook_result {
        Ok(_) => info!("Webhook processing succeeded"),
        Err(e) => {
            error!("Webhook processing failed: {}", e);
            return Err(TestError::WebhookProcessingFailed(e));
        }
    }

    let processing_time = start_time.elapsed();
    info!("Processing completed in {:?}", processing_time);

    Ok(())
}
```

## Related Documents

- **[Unit Testing](./unit-testing.md)**: Foundation testing for individual components
- **[End-to-End Testing](./end-to-end-testing.md)**: Complete system validation
- **[Performance Testing](./performance-testing.md)**: Load and performance validation
- **[Functional Requirements](../requirements/functional-requirements.md)**: Requirements validated through integration testing
- **[System Overview](../architecture/system-overview.md)**: Architecture components being tested

## Behavioral Assertions

1. Integration test framework must successfully create and configure test repositories in the `glitchgrove` GitHub organization using GitHub API
2. Bot instance configuration must complete webhook setup and verify required permissions within 30 seconds using local development or test endpoints
3. Pull request validation workflow must process webhooks and apply appropriate labels within 15 seconds using mock Azure services when real services unavailable
4. Configuration changes to merge-warden.toml must be detected and applied to subsequent PR processing within mock service environment
5. System must gracefully handle Azure service simulation failures and recover when mock services are restored
6. Concurrent processing of multiple pull requests must maintain data integrity and consistent results in test environment
7. All test resources (repositories, webhooks, installations) in `glitchgrove` organization must be properly cleaned up after test completion
8. Integration tests must achieve 95% success rate in CI environment over 100+ consecutive runs using mock services
9. Test environment setup and teardown must complete within 60 seconds per test case including local Functions host startup
10. Mock Azure services must accurately simulate real service behavior including failure modes, timeouts, and configuration updates
