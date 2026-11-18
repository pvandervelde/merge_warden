//! Integration tests for Issue #336 — application-level policy controlling how
//! merge-warden reacts to a broken `.github/merge-warden.toml` on the default
//! branch.
//!
//! These exercise `resolve_pull_request_config` end-to-end with an in-process
//! mock `ConfigFetcher` returning malformed TOML, asserting the populated
//! `broken_repo_config` notice for each of the three operator modes. No live
//! GitHub credentials are required.
//!
//! RED test suite (TDD): references the not-yet-implemented
//! `BrokenRepoConfigBehaviour` enum and the
//! `CurrentPullRequestValidationConfiguration::broken_repo_config` field.
//! Expected to FAIL TO COMPILE until the implementation lands.

use async_trait::async_trait;
use merge_warden_core::config::{
    resolve_pull_request_config, ApplicationDefaults, BrokenRepoConfigBehaviour,
};
use merge_warden_core::NoopMetricsRecorder;
use merge_warden_developer_platforms::{errors::Error as PlatformError, ConfigFetcher};

/// Returns the same (malformed) content for every fetch call.
struct ConstantFetcher {
    content: Option<String>,
}

impl ConstantFetcher {
    fn returns(content: &str) -> Self {
        Self {
            content: Some(content.to_string()),
        }
    }
}

#[async_trait]
impl ConfigFetcher for ConstantFetcher {
    async fn fetch_config(
        &self,
        _owner: &str,
        _repo: &str,
        _path: &str,
    ) -> Result<Option<String>, PlatformError> {
        Ok(self.content.clone())
    }

    async fn fetch_config_at_ref(
        &self,
        _owner: &str,
        _repo: &str,
        _path: &str,
        _git_ref: &str,
    ) -> Result<Option<String>, PlatformError> {
        Ok(self.content.clone())
    }
}

/// Malformed TOML that forces a BROKEN-config error in the repo tier.
const MALFORMED_TOML: &str = "not = valid = toml [[[";

#[tokio::test]
async fn malformed_repo_config_silent_fallback_yields_no_notice() {
    let fetcher = ConstantFetcher::returns(MALFORMED_TOML);
    let app = ApplicationDefaults::default(); // SilentFallback

    let cfg = resolve_pull_request_config(
        "owner",
        "repo",
        ".github/merge-warden.toml",
        &fetcher,
        &app,
        None,
        &NoopMetricsRecorder,
    )
    .await
    .expect("resolution must degrade gracefully");

    assert_eq!(
        cfg.broken_repo_config, None,
        "SilentFallback must reproduce today's behaviour: no notice"
    );
}

#[tokio::test]
async fn malformed_repo_config_notify_yields_notify_notice() {
    let fetcher = ConstantFetcher::returns(MALFORMED_TOML);
    let mut app = ApplicationDefaults::default();
    app.broken_repo_config_behaviour = BrokenRepoConfigBehaviour::Notify;

    let cfg = resolve_pull_request_config(
        "owner",
        "repo",
        ".github/merge-warden.toml",
        &fetcher,
        &app,
        None,
        &NoopMetricsRecorder,
    )
    .await
    .expect("resolution must degrade gracefully");

    let notice = cfg
        .broken_repo_config
        .expect("Notify must produce a notice for a broken repo config");
    assert_eq!(notice.behaviour, BrokenRepoConfigBehaviour::Notify);
    assert!(
        !notice.error_detail.trim().is_empty(),
        "the notice must carry a non-empty error detail"
    );
}

#[tokio::test]
async fn malformed_repo_config_block_yields_block_notice() {
    let fetcher = ConstantFetcher::returns(MALFORMED_TOML);
    let mut app = ApplicationDefaults::default();
    app.broken_repo_config_behaviour = BrokenRepoConfigBehaviour::Block;

    let cfg = resolve_pull_request_config(
        "owner",
        "repo",
        ".github/merge-warden.toml",
        &fetcher,
        &app,
        None,
        &NoopMetricsRecorder,
    )
    .await
    .expect("resolution must degrade gracefully");

    assert_eq!(
        cfg.broken_repo_config.map(|n| n.behaviour),
        Some(BrokenRepoConfigBehaviour::Block),
        "Block must produce a Block notice for a broken repo config"
    );
}
