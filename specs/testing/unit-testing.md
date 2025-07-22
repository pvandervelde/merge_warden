# Unit Testing

Comprehensive unit testing strategy and implementation guidelines for Merge Warden components, ensuring robust, maintainable, and reliable code through systematic testing approaches.

## Overview

This document defines the unit testing standards, patterns, and requirements for Merge Warden development. Unit tests form the foundation of our testing pyramid, providing fast feedback, regression protection, and code quality assurance for individual components and functions.

## Testing Philosophy

### Core Principles

**Test-Driven Development (TDD):**

- Write tests before implementation when feasible
- Use tests to drive design decisions
- Maintain tests as living documentation
- Ensure tests describe expected behavior clearly

**Test Pyramid Foundation:**

- Unit tests: 70% of total test coverage
- Integration tests: 20% of total test coverage
- End-to-end tests: 10% of total test coverage

**Quality Standards:**

- Every public function must have unit tests
- Critical business logic requires 100% test coverage
- Edge cases and error conditions must be tested
- Tests must be deterministic and fast

## Unit Testing Framework

### Rust Testing Stack

**Built-in Testing:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_output);
    }
}
```

**Testing Dependencies:**

```toml
[dev-dependencies]
tokio-test = "0.4"      # Async testing utilities
mockall = "0.11"        # Mock generation
proptest = "1.0"        # Property-based testing
assert_matches = "1.5"  # Enhanced assertions
tempfile = "3.0"        # Temporary file testing
wiremock = "0.5"        # HTTP service mocking
```

### Test Organization

**File Structure:**

```
crates/
├── core/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── validation.rs
│   │   └── validation_tests.rs  # Inline tests
│   └── tests/
│       └── integration_tests.rs
├── azure-functions/
│   ├── src/
│   │   ├── main.rs
│   │   └── webhook_handler.rs
│   └── tests/
│       └── webhook_tests.rs
```

**Test Module Conventions:**

- Unit tests in same file as implementation: `mod tests`
- Complex test utilities in separate files: `_tests.rs` suffix
- Test-only dependencies in `[dev-dependencies]`
- Shared test utilities in `tests/common/` directory

## Testing Patterns and Standards

### AAA Pattern (Arrange-Act-Assert)

**Standard Structure:**

```rust
#[test]
fn should_validate_conventional_commit_title() {
    // Arrange
    let title = "feat: add new validation rule";
    let config = ValidationConfig::default();
    let validator = TitleValidator::new(&config);

    // Act
    let result = validator.validate(title);

    // Assert
    assert!(result.is_valid());
    assert_eq!(result.commit_type(), Some("feat"));
}
```

### Test Naming Conventions

**Descriptive Test Names:**

```rust
// Good: Describes behavior and context
#[test]
fn should_return_error_when_title_missing_type() { }

#[test]
fn should_apply_size_label_when_pr_exceeds_large_threshold() { }

#[test]
fn should_bypass_validation_when_user_has_admin_permission() { }

// Bad: Vague or implementation-focused
#[test]
fn test_validation() { }

#[test]
fn test_title_parser_edge_case() { }
```

### Parameterized Testing

**Test Cases with Multiple Inputs:**

```rust
#[cfg(test)]
mod title_validation_tests {
    use super::*;
    use test_case::test_case;

    #[test_case("feat: add feature", true; "valid feature")]
    #[test_case("fix: resolve bug", true; "valid fix")]
    #[test_case("invalid title", false; "missing type")]
    #[test_case("feat:", false; "missing description")]
    fn should_validate_title_format(title: &str, expected_valid: bool) {
        let validator = TitleValidator::default();
        let result = validator.validate(title);
        assert_eq!(result.is_valid(), expected_valid);
    }
}
```

### Property-Based Testing

**Testing with Generated Data:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn pr_size_calculation_should_never_be_negative(
        additions in 0u32..10000,
        deletions in 0u32..10000
    ) {
        let pr_info = PullRequestInfo {
            additions,
            deletions,
            changed_files: 1,
        };

        let size = calculate_pr_size(&pr_info);
        prop_assert!(size.total_lines() >= 0);
    }
}
```

## Component-Specific Testing

### Core Validation Logic

**Validation Engine Tests:**

```rust
#[cfg(test)]
mod validation_engine_tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn should_validate_all_policies_when_enabled() {
        // Arrange
        let mut mock_title_validator = MockTitleValidator::new();
        mock_title_validator
            .expect_validate()
            .with(eq("feat: add feature"))
            .returning(|_| ValidationResult::valid());

        let mut mock_workitem_validator = MockWorkItemValidator::new();
        mock_workitem_validator
            .expect_validate()
            .with(eq("Fixes #123"))
            .returning(|_| ValidationResult::valid());

        let engine = ValidationEngine::new(
            Box::new(mock_title_validator),
            Box::new(mock_workitem_validator),
        );

        let pr = PullRequest {
            title: "feat: add feature".to_string(),
            body: "Fixes #123".to_string(),
            ..Default::default()
        };

        // Act
        let result = engine.validate(&pr);

        // Assert
        assert!(result.is_valid());
        assert_eq!(result.violations().len(), 0);
    }
}
```

### Configuration Management Tests

**Configuration Loading and Validation:**

```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn should_load_valid_toml_configuration() {
        // Arrange
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
            schemaVersion = 1

            [policies.pullRequests.prTitle]
            format = "conventional-commits"
            "#
        ).unwrap();

        // Act
        let config = Config::load_from_file(temp_file.path()).unwrap();

        // Assert
        assert_eq!(config.schema_version, 1);
        assert_eq!(
            config.policies.pull_requests.pr_title.format,
            TitleFormat::ConventionalCommits
        );
    }

    #[test]
    fn should_return_error_for_invalid_schema_version() {
        // Arrange
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"schemaVersion = 999"#).unwrap();

        // Act
        let result = Config::load_from_file(temp_file.path());

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::UnsupportedSchemaVersion(999)
        ));
    }
}
```

### Labeling System Tests

**Label Application Logic:**

```rust
#[cfg(test)]
mod labeling_tests {
    use super::*;

    #[test]
    fn should_apply_correct_size_label() {
        // Arrange
        let labeler = SizeLabeler::new(&SizeLabelConfig::default());
        let pr_info = PullRequestInfo {
            additions: 150,
            deletions: 50,
            changed_files: 5,
        };

        // Act
        let labels = labeler.generate_labels(&pr_info);

        // Assert
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "size/M");
        assert_eq!(labels[0].color, "fbca04");
    }

    #[test]
    fn should_map_conventional_commit_to_repository_labels() {
        // Arrange
        let repo_labels = vec![
            Label { name: "enhancement".to_string(), color: "0075ca".to_string() },
            Label { name: "bug".to_string(), color: "d73a4a".to_string() },
        ];

        let labeler = ChangeTypeLabeler::new(&repo_labels);
        let commits = vec![
            Commit { message: "feat: add new feature".to_string() },
        ];

        // Act
        let labels = labeler.generate_labels(&commits);

        // Assert
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "enhancement");
    }
}
```

### GitHub Integration Tests

**API Client Unit Tests:**

```rust
#[cfg(test)]
mod github_client_tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

    #[tokio::test]
    async fn should_create_pr_comment_successfully() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/issues/123/comments"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::new(
            "test-token".to_string(),
            mock_server.uri()
        );

        // Act
        let result = client.create_comment(
            "owner",
            "repo",
            123,
            "Test comment"
        ).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

## Mocking and Test Doubles

### Trait-Based Mocking

**Mockable Traits:**

```rust
#[cfg_attr(test, automock)]
pub trait GitHubApi {
    async fn get_pull_request(&self, owner: &str, repo: &str, number: u64)
        -> Result<PullRequest, GitHubError>;

    async fn create_comment(&self, owner: &str, repo: &str, number: u64, body: &str)
        -> Result<Comment, GitHubError>;

    async fn add_labels(&self, owner: &str, repo: &str, number: u64, labels: &[String])
        -> Result<(), GitHubError>;
}

// Implementation
pub struct GitHubClient {
    token: String,
    base_url: String,
}

impl GitHubApi for GitHubClient {
    async fn get_pull_request(&self, owner: &str, repo: &str, number: u64)
        -> Result<PullRequest, GitHubError> {
        // Implementation
    }
}
```

**Using Mocks in Tests:**

```rust
#[cfg(test)]
mod service_tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn should_process_webhook_successfully() {
        // Arrange
        let mut mock_github = MockGitHubApi::new();
        mock_github
            .expect_get_pull_request()
            .with(eq("owner"), eq("repo"), eq(123))
            .returning(|_, _, _| Ok(test_pull_request()));

        mock_github
            .expect_add_labels()
            .with(eq("owner"), eq("repo"), eq(123), eq(vec!["size/M".to_string()]))
            .returning(|_, _, _, _| Ok(()));

        let service = WebhookService::new(Box::new(mock_github));

        // Act
        let result = service.process_pull_request_event(&webhook_payload()).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

### Test Data Builders

**Builder Pattern for Test Data:**

```rust
#[cfg(test)]
pub struct PullRequestBuilder {
    pr: PullRequest,
}

impl PullRequestBuilder {
    pub fn new() -> Self {
        Self {
            pr: PullRequest {
                number: 1,
                title: "Default title".to_string(),
                body: "Default body".to_string(),
                additions: 10,
                deletions: 5,
                changed_files: 2,
                ..Default::default()
            }
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.pr.title = title.to_string();
        self
    }

    pub fn additions(mut self, additions: u32) -> Self {
        self.pr.additions = additions;
        self
    }

    pub fn build(self) -> PullRequest {
        self.pr
    }
}

// Usage in tests
#[test]
fn should_label_large_pr() {
    let pr = PullRequestBuilder::new()
        .title("feat: add large feature")
        .additions(1500)
        .build();

    let labels = generate_size_labels(&pr);
    assert!(labels.contains(&"size/L".to_string()));
}
```

## Async Testing

### Tokio Test Runtime

**Async Test Setup:**

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn should_process_webhook_asynchronously() {
        let service = WebhookService::new();
        let payload = webhook_payload();

        let result = service.process(payload).await;

        assert!(result.is_ok());
    }

    #[test]
    fn should_timeout_slow_operations() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let future = slow_operation();
            let result = tokio::time::timeout(
                Duration::from_secs(1),
                future
            ).await;

            assert!(result.is_err()); // Should timeout
        });
    }
}
```

### Testing Error Handling

**Error Condition Testing:**

```rust
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn should_handle_network_timeout_gracefully() {
        let mut mock_client = MockHttpClient::new();
        mock_client
            .expect_post()
            .returning(|_| Err(HttpError::Timeout));

        let service = Service::new(Box::new(mock_client));
        let result = service.send_notification("test").await;

        assert!(matches!(result, Err(ServiceError::NetworkTimeout)));
    }

    #[test]
    fn should_retry_on_transient_failures() {
        let mut mock_client = MockHttpClient::new();
        mock_client
            .expect_post()
            .times(3)
            .returning(|_| Err(HttpError::ServerError(500)));

        let service = Service::with_retry_policy(
            Box::new(mock_client),
            RetryPolicy::new().max_attempts(3)
        );

        let result = service.send_notification("test").await;
        assert!(result.is_err());
    }
}
```

## Performance and Benchmarking

### Criterion Benchmarks

**Benchmark Setup:**

```rust
// benches/validation_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use merge_warden_core::validation::TitleValidator;

fn benchmark_title_validation(c: &mut Criterion) {
    let validator = TitleValidator::default();
    let titles = vec![
        "feat: add new feature",
        "fix: resolve critical bug",
        "docs: update README",
    ];

    c.bench_function("title_validation", |b| {
        b.iter(|| {
            for title in &titles {
                validator.validate(black_box(title));
            }
        })
    });
}

criterion_group!(benches, benchmark_title_validation);
criterion_main!(benches);
```

### Memory Usage Testing

**Memory Leak Detection:**

```rust
#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn should_not_leak_memory_during_validation() {
        let validator = ValidationEngine::new();
        let initial_memory = get_memory_usage();

        // Process many PRs
        for i in 0..1000 {
            let pr = generate_test_pr(i);
            validator.validate(&pr);
        }

        let final_memory = get_memory_usage();
        let memory_growth = final_memory - initial_memory;

        // Allow for some growth but detect significant leaks
        assert!(memory_growth < 10_000_000); // 10MB threshold
    }
}
```

## Coverage and Quality Metrics

### Coverage Requirements

**Minimum Coverage Targets:**

- Core validation logic: 95%
- Configuration management: 90%
- GitHub integration: 85%
- CLI components: 80%
- Overall project: 85%

**Coverage Collection:**

```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

### Code Quality Checks

**Automated Quality Gates:**

```bash
# Run all unit tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Run security audit
cargo audit

# Generate coverage report
cargo tarpaulin --fail-under 85
```

## Continuous Integration

### GitHub Actions Integration

**Test Workflow:**

```yaml
name: Unit Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt

      - name: Run tests
        run: cargo test --all

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: ./cobertura.xml
```

## Related Documents

- **[Integration Testing](./integration-testing.md)**: Component integration testing strategies
- **[End-to-End Testing](./end-to-end-testing.md)**: Full system testing approaches
- **[Performance Testing](./performance-testing.md)**: Performance and load testing
- **[Core Components](../architecture/core-components.md)**: Architecture of testable components
