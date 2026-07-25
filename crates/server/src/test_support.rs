// Shared test-only fixtures for `crate::webhook` and `crate::health` tests.
//
// Extracted because `webhook_tests.rs` and `health_tests.rs` had each grown a
// near-identical `AppState`/`GitHubClient` construction helper (see
// `docs/catalog.md` for the registered abstraction). Only compiled under
// `#[cfg(test)]` (see `mod test_support;` in `main.rs`).

use std::sync::Arc;

use github_bot_sdk::client::{ClientConfig, GitHubClient};
use merge_warden_core::config::ApplicationDefaults;
use merge_warden_developer_platforms::app_auth::AppAuthProvider;
use queue_runtime::QueueName;

use crate::health::HealthCheckConfig;
use crate::metrics::{Metrics, MetricsConfig};
use crate::webhook::AppState;

/// RSA private key used only in tests. Generated offline; never used in
/// production. Must be a valid PEM-encoded PKCS#8 or traditional RSA key so
/// `AppAuthProvider` can parse it.
pub(crate) const TEST_PEM: &str =
    include_str!("../../developer_platforms/testdata/test-rsa-key.pem");

/// Builds a `GitHubClient` whose GitHub App auth targets `base_url`.
///
/// Use `"https://api.github.com"` for tests that only exercise routing/wiring
/// and never make a real network call. Use an unroutable address such as
/// `"http://127.0.0.1:1"` (a loopback port nothing listens on) for tests that
/// need every call to fail near-instantly with `ECONNREFUSED` — a fast,
/// deterministic stand-in for "GitHub API unreachable".
pub(crate) fn github_client_for(base_url: &str) -> GitHubClient {
    let auth = AppAuthProvider::new(12345, TEST_PEM, base_url).expect("test RSA key must be valid");
    GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .expect("GitHub client must build even with an unreachable base URL")
}

/// Fields of [`test_app_state`] that vary across call sites. Every field
/// defaults to an inert "webhook mode, nothing enabled" baseline via
/// `Default`; override only what a given test needs.
pub(crate) struct TestAppStateOptions {
    /// Passed to [`github_client_for`]. Defaults to a syntactically valid,
    /// never-dialled GitHub API URL.
    pub github_base_url: &'static str,
    /// `MetricsConfig::prometheus_enabled`. Defaults to `false`.
    pub prometheus_enabled: bool,
    /// `HealthCheckConfig::full_checks_enabled`. Defaults to `false`.
    pub full_checks_enabled: bool,
    /// `AppState::queue_client` / `AppState::queue_name`, unzipped. Defaults
    /// to `None` (webhook mode).
    pub queue: Option<(Arc<dyn queue_runtime::QueueClient>, QueueName)>,
}

impl Default for TestAppStateOptions {
    fn default() -> Self {
        TestAppStateOptions {
            github_base_url: "https://api.github.com",
            prometheus_enabled: false,
            full_checks_enabled: false,
            queue: None,
        }
    }
}

/// Builds an `AppState` for router/handler tests.
///
/// `receiver` is always `None` and `metrics` is always the inert
/// [`Metrics::default`] sink — no test in this crate currently needs to
/// assert on emitted metric values, only on routing/handler behaviour, so
/// these two fields are not exposed via [`TestAppStateOptions`]. Add a field
/// here if a future test needs to vary them.
pub(crate) fn test_app_state(options: TestAppStateOptions) -> Arc<AppState> {
    let (queue_client, queue_name) = match options.queue {
        Some((client, name)) => (Some(client), Some(name)),
        None => (None, None),
    };

    Arc::new(AppState {
        receiver: None,
        github_client: github_client_for(options.github_base_url),
        policies: ApplicationDefaults::default(),
        metrics: Metrics::default(),
        metrics_config: MetricsConfig {
            otlp_endpoint: None,
            service_name: "test".to_string(),
            service_version: "0.0.0".to_string(),
            prometheus_enabled: options.prometheus_enabled,
        },
        health_check_config: HealthCheckConfig {
            full_checks_enabled: options.full_checks_enabled,
        },
        queue_client,
        queue_name,
    })
}
