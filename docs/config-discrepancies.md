# Configuration Documentation Discrepancies

This document lists discrepancies found between the Rust source code configuration structs
and the user-facing reference documentation in `docs/user/reference/`.

---

## 1. `change_type_labels.enabled` — wrong default in per-repo docs

**Category:** Default value mismatch

**Config option:** `[change_type_labels] enabled`

| | Value |
| :--- | :--- |
| **Code** | `true` (`ChangeTypeLabelConfig::default_enabled()`) |
| **`docs/user/reference/per-repo-config.md`** | `false` |

**Code location:** `crates/core/src/config.rs` — `ChangeTypeLabelConfig::default_enabled`

```rust
fn default_enabled() -> bool {
    true
}
```

**Docs location:** `docs/user/reference/per-repo-config.md`, `[change_type_labels]` table

```
| `enabled` | bool | `false` | When `true`, Merge Warden maps the PR title's commit type to a repository label. |
```

**Impact:** Users who rely on the documented default of `false` to infer that change-type label
detection is opt-in will be surprised when Merge Warden applies labels without any explicit
configuration. The default is actually **opt-out** (`enabled = true`).

---

## 2. `wip_label` — wrong default in both per-repo and app-level docs

**Category:** Default value mismatch

**Config option:** `[policies.pullRequests.wip] wip_label` (per-repo) / `[policies.wip_check] wip_label` (app)

| | Value |
| :--- | :--- |
| **Code** | `Some("WIP")` (`WipCheckConfig::default_wip_label()`) |
| **`docs/user/reference/per-repo-config.md`** | `*(none)*` |
| **`docs/user/reference/app-config.md`** | `*(none)*` |

**Code location:** `crates/core/src/config.rs` — `WipCheckConfig::default_wip_label`

```rust
fn default_wip_label() -> Option<String> {
    Some("WIP".to_string())
}
```

Confirmed by the inline doc-test in the same file:

```rust
/// let config = WipCheckConfig::default();
/// assert_eq!(config.wip_label, Some("WIP".to_string()));
```

**Docs location:**
- `docs/user/reference/per-repo-config.md`, `[policies.pullRequests.wip]` table: `*(none)*`
- `docs/user/reference/app-config.md`, `[policies.wip_check]` table: `*(none)*`

**Impact:** Both docs imply that WIP labeling is disabled by default (no label applied). In reality,
the default label is `"WIP"`, so any PR with a matching WIP title pattern will receive that label
without any explicit configuration.

---

## 3. `[policies.change_type_labels]` section missing from app-level docs

**Category:** Missing from docs

**Config option:** `[policies.change_type_labels]` and all its sub-sections in the application config

| | Status |
| :--- | :--- |
| **Code** | `ApplicationDefaults.change_type_labels: ChangeTypeLabelConfig` — fully configurable |
| **`samples/app-config.sample.toml`** | Documented with a complete example |
| **`docs/user/reference/app-config.md`** | **Not documented at all** |

**Code location:** `crates/core/src/config.rs`

```rust
pub struct ApplicationDefaults {
    // ...
    #[serde(default)]
    pub change_type_labels: ChangeTypeLabelConfig,
    // ...
}
```

The server wraps `ApplicationDefaults` under a `[policies]` section, so the TOML key is
`[policies.change_type_labels]` and all its sub-sections:

- `[policies.change_type_labels]` — `enabled`
- `[policies.change_type_labels.conventional_commit_mappings]` — per-type label lists
- `[policies.change_type_labels.fallback_label_settings]` — `name_format`, `create_if_missing`
- `[policies.change_type_labels.fallback_label_settings.color_scheme]` — per-type hex colors
- `[policies.change_type_labels.detection_strategy]` — `exact_match`, `prefix_match`, `description_match`, `common_prefixes`
- `[policies.change_type_labels.keyword_labels]` — `breaking_change`, `security`, `hotfix`, `tech_debt`

**Sample file location:** `samples/app-config.sample.toml` lines 98–149 (fully documented there)

**Docs location:** `docs/user/reference/app-config.md` — the section is entirely absent.

**Impact:** Operators cannot discover that change-type labels (including keyword labels such as
`breaking-change`, `security`, `hotfix`, `tech-debt`) can be customised at the application level
without reading the sample file.

---

## 4. `[policies.pr_size_check.thresholds]` sub-section missing from app-level docs

**Category:** Missing from docs

**Config option:** `[policies.pr_size_check.thresholds]` in the application config

| | Status |
| :--- | :--- |
| **Code** | `PrSizeCheckConfig.thresholds: Option<SizeThresholds>` — configurable |
| **`samples/app-config.sample.toml`** | Present (commented out) |
| **`docs/user/reference/app-config.md`** | `[policies.pr_size_check]` table does not mention `thresholds` |
| **`docs/user/reference/per-repo-config.md`** | `[policies.pullRequests.prSize.thresholds]` fully documented |

**Code location:** `crates/core/src/config.rs`

```rust
pub struct PrSizeCheckConfig {
    // ...
    #[serde(default)]
    pub thresholds: Option<SizeThresholds>,
    // ...
}
```

The `SizeThresholds` struct has fields `xs`, `s`, `m`, `l`, `xl` with defaults `10`, `50`, `100`, `250`, `500`.

**Docs location:** `docs/user/reference/app-config.md`, `[policies.pr_size_check]` table — `thresholds` field not listed.

**Impact:** Operators reading the app-config reference cannot discover that PR size thresholds are
also customisable at the application level, even though the per-repo reference documents the same
feature.

---

## 5. App-level bypass rules incorrectly described as non-overridable by per-repo config

**Category:** Wrong description

**Config option:** `[policies.bypass_rules.*]` in app-config

**Docs location:** `docs/user/reference/app-config.md`, `[policies.bypass_rules.*]` section:

> Bypass rules in the application config apply across all repositories and cannot be overridden by
> per-repo configs.

**Code behaviour:** Per-repository bypass rules **can** override application-level bypass rules.
In `resolve_pull_request_config` the full merge chain is:

```
app_defaults → org_defaults → conditional_defaults* → repo → conditional_enforced* → org_enforced → app_enforced
```

App-level bypass rules are loaded via `PolicySet::from_application_defaults` as the lowest-priority
baseline. `BypassRules::merge` then lets any higher-priority tier (including the repo tier) win for
any sub-rule that is explicitly configured in that tier:

```rust
title_convention: if is_configured(&over.title_convention) {
    over.title_convention.clone()  // repo wins
} else {
    base.title_convention.clone()  // app default
},
```

**Code location:** `crates/core/src/config.rs` — `BypassRules::merge` and `resolve_pull_request_config`

**Impact:** Operators may believe that setting bypass rules in the application config is the only
way to control them, without realising that per-repository `.github/merge-warden.toml` files can
add or replace those rules. This is particularly relevant for security/compliance teams that want
to enforce a fixed bypass list across all repositories.

Note: bypass rules configured in the org-policy `[enforced]` section **cannot** be overridden by
per-repo config, as that tier is applied after the repo tier. Only the app-level defaults (under
`[policies.bypass_rules.*]`) are overridable per-repo.

---

## Summary table

| # | File | Section | Category | Short description |
| :--- | :--- | :--- | :--- | :--- |
| 1 | `per-repo-config.md` | `[change_type_labels]` | Default mismatch | `enabled` documented as `false`; code default is `true` |
| 2 | `per-repo-config.md`, `app-config.md` | `wip_label` | Default mismatch | `wip_label` documented as `*(none)*`; code default is `"WIP"` |
| 3 | `app-config.md` | `[policies.change_type_labels]` | Missing from docs | Entire section absent from app-config reference |
| 4 | `app-config.md` | `[policies.pr_size_check]` | Missing from docs | `thresholds` sub-section not mentioned |
| 5 | `app-config.md` | `[policies.bypass_rules.*]` | Wrong description | Bypass rules described as non-overridable by per-repo config; they are overridable |
