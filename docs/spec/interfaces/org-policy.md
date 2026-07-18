# Interface Specification: Org-Level Policy Configuration

**Version:** 1.0
**Last Updated:** 2026-05-28
**ADR reference:** [ADR-003-org-level-policy.md](../../adr/ADR-003-org-level-policy.md)
**Architecture:** [docs/spec/architecture/org-policy-config.md](../architecture/org-policy-config.md)
**Issue:** task 5.0 in `.llm/tasks.md`

---

## Overview

This document specifies the concrete types, method signatures, and behavioural contracts
for org-level policy configuration.  All items live in `crates/core/src/config.rs` unless
stated otherwise.

---

## 1. `OrgPolicySource`

```rust
/// Locates the org-level policy TOML file within a GitHub repository.
///
/// Added to `ApplicationDefaults` as an optional field:
/// `org_policy_source: Option<OrgPolicySource>`.
///
/// # TOML example
///
/// ```toml
/// [policies.org_policy_source]
/// owner = "my-org"
/// repo  = "platform-configs"
/// path  = "merge-warden/org-policy.toml"
/// # fail_if_unreachable = false   # optional; default false
/// ```
///
/// Note the `[policies.*]` nesting: `org_policy_source` is a field on
/// `ApplicationDefaults`, and the server/CLI config loaders only map the file's
/// `[policies]` table onto `ApplicationDefaults`. A top-level `[org_policy_source]`
/// table is silently ignored.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct OrgPolicySource {
    /// GitHub organisation or user name that owns the policy repository.
    pub owner: String,

    /// Name of the repository that holds the org policy file.
    pub repo: String,

    /// Path to the org policy TOML file within the repository,
    /// relative to the repository root.
    pub path: String,

    /// When `true`, an unreachable or unparseable org policy causes
    /// `resolve_pull_request_config` to return
    /// `Err(ConfigLoadError::OrgPolicyUnavailable)`.
    /// When `false` (default), failures degrade gracefully to the
    /// three-tier system.
    #[serde(default)]
    pub fail_if_unreachable: bool,
}
```

### Extension to `ApplicationDefaults`

```rust
pub struct ApplicationDefaults {
    // ... existing fields unchanged ...

    /// Optional pointer to an org-level policy file.
    ///
    /// When `None`, the system behaves identically to the pre-task-5
    /// three-tier configuration model.
    #[serde(default)]
    pub org_policy_source: Option<OrgPolicySource>,
}
```

---

## 2. `OrgPolicy`

```rust
/// Parsed and validated org-level policy.
///
/// Contains two `PolicySet` values at different precedence levels.
/// Both fields are populated from the `[enforced]` and `[defaults]`
/// sections of the org policy TOML file.
///
/// When either section is absent from the TOML file, the corresponding
/// `PolicySet` is `PolicySet::default()`, which has no effect on the
/// merge chain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrgPolicy {
    /// Settings that CANNOT be overridden by repo-level config.
    ///
    /// Applied as the highest-priority tier in the merge chain (after
    /// `app_enforced_ps`).
    pub enforced: PolicySet,

    /// Settings that CAN be overridden by repo-level config.
    ///
    /// Applied between `app_defaults_ps` and `repo_ps` in the merge chain.
    pub defaults: PolicySet,
}
```

### `OrgPolicyRaw` — serde intermediate

The org policy TOML mirrors the `policies.*` structure of `RepositoryProvidedConfig` but
**without** `schemaVersion` inside the subsections. `RepositoryProvidedConfig` cannot be
used directly here because its `schema_version: u32` field has no `#[serde(default)]` and
would fail deserialization when absent from `[enforced]` / `[defaults]` subsections.

An intermediate deserialisation struct `OrgPolicyRaw` is used internally by `load_org_policy`
to parse the TOML before converting to `OrgPolicy`:

```rust
/// Internal deserialisation target for the org policy TOML.
/// Not part of the public API.
///
/// Uses `OrgPolicySectionRaw` for the two sections rather than
/// `RepositoryProvidedConfig` directly, to avoid requiring a nested
/// `schemaVersion` key inside each subsection.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct OrgPolicyRaw {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,

    /// Enforced policy section.
    #[serde(default)]
    pub enforced: OrgPolicySectionRaw,

    /// Default policy section.
    #[serde(default)]
    pub defaults: OrgPolicySectionRaw,
}

/// One section (`[enforced]` or `[defaults]`) of the org policy TOML.
///
/// Contains the same policy-relevant fields as `RepositoryProvidedConfig`
/// minus `schema_version`, so neither subsection requires a `schemaVersion`
/// key in the TOML.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct OrgPolicySectionRaw {
    #[serde(default)]
    pub policies: PoliciesConfig,

    #[serde(default)]
    pub change_type_labels: Option<ChangeTypeLabelConfig>,
}
```

`load_org_policy` converts each `OrgPolicySectionRaw` to a `PolicySet` via a helper analogous
to `PolicySet::from_repository_config` — i.e., `PolicySet::from_org_section(section: &OrgPolicySectionRaw)`.

---

## 3. `load_org_policy`

```rust
/// Fetches and parses the org-level policy file.
///
/// # Returns
///
/// - `Ok(Some(OrgPolicy))` — file exists, parses successfully, schema version is `1`.
/// - `Ok(None)` — file does not exist *and* `source.fail_if_unreachable` is `false`.
/// - `Err(ConfigLoadError::OrgPolicyUnavailable)` — fetch/parse error *and*
///   `source.fail_if_unreachable` is `true`.
///
/// When `source.fail_if_unreachable` is `false` (default), all failure conditions
/// produce `Ok(None)` after emitting a `warn!` log entry with structured fields:
/// `org_owner`, `org_repo`, `org_path`, and `error` (when applicable).
///
/// # Arguments
///
/// * `source` — coordinates of the org policy file.
/// * `fetcher` — `ConfigFetcher` implementation (typically `GitHubProvider`).
pub async fn load_org_policy(
    source: &OrgPolicySource,
    fetcher: &dyn ConfigFetcher,
) -> Result<Option<OrgPolicy>, ConfigLoadError>;
```

### Behaviour contract

| Condition | `fail_if_unreachable = false` | `fail_if_unreachable = true` |
|---|---|---|
| File not found (`Ok(None)`) | `Ok(None)` + `warn!` | `Ok(None)` + `warn!` |
| Fetch error (`Err(...)`) | `Ok(None)` + `warn!` | `Err(OrgPolicyUnavailable)` |
| Parse error (bad TOML) | `Ok(None)` + `warn!` | `Err(OrgPolicyUnavailable)` |
| Schema version ≠ 1 | `Ok(None)` + `warn!` | `Err(OrgPolicyUnavailable)` |
| File valid | `Ok(Some(OrgPolicy))` | `Ok(Some(OrgPolicy))` |

---

## 4. `ConfigLoadError` extension

```rust
pub enum ConfigLoadError {
    // ... existing variants unchanged ...

    /// The org policy file could not be loaded or parsed, and
    /// `OrgPolicySource.fail_if_unreachable` is `true`.
    ///
    /// The inner `String` contains a human-readable description of the failure
    /// (fetch error message or parse error detail).
    OrgPolicyUnavailable(String),
}
```

---

## 5. `PolicySet` additions

### 5.1 `PolicySet::from_app_enforcement_flags`

```rust
impl PolicySet {
    /// Constructs a `PolicySet` containing only the settings forced by the
    /// four app-level enforcement flags on `ApplicationDefaults`.
    ///
    /// All other fields are `PolicySet::default()` so they do not override
    /// anything when this `PolicySet` is applied as the last merge tier.
    ///
    /// Fields mapped:
    /// - `enable_title_validation = true`  → `result.title.required = true`
    /// - `enable_work_item_validation = true` → `result.work_item.required = true`
    /// - `pr_size_check.enabled = true`    → `result.size.enabled = true`
    /// - `wip_check.enforce_wip_blocking = true` → `result.wip.enforce_wip_blocking = true`
    pub(crate) fn from_app_enforcement_flags(app: &ApplicationDefaults) -> PolicySet;
}
```

### 5.2 `PolicySet::to_validation_config`

```rust
impl PolicySet {
    /// Converts a fully-merged `PolicySet` into a
    /// `CurrentPullRequestValidationConfiguration`.
    ///
    /// This replaces the write-back-into-`RepositoryProvidedConfig` +
    /// `to_validation_config` pattern used in `load_merge_warden_config`.
    /// Called once per webhook event from `resolve_pull_request_config`.
    ///
    /// # Arguments
    ///
    /// * `app_defaults` — needed for fields not covered by `PolicySet`
    ///   (`bot_mention`, `default_invalid_title_label`, etc.).
    pub(crate) fn to_validation_config(
        &self,
        app_defaults: &ApplicationDefaults,
    ) -> CurrentPullRequestValidationConfiguration;
}
```

### 5.3 `PolicySet::from_org_section`

```rust
impl PolicySet {
    /// Constructs a `PolicySet` from one section of the org policy TOML
    /// (`[enforced]` or `[defaults]`).
    ///
    /// Equivalent to `from_repository_config` but operates on
    /// `OrgPolicySectionRaw` instead of `RepositoryProvidedConfig`.
    pub(crate) fn from_org_section(section: &OrgPolicySectionRaw) -> PolicySet;
}
```

---

```rust
/// Orchestrates the four-tier PR configuration resolution chain.
///
/// This is the primary entry point for platform handlers (server, CLI).
/// It replaces direct calls to `load_merge_warden_config`.
///
/// # Resolution order (highest priority last in merge chain)
///
/// 1. Application defaults (`PolicySet::from_application_defaults`)
/// 2. Org defaults (from `OrgPolicy.defaults`, if `org_policy_source` is set)
/// 3. Repository config (`load_merge_warden_config` result)
/// 4. Org enforced (from `OrgPolicy.enforced`, if `org_policy_source` is set)
/// 5. App-level enforcement flags (`PolicySet::from_app_enforcement_flags`)
///
/// # Arguments
///
/// * `repo_owner` — GitHub repository owner.
/// * `repo_name` — GitHub repository name.
/// * `config_path` — path to the repo policy TOML file
///   (typically `".github/merge-warden.toml"`).
/// * `fetcher` — `ConfigFetcher` implementation used for both repo config
///   and org policy fetches.
/// * `app_defaults` — application-level policy defaults and configuration
///   pointers (including `org_policy_source`).
///
/// # Returns
///
/// * `Ok(CurrentPullRequestValidationConfiguration)` — always returned
///   unless `app_defaults.org_policy_source.fail_if_unreachable = true`
///   and the org policy cannot be loaded.
/// * `Err(ConfigLoadError::OrgPolicyUnavailable)` — only when strict mode
///   is enabled and the org policy is unreachable or unparseable.
///
/// # Errors
///
/// Repo config load failures (file missing, parse error) are handled
/// internally by falling back to `PolicySet::default()` for the repo tier,
/// matching the current behaviour of the platform handlers.
pub async fn resolve_pull_request_config(
    repo_owner: &str,
    repo_name: &str,
    config_path: &str,
    fetcher: &dyn ConfigFetcher,
    app_defaults: &ApplicationDefaults,
) -> Result<CurrentPullRequestValidationConfiguration, ConfigLoadError>;
```

---

## 7. `load_merge_warden_config` changes

The existing function retains its signature unchanged but loses the four ad-hoc enforcement
override lines that have been moved to `resolve_pull_request_config`:

```rust
// REMOVED from load_merge_warden_config — now in resolve_pull_request_config via
// PolicySet::from_app_enforcement_flags:
//
// if app_defaults.enable_title_validation       { merged_ps.title.required = true; }
// if app_defaults.enable_work_item_validation   { merged_ps.work_item.required = true; }
// if app_defaults.pr_size_check.enabled         { merged_ps.size.enabled = true; }
// if app_defaults.wip_check.enforce_wip_blocking { merged_ps.wip.enforce_wip_blocking = true; }
```

The function continues to return `RepositoryProvidedConfig` and applies the `PolicySet`
two-tier merge (app defaults + repo config) as before — minus the enforcement flags.

---

## 8. Platform handler migration

### `crates/server/src/webhook.rs`

```rust
// BEFORE:
let validation_config = match load_merge_warden_config(
    repo_owner, repo_name, merge_warden_config_path, &provider, &self.policies,
).await {
    Ok(config) => config.to_validation_config(&self.policies.bypass_rules),
    Err(e) => {
        warn!("Failed to load ... Using defaults.");
        CurrentPullRequestValidationConfiguration { /* large inline fallback */ }
    }
};

// AFTER:
let validation_config = match resolve_pull_request_config(
    repo_owner, repo_name, merge_warden_config_path, &provider, &self.policies,
).await {
    Ok(config) => config,
    Err(e) => {
        warn!("Failed to resolve PR config: {}. Using compiled-in defaults.", e);
        CurrentPullRequestValidationConfiguration::from_app_defaults(&self.policies)
    }
};
```

`CurrentPullRequestValidationConfiguration::from_app_defaults` is a new convenience
constructor that builds a baseline CPVRC from `ApplicationDefaults` without any repo or org
overrides. It replaces the large inline fallback struct construction currently in the handler.

### `crates/cli/src/commands/check_pr.rs`

Same pattern as above — replace `load_merge_warden_config` with `resolve_pull_request_config`.

---

## 9. Test requirements

All items must have unit tests in `config_tests.rs`.  Integration-level tests for the four-tier
merge chain go in `crates/integration-tests/`.

### 9.1 `load_org_policy` unit tests

| Scenario | Expected |
|---|---|
| `fetcher` returns `Ok(None)` (file absent), `fail_if_unreachable = false` | `Ok(None)`, `warn!` emitted |
| `fetcher` returns `Err(...)`, `fail_if_unreachable = false` | `Ok(None)`, `warn!` emitted |
| TOML parse fails, `fail_if_unreachable = false` | `Ok(None)`, `warn!` emitted |
| `schemaVersion ≠ 1`, `fail_if_unreachable = false` | `Ok(None)`, `warn!` emitted |
| `fetcher` returns `Ok(None)`, `fail_if_unreachable = true` | `Ok(None)` |
| `fetcher` returns `Err(...)`, `fail_if_unreachable = true` | `Err(OrgPolicyUnavailable)` |
| TOML parse fails, `fail_if_unreachable = true` | `Err(OrgPolicyUnavailable)` |
| Valid TOML with `[enforced]` only | `Ok(Some(OrgPolicy { enforced: <parsed>, defaults: default }))` |
| Valid TOML with `[defaults]` only | `Ok(Some(OrgPolicy { enforced: default, defaults: <parsed> }))` |
| Valid TOML with both sections | `Ok(Some(OrgPolicy { enforced: <parsed>, defaults: <parsed> }))` |
| Empty TOML (`schemaVersion = 1` only) | `Ok(Some(OrgPolicy::default()))` |

### 9.2 `PolicySet::from_app_enforcement_flags` unit tests

| Scenario | Expected |
|---|---|
| `enable_title_validation = true` | `result.title.required = true`; all other fields default |
| `enable_work_item_validation = true` | `result.work_item.required = true`; all other fields default |
| `pr_size_check.enabled = true` | `result.size.enabled = true`; all other fields default |
| `wip_check.enforce_wip_blocking = true` | `result.wip.enforce_wip_blocking = true`; all other fields default |
| All four flags `false` | `result == PolicySet::default()` |

### 9.3 `resolve_pull_request_config` — four-tier merge unit tests

| Scenario | Expected |
|---|---|
| No `org_policy_source` → behaves identically to old `load_merge_warden_config` | All existing `config_tests.rs` scenarios pass |
| Org `[enforced]` sets `title.required = true`; repo sets `required = false` | `result.enforce_title_convention = true` |
| Org `[defaults]` sets `work_item.required = true`; repo sets `required = true` | `result.enforce_work_item_references = true` |
| Org `[defaults]` sets `work_item.pattern = "JIRA-\d+"`;  repo sets `pattern = "GH-\d+"` | `result.work_item_reference_pattern = "GH-\\d+"` |
| Org `[defaults]` sets `work_item.pattern = "JIRA-\d+"`; repo omits pattern | `result.work_item_reference_pattern = "JIRA-\\d+"` |
| `load_org_policy` returns `Ok(None)` (degraded) | Result equals three-tier system result |
| `fail_if_unreachable = true` and org policy unreachable | `Err(ConfigLoadError::OrgPolicyUnavailable)` |

### 9.4 App-level enforcement flags removed from `load_merge_warden_config`

Verify that `load_merge_warden_config` no longer applies `enable_title_validation` etc. by
asserting that the returned `RepositoryProvidedConfig.title_policies.required` is `false`
when the app defaults have `enable_title_validation = true` but the repo TOML does not set
`required = true`.

### 9.5 `CurrentPullRequestValidationConfiguration::from_app_defaults` unit tests

| Scenario | Expected |
|---|---|
| `app_defaults.enable_title_validation = true` | `result.enforce_title_convention = true` |
| `app_defaults.default_title_pattern = "x"` | `result.title_pattern = "x"` |
| Default `ApplicationDefaults` | All fields at their known default values |

### 9.6 Backward compatibility

All existing `config_tests.rs` tests must pass without modification after the refactor.
This verifies that `resolve_pull_request_config` produces identical results to the old
`load_merge_warden_config` + inline enforcement pattern for the no-org-policy case.

---

## 10. Dependency map

```
config.rs
  └── OrgPolicySource             (new) — field on ApplicationDefaults
  └── OrgPolicy                   (new) — enforced: PolicySet, defaults: PolicySet
  └── OrgPolicyRaw                (new, pub(crate)) — serde root for org policy TOML
  └── OrgPolicySectionRaw         (new, pub(crate)) — serde type for [enforced]/[defaults]
  └── load_org_policy             (new) — fetch + parse + validate
  └── PolicySet::from_org_section (new, pub(crate)) — OrgPolicySectionRaw → PolicySet
  └── PolicySet::from_app_enforcement_flags  (new) — builds enforcement tier
  └── PolicySet::to_validation_config        (new) — replaces write-back pattern
  └── resolve_pull_request_config (new) — four-tier orchestrator
  └── load_merge_warden_config    (modified) — enforcement flags removed

  └── ConfigLoadError::OrgPolicyUnavailable  (new variant)

  └── CurrentPullRequestValidationConfiguration::from_app_defaults  (new)

webhook.rs / check_pr.rs
  └── resolve_pull_request_config replaces load_merge_warden_config
  └── CurrentPullRequestValidationConfiguration::from_app_defaults replaces inline fallback
```
