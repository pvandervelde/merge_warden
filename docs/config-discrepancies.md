# Configuration Documentation Discrepancies

This document lists discrepancies found between the Rust source code configuration structs
and the user-facing documentation under `docs/user/` (reference, how-to, explanation, and
tutorial docs). All non-reference docs (`how-to/`, `explanation/`, `tutorials/`) were
audited; only the items listed here were found to be inaccurate.

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

## 6. `configure-org-policy.md` "Available policy settings" table is incomplete

**Category:** Missing from docs

**Docs location:** `docs/user/how-to/configure-org-policy.md`, `## Available policy settings` table

The table lists the following as the complete set of configurable org-policy settings:

```
prTitle:       required, pattern, label_if_missing
workItem:      required, pattern, label_if_missing
prSize:        enabled, fail_on_oversized, label_prefix, add_comment
wip:           enforce_wip_blocking
bypassRules.*: enabled, users
```

**Code behaviour:** The `OrgPolicySectionRaw` struct contains the full `PoliciesConfig` and an
optional `ChangeTypeLabelConfig`, both of which expose substantially more settings than the table
shows. The complete list of omissions:

| Missing section/field | Code location |
| :--- | :--- |
| `[*.policies.pullRequests.prState]` (entire section) | `PrStateLabelsConfig` |
| `[*.policies.pullRequests.issuePropagation]` (entire section) | `IssuePropagationConfig` |
| `[*.policies.pullRequests.renovateStability]` (entire section) | `RenovateStabilityConfig` |
| `[*.change_type_labels]` (entire section) | `ChangeTypeLabelConfig` |
| `[*.policies.pullRequests.prSize]` — `excluded_file_patterns`, `thresholds`, `ignore_deletions` | `PrSizeCheckConfig` |
| `[*.policies.pullRequests.wip]` — `wip_label`, `wip_title_patterns`, `wip_description_patterns` | `WipCheckConfig` |

All these sections and fields are present in `samples/merge-warden-org-policy.sample.toml`.

**Code location:** `crates/core/src/config.rs` — `OrgPolicySectionRaw`, `PullRequestsPoliciesConfig`,
`PrSizeCheckConfig`, `WipCheckConfig`

```rust
pub(crate) struct OrgPolicySectionRaw {
    pub policies: PoliciesConfig,
    pub change_type_labels: Option<ChangeTypeLabelConfig>,
}
```

**Impact:** Platform engineers reading `configure-org-policy.md` will believe that WIP label
customisation, PR state labels, issue propagation, Renovate stability, and change-type labels
cannot be controlled at the org level. In practice all of those can be set in the
`[enforced.*]` and `[defaults.*]` sections.

---

## 7. `bypass-rules.md` (explanation) incorrectly states app-level bypass rules cannot be overridden

**Category:** Wrong description

**Config option:** `[policies.bypass_rules.*]` in the application config (`MERGE_WARDEN_CONFIG_FILE`)

**Docs location:** `docs/user/explanation/bypass-rules.md`, `## Security considerations` section:

> If this is a concern for your organisation, define bypass rules in the **application-level
> configuration** (`MERGE_WARDEN_CONFIG_FILE`) instead. The application config is controlled
> by the operator and **cannot be overridden by per-repo configs**.

**Code behaviour:** This is the same underlying bug as discrepancy #5 but repeated in a different
file. Per-repository bypass rules **can** override application-level bypass rules through
`BypassRules::merge`. Only org-policy `[enforced]`-tier bypass rules are truly non-overridable.

**Code location:** `crates/core/src/config.rs` — `BypassRules::merge`

**Impact:** Security-conscious operators who want to enforce a fixed bypass list will follow this
advice, configure app-level bypass rules, and be surprised when repositories with
`.github/merge-warden.toml` files can still override those rules.

---

## 8. `configure-renovate-stability.md` — per-repo `enabled = false` cannot override app-level `enabled = true`

**Category:** Wrong instruction

**Config option:** `[policies.pullRequests.renovateStability] enabled`

**Docs location:** `docs/user/how-to/configure-renovate-stability.md`, `## Disabling the feature`:

> To fully disable the feature for a repository, set `enabled = false` explicitly in
> that repository's `.github/merge-warden.toml`:
>
> ```toml
> [policies.pullRequests.renovateStability]
> enabled = false
> ```

**Code behaviour:** The `enabled` field for Renovate stability uses OR-merge semantics:

```rust
// RenovateStabilityConfig::merge
enabled: base.enabled || over.enabled,
```

The compiled-in default for `RenovateStabilityConfig::default_enabled()` is `true`. Since the
application-level defaults are the first tier in the merge chain, the accumulated base before
reaching the repo tier already has `enabled = true`. Merging with per-repo `enabled = false`
gives `true || false = true` — the feature remains **enabled**.

The guide itself explains OR semantics one paragraph earlier and correctly states that
"application-level `enabled = false` alone is not enough to disable the feature for a repository
that has its own config file." It then contradicts itself by suggesting that per-repo `false` is
sufficient.

Per-repo `enabled = false` can only disable the feature for a specific repository when the
**entire merge chain up to that point has `enabled = false`** (i.e. app defaults, org defaults,
and all matching conditional defaults all have `enabled = false`). The guide omits this prerequisite.

**Code location:** `crates/core/src/config.rs` — `RenovateStabilityConfig::merge`,
`RenovateStabilityConfig::default_enabled`

**Impact:** A repository maintainer following this guide and setting `enabled = false` in their
`.github/merge-warden.toml` will find the Renovate stability feature still active, with no
indication of why their configuration change had no effect.

---

## Summary table

| # | File | Section | Category | Short description |
| :--- | :--- | :--- | :--- | :--- |
| 1 | `per-repo-config.md` | `[change_type_labels]` | Default mismatch | `enabled` documented as `false`; code default is `true` |
| 2 | `per-repo-config.md`, `app-config.md` | `wip_label` | Default mismatch | `wip_label` documented as `*(none)*`; code default is `"WIP"` |
| 3 | `app-config.md` | `[policies.change_type_labels]` | Missing from docs | Entire section absent from app-config reference |
| 4 | `app-config.md` | `[policies.pr_size_check]` | Missing from docs | `thresholds` sub-section not mentioned |
| 5 | `app-config.md` | `[policies.bypass_rules.*]` | Wrong description | Bypass rules described as non-overridable by per-repo config; they are overridable |
| 6 | `how-to/configure-org-policy.md` | Available policy settings table | Incomplete | Table omits `prState`, `issuePropagation`, `renovateStability`, `change_type_labels`, and several `prSize`/`wip` fields |
| 7 | `explanation/bypass-rules.md` | Security considerations | Wrong description | App-level bypass rules claimed non-overridable by per-repo config; they are overridable |
| 8 | `how-to/configure-renovate-stability.md` | Disabling the feature | Wrong instruction | Per-repo `enabled = false` cannot override app-level `enabled = true` with OR-merge semantics |
