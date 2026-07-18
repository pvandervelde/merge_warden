# ADR-003: Org-Level Policy Repository Configuration

Status: Accepted
Date: 2026-05-28
Owners: merge_warden team

> **Errata (2026-07-16, PR #335):** The TOML example in Decision 1 originally showed
> `[org_policy_source]` (top-level). The correct form is `[policies.org_policy_source]`,
> since `OrgPolicySource` is a field on `ApplicationDefaults`, which only maps from the
> file's `[policies]` table. This document has been corrected in place; see PR #335 for
> the production incident this caused.

## Context

Task 5.0 adds a fourth configuration tier to Merge Warden: a designated org-wide policy repository
that holds `merge-warden-org-policy.toml`. This enables platform operators to enforce mandatory
rules across all repositories in an organisation without requiring changes to individual repo
configs or the infrastructure-level `ApplicationDefaults`.

The existing three-tier system is:

```
Repository config (.github/merge-warden.toml)
  ↓ overrides
Infrastructure config (ApplicationDefaults / app-config.toml)
  ↓ overrides
System defaults (compiled-in Rust defaults)
```

ADR-002 introduced `PolicySet` and `PolicySet::merge` as the composable abstraction, and
explicitly noted that `OrgPolicy { enforced: PolicySet, defaults: PolicySet }` would be
introduced here using the same merge method.

The four open decisions for this ADR are:

1. Where does `OrgPolicySource` live?
2. Does `load_merge_warden_config` grow a parameter, or does a new orchestrator own the
   four-tier chain?
3. How do platform handlers acquire the org policy (startup vs. per-event)?
4. What is the failure policy when the org policy file is unreachable?

## Decision

### Decision 1 — `OrgPolicySource` location

`OrgPolicySource` is added as an optional field on `ApplicationDefaults`:

```toml
[policies.org_policy_source]
owner = "my-org"
repo  = "org-configs"
path  = "merge-warden-org-policy.toml"
```

`ApplicationDefaults` is the operator-controlled infrastructure config and is the natural
home for a pointer that says "this installation should enforce org-wide rules from this repo".
It is not per-repo (that would be circular), not per-webhook event, and not per
deployment environment. The field uses `#[serde(default)]` so existing deployments without
`org_policy_source` are unaffected.

### Decision 2 — New orchestrator replaces direct `load_merge_warden_config` calls

A new public function `resolve_pull_request_config` is introduced in `crates/core/src/config.rs`.
Platform handlers (server, CLI) call this function instead of `load_merge_warden_config`.
The existing `load_merge_warden_config` is retained as the repo-config loader with its
current signature — it is not modified.

`resolve_pull_request_config` owns the four-tier resolution chain:

```
app defaults (lowest priority)
  ↓ .merge()
org defaults
  ↓ .merge()
repo config
  ↓ .merge()
org enforced  (highest priority)
  ↓
apply app-level enforcement flags (enable_title_validation etc.)
  ↓
CurrentPullRequestValidationConfiguration
```

This migration also removes the four ad-hoc enforcement overrides (`enable_title_validation`,
`enable_work_item_validation`, `pr_size_check.enabled`, `wip_check.enforce_wip_blocking`) from
`load_merge_warden_config` and re-expresses them as an explicit `app_enforced_ps` tier applied
last in the `resolve_pull_request_config` merge chain. This closes the technical debt noted in
ADR-002.

`load_merge_warden_config` remains public for callers that only need the repo-level parse +
two-tier app/repo merge (e.g. config validation, test utilities). Its four ad-hoc enforcement
overrides are removed and it returns a plain `RepositoryProvidedConfig` as before, but no longer
applies enforcement flags internally — these now live solely in `resolve_pull_request_config`.

### Decision 3 — Org policy loaded per webhook event

The org policy is fetched from GitHub on every webhook event via the existing `ConfigFetcher`
trait, identical to how repo config is fetched. This means:

- Policy changes take effect on the next webhook event without a server restart.
- No startup-time loading complexity; no `Arc<Mutex<OrgPolicy>>` or cache infrastructure needed
  in this phase.
- The `GitHubProvider` `ConfigFetcher` already handles per-request installation-scoped tokens.

A TTL-based in-memory cache can be layered in later (as an implementation detail of a
`CachingConfigFetcher` wrapper) without any interface change.

### Decision 4 — Graceful degradation on org policy unavailability

| Condition | Behaviour |
|---|---|
| `org_policy_source` absent from `ApplicationDefaults` | Three-tier system; no change to behaviour |
| Org policy file **not found** (`fetch_config` returns `Ok(None)`) | Log `warn!`; continue with three-tier merge as if no org policy exists |
| Org policy file **parse error** (invalid TOML or unsupported `schemaVersion`) | Log `warn!` with detail; continue with three-tier merge |
| Org policy file **fetch error** (network failure, GitHub API error, auth error) | Log `warn!` with detail; continue with three-tier merge |

Hard-failing would block PR processing across every repository in the organisation during
transient GitHub API outages or rate-limit windows. The risk of silently degrading for a single
event is lower than the risk of blocking all events indefinitely.

The `OrgPolicySource` struct carries an optional `fail_if_unreachable: bool` field (default
`false`) for operators who prefer strict enforcement and accept the availability trade-off.
When `true`, an unreachable or missing org policy file causes `resolve_pull_request_config`
to return `Err(ConfigLoadError::OrgPolicyUnavailable)`, which the platform handler propagates
as a non-200 response to GitHub (triggering a GitHub retry).

## Consequences

**Enables:**

- Org-wide mandatory policy enforcement without touching per-repo configs.
- The `PolicySet::merge` abstraction introduced in ADR-002 is used directly — no new merge
  semantics.
- The four ad-hoc enforcement overrides in `load_merge_warden_config` are formally migrated
  into the tier model, closing ADR-002 technical debt.
- Platform handlers are simplified: they call one function and receive a
  `CurrentPullRequestValidationConfiguration` directly; no fallback construction inline.
- Phase 4 conditional policies can extend `OrgPolicy` with a `conditional_policies` array
  without touching `resolve_pull_request_config`'s core merge chain.

**Forbids:**

- Adding new enforcement mechanisms outside `resolve_pull_request_config`. All enforcement
  logic lives in the merge chain ordering or in the `app_enforced_ps` tier.
- Storing org policy state at the handler level (in `AppState`, `MergeWardenWebhookHandler`,
  etc.). Org policy is always derived from a fresh fetch inside `resolve_pull_request_config`.

**Trade-offs:**

- Per-event fetching adds one extra HTTP call per webhook event (to fetch the org policy file).
  This is acceptable at the scale Merge Warden targets; a caching layer can be added later.
- Graceful degradation means a misconfigured org policy (e.g., wrong repo name) silently
  produces no enforcement rather than alerting loudly. Operators should monitor the
  `warn!` log lines. The `fail_if_unreachable` flag is available for strict enforcement.

## Alternatives considered

### Alt 1 — Load org policy at startup, store in `AppState`

Load the org policy once on server startup, cache in `Arc<RwLock<OrgPolicy>>`, refresh on a
timer. This reduces per-event GitHub API calls but requires background refresh logic, a restart
to pick up changes in the config pointer itself, and complicates testing.

**Why not:** Adds significant infrastructure complexity for a feature that can be cached as an
implementation detail later. Per-event loading is simpler and correct now.

### Alt 2 — Add `Option<&OrgPolicy>` parameter to `load_merge_warden_config`

Extend the existing function's signature instead of introducing a new orchestrator.

**Why not:** `load_merge_warden_config` already has 5 parameters. Adding a sixth makes the
caller responsible for loading the org policy before calling it, spreading the orchestration
logic across platform handlers. The new orchestrator centralises this cleanly. Platform handlers
should not need to know the four-tier algorithm.

### Alt 3 — Hard fail when org policy is unreachable

Return an error from `resolve_pull_request_config` when the org policy source is configured
but the file cannot be loaded.

**Why not:** Transient GitHub API failures (rate limits, outages) would block PR processing
for the entire organisation. The `fail_if_unreachable: bool` field provides this behaviour for
operators who explicitly opt in.

### Alt 4 — Per-entry `enforced: bool` flag on `PolicySet` fields

Give each `PolicySet` field an enforcement flag so any tier can mark its settings as
non-overridable. Considered and rejected in ADR-002; still not the right approach here.
Tier-placement achieves the same result with less complexity.

## Implementation notes

### `load_merge_warden_config` changes

- Remove the four ad-hoc enforcement override lines (`if app_defaults.enable_*`).
- The function continues to return `RepositoryProvidedConfig` (unchanged return type).
- `PolicySet::from_application_defaults` and `PolicySet::from_repository_config` remain
  the internal conversion helpers.
- After this change, callers of `load_merge_warden_config` that previously relied on
  enforcement flag application must migrate to `resolve_pull_request_config`.

### `resolve_pull_request_config` internals

```rust
// 1. Fetch repo config and org policy concurrently (independent remote reads)
let (repo_config_res, org_policy_res) = tokio::join!(
    parse_repo_config(repo_owner, repo_name, config_path, fetcher),  // raw repo TOML, no app defaults
    async {
        match &app_defaults.org_policy_source {
            None => Ok(None),
            Some(source) => load_org_policy(source, fetcher).await,
        }
    }
);

// repo config failures degrade gracefully (empty defaults for repo tier)
let repo_config = repo_config_res.unwrap_or_default();
// org policy errors propagate when fail_if_unreachable = true
let org_policy: Option<OrgPolicy> = org_policy_res?;

// 2. Build policy sets for each tier
let app_defaults_ps = PolicySet::from_application_defaults(app_defaults);
let (org_defaults_ps, org_enforced_ps) = match &org_policy {
    Some(p) => (p.defaults.clone(), p.enforced.clone()),
    None    => (PolicySet::default(), PolicySet::default()),
};
let repo_ps = PolicySet::from_repository_config(&repo_config);

// 3. Build app-level enforced tier from the four enforcement flags
let app_enforced_ps = PolicySet::from_app_enforcement_flags(app_defaults);

// 5. Four-tier merge chain
let effective_ps = app_defaults_ps
    .merge(&org_defaults_ps)
    .merge(&repo_ps)
    .merge(&org_enforced_ps)
    .merge(&app_enforced_ps);

// 6. Convert to CPVRC using the bypass rules from the effective policy
effective_ps.to_validation_config(repo_owner, repo_name)
```

`PolicySet::from_app_enforcement_flags` constructs a `PolicySet` that contains only the
settings forced by `enable_title_validation`, `enable_work_item_validation`,
`pr_size_check.enabled`, and `wip_check.enforce_wip_blocking`. All other fields are
`PolicySet::default()` values so they do not override anything.

`PolicySet::to_validation_config` is a new method on `PolicySet` that replaces the
write-back-into-RepositoryProvidedConfig + `to_validation_config` pattern currently used in
`load_merge_warden_config`. It produces `CurrentPullRequestValidationConfiguration` directly
from a fully-merged `PolicySet`.

### `load_org_policy` internals

```rust
pub async fn load_org_policy(
    source: &OrgPolicySource,
    fetcher: &dyn ConfigFetcher,
) -> Option<OrgPolicy>
```

- Returns `None` on any error (fetch error, parse error, wrong schema version), after logging
  a `warn!` with the specific failure detail.
- Returns `None` when the file does not exist (`Ok(None)` from `fetch_config`), after logging
  a `warn!` (file absence is intentional in some org setups but still worth logging).
- Returns `Some(OrgPolicy)` only when the file exists, parses successfully, and has
  `schemaVersion = 1`.
- When `source.fail_if_unreachable = true`, returns `None` only if the file is intentionally
  absent (`Ok(None)`); propagates fetch/parse errors as `Err(ConfigLoadError::OrgPolicyUnavailable)`.

### Org policy TOML schema

```toml
schemaVersion = 1

# Settings that CANNOT be overridden by repo-level config.
[enforced.policies.pullRequests.prTitle]
required = true
pattern  = "^(feat|fix|chore|docs|style|refactor|perf|test)(\\([a-z0-9_-]+\\))?!?: .+"

# Settings that CAN be overridden by repo-level config.
[defaults.policies.pullRequests.workItem]
required = true
pattern  = "#\\d+"
```

Both `[enforced]` and `[defaults]` sections are optional. The internal structs mirror
`RepositoryProvidedConfig.policies` so the same serde-derived parsing applies.

### Platform handler migration

Both `crates/server/src/webhook.rs` and `crates/cli/src/commands/check_pr.rs` are updated:

- Replace `load_merge_warden_config(...)` calls with `resolve_pull_request_config(...)`.
- Remove the inline fallback `CurrentPullRequestValidationConfiguration` construction
  (the orchestrator now handles this).
- `resolve_pull_request_config` returns `Result<CurrentPullRequestValidationConfiguration, ConfigLoadError>`;
  on `Err`, handlers apply their existing fallback logic (log warning, use system defaults).

## Examples

### Org enforces conventional commit titles

`merge-warden-org-policy.toml`:

```toml
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = true
pattern  = "^(feat|fix|chore|docs|style|refactor|perf|test)(\\([a-z0-9_-]+\\))?!?: .+"
```

Even if a repo's `.github/merge-warden.toml` sets `required = false` for title validation,
the org-enforced tier applies last and overrides it to `required = true`.

### Org sets default work item pattern, repos can override

`merge-warden-org-policy.toml`:

```toml
schemaVersion = 1

[defaults.policies.pullRequests.workItem]
required = true
pattern  = "JIRA-\\d+"
```

A repo that sets `pattern = "GH-\\d+"` in its own config will use its pattern.
A repo with no `workItem` config will inherit the org default `JIRA-\\d+`.

### Infrastructure pointing at the org policy repo

`app-config.toml`:

```toml
[policies.org_policy_source]
owner = "my-org"
repo  = "platform-configs"
path  = "merge-warden/org-policy.toml"
# fail_if_unreachable = false  # default; set true for strict enforcement
```

## References

- ADR-002: [ADR-002-policy-engine.md](ADR-002-policy-engine.md) — `PolicySet` and merge semantics
- Issue #162: Minimal policy engine refactor (Phase 2 — foundation for this ADR)
- Task 5.0 in `.llm/tasks.md` — implementation task
- Interface spec: [docs/spec/interfaces/org-policy.md](../spec/interfaces/org-policy.md)
