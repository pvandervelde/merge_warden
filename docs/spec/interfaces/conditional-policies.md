# Interface Specification: Conditional Org Policies

**Version:** 1.0
**Last Updated:** 2026-05-30
**ADR reference:** [ADR-004-conditional-org-policies.md](../../adr/ADR-004-conditional-org-policies.md)
**Issue:** task 6.0 in `.llm/tasks.md`

---

## Overview

This document specifies the concrete types, method signatures, and behavioural contracts
for conditional org-level policies based on repository topics and custom properties.

All config types live in `crates/core/src/config.rs`.  The `RepositoryMetadataProvider`
trait and `RepositoryContext` model live in `crates/developer_platforms/src/`.

---

## 1. `RepositoryContext` (developer_platforms::models)

```rust
/// Repository metadata fetched to evaluate conditional org policy conditions.
///
/// Populated by [`RepositoryMetadataProvider::get_repository_context`] before
/// conditional policy evaluation in [`resolve_pull_request_config`].
///
/// # Enterprise note
///
/// `custom_properties` is empty on non-enterprise GitHub plans because the
/// custom properties API returns 403/404 for non-enterprise organisations.
/// All topic-based conditions continue to work on all plans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryContext {
    /// Repository topics as set in GitHub repository settings.
    ///
    /// All topic strings are lowercased to match GitHub's canonical representation.
    pub topics: Vec<String>,

    /// Repository-level custom properties (GitHub Enterprise only).
    ///
    /// Maps property name to its string value.  Empty on non-enterprise plans
    /// or when the app lacks the `org_custom_property: read` permission.
    pub custom_properties: HashMap<String, String>,
}
```

---

## 2. `RepositoryMetadataProvider` (developer_platforms)

```rust
/// Fetches repository-level metadata required for conditional policy evaluation.
///
/// Implemented by [`github::GitHubProvider`] using two independent GitHub API calls:
/// - `GET /repos/{owner}/{repo}/topics` — available on all GitHub plans.
/// - `GET /repos/{owner}/{repo}/properties/values` — GitHub Enterprise only.
///
/// # Graceful degradation
///
/// - Topics API errors produce `Err` (topics are expected to work on all plans).
/// - Custom properties API 403/404 is treated as an empty property map and logged
///   at `debug!` — this is the expected response on non-enterprise GitHub plans.
///
/// # Examples
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::RepositoryMetadataProvider;
/// use merge_warden_developer_platforms::errors::Error;
/// use async_trait::async_trait;
///
/// #[derive(Debug)]
/// struct MyProvider;
///
/// #[async_trait]
/// impl RepositoryMetadataProvider for MyProvider {
///     async fn get_repository_context(
///         &self,
///         repo_owner: &str,
///         repo_name: &str,
///     ) -> Result<merge_warden_developer_platforms::models::RepositoryContext, Error> {
///         use std::collections::HashMap;
///         use merge_warden_developer_platforms::models::RepositoryContext;
///         Ok(RepositoryContext {
///             topics: vec!["payments".to_string()],
///             custom_properties: HashMap::new(),
///         })
///     }
/// }
/// ```
#[async_trait]
pub trait RepositoryMetadataProvider: std::fmt::Debug + Sync + Send {
    /// Fetches topics and custom properties for the specified repository.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` — Repository owner (org or user).
    /// * `repo_name` — Repository name.
    ///
    /// # Returns
    ///
    /// - `Ok(RepositoryContext)` — topics and/or custom properties fetched (custom
    ///   properties may be empty on non-enterprise plans).
    /// - `Err(Error)` — topics fetch failed; the caller logs a `warn!` and uses
    ///   `RepositoryContext::default()` (empty topics and properties).
    async fn get_repository_context(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<RepositoryContext, Error>;
}
```

---

## 3. `PolicyCondition` (core::config)

```rust
/// Condition that gates a [`ConditionalPolicy`] block.
///
/// A condition matches a repository when **all** non-empty sub-conditions hold:
/// - `has_any_topic`: at least one listed topic is present (OR within the list).
/// - `has_custom_property`: every listed key=value pair matches (AND within the map).
///
/// An empty condition (no topics, no properties) always matches.
///
/// # Examples
///
/// ```toml
/// [conditional_policies.condition]
/// has_any_topic = ["payments", "financial"]
///
/// [conditional_policies.condition.has_custom_property]
/// team = "security"
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PolicyCondition {
    /// Topics: at least one must be present (OR semantics).
    /// Empty list means this sub-condition is not evaluated (always passes).
    #[serde(default)]
    pub has_any_topic: Vec<String>,

    /// Custom properties: all must match (AND semantics).
    /// Empty map means this sub-condition is not evaluated (always passes).
    #[serde(default)]
    pub has_custom_property: HashMap<String, String>,
}

impl PolicyCondition {
    /// Returns `true` when the condition matches `context`.
    ///
    /// # Matching rules
    ///
    /// - `has_any_topic` empty → sub-condition passes.
    /// - `has_any_topic` non-empty → at least one topic in `context.topics` must
    ///   appear in `has_any_topic` (case-insensitive comparison via lowercasing).
    /// - `has_custom_property` empty → sub-condition passes.
    /// - `has_custom_property` non-empty → every (key, value) pair must appear in
    ///   `context.custom_properties` (exact match).
    /// - Both sub-conditions must pass for the overall condition to match.
    pub fn matches(&self, context: &RepositoryContext) -> bool;
}
```

---

## 4. `ConditionalPolicy` (core::config)

```rust
/// A conditional org policy block.
///
/// Applied to repositories that match `condition`.  Contains the same
/// `enforced` / `defaults` dual-tier as [`OrgPolicy`] itself.
///
/// Both policy tiers are inserted into the merge chain (see ADR-004 §Decision 3):
/// - `defaults` is merged after `org_defaults` but before `repo`.
/// - `enforced` is merged after `repo` but before `org_enforced`.
///
/// Multiple matching blocks are merged in declaration order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConditionalPolicy {
    /// Condition that must be met for this policy block to apply.
    pub condition: PolicyCondition,
    /// Settings that CANNOT be overridden by repo-level config.
    pub enforced: PolicySet,
    /// Settings that CAN be overridden by repo-level config.
    pub defaults: PolicySet,
}
```

### Internal deserialisation type

```rust
/// Internal deserialisation target for one `[[conditional_policies]]` entry.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ConditionalPolicyRaw {
    #[serde(default)]
    pub condition: PolicyConditionRaw,
    #[serde(default)]
    pub enforced: OrgPolicySectionRaw,
    #[serde(default)]
    pub defaults: OrgPolicySectionRaw,
}

/// Deserialisation form of [`PolicyCondition`].
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct PolicyConditionRaw {
    #[serde(default)]
    pub has_any_topic: Vec<String>,
    #[serde(default)]
    pub has_custom_property: HashMap<String, String>,
}
```

---

## 5. `OrgPolicy` extension (core::config)

```rust
pub struct OrgPolicy {
    /// Settings that CANNOT be overridden by repo-level config.
    pub enforced: PolicySet,
    /// Settings that CAN be overridden by repo-level config.
    pub defaults: PolicySet,
    /// Conditional policy blocks evaluated at config resolution time.  Empty when
    /// the `[[conditional_policies]]` array is absent from the org policy TOML.
    pub conditional_policies: Vec<ConditionalPolicy>,
}
```

### `OrgPolicyRaw` extension

```rust
pub(crate) struct OrgPolicyRaw {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(default)]
    pub enforced: OrgPolicySectionRaw,
    #[serde(default)]
    pub defaults: OrgPolicySectionRaw,
    /// Array of conditional policy entries.
    #[serde(default)]
    pub conditional_policies: Vec<ConditionalPolicyRaw>,
}
```

---

## 6. `resolve_pull_request_config` extension (core::config)

```rust
/// Resolves the effective PR validation configuration using the full
/// policy precedence chain.
///
/// Extends the four-tier chain from ADR-003 with optional conditional policy tiers
/// evaluated against the repository's topics and custom properties.
///
/// # Merge order (lowest → highest priority)
///
/// 1. `app_defaults`
/// 2. `org_defaults`
/// 3. `conditional_defaults` (matching blocks, declaration order)
/// 4. `repo`
/// 5. `conditional_enforced` (matching blocks, declaration order)
/// 6. `org_enforced`
/// 7. `app_enforced`
///
/// # Arguments
///
/// * `repo_owner` — repository owner
/// * `repo_name` — repository name
/// * `config_path` — path to the per-repo config file
/// * `fetcher` — config fetcher (typically `GitHubProvider`)
/// * `app_defaults` — application-level defaults
/// * `metadata_provider` — optional provider for fetching repo metadata;
///   pass `Some(&provider)` in production, `None` in tests that do not
///   need conditional policy evaluation.
///
/// # Behaviour when `metadata_provider` is `None`
///
/// When `org_policy.conditional_policies` is non-empty and `metadata_provider` is
/// `None`, all conditional blocks are skipped (treated as not matching) and a `warn!`
/// is emitted.  This preserves backward compatibility for existing callers.
pub async fn resolve_pull_request_config(
    repo_owner: &str,
    repo_name: &str,
    config_path: &str,
    fetcher: &dyn ConfigFetcher,
    app_defaults: &ApplicationDefaults,
    metadata_provider: Option<&dyn RepositoryMetadataProvider>,
) -> Result<CurrentPullRequestValidationConfiguration, ConfigLoadError>;
```

---

## 7. Test requirements

### Unit tests for `PolicyCondition::matches`

| Scenario | Expected |
|---|---|
| Empty condition | `true` (always matches) |
| `has_any_topic = ["payments"]`, repo has `"payments"` | `true` |
| `has_any_topic = ["payments"]`, repo has `"docs"` | `false` |
| `has_any_topic = ["payments", "financial"]`, repo has `"financial"` | `true` |
| `has_any_topic = ["payments"]`, topics are empty | `false` |
| `has_custom_property = {"team": "security"}`, property matches | `true` |
| `has_custom_property = {"team": "security"}`, property absent | `false` |
| `has_custom_property = {"team": "security"}`, wrong value | `false` |
| Both `has_any_topic` and `has_custom_property` set, both match | `true` |
| Both set, topic matches but property does not | `false` |
| Both set, property matches but topic does not | `false` |

### Unit tests for conditional merge in `resolve_pull_request_config`

- Matching conditional block's `enforced` overrides repo config.
- Matching conditional block's `defaults` is overridden by repo config.
- Non-matching block has no effect.
- Two matching blocks merged in declaration order (second wins in `enforced` tier).
- Missing `metadata_provider` with conditional policies: `warn!` emitted, blocks skipped.
- `get_repository_context` error: `warn!` emitted, blocks skipped, no `Err` propagated.

### Unit tests for `GitHubProvider::get_repository_context`

- Returns topics from `/repos/{owner}/{repo}/topics` endpoint.
- Returns empty `custom_properties` on 403 from properties endpoint (no error).
- Returns empty `custom_properties` on 404 from properties endpoint (no error).
- Returns populated `custom_properties` on success.

### Integration-level tests

- Repo with matching topic receives conditional `enforced` policy (cannot be overridden by repo config).
- Repo with matching topic receives conditional `defaults` policy (can be overridden by repo config).
- Repo without matching topic is unaffected by conditional block.
