//! Integration tests for four-tier org-level policy configuration resolution.
//!
//! These tests exercise the complete config resolution chain
//! (`resolve_pull_request_config`) using in-process mock `ConfigFetcher`
//! implementations.  No live GitHub credentials are required.
//!
//! Covered behaviours:
//!
//! - Four-tier merge order: app defaults → org defaults → repo → org enforced
//! - `org_policy_source` absent → identical output to three-tier system
//! - Org enforced settings cannot be overridden by repo config
//! - Org defaults are overridable by repo config
//! - `fail_if_unreachable = false` — org policy unavailable degrades gracefully
//! - `fail_if_unreachable = true`  — org policy unavailable returns error
//! - Enforcement flags (app-level) always win as the top tier
//! - Schema version validation (non-1 rejected)
//! - Sample org policy TOML parses successfully

use async_trait::async_trait;
use merge_warden_core::config::{
    resolve_pull_request_config, ApplicationDefaults, OrgPolicySource,
};
use merge_warden_developer_platforms::{errors::Error as PlatformError, ConfigFetcher};

// ---------------------------------------------------------------------------
// Mock ConfigFetcher implementations
// ---------------------------------------------------------------------------

/// Returns the same content for every fetch call.
struct ConstantFetcher {
    content: Option<String>,
}

impl ConstantFetcher {
    fn returns(content: &str) -> Self {
        Self {
            content: Some(content.to_string()),
        }
    }

    fn returns_none() -> Self {
        Self { content: None }
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

/// Routes fetches based on the requested path only.
///
/// **Note:** `owner` and `repo` arguments are ignored — routing is purely
/// path-based. This is intentional for test simplicity: tests register a
/// path string and any call to that path returns the associated content
/// regardless of which owner/repo is requested.
struct PathRoutingFetcher {
    routes: Vec<(String, String)>,
}

impl PathRoutingFetcher {
    fn new(routes: Vec<(&str, &str)>) -> Self {
        Self {
            routes: routes
                .into_iter()
                .map(|(p, c)| (p.to_string(), c.to_string()))
                .collect(),
        }
    }
}

#[async_trait]
impl ConfigFetcher for PathRoutingFetcher {
    async fn fetch_config(
        &self,
        _owner: &str,
        _repo: &str,
        path: &str,
    ) -> Result<Option<String>, PlatformError> {
        for (route_path, content) in &self.routes {
            if path == route_path {
                return Ok(Some(content.clone()));
            }
        }
        Ok(None)
    }

    async fn fetch_config_at_ref(
        &self,
        _o: &str,
        _r: &str,
        _p: &str,
        _ref: &str,
    ) -> Result<Option<String>, PlatformError> {
        Ok(None)
    }
}

/// Always returns a fetch error.
struct AlwaysFailingFetcher;

#[async_trait]
impl ConfigFetcher for AlwaysFailingFetcher {
    async fn fetch_config(
        &self,
        _owner: &str,
        _repo: &str,
        _path: &str,
    ) -> Result<Option<String>, PlatformError> {
        Err(PlatformError::ApiError())
    }

    async fn fetch_config_at_ref(
        &self,
        _o: &str,
        _r: &str,
        _p: &str,
        _ref: &str,
    ) -> Result<Option<String>, PlatformError> {
        Err(PlatformError::ApiError())
    }
}

// ---------------------------------------------------------------------------
// Helper: standard org policy source pointing at "org-policy.toml"
// ---------------------------------------------------------------------------

fn org_source(fail: bool) -> OrgPolicySource {
    OrgPolicySource {
        owner: "my-org".to_string(),
        repo: "platform-configs".to_string(),
        path: "org-policy.toml".to_string(),
        fail_if_unreachable: fail,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// When `org_policy_source` is absent the result must be identical to the
/// three-tier system driven purely by app defaults and repo config.
#[tokio::test]
async fn no_org_source_behaves_like_three_tier_system() {
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
pattern = "^feat:"
"#;
    let fetcher = ConstantFetcher::returns(repo_toml);
    let app = ApplicationDefaults::default();

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    assert!(cfg.enforce_title_convention);
    assert_eq!(cfg.title_pattern, "^feat:");
}

/// App enforcement flags win over every other tier.
#[tokio::test]
async fn app_enforcement_flags_are_highest_priority() {
    // Org enforced and repo both set title required = false.
    let org_toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = false

[defaults]
"#;

    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = false
"#;

    let fetcher = PathRoutingFetcher::new(vec![
        ("org-policy.toml", org_toml),
        ("repo.toml", repo_toml),
    ]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(false));
    app.enable_title_validation = true; // app enforcement flag → always wins

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    assert!(
        cfg.enforce_title_convention,
        "App enforcement flag must override org enforced + repo setting"
    );
}

/// Org enforced setting beats repo config.
#[tokio::test]
async fn org_enforced_overrides_repo_config() {
    let org_toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = true
pattern = "^ORG-ENFORCED:"

[defaults]
"#;

    // Repo explicitly disables title enforcement.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = false
"#;

    let fetcher = PathRoutingFetcher::new(vec![
        ("org-policy.toml", org_toml),
        ("repo.toml", repo_toml),
    ]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(false));

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    assert!(
        cfg.enforce_title_convention,
        "Org enforced title must override repo disabled setting"
    );
    assert_eq!(cfg.title_pattern, "^ORG-ENFORCED:");
}

/// Org defaults can be overridden by repo config.
#[tokio::test]
async fn repo_config_overrides_org_defaults() {
    let org_toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.prTitle]
required = true
pattern = "^ORG-DEFAULT:"
"#;

    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
pattern = "^REPO:"
"#;

    let fetcher = PathRoutingFetcher::new(vec![
        ("org-policy.toml", org_toml),
        ("repo.toml", repo_toml),
    ]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(false));

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    assert!(cfg.enforce_title_convention);
    assert_eq!(
        cfg.title_pattern, "^REPO:",
        "Repo pattern must take precedence over org default"
    );
}

/// When repo omits a field the org default is used.
#[tokio::test]
async fn org_defaults_fill_fields_absent_in_repo_config() {
    let org_toml = r#"
schemaVersion = 1

[enforced]

[defaults.policies.pullRequests.workItem]
required = true
pattern = "JIRA-[0-9]+"
"#;

    // Repo does not mention workItem at all.
    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = false
"#;

    let fetcher = PathRoutingFetcher::new(vec![
        ("org-policy.toml", org_toml),
        ("repo.toml", repo_toml),
    ]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(false));

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    assert!(
        cfg.enforce_work_item_references,
        "Org default work_item.required must apply when repo omits it"
    );
    assert_eq!(
        cfg.work_item_reference_pattern, "JIRA-[0-9]+",
        "Org default work_item pattern must apply when repo omits it"
    );
}

/// Lenient mode: org policy fetch fails → degrade to three-tier, return Ok.
#[tokio::test]
async fn org_fetch_failure_lenient_degrades_gracefully() {
    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(false)); // fail_if_unreachable = false

    // AlwaysFailingFetcher will fail the org fetch; repo is also unavailable.
    let result = resolve_pull_request_config(
        "owner",
        "repo",
        "repo.toml",
        &AlwaysFailingFetcher,
        &app,
        None,
    )
    .await;

    assert!(
        result.is_ok(),
        "Lenient mode must degrade gracefully, not return Err"
    );
    let cfg = result.unwrap();
    // Falls back to app defaults only.
    assert!(!cfg.enforce_title_convention);
}

/// Strict mode: org policy fetch fails → Err(OrgPolicyUnavailable).
#[tokio::test]
async fn org_fetch_failure_strict_returns_error() {
    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(true)); // fail_if_unreachable = true

    let result = resolve_pull_request_config(
        "owner",
        "repo",
        "repo.toml",
        &AlwaysFailingFetcher,
        &app,
        None,
    )
    .await;

    assert!(
        result.is_err(),
        "Strict mode must return Err when org policy is unreachable"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Org policy unavailable"),
        "Error message should mention org policy unavailable; got: {err}"
    );
}

/// Strict mode: org policy has unsupported schema version → Err.
#[tokio::test]
async fn org_unsupported_schema_strict_returns_error() {
    let org_toml = "schemaVersion = 99\n[enforced]\n[defaults]\n";

    let fetcher = PathRoutingFetcher::new(vec![("org-policy.toml", org_toml)]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(true));

    let result =
        resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None).await;

    assert!(
        result.is_err(),
        "Strict mode must return Err for unsupported schema version"
    );
}

/// Strict mode: org policy has malformed TOML → Err.
#[tokio::test]
async fn org_malformed_toml_strict_returns_error() {
    let fetcher = PathRoutingFetcher::new(vec![("org-policy.toml", "this is not valid toml %%%")]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(true));

    let result =
        resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None).await;

    assert!(
        result.is_err(),
        "Strict mode must return Err for malformed org policy TOML"
    );
}

/// No repo config, no org config → uses app defaults only.
#[tokio::test]
async fn no_config_at_all_uses_app_defaults() {
    let fetcher = ConstantFetcher::returns_none();
    let mut app = ApplicationDefaults::default();
    app.enable_title_validation = true;
    app.default_title_pattern = "^APP-DEFAULT:".to_string();

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    assert!(
        cfg.enforce_title_convention,
        "App enforcement flag must apply even with no configs"
    );
    assert_eq!(cfg.title_pattern, "^APP-DEFAULT:");
}

/// Full four-tier stack: all tiers active simultaneously.
///
/// Resolution chain:
///   app defaults (title pattern "^APP:")
///   → org defaults (work_item required, pattern "JIRA")
///   → repo (title required, pattern "^REPO:")
///   → org enforced (work_item pattern "ORG-ENFORCED-[0-9]+", required = true)
///   → app enforcement flags (none set)
///
/// Expected:
///   title: required=true, pattern="^REPO:" (repo beats org default)
///   work_item: required=true, pattern="ORG-ENFORCED-[0-9]+" (enforced beats repo)
#[tokio::test]
async fn four_tier_full_stack_merge_order() {
    let org_toml = r#"
schemaVersion = 1

[enforced.policies.pullRequests.workItem]
required = true
pattern = "ORG-ENFORCED-[0-9]+"

[defaults.policies.pullRequests.workItem]
required = true
pattern = "JIRA-[0-9]+"
"#;

    let repo_toml = r#"
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
pattern = "^REPO:"

[policies.pullRequests.workItem]
required = true
pattern = "GH-[0-9]+"
"#;

    let fetcher = PathRoutingFetcher::new(vec![
        ("org-policy.toml", org_toml),
        ("repo.toml", repo_toml),
    ]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(org_source(false));
    app.default_title_pattern = "^APP:".to_string();

    let cfg = resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None)
        .await
        .expect("should succeed");

    // Repo title beats org default (no org enforced title).
    assert!(cfg.enforce_title_convention);
    assert_eq!(
        cfg.title_pattern, "^REPO:",
        "Repo title pattern must beat app default"
    );

    // Org enforced work_item beats repo.
    assert!(cfg.enforce_work_item_references);
    assert_eq!(
        cfg.work_item_reference_pattern, "ORG-ENFORCED-[0-9]+",
        "Org enforced work_item pattern must beat repo pattern"
    );
}

/// The sample org policy TOML file must parse successfully and produce a valid
/// resolved configuration when used as an org policy source.
#[tokio::test]
async fn sample_org_policy_toml_parses_successfully() {
    let sample = include_str!("../../../samples/merge-warden-org-policy.sample.toml");

    let source = org_source(false);
    // Route the org policy path to the sample; repo config path has no route
    // (PathRoutingFetcher returns None for unregistered paths).
    let fetcher = PathRoutingFetcher::new(vec![(&source.path, sample)]);

    let mut app = ApplicationDefaults::default();
    app.org_policy_source = Some(source);

    let result =
        resolve_pull_request_config("owner", "repo", "repo.toml", &fetcher, &app, None).await;

    assert!(
        result.is_ok(),
        "Sample org policy TOML must resolve without error, got: {:?}",
        result.err()
    );
}
