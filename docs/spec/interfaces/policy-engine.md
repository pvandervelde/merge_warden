# Interface Specification: Policy Engine

**Version:** 1.0
**Last Updated:** 2026-05-21
**ADR reference:** [ADR-002-policy-engine.md](../../adr/ADR-002-policy-engine.md)
**Issue:** #162 — Minimal policy engine refactor

---

## Overview

This document specifies the concrete types and method signatures that implement the `PolicySet`
abstraction described in ADR-002. All items live in `crates/core/src/config.rs` unless stated
otherwise.

> **Scope guard:** This spec covers the initial `PolicySet` implementation.
> Org-level enforcement (`OrgPolicy`) and conditional policies will have their own interface specs.

---

## 1. `PolicySet`

```rust
/// A complete set of PR validation policies for one tier of the configuration hierarchy.
///
/// Used as both a standalone policy container and as the building block for the
/// planned `OrgPolicy { enforced: PolicySet, defaults: PolicySet }`.
///
/// # Enforcement model
///
/// Enforcement is achieved by **call-site ordering** — apply the enforcing tier last:
///
/// ```text
/// // Two tiers (application defaults + repository config):
/// let effective = app_defaults_ps.merge(&repo_ps);
///
/// // Four tiers (with org-level configuration, illustrative):
/// let effective = app_defaults_ps
///     .merge(&org_defaults_ps)
///     .merge(&repo_ps)
///     .merge(&org_enforced_ps);  // applied last — wins unconditionally
/// ```
///
/// There is no `enforced: bool` flag within `PolicySet` itself.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolicySet {
    /// PR title format and label configuration.
    pub title: PullRequestsTitlePolicyConfig,
    /// Work item reference requirement and label configuration.
    pub work_item: WorkItemPolicyConfig,
    /// PR size thresholds and labelling configuration.
    pub size: PrSizeCheckConfig,
    /// WIP detection and blocking configuration.
    pub wip: WipCheckConfig,
    /// State-based PR lifecycle label configuration.
    pub pr_state: PrStateLabelsConfig,
    /// Issue metadata propagation configuration.
    pub issue_propagation: IssuePropagationConfig,
    /// Change-type label detection, mapping, and keyword-label configuration.
    pub change_type_labels: ChangeTypeLabelConfig,
    /// Bypass rules for skipping specific validation checks.
    pub bypass_rules: BypassRules,
}

impl PolicySet {
    /// Merges `over` on top of `self` (lower-priority base).
    ///
    /// For each constituent policy, delegates to that policy struct's own static
    /// `merge` method. Higher-priority values from `over` win when they are
    /// non-default; otherwise the base value is preserved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_core::config::{PolicySet, PullRequestsTitlePolicyConfig};
    ///
    /// let base = PolicySet::default();
    /// let mut over = PolicySet::default();
    /// over.title.required = true;
    ///
    /// let merged = base.merge(&over);
    /// assert!(merged.title.required);
    /// ```
    pub fn merge(&self, over: &PolicySet) -> PolicySet;

    /// Constructs a `PolicySet` from an `ApplicationDefaults`.
    ///
    /// Extracts the policy-relevant fields from `app` into the appropriate
    /// constituent config structs. The `enable_title_validation` and
    /// `enable_work_item_validation` flags on `app` are NOT applied here — they
    /// are applied as post-merge enforcement overrides in `load_merge_warden_config`.
    pub fn from_application_defaults(app: &ApplicationDefaults) -> PolicySet;

    /// Constructs a `PolicySet` from a `RepositoryProvidedConfig`.
    ///
    /// Extracts the `policies.pull_requests.*` fields and `change_type_labels` from
    /// `repo` into the appropriate constituent config structs.
    pub fn from_repository_config(repo: &RepositoryProvidedConfig) -> PolicySet;
}
```

---

## 2. Merge Methods on Constituent Config Types

Each config type listed below gains a `pub(crate)` static `merge` method. The method is
`pub(crate)` because it is only called by `PolicySet::merge`; external callers compose
policies through `PolicySet`.

### 2.1 `PullRequestsTitlePolicyConfig::merge`

```rust
impl PullRequestsTitlePolicyConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `required`: `base.required || over.required` (OR — once required, stays required)
    /// - `pattern`: `over.pattern` if non-empty and not equal to
    ///   `CONVENTIONAL_COMMIT_REGEX`; otherwise `base.pattern`
    /// - `label_if_missing`: `over.label_if_missing.or_else(|| base.label_if_missing.clone())`
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.2 `WorkItemPolicyConfig::merge`

```rust
impl WorkItemPolicyConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `required`: `base.required || over.required`
    /// - `pattern`: `over.pattern` if non-empty and not equal to `WORK_ITEM_REGEX`;
    ///   otherwise `base.pattern`
    /// - `label_if_missing`: `over.label_if_missing.or_else(|| base.label_if_missing.clone())`
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.3 `PrSizeCheckConfig::merge`

```rust
impl PrSizeCheckConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - `fail_on_oversized`: `over.fail_on_oversized` wins unconditionally
    /// - `thresholds`: `over.thresholds.or_else(|| base.thresholds.clone())`
    /// - `excluded_file_patterns`: `over` if non-empty; otherwise `base`
    /// - `label_prefix`: `over.label_prefix` if not equal to the default `"size/"`;
    ///   otherwise `base.label_prefix`
    /// - `add_comment`: `over.add_comment` wins unconditionally
    /// - `ignore_deletions`: `over.ignore_deletions` wins unconditionally
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.4 `WipCheckConfig::merge`

```rust
impl WipCheckConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `enforce_wip_blocking`: `base.enforce_wip_blocking || over.enforce_wip_blocking`
    /// - `wip_label`: `over.wip_label` if not equal to the `WipCheckConfig::default()` label;
    ///   otherwise `base.wip_label`
    /// - `wip_title_patterns`: `over` if not equal to `WipCheckConfig::default().wip_title_patterns`;
    ///   otherwise `base`
    /// - `wip_description_patterns`: `over` if non-empty; otherwise `base`
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.5 `PrStateLabelsConfig::merge`

```rust
impl PrStateLabelsConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - All label name fields (`draft_label`, `review_label`, `approved_label`):
    ///   `over` value if `Some`; otherwise `base` value
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.6 `IssuePropagationConfig::merge`

```rust
impl IssuePropagationConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - All remaining boolean flags: `over` wins unconditionally
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.7 `ChangeTypeLabelConfig::merge`

This is the highest-value merge to encapsulate — it replaces the 11+ if-blocks currently
inlined in `load_merge_warden_config`.

```rust
impl ChangeTypeLabelConfig {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - `enabled`: `base.enabled || over.enabled`
    /// - `conventional_commit_mappings.*` (11 `Vec<String>` fields):
    ///   `over.field` if non-empty; otherwise `base.field`
    /// - `detection_strategy.exact_match`, `.prefix_match`, `.description_match`:
    ///   `over` wins unconditionally
    /// - `detection_strategy.common_prefixes`:
    ///   `over` if non-empty; otherwise `base`
    /// - `fallback_label_settings.name_format`:
    ///   `over` if not equal to `FallbackLabelSettings::default().name_format`; otherwise `base`
    /// - `fallback_label_settings.color_scheme`:
    ///   merge per-key: `over` key wins if present; missing keys fall through to `base`
    /// - `fallback_label_settings.create_if_missing`:
    ///   `over` wins unconditionally
    /// - `keyword_labels.breaking_change`, `.security`, `.hotfix`, `.tech_debt`:
    ///   `over.field` if `Some`; otherwise `base.field`
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

### 2.8 `BypassRules::merge`

```rust
impl BypassRules {
    /// Merges `over` on top of `base`.
    ///
    /// Field-level rules:
    /// - Each sub-rule (`title_convention`, `work_item_convention`, `size`):
    ///   `over` sub-rule if it has been explicitly configured (its user list is non-empty
    ///   or its `enabled` flag differs from the default); otherwise `base` sub-rule
    pub(crate) fn merge(base: &Self, over: &Self) -> Self;
}
```

---

## 3. Updates to `load_merge_warden_config`

The function signature is **unchanged**. Internally, after config loading, the ad-hoc ~350-line
merge block is replaced with:

```rust
// Build policy sets from each tier
let app_ps = PolicySet::from_application_defaults(app_defaults);
let repo_ps = PolicySet::from_repository_config(&config);

// Merge: app defaults are the base; repo config overrides
let mut merged_ps = app_ps.merge(&repo_ps);

// Preserved enforcement overrides — to be removed when OrgPolicy is introduced
if app_defaults.enable_title_validation {
    merged_ps.title.required = true;
}
if app_defaults.enable_work_item_validation {
    merged_ps.work_item.required = true;
}
if app_defaults.pr_size_check.enabled {
    merged_ps.size.enabled = true;
}
if app_defaults.wip_check.enforce_wip_blocking {
    merged_ps.wip.enforce_wip_blocking = true;
}

// Write merged policies back into config for conversion to CPVRC
config.policies.pull_requests.title_policies = merged_ps.title;
config.policies.pull_requests.work_item_policies = merged_ps.work_item;
config.policies.pull_requests.size_policies = merged_ps.size;
config.policies.pull_requests.wip_policies = merged_ps.wip;
config.policies.pull_requests.pr_state_policies = merged_ps.pr_state;
config.policies.pull_requests.issue_propagation = merged_ps.issue_propagation;
config.change_type_labels = Some(merged_ps.change_type_labels);
```

> **Note:** The `config.policies.bypass_rules` merge path follows the existing per-sub-rule
> pattern already present in `to_validation_config`. Bypass rule merging moves to
> `BypassRules::merge` as a follow-up cleanup.

---

## 4. No Changes to `CurrentPullRequestValidationConfiguration`

`CurrentPullRequestValidationConfiguration` (CPVRC) and
`RepositoryProvidedConfig::to_validation_config` are **unchanged**. `PolicySet` is purely a
merge-layer type. After merging, the result is written back into `RepositoryProvidedConfig`
fields, and the existing `to_validation_config` path produces CPVRC as before.

---

## 5. Test Requirements

All items below must have unit tests in `config_tests.rs`.

### 5.1 `PolicySet::merge` — structural

| Scenario | Expected |
| :--- | :--- |
| Both `base` and `over` are `PolicySet::default()` | Result equals `PolicySet::default()` |
| `over` is `PolicySet::default()` | Result equals `base` |
| `base` is `PolicySet::default()` | Result equals `over` |

### 5.2 Title policy merge

| Scenario | Expected |
| :--- | :--- |
| `base.required = true`, `over.required = false` | `result.required = true` |
| `base.required = false`, `over.required = true` | `result.required = true` |
| `over.pattern` is non-empty and non-default | `result.pattern = over.pattern` |
| `over.pattern` is empty | `result.pattern = base.pattern` |
| `over.label_if_missing = Some("x")` | `result.label_if_missing = Some("x")` |
| `over.label_if_missing = None`, `base = Some("x")` | `result.label_if_missing = Some("x")` |

### 5.3 Work-item policy merge

Mirror of title policy test cases for `WorkItemPolicyConfig`.

### 5.4 Size policy merge

| Scenario | Expected |
| :--- | :--- |
| `base.enabled = true`, `over.enabled = false` | `result.enabled = true` |
| `over.label_prefix = "pr/"` (non-default) | `result.label_prefix = "pr/"` |
| `over.thresholds = Some(custom)` | `result.thresholds = Some(custom)` |
| `over.excluded_file_patterns` non-empty | `result.excluded_file_patterns = over` |

### 5.5 WIP policy merge

| Scenario | Expected |
| :--- | :--- |
| `base.enforce_wip_blocking = true`, `over = false` | `result.enforce_wip_blocking = true` |
| `over.wip_label` is non-default | `result.wip_label = over.wip_label` |
| `over.wip_description_patterns` non-empty | `result = over` |

### 5.6 `ChangeTypeLabelConfig::merge` — commit-type mappings

| Scenario | Expected |
| :--- | :--- |
| `over.conventional_commit_mappings.feat` is non-empty | `result.feat = over.feat` |
| `over.conventional_commit_mappings.feat` is empty | `result.feat = base.feat` |
| Test repeated for all 11 commit types | Same rule applies |

### 5.7 `ChangeTypeLabelConfig::merge` — keyword labels

| Scenario | Expected |
| :--- | :--- |
| `over.keyword_labels.breaking_change = Some("semver-major")` | `result = Some("semver-major")` |
| `over.keyword_labels.breaking_change = None` | `result = base.breaking_change` |
| Same for security, hotfix, tech_debt | Same rule applies |

### 5.8 End-to-end: `load_merge_warden_config` produces identical results

For each of the 6 existing integration scenarios tested in `config_tests.rs`, verify that
the refactored `load_merge_warden_config` (using `PolicySet::merge`) produces exactly the same
`CurrentPullRequestValidationConfiguration` as the old ad-hoc code did.

---

## 6. Dependency Map

```
config.rs
  └── PolicySet (new)
        ├── from_application_defaults  → reads ApplicationDefaults
        ├── from_repository_config     → reads RepositoryProvidedConfig
        └── merge                      → delegates to constituent merge methods
              ├── PullRequestsTitlePolicyConfig::merge
              ├── WorkItemPolicyConfig::merge
              ├── PrSizeCheckConfig::merge
              ├── WipCheckConfig::merge
              ├── PrStateLabelsConfig::merge
              ├── IssuePropagationConfig::merge
              ├── ChangeTypeLabelConfig::merge
              └── BypassRules::merge

load_merge_warden_config (modified)
  └── uses PolicySet::from_* + PolicySet::merge
  └── writes merged fields back into RepositoryProvidedConfig
  └── calls existing to_validation_config → CurrentPullRequestValidationConfiguration (unchanged)
```
