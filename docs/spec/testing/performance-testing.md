# Performance Testing

Comprehensive performance testing strategy for Merge Warden to ensure the system meets performance requirements under various load conditions and can handle production-scale workloads efficiently.

## Overview

Performance testing validates that Merge Warden can handle expected load volumes while maintaining acceptable response times, resource utilization, and system stability. This includes load testing, stress testing, spike testing, and endurance testing across all system components.

## Performance Requirements

### Response Time Requirements

**Webhook Processing:**

- Initial webhook response: < 5 seconds (GitHub timeout protection)
- Complete validation processing: < 15 seconds
- Configuration reload: < 30 seconds
- Error recovery: < 60 seconds

**GitHub API Interactions:**

- Comment creation: < 3 seconds
- Label application: < 2 seconds
- Status check updates: < 2 seconds
- Pull request data retrieval: < 5 seconds

**Azure Service Integration:**

- App Config read operations: < 1 second
- Key Vault secret retrieval: < 2 seconds
- Application Insights logging: < 500ms (async)

### Throughput Requirements

**Concurrent Processing:**

- 50 simultaneous pull requests without degradation
- 100 webhook events per minute sustained
- 500 webhook events per minute peak capacity
- 1000+ GitHub API calls per hour

**Scalability Targets:**

- Support 100+ repositories per instance
- Handle 10,000+ pull requests per day
- Process 50,000+ webhook events per day
- Maintain performance with 1M+ configuration reads per day

## Performance Testing Framework

### Load Testing Infrastructure

**Test Environment Setup:**

```rust
pub struct PerformanceTestEnvironment {
    load_generators: Vec<LoadGenerator>,
    monitoring_systems: Vec<MonitoringAgent>,
    test_repositories: Vec<Repository>,
    metrics_collector: MetricsCollector,
}

impl PerformanceTestEnvironment {
    pub async fn setup_for_load_test(test_config: &LoadTestConfig) -> Result<Self, TestError> {
        let load_generators = Self::create_load_generators(test_config).await?;
        let monitoring_systems = Self::setup_monitoring().await?;
        let test_repositories = Self::create_test_repositories(test_config.repository_count).await?;
        let metrics_collector = MetricsCollector::new();

        Ok(Self {
            load_generators,
            monitoring_systems,
            test_repositories,
            metrics_collector,
        })
    }

    async fn create_load_generators(config: &LoadTestConfig) -> Result<Vec<LoadGenerator>, TestError> {
        let generators = futures::future::try_join_all(
            (0..config.generator_count).map(|i| async move {
                LoadGenerator::new(&LoadGeneratorConfig {
                    id: format!("generator-{}", i),
                    webhook_endpoint: config.webhook_endpoint.clone(),
                    github_token: config.github_tokens[i % config.github_tokens.len()].clone(),
                    rate_limit: config.requests_per_second / config.generator_count,
                }).await
            })
        ).await?;

        Ok(generators)
    }
}
```

### Metrics Collection and Analysis

**Performance Metrics Framework:**

```rust
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub webhook_response_times: Vec<Duration>,
    pub processing_completion_times: Vec<Duration>,
    pub github_api_response_times: HashMap<String, Vec<Duration>>,
    pub azure_service_response_times: HashMap<String, Vec<Duration>>,
    pub error_rates: HashMap<String, f64>,
    pub throughput_metrics: ThroughputMetrics,
    pub resource_utilization: ResourceUtilization,
}

impl PerformanceMetrics {
    pub fn calculate_percentiles(&self) -> PerformancePercentiles {
        PerformancePercentiles {
            webhook_response_p50: self.calculate_percentile(&self.webhook_response_times, 50.0),
            webhook_response_p95: self.calculate_percentile(&self.webhook_response_times, 95.0),
            webhook_response_p99: self.calculate_percentile(&self.webhook_response_times, 99.0),
            processing_completion_p50: self.calculate_percentile(&self.processing_completion_times, 50.0),
            processing_completion_p95: self.calculate_percentile(&self.processing_completion_times, 95.0),
            processing_completion_p99: self.calculate_percentile(&self.processing_completion_times, 99.0),
        }
    }

    pub fn analyze_performance_trends(&self, baseline: &PerformanceMetrics) -> PerformanceAnalysis {
        PerformanceAnalysis {
            response_time_regression: self.detect_regression(&baseline.webhook_response_times, &self.webhook_response_times),
            throughput_change: self.calculate_throughput_change(&baseline.throughput_metrics, &self.throughput_metrics),
            error_rate_increase: self.compare_error_rates(&baseline.error_rates, &self.error_rates),
            resource_efficiency: self.analyze_resource_efficiency(&baseline.resource_utilization, &self.resource_utilization),
        }
    }
}
```

## Load Testing Scenarios

### Baseline Load Testing

**Normal Operation Load Test:**

```rust
#[tokio::test]
async fn test_baseline_performance_under_normal_load() {
    let test_env = PerformanceTestEnvironment::setup_for_load_test(&LoadTestConfig {
        generator_count: 5,
        repository_count: 10,
        requests_per_second: 10,
        test_duration: Duration::from_minutes(10),
        webhook_endpoint: "https://merge-warden-perf.azurewebsites.net/api/webhook".to_string(),
        github_tokens: vec![
            "token1".to_string(),
            "token2".to_string(),
            "token3".to_string(),
        ],
    }).await.unwrap();

    // Configure realistic pull request scenarios
    let pr_scenarios = vec![
        PullRequestScenario {
            title_format: "feat: implement feature {{id}}",
            body_template: "This PR implements feature {{id}}.\n\nCloses #{{work_item}}",
            file_changes: 3,
            lines_added: 150,
            lines_deleted: 50,
            weight: 0.4, // 40% of PRs
        },
        PullRequestScenario {
            title_format: "fix: resolve bug {{id}}",
            body_template: "This PR fixes bug {{id}}.\n\nFixes #{{work_item}}",
            file_changes: 1,
            lines_added: 20,
            lines_deleted: 10,
            weight: 0.3, // 30% of PRs
        },
        PullRequestScenario {
            title_format: "docs: update documentation {{id}}",
            body_template: "Updates documentation for {{id}}",
            file_changes: 1,
            lines_added: 100,
            lines_deleted: 20,
            weight: 0.3, // 30% of PRs
        },
    ];

    // Execute load test
    let load_test_results = test_env.execute_load_test(&LoadTestExecution {
        scenarios: pr_scenarios,
        ramp_up_duration: Duration::from_minutes(2),
        steady_state_duration: Duration::from_minutes(6),
        ramp_down_duration: Duration::from_minutes(2),
    }).await.unwrap();

    // Analyze results
    let metrics = load_test_results.aggregate_metrics();
    let percentiles = metrics.calculate_percentiles();

    // Performance assertions
    assert!(percentiles.webhook_response_p95 < Duration::from_secs(5),
        "95th percentile webhook response time should be < 5s, got {:?}", percentiles.webhook_response_p95);

    assert!(percentiles.processing_completion_p95 < Duration::from_secs(15),
        "95th percentile processing completion should be < 15s, got {:?}", percentiles.processing_completion_p95);

    assert!(metrics.error_rates.get("webhook_errors").unwrap_or(&0.0) < &0.01,
        "Webhook error rate should be < 1%");

    assert!(metrics.throughput_metrics.requests_per_second >= 9.0,
        "Should maintain at least 90% of target throughput");

    // Resource utilization checks
    assert!(metrics.resource_utilization.cpu_usage_percent < 80.0,
        "CPU usage should be < 80% under normal load");

    assert!(metrics.resource_utilization.memory_usage_percent < 70.0,
        "Memory usage should be < 70% under normal load");

    test_env.cleanup().await.unwrap();
}
```

### Stress Testing

**Peak Load Stress Test:**

```rust
#[tokio::test]
async fn test_performance_under_peak_stress() {
    let test_env = PerformanceTestEnvironment::setup_for_load_test(&LoadTestConfig {
        generator_count: 20,
        repository_count: 50,
        requests_per_second: 100, // 10x normal load
        test_duration: Duration::from_minutes(15),
        webhook_endpoint: "https://merge-warden-perf.azurewebsites.net/api/webhook".to_string(),
        github_tokens: get_performance_test_tokens(),
    }).await.unwrap();

    // Create high-stress scenarios
    let stress_scenarios = vec![
        PullRequestScenario {
            title_format: "feat: large feature {{id}}",
            body_template: "Large feature implementation {{id}}.\n\nCloses #{{work_item}}",
            file_changes: 15,
            lines_added: 1000,
            lines_deleted: 200,
            weight: 0.6, // Primarily large PRs
        },
        PullRequestScenario {
            title_format: "refactor: major refactoring {{id}}",
            body_template: "Major refactoring effort {{id}}.\n\nCloses #{{work_item}}",
            file_changes: 25,
            lines_added: 2000,
            lines_deleted: 1500,
            weight: 0.4, // Very large PRs
        },
    ];

    let stress_test_results = test_env.execute_load_test(&LoadTestExecution {
        scenarios: stress_scenarios,
        ramp_up_duration: Duration::from_minutes(3),
        steady_state_duration: Duration::from_minutes(9),
        ramp_down_duration: Duration::from_minutes(3),
    }).await.unwrap();

    let metrics = stress_test_results.aggregate_metrics();
    let percentiles = metrics.calculate_percentiles();

    // Stress test assertions (relaxed but still functional)
    assert!(percentiles.webhook_response_p95 < Duration::from_secs(10),
        "95th percentile webhook response under stress should be < 10s, got {:?}", percentiles.webhook_response_p95);

    assert!(percentiles.processing_completion_p95 < Duration::from_secs(30),
        "95th percentile processing under stress should be < 30s, got {:?}", percentiles.processing_completion_p95);

    assert!(metrics.error_rates.get("webhook_errors").unwrap_or(&0.0) < &0.05,
        "Error rate under stress should be < 5%");

    // Should handle at least 50% of peak load successfully
    assert!(metrics.throughput_metrics.successful_requests_per_second >= 50.0,
        "Should handle at least 50 successful requests/second under stress");

    // Resource limits under stress
    assert!(metrics.resource_utilization.cpu_usage_percent < 95.0,
        "CPU usage should not exceed 95% under stress");

    test_env.cleanup().await.unwrap();
}
```

### Spike Testing

**Traffic Spike Handling Test:**

```rust
#[tokio::test]
async fn test_traffic_spike_handling() {
    let test_env = PerformanceTestEnvironment::setup_for_load_test(&LoadTestConfig {
        generator_count: 10,
        repository_count: 20,
        requests_per_second: 10, // Normal baseline
        test_duration: Duration::from_minutes(20),
        webhook_endpoint: "https://merge-warden-perf.azurewebsites.net/api/webhook".to_string(),
        github_tokens: get_performance_test_tokens(),
    }).await.unwrap();

    // Define spike pattern
    let spike_pattern = SpikeTestPattern {
        baseline_rps: 10,
        spike_rps: 200, // 20x spike
        spike_duration: Duration::from_minutes(2),
        recovery_duration: Duration::from_minutes(3),
        spike_count: 3,
    };

    let spike_test_results = test_env.execute_spike_test(&spike_pattern).await.unwrap();

    // Analyze spike response
    let baseline_metrics = spike_test_results.get_baseline_period_metrics();
    let spike_metrics = spike_test_results.get_spike_period_metrics();
    let recovery_metrics = spike_test_results.get_recovery_period_metrics();

    // Baseline performance should be maintained
    assert!(baseline_metrics.calculate_percentiles().webhook_response_p95 < Duration::from_secs(5));

    // During spikes, system should degrade gracefully
    assert!(spike_metrics.calculate_percentiles().webhook_response_p95 < Duration::from_secs(15),
        "Spike response time should degrade gracefully");

    // Should recover to baseline performance
    assert!(recovery_metrics.calculate_percentiles().webhook_response_p95 < Duration::from_secs(6),
        "Should recover to near-baseline performance after spike");

    // Error rates should be controlled during spikes
    assert!(spike_metrics.error_rates.get("webhook_errors").unwrap_or(&0.0) < &0.10,
        "Error rate during spikes should be < 10%");

    // System should not crash or become unresponsive
    assert!(spike_test_results.system_remained_responsive(),
        "System should remain responsive throughout spike test");

    test_env.cleanup().await.unwrap();
}
```

### Endurance Testing

**Long-Duration Stability Test:**

```rust
#[tokio::test]
async fn test_long_duration_stability() {
    let test_env = PerformanceTestEnvironment::setup_for_load_test(&LoadTestConfig {
        generator_count: 8,
        repository_count: 15,
        requests_per_second: 15,
        test_duration: Duration::from_hours(4), // Extended test
        webhook_endpoint: "https://merge-warden-perf.azurewebsites.net/api/webhook".to_string(),
        github_tokens: get_performance_test_tokens(),
    }).await.unwrap();

    let endurance_scenarios = vec![
        PullRequestScenario {
            title_format: "feat: endurance feature {{id}}",
            body_template: "Endurance test feature {{id}}.\n\nCloses #{{work_item}}",
            file_changes: 5,
            lines_added: 200,
            lines_deleted: 75,
            weight: 1.0,
        },
    ];

    let endurance_results = test_env.execute_endurance_test(&EnduranceTestExecution {
        scenarios: endurance_scenarios,
        ramp_up_duration: Duration::from_minutes(10),
        steady_state_duration: Duration::from_hours(3).saturating_sub(Duration::from_minutes(20)),
        ramp_down_duration: Duration::from_minutes(10),
        measurement_intervals: Duration::from_minutes(15),
    }).await.unwrap();

    // Analyze stability over time
    let time_series_metrics = endurance_results.get_time_series_metrics();
    let stability_analysis = time_series_metrics.analyze_stability();

    // Performance should not degrade over time
    assert!(stability_analysis.response_time_trend < 0.05, // < 5% degradation
        "Response time should not degrade significantly over time");

    assert!(stability_analysis.throughput_trend > -0.02, // > -2% degradation
        "Throughput should remain stable over time");

    // Memory usage should be stable (no leaks)
    assert!(stability_analysis.memory_growth_rate < 0.01, // < 1% per hour
        "Memory usage should not grow significantly over time");

    // Error rates should remain low and stable
    assert!(stability_analysis.error_rate_trend < 0.001, // < 0.1% increase
        "Error rates should remain stable over time");

    // Resource utilization should be consistent
    assert!(stability_analysis.cpu_usage_variance < 0.1,
        "CPU usage should be consistent over time");

    test_env.cleanup().await.unwrap();
}
```

## Component-Specific Performance Testing

### Azure Functions Cold Start Testing

**Cold Start Performance Test:**

```rust
#[tokio::test]
async fn test_azure_functions_cold_start_performance() {
    let test_env = PerformanceTestEnvironment::setup_for_component_test("azure-functions").await.unwrap();

    // Simulate cold start conditions
    test_env.force_function_app_shutdown().await.unwrap();
    tokio::time::sleep(Duration::from_minutes(5)).await; // Ensure cold state

    let cold_start_results = test_env.execute_cold_start_test(&ColdStartTestConfig {
        concurrent_requests: 10,
        request_spacing: Duration::from_millis(100),
        warmup_disabled: true,
    }).await.unwrap();

    // Cold start performance assertions
    assert!(cold_start_results.first_request_response_time < Duration::from_secs(10),
        "First request (cold start) should complete within 10 seconds");

    assert!(cold_start_results.subsequent_requests_avg_time < Duration::from_secs(3),
        "Subsequent requests should be fast after warm-up");

    // Optimization verification (from issue 178)
    assert!(cold_start_results.initialization_order.webhook_validation_first,
        "Webhook validation should occur before external service connections");

    assert!(cold_start_results.lazy_initialization_working,
        "Lazy initialization should reduce cold start impact");

    test_env.cleanup().await.unwrap();
}
```

### GitHub API Rate Limit Testing

**Rate Limit Handling Test:**

```rust
#[tokio::test]
async fn test_github_api_rate_limit_handling() {
    let test_env = PerformanceTestEnvironment::setup_for_component_test("github-integration").await.unwrap();

    // Configure test to approach rate limits
    let rate_limit_test = test_env.execute_rate_limit_test(&RateLimitTestConfig {
        requests_per_hour: 4500, // Approaching 5000 limit
        test_duration: Duration::from_minutes(30),
        monitor_headers: true,
        test_backoff_behavior: true,
    }).await.unwrap();

    // Rate limit handling assertions
    assert!(rate_limit_test.successful_requests_percentage > 95.0,
        "Should successfully handle > 95% of requests despite rate limiting");

    assert!(rate_limit_test.backoff_strategy_used,
        "Should use exponential backoff when approaching limits");

    assert!(rate_limit_test.no_rate_limit_exceeded_errors,
        "Should not exceed GitHub rate limits");

    // Response time should increase gracefully under rate pressure
    assert!(rate_limit_test.average_response_time_under_pressure < Duration::from_secs(8),
        "Average response time should remain reasonable under rate pressure");

    test_env.cleanup().await.unwrap();
}
```

### Configuration Service Performance

**Azure App Config Performance Test:**

```rust
#[tokio::test]
async fn test_azure_app_config_performance() {
    let test_env = PerformanceTestEnvironment::setup_for_component_test("app-config").await.unwrap();

    let config_perf_test = test_env.execute_config_performance_test(&ConfigPerformanceTestConfig {
        concurrent_reads: 50,
        configuration_updates_per_minute: 5,
        cache_effectiveness_test: true,
        test_duration: Duration::from_minutes(10),
    }).await.unwrap();

    // Configuration performance assertions
    assert!(config_perf_test.config_read_p95 < Duration::from_millis(500),
        "95th percentile config reads should be < 500ms");

    assert!(config_perf_test.cache_hit_rate > 0.9,
        "Configuration cache hit rate should be > 90%");

    assert!(config_perf_test.config_change_propagation_time < Duration::from_secs(30),
        "Configuration changes should propagate within 30 seconds");

    assert!(config_perf_test.no_config_read_timeouts,
        "Configuration reads should not timeout under load");

    test_env.cleanup().await.unwrap();
}
```

## Performance Monitoring and Alerting

### Real-Time Performance Monitoring

**Performance Dashboard Metrics:**

```rust
pub struct PerformanceDashboard {
    pub real_time_metrics: RealTimeMetrics,
    pub historical_trends: HistoricalTrends,
    pub performance_alerts: Vec<PerformanceAlert>,
}

impl PerformanceDashboard {
    pub async fn collect_real_time_metrics(&mut self) -> Result<(), MonitoringError> {
        self.real_time_metrics = RealTimeMetrics {
            current_rps: self.measure_current_requests_per_second().await?,
            active_webhook_processing: self.count_active_webhook_processing().await?,
            avg_response_time_5min: self.calculate_avg_response_time_window(Duration::from_minutes(5)).await?,
            error_rate_5min: self.calculate_error_rate_window(Duration::from_minutes(5)).await?,
            cpu_utilization: self.get_cpu_utilization().await?,
            memory_utilization: self.get_memory_utilization().await?,
            github_api_quota_remaining: self.get_github_api_quota().await?,
        };

        self.check_performance_thresholds().await?;
        Ok(())
    }

    async fn check_performance_thresholds(&mut self) -> Result<(), MonitoringError> {
        let thresholds = PerformanceThresholds {
            max_avg_response_time: Duration::from_secs(5),
            max_error_rate: 0.02, // 2%
            max_cpu_utilization: 0.8, // 80%
            max_memory_utilization: 0.75, // 75%
            min_github_quota_remaining: 500,
        };

        if self.real_time_metrics.avg_response_time_5min > thresholds.max_avg_response_time {
            self.performance_alerts.push(PerformanceAlert {
                severity: AlertSeverity::Warning,
                message: format!("Average response time {} exceeds threshold",
                    humantime::format_duration(self.real_time_metrics.avg_response_time_5min)),
                timestamp: Utc::now(),
                metric: "avg_response_time".to_string(),
            });
        }

        if self.real_time_metrics.error_rate_5min > thresholds.max_error_rate {
            self.performance_alerts.push(PerformanceAlert {
                severity: AlertSeverity::Critical,
                message: format!("Error rate {:.2}% exceeds threshold {:.2}%",
                    self.real_time_metrics.error_rate_5min * 100.0,
                    thresholds.max_error_rate * 100.0),
                timestamp: Utc::now(),
                metric: "error_rate".to_string(),
            });
        }

        Ok(())
    }
}
```

### Performance Regression Detection

**Automated Performance Regression Testing:**

```rust
#[tokio::test]
async fn test_performance_regression_detection() {
    let test_env = PerformanceTestEnvironment::setup_for_regression_test().await.unwrap();

    // Load baseline performance metrics
    let baseline_metrics = test_env.load_baseline_performance_metrics().await.unwrap();

    // Execute current performance test
    let current_metrics = test_env.execute_performance_baseline_test().await.unwrap();

    // Compare against baseline
    let regression_analysis = RegressionAnalyzer::analyze(&baseline_metrics, &current_metrics);

    // Regression detection assertions
    assert!(!regression_analysis.has_significant_response_time_regression(),
        "Significant response time regression detected: {}", regression_analysis.response_time_change_summary());

    assert!(!regression_analysis.has_significant_throughput_regression(),
        "Significant throughput regression detected: {}", regression_analysis.throughput_change_summary());

    assert!(!regression_analysis.has_significant_error_rate_increase(),
        "Significant error rate increase detected: {}", regression_analysis.error_rate_change_summary());

    // If performance improved, update baseline
    if regression_analysis.overall_performance_improved() {
        test_env.update_baseline_metrics(&current_metrics).await.unwrap();
        println!("Performance baseline updated due to improvements");
    }

    test_env.cleanup().await.unwrap();
}
```

## Load Test Data Generation

### Realistic Test Data Generation

**Test Data Factory:**

```rust
pub struct TestDataFactory {
    pr_title_generator: PullRequestTitleGenerator,
    commit_message_generator: CommitMessageGenerator,
    file_change_generator: FileChangeGenerator,
    user_behavior_simulator: UserBehaviorSimulator,
}

impl TestDataFactory {
    pub fn generate_realistic_pull_request(&self, scenario: &PullRequestScenario) -> PullRequestData {
        let title = self.pr_title_generator.generate(&scenario.title_format);
        let body = self.commit_message_generator.generate(&scenario.body_template);
        let file_changes = self.file_change_generator.generate_changes(
            scenario.file_changes,
            scenario.lines_added,
            scenario.lines_deleted,
        );

        PullRequestData {
            title,
            body,
            files: file_changes,
            user_behavior: self.user_behavior_simulator.generate_behavior(),
        }
    }

    pub fn generate_configuration_scenarios(&self) -> Vec<ConfigurationTestScenario> {
        vec![
            ConfigurationTestScenario {
                name: "strict-validation".to_string(),
                config: include_str!("../test-configs/strict-validation.toml").to_string(),
                expected_behavior: ValidationBehavior::Strict,
                weight: 0.3,
            },
            ConfigurationTestScenario {
                name: "permissive-validation".to_string(),
                config: include_str!("../test-configs/permissive-validation.toml").to_string(),
                expected_behavior: ValidationBehavior::Permissive,
                weight: 0.4,
            },
            ConfigurationTestScenario {
                name: "enterprise-validation".to_string(),
                config: include_str!("../test-configs/enterprise-validation.toml").to_string(),
                expected_behavior: ValidationBehavior::Enterprise,
                weight: 0.3,
            },
        ]
    }
}
```

## CI/CD Performance Testing Integration

### Automated Performance Testing Pipeline

**GitHub Actions Performance Workflow:**

```yaml
name: Performance Testing

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
    paths:
      - 'crates/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  performance-test:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' || contains(github.event.pull_request.labels.*.name, 'performance-test')

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup performance test environment
        env:
          AZURE_SUBSCRIPTION_ID: ${{ secrets.AZURE_SUBSCRIPTION_ID }}
          AZURE_CLIENT_ID: ${{ secrets.AZURE_CLIENT_ID }}
          AZURE_CLIENT_SECRET: ${{ secrets.AZURE_CLIENT_SECRET }}
          AZURE_TENANT_ID: ${{ secrets.AZURE_TENANT_ID }}
        run: |
          # Provision performance test environment
          cargo run --bin perf-env-setup

      - name: Run baseline performance tests
        env:
          PERF_TEST_DURATION_MINUTES: 10
          PERF_TEST_TARGET_RPS: 10
          GITHUB_PERF_TOKENS: ${{ secrets.GITHUB_PERF_TOKENS }}
        run: |
          cargo test --release --test performance_tests baseline_performance_test -- --nocapture

      - name: Run stress tests
        if: github.event_name == 'schedule'
        env:
          PERF_TEST_DURATION_MINUTES: 15
          PERF_TEST_TARGET_RPS: 100
          GITHUB_PERF_TOKENS: ${{ secrets.GITHUB_PERF_TOKENS }}
        run: |
          cargo test --release --test performance_tests stress_performance_test -- --nocapture

      - name: Analyze performance results
        run: |
          cargo run --bin perf-analyzer -- --results-path ./test-results/performance

      - name: Upload performance reports
        uses: actions/upload-artifact@v3
        with:
          name: performance-test-results
          path: |
            test-results/performance/
            test-logs/performance/

      - name: Comment performance results on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const perfResults = fs.readFileSync('./test-results/performance/summary.md', 'utf8');

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Performance Test Results\n\n${perfResults}`
            });

      - name: Cleanup performance test environment
        if: always()
        run: |
          cargo run --bin perf-env-cleanup
```

## Performance Optimization Validation

### Performance Improvement Verification

**Optimization Impact Testing:**

```rust
#[tokio::test]
async fn test_cold_start_optimization_impact() {
    // Test for issue 178 - cold start performance improvement
    let test_env = PerformanceTestEnvironment::setup_for_optimization_test().await.unwrap();

    // Test original behavior (before optimization)
    let pre_optimization_results = test_env.execute_cold_start_test(&ColdStartTestConfig {
        initialization_strategy: InitializationStrategy::EagerAll,
        concurrent_requests: 10,
        request_spacing: Duration::from_millis(100),
    }).await.unwrap();

    // Test optimized behavior (after optimization)
    let post_optimization_results = test_env.execute_cold_start_test(&ColdStartTestConfig {
        initialization_strategy: InitializationStrategy::LazyAfterValidation,
        concurrent_requests: 10,
        request_spacing: Duration::from_millis(100),
    }).await.unwrap();

    // Verify optimization impact
    let improvement = OptimizationImpactAnalyzer::analyze(
        &pre_optimization_results,
        &post_optimization_results,
    );

    assert!(improvement.cold_start_time_reduction > 0.3, // 30% improvement
        "Cold start optimization should reduce time by at least 30%");

    assert!(improvement.webhook_timeout_rate_reduction > 0.8, // 80% reduction
        "Webhook timeout rate should be reduced by at least 80%");

    assert!(post_optimization_results.webhook_validation_completes_first,
        "Webhook validation should complete before external service connections");

    test_env.cleanup().await.unwrap();
}
```

## Quality Gates and Acceptance Criteria

### Performance Quality Gates

**Automated Performance Criteria:**

- [ ] Webhook response time 95th percentile < 5 seconds under normal load
- [ ] Processing completion 95th percentile < 15 seconds under normal load
- [ ] Error rate < 1% under normal load conditions
- [ ] System handles 50+ concurrent pull requests without significant degradation
- [ ] Cold start optimization reduces initial response time by 30%
- [ ] Memory usage remains stable over 4+ hour endurance tests
- [ ] Performance regression detection catches > 10% degradations
- [ ] Rate limit handling maintains > 95% success rate
- [ ] Configuration service reads < 500ms 95th percentile
- [ ] System recovers to baseline performance within 3 minutes after traffic spikes

### Performance Reporting

**Performance Test Report Generation:**

```rust
pub struct PerformanceReport {
    pub test_summary: TestSummary,
    pub response_time_analysis: ResponseTimeAnalysis,
    pub throughput_analysis: ThroughputAnalysis,
    pub resource_utilization: ResourceUtilizationAnalysis,
    pub error_analysis: ErrorAnalysis,
    pub recommendations: Vec<PerformanceRecommendation>,
}

impl PerformanceReport {
    pub fn generate_markdown_report(&self) -> String {
        format!(r#"
# Performance Test Report

## Test Summary
- **Test Duration**: {}
- **Target Load**: {} requests/second
- **Total Requests**: {}
- **Success Rate**: {:.2}%

## Response Time Analysis
- **Mean**: {:?}
- **95th Percentile**: {:?}
- **99th Percentile**: {:?}
- **Max**: {:?}

## Throughput Analysis
- **Achieved RPS**: {:.2}
- **Peak RPS**: {:.2}
- **Throughput Efficiency**: {:.1}%

## Resource Utilization
- **Peak CPU**: {:.1}%
- **Peak Memory**: {:.1}%
- **Average CPU**: {:.1}%
- **Average Memory**: {:.1}%

## Error Analysis
- **Total Errors**: {}
- **Error Rate**: {:.3}%
- **Timeout Errors**: {}
- **Server Errors**: {}

## Recommendations
{}

"#,
            humantime::format_duration(self.test_summary.duration),
            self.test_summary.target_rps,
            self.test_summary.total_requests,
            self.test_summary.success_rate * 100.0,
            self.response_time_analysis.mean,
            self.response_time_analysis.p95,
            self.response_time_analysis.p99,
            self.response_time_analysis.max,
            self.throughput_analysis.achieved_rps,
            self.throughput_analysis.peak_rps,
            self.throughput_analysis.efficiency * 100.0,
            self.resource_utilization.peak_cpu * 100.0,
            self.resource_utilization.peak_memory * 100.0,
            self.resource_utilization.avg_cpu * 100.0,
            self.resource_utilization.avg_memory * 100.0,
            self.error_analysis.total_errors,
            self.error_analysis.error_rate * 100.0,
            self.error_analysis.timeout_errors,
            self.error_analysis.server_errors,
            self.recommendations.iter()
                .map(|r| format!("- {}", r.description))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
```

## Related Documents

- **[Integration Testing](./integration-testing.md)**: Integration testing framework and scenarios
- **[End-to-End Testing](./end-to-end-testing.md)**: Complete system validation
- **[Unit Testing](./unit-testing.md)**: Foundation testing for individual components
- **[Monitoring](../operations/monitoring.md)**: Production monitoring and observability
- **[Deployment](../operations/deployment.md)**: Deployment procedures and infrastructure

## Behavioral Assertions

1. Webhook processing must complete initial response within 5 seconds to prevent GitHub timeout failures
2. Complete pull request validation must finish within 15 seconds under normal load conditions
3. System must handle 50 concurrent pull requests without response time degradation exceeding 20%
4. Cold start optimization must reduce initial Azure Function response time by minimum 30%
5. Error rates must remain below 1% during normal load testing scenarios
6. Memory usage must remain stable with growth rate below 1% per hour during endurance tests
7. Performance regression detection must identify degradations exceeding 10% automatically
8. GitHub API rate limit handling must maintain above 95% success rate when approaching limits
9. Configuration service reads must complete within 500ms at 95th percentile under load
10. System must recover to baseline performance within 3 minutes after traffic spike completion
