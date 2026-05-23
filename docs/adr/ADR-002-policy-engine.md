# ADR-002: Minimal Policy Engine for Merge Warden

Status: Accepted
Date: 2026-05-21
Owners: merge_warden team

## Context

`load_merge_warden_config` in `crates/core/src/config.rs` contains an approximately 350-line
ad-hoc merge block. It manually walks every field of `ChangeTypeLabelConfig` (11 commit-type
mapping vectors, fallback label settings, detection strategy flags, and keyword label overrides)
with explicit `if !field.is_empty()` guards, and separately handles WIP, PR-state, title, and
work-item policy enforcement with scattered `if app_defaults.enable_*` checks.

Problems this creates:

- Every new config field requires an additional if-block in an already-large function.
- There is no reusable or testable abstraction for "merge lower-priority defaults with
  higher-priority overrides".
- There is no formal model for "enforcement" (settings that a higher-tier config can lock
  against lower-tier override). The current ad-hoc enforcement (`enable_title_validation`,
  `enable_work_item_validation`, `pr_size_check.enabled`, `wip_check.enforce_wip_blocking`)
  is fragile and not composable.
- The planned org-level enforcement and conditional policies
  cannot be cleanly added without a formal merge abstraction.

The scope of this decision is deliberately narrow: **minimum viable change to replace the
ad-hoc merge block and lay the structural groundwork for org-level enforcement**. It is not
a general policy DSL and does not introduce runtime rule evaluation.

## Decision

Introduce `PolicySet` — a plain data struct that groups the 6 PR-validation rule configs plus
change-type labels and bypass rules. Implement `PolicySet::merge(&self, over: &PolicySet) ->
PolicySet` with "over wins for non-default values" semantics, delegating per-field logic to
`merge` methods on each constituent config type.

Replace the ad-hoc merge block in `load_merge_warden_config` with
`base_policy_set.merge(&repo_policy_set)`.

The current 4 ad-hoc enforcement overrides are preserved as explicit post-merge steps in
`load_merge_warden_config`. They will be migrated to the tier-placement model when
`OrgPolicy` is introduced.

**There is no `Policy` trait, no per-entry `enforced` flag, and no dynamic dispatch.** The
6 rule types are a fixed, known set. Enforcement semantics are achieved by call-site ordering
of `merge` calls, not by flags within `PolicySet`.

## Consequences

**Enables:**

- A reusable, testable `PolicySet::merge` that replaces the 350-line ad-hoc block.
- Later phases can introduce `OrgPolicy { enforced: PolicySet, defaults: PolicySet }` and use the
  same `merge` method: `app_defaults_ps.merge(&org_defaults).merge(&repo).merge(&org_enforced)`.
- Each constituent config type owns its own merge logic, making it easy to add new config
  fields without touching `load_merge_warden_config`.
- `PolicySet` can be unit-tested independently of the full config loading path.

**Forbids:**

- New ad-hoc merge logic added directly in `load_merge_warden_config`. All new config fields
  must be handled by the owning config type's `merge` method.
- Per-entry `enforced` flags on `PolicySet` fields (enforcement is tier-placement, not flags).
- Changing `CurrentPullRequestValidationConfiguration`'s public shape in this refactor.
  CPVRC remains the runtime consumption type; `PolicySet` is only the merge layer.

**Trade-offs:**

- This design does not yet fully implement enforcement semantics. The 4 ad-hoc overrides
  remain as technical debt until `OrgPolicy` is introduced, which is the right place to
  formalize them.
- The "non-default wins" heuristic used in `merge` requires that config types use `#[serde(default)]`
  consistently so their default states are always well-defined.
- `PolicySet` adds an additional abstraction layer but keeps the overall function surface small
  (one new struct, one new `merge` method).

## Alternatives considered

### Option A: Per-entry `PolicyEntry<T>` wrapper with an `enforced: bool` flag

Each `PolicySet` field would be `PolicyEntry<T> { value: T, enforced: bool }`. `merge` would
check the flag and refuse to override entries marked `enforced`.

**Why not**: The flag needs to be set by the *source tier*, which means callers
must construct `PolicySet` with explicit `enforced` values. This requires changing how
`ApplicationDefaults` is converted to a `PolicySet`. It also conflates "this tier enforces
this policy" with "the merged result must not be overridden", blurring responsibilities. The
tier-placement model achieves the same result more cleanly and is simpler to implement.

### Option B: A `Policy` trait with dynamic dispatch

Define a `Policy` trait with `fn name() -> &str` and `fn merge(base: &dyn Policy, over: &dyn Policy) -> Box<dyn Policy>`. `PolicySet` would hold `Vec<Box<dyn Policy>>`.

**Why not**: The rule types are a fixed, known set — trait objects add runtime cost and
complexity without benefit. Matching on policy names to access specific rule configs would be
error-prone and non-idiomatic Rust.

### Option C: Keep the ad-hoc merge block, only extract `ChangeTypeLabelConfig::merge`

Extract only the 11-field `ChangeTypeLabelConfig` merge block into a method, leaving other
ad-hoc logic in place.

**Why not**: This solves the most acute pain point but leaves the underlying structural problem
unaddressed. Next phase would still need a general merge abstraction. Doing a half-measure here
makes the next phase harder, not easier.

### Option D: Use `Option<T>` for all config fields to distinguish "explicitly set" vs "default"

Wrap every config field in `Option<T>` so `None` = "not set, use lower tier" and `Some(v)` =
"explicitly configured to v". `merge` becomes trivial: `over.field.or(base.field)`.

**Why not**: This requires changing every config struct and every TOML deserialization path.
It is a large, breaking change to the public config API. It also makes defaults harder to
reason about. The "non-default value" heuristic used by the existing code and preserved in
this design is good enough for the use cases at hand.

## Implementation notes

### `PolicySet` struct

```rust
/// A complete set of PR validation policies for one tier of the configuration hierarchy.
///
/// Used as both a standalone policy container and as the building block for the
/// planned `OrgPolicy { enforced: PolicySet, defaults: PolicySet }`.
///
/// # Merge semantics
///
/// `PolicySet::merge` takes `&self` as the lower-priority base and `over` as the
/// higher-priority override. For each constituent policy, merge delegates to that
/// policy's own `merge` method.
///
/// Enforcement is **not** encoded as flags within `PolicySet`. Instead, enforcement
/// is achieved by applying a higher-priority `PolicySet` last in the call chain:
///
/// ```text
/// // Two tiers (application defaults + repository config):
/// let effective = app_defaults_ps.merge(&repo_policy_set);
///
/// // Four tiers (once org-level configuration is available):
/// let effective = app_defaults_ps
///     .merge(&org_defaults_ps)
///     .merge(&repo_policy_set)
///     .merge(&org_enforced_ps);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySet {
    pub title: PullRequestsTitlePolicyConfig,
    pub work_item: WorkItemPolicyConfig,
    pub size: PrSizeCheckConfig,
    pub wip: WipCheckConfig,
    pub pr_state: PrStateLabelsConfig,
    pub issue_propagation: IssuePropagationConfig,
    pub change_type_labels: ChangeTypeLabelConfig,
    pub bypass_rules: BypassRules,
}
```

### Merge method on `PolicySet`

```rust
impl PolicySet {
    /// Merges `over` on top of `self` (lower-priority base).
    ///
    /// For each constituent policy, delegates to that policy's own `merge`
    /// implementation. Higher-priority values from `over` replace lower-priority
    /// values from `self` when they are non-default.
    pub fn merge(&self, over: &PolicySet) -> PolicySet {
        PolicySet {
            title: PullRequestsTitlePolicyConfig::merge(&self.title, &over.title),
            work_item: WorkItemPolicyConfig::merge(&self.work_item, &over.work_item),
            size: PrSizeCheckConfig::merge(&self.size, &over.size),
            wip: WipCheckConfig::merge(&self.wip, &over.wip),
            pr_state: PrStateLabelsConfig::merge(&self.pr_state, &over.pr_state),
            issue_propagation: IssuePropagationConfig::merge(
                &self.issue_propagation,
                &over.issue_propagation,
            ),
            change_type_labels: ChangeTypeLabelConfig::merge(
                &self.change_type_labels,
                &over.change_type_labels,
            ),
            bypass_rules: BypassRules::merge(&self.bypass_rules, &over.bypass_rules),
        }
    }
}
```

### Merge semantics per config type

Each constituent config type implements `fn merge(base: &Self, over: &Self) -> Self`. The
merge rules follow the current ad-hoc behavior exactly, now encoded in the owning type:

| Field type | Merge rule |
| :--- | :--- |
| `bool` — activation flag (`required`, `enabled`, `enforce_*`, `sync_milestone_from_issue`, `sync_project_from_issue`) | `base \|\| over` — once activated by either tier, stays active |
| `String` — regex pattern or label prefix | `over` if non-empty and not equal to the type's `default()` value; otherwise `base` |
| `Option<String>` — optional label | `over.or(base)` |
| `Vec<String>` — label name candidates | `over` if non-empty; otherwise `base` |
| `bool` — non-activation flag (`fail_on_oversized`, `add_comment`, `exact_match`, `prefix_match`, `description_match`, etc.) | `over` wins unconditionally (higher tier's preference applies) |
| `Option<SizeThresholds>` | `over.or(base)` |
| `HashMap<String, String>` — colour scheme | merge per-key: `over` key wins if present |

> **Trade-off — `bool` non-activation flags with `true` defaults:** Fields that default to
> `true` (e.g., `add_comment`, `exact_match`, `prefix_match`, `description_match`) are
> asymmetric. If the app tier sets one to `false` but the repo config omits the field
> (serde default = `true`), "over wins unconditionally" causes the repo's serde default
> to override the operator's explicit `false`. Fixing this correctly requires `Option<bool>`
> on those fields, which is a wider refactor deferred to a future cleanup. Operators who
> need to enforce a `false` value should use the org-policy enforcement tier once available.

### Conversion helpers

Add `PolicySet::from_application_defaults(app: &ApplicationDefaults) -> PolicySet` and
`PolicySet::from_repository_config(repo: &RepositoryProvidedConfig) -> PolicySet` to build
`PolicySet` instances from the existing types. These replace the conversion logic currently
inlined in `load_merge_warden_config`.

### Preserved ad-hoc enforcement overrides

After calling `app_defaults_ps.merge(&repo_ps)`, the following enforcement overrides
are applied on the merged `PolicySet` before converting to
`CurrentPullRequestValidationConfiguration`:

- `if app_defaults.enable_title_validation { merged.title.required = true; }`
- `if app_defaults.enable_work_item_validation { merged.work_item.required = true; }`
- `if app_defaults.pr_size_check.enabled { merged.size.enabled = true; }`
- `if app_defaults.wip_check.enforce_wip_blocking { merged.wip.enforce_wip_blocking = true; }`

These will be removed when the org-level enforcement tier (`OrgPolicy`) is introduced.

### `RepositoryProvidedConfig` and `ApplicationDefaults` relationship

`RepositoryProvidedConfig` and `ApplicationDefaults` are unchanged. The new
`PolicySet::from_*` helpers extract the relevant policy fields into a `PolicySet`. The
existing `RepositoryProvidedConfig::to_validation_config` method is unchanged and continues
to convert the merged result to `CurrentPullRequestValidationConfiguration`.

### Testing

Unit tests for `PolicySet::merge` must cover:

- All 8 constituent policies are merged independently.
- Merge of two defaults produces defaults.
- An empty repo `PolicySet` (all defaults) leaves app defaults unchanged.
- Each individual non-default field in the `over` position wins over the base value.
- Each activation boolean follows OR semantics.
- `ChangeTypeLabelConfig::merge` covers all 11 commit-type mapping vectors and fallback settings.
- The post-merge enforcement overrides are applied correctly in `load_merge_warden_config`.

## References

- Issue #162 — Minimal policy engine refactor
- Issue #192 — Conditional policies (next phase, blocked on this ADR)
- `crates/core/src/config.rs` — `load_merge_warden_config`, `ApplicationDefaults`,
  `RepositoryProvidedConfig`, `CurrentPullRequestValidationConfiguration`
- ADR-001 — Hexagonal architecture (broader architectural context)
