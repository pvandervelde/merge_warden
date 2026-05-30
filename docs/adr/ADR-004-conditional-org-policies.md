# ADR-004: Conditional Org Policies Based on Repository Topics and Custom Properties

Status: Accepted
Date: 2026-05-30
Owners: merge_warden team

## Context

A conditional policy blocks to the org policy file introduced in ADR-003 has been added.
The motivation is that a single organisation may contain repositories with different
risk profiles — payment services, security tools, documentation sites — that need
different validation rules without requiring separate Merge Warden deployments.

Conditional policies allow an org policy to say: "if this repository has the topic
`payments`, enforce stricter title conventions and require a work-item reference".
Conditions can be based on:

- **Repository topics** — available on all GitHub plans, set by repository
  administrators via the repository settings or API.
- **Repository custom properties** — available on GitHub Enterprise only (Team and
  Enterprise Cloud); returning 403/404 on other plans is the expected and documented
  behaviour.

ADR-003 introduced `OrgPolicy { enforced: PolicySet, defaults: PolicySet }` and the
`resolve_pull_request_config` orchestrator.  Conditional policies extend `OrgPolicy`
with a `conditional_policies: Vec<ConditionalPolicy>` array.  Each entry carries its
own `condition: PolicyCondition`, `enforced: PolicySet`, and `defaults: PolicySet`.

The four open questions for this ADR are:

1. Does fetching repo metadata go on `PullRequestProvider`, a new trait, or as passed-in context?
2. Does condition evaluation happen inside `resolve_pull_request_config` or in a separate function?
3. How are custom properties handled on non-enterprise plans (403/404)?
4. What are the condition semantics — AND or OR?

## Decision

### Decision 1 — New `RepositoryMetadataProvider` trait

A new `RepositoryMetadataProvider` trait is added to `crates/developer_platforms/src/lib.rs`.
It follows the `IssueMetadataProvider` pattern exactly:

```rust
pub trait RepositoryMetadataProvider: std::fmt::Debug + Sync + Send {
    async fn get_repository_context(
        &self,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<RepositoryContext, Error>;
}
```

`GitHubProvider` implements `RepositoryMetadataProvider` by making two independent API calls:

- `GET /repos/{owner}/{repo}/topics` → `topics: Vec<String>`
- `GET /repos/{owner}/{repo}/properties/values` → `custom_properties: HashMap<String, String>`
  (returns `Ok` with empty map on 403/404; logs at `debug!`)

**Why not on `PullRequestProvider`?**  Repository metadata is a distinct concern unrelated to
PR operations.  Adding it to `PullRequestProvider` would force every existing mock and test
implementation to implement two new methods even when they have no need for them.  The
`IssueMetadataProvider` precedent confirms this separation is correct.

**Why not pass context directly?**  Platform handlers do not know which repos will have
conditional policies at dispatch time.  Making the core orchestrator responsible for
fetching keeps the handler code simple (ADR-003 decision 2 goal).

### Decision 2 — Condition evaluation inside `resolve_pull_request_config`

`resolve_pull_request_config` is extended with one additional optional parameter:

```rust
pub async fn resolve_pull_request_config(
    repo_owner: &str,
    repo_name: &str,
    config_path: &str,
    fetcher: &dyn ConfigFetcher,
    app_defaults: &ApplicationDefaults,
    metadata_provider: Option<&dyn RepositoryMetadataProvider>,
) -> Result<CurrentPullRequestValidationConfiguration, ConfigLoadError>
```

Context is fetched lazily — only when `org_policy.conditional_policies` is non-empty.
If `metadata_provider` is `None` but conditional policies are configured, a `warn!` is
emitted and conditional blocks are skipped (treated as not matching).

**Why not a separate function?**  The four-tier merge chain is already centralised in
`resolve_pull_request_config`.  Conditional policies are just additional tiers inserted
between the org-defaults and repo tier for matching conditions.  Splitting evaluation
into a separate function would fragment the merge algorithm.

### Decision 3 — AND semantics within one block; all matching blocks merged in order

Within a single `PolicyCondition`:

- `has_any_topic`: repository must have **at least one** of the listed topics (OR among topics).
- `has_custom_property`: repository must have **every** listed key=value pair (AND among pairs).
- When both fields are present: **both** sub-conditions must hold (AND between them).
- An empty condition (no topics, no properties) always matches — equivalent to an unconditional
  policy; operators should use `[enforced]` / `[defaults]` sections instead.

When multiple `conditional_policies` blocks match, **all** are applied:

- Their `defaults` PolicySets are merged in declaration order after `org_defaults_ps` but before
  `repo_ps`.
- Their `enforced` PolicySets are merged in declaration order before `org_enforced_ps`.

The merge ordering (lowest → highest priority):

```
app_defaults
  ↓ .merge(org_defaults)
  ↓ .merge(conditional_defaults₁)   # matching blocks, declaration order
  ↓ .merge(conditional_defaults₂)
  ↓ .merge(repo)
  ↓ .merge(conditional_enforced₁)   # matching blocks, declaration order
  ↓ .merge(conditional_enforced₂)
  ↓ .merge(org_enforced)
  ↓ .merge(app_enforced)
```

**Why not first-match-wins?**  If a repo is tagged `payments` AND `security`, both conditional
policy blocks should apply independently.  First-match-wins would silently drop the second.

### Decision 4 — Custom properties degrade gracefully; logged at `debug!`

When `GET /repos/{owner}/{repo}/properties/values` returns 403 or 404:

- `custom_properties` in `RepositoryContext` is set to an empty `HashMap`.
- The error is logged at `debug!` level (expected on non-enterprise plans, not a warning).
- Topics-based conditions continue to function normally.
- Any condition that requires a custom property will not match (empty map fails the `has_custom_property` check).

When the topics endpoint returns an error:

- `topics` is set to an empty `Vec<String>`.
- The error is logged at `warn!` (topics are supported on all plans; a failure is unexpected).

## Consequences

**Enables:**

- Repository-specific policy differentiation within a single org policy file.
- Gradual rollout: topics can be added to repos incrementally.
- Enterprise-only custom property conditions are silently no-ops on non-enterprise plans.

**Forbids:**

- Conditions referencing PR content (e.g., branch name patterns) — that is out of scope.
- Nested conditional policies (conditions inside conditional policy blocks).

**Trade-offs:**

- Adds one extra GitHub API call per webhook event when conditional policies are configured
  (to fetch topics; optionally two calls if custom properties are also needed).
- Empty condition always matches — operators must be explicit about the TOML documentation to
  avoid accidentally creating always-on conditional policies.
- Custom property failures are silent on non-enterprise plans — operators relying on custom
  properties on an enterprise plan should verify their app permission (`org_custom_property` read).

## Alternatives considered

### Alt 1 — Add methods to `PullRequestProvider`

Add `get_repository_topics` and `get_repository_custom_properties` directly to
`PullRequestProvider`. Rejected: forces every mock to implement two unrelated methods,
violates single-responsibility, no precedent in this codebase.

### Alt 2 — Separate `evaluate_conditional_policies` function

Evaluate conditions outside `resolve_pull_request_config`. Rejected: the merge chain
logic is already in `resolve_pull_request_config`; a separate function would require
passing intermediate `PolicySet` values across a function boundary for no benefit.

### Alt 3 — First-match-wins semantics

Stop evaluating after the first matching conditional block. Rejected: a repository
legitimately belonging to multiple groups (e.g., `payments` and `security`) should
receive both groups' policy constraints without requiring the operator to duplicate
policy text.

### Alt 4 — Hard fail when `metadata_provider` is `None` and conditional policies exist

Return `ConfigLoadError` when the provider is absent. Rejected: same graceful-degradation
philosophy as org policy unavailability in ADR-003 — conditional blocks are skipped with a
`warn!`, not blocking PR processing.

## Implementation notes

### `RepositoryContext` struct

```rust
pub struct RepositoryContext {
    /// Repository topics set via GitHub's repository settings.
    pub topics: Vec<String>,
    /// Repository-level custom properties (GitHub Enterprise only).
    /// Empty on non-enterprise plans or when the permission is absent.
    pub custom_properties: HashMap<String, String>,
}
```

### `PolicyCondition` struct

```rust
pub struct PolicyCondition {
    /// Repository must have at least one of these topics (OR semantics).
    /// Empty slice means this sub-condition is not evaluated (always passes).
    pub has_any_topic: Vec<String>,
    /// Repository must have every listed property matching the given value (AND semantics).
    /// Empty map means this sub-condition is not evaluated (always passes).
    pub has_custom_property: HashMap<String, String>,
}
```

### `ConditionalPolicy` struct

```rust
pub struct ConditionalPolicy {
    pub condition: PolicyCondition,
    pub enforced: PolicySet,
    pub defaults: PolicySet,
}
```

### Org policy TOML schema extension

```toml
schemaVersion = 1

[enforced.policies.pullRequests.prTitle]
required = true

# Conditional: stricter rules for payment services repos
[[conditional_policies]]

[conditional_policies.condition]
has_any_topic = ["payments", "financial"]

[conditional_policies.enforced.policies.pullRequests.prTitle]
required = true
pattern = "^(feat|fix|chore)(\\([a-z0-9_-]+\\))?!?: .+"

# Conditional: security-team repos via custom property (enterprise only)
[[conditional_policies]]

[conditional_policies.condition]
[conditional_policies.condition.has_custom_property]
team = "security"

[conditional_policies.enforced.policies.pullRequests.workItem]
required = true
```

### `resolve_pull_request_config` extension

```rust
// After loading org_policy:
if let Some(org_policy) = &org_policy {
    if !org_policy.conditional_policies.is_empty() {
        let context = match metadata_provider {
            Some(provider) => provider.get_repository_context(repo_owner, repo_name).await
                .unwrap_or_else(|e| {
                    warn!(error = %e, "Failed to fetch repository context; skipping conditional policies");
                    RepositoryContext::default()
                }),
            None => {
                warn!("Conditional policies present but no metadata_provider supplied; skipping");
                RepositoryContext::default()
            }
        };
        for cp in &org_policy.conditional_policies {
            if cp.condition.matches(&context) {
                conditional_defaults.push(cp.defaults.clone());
                conditional_enforced.push(cp.enforced.clone());
            }
        }
    }
}

// Extended merge chain:
let effective_ps = app_defaults_ps
    .merge(&org_defaults_ps)
    .merge_all(&conditional_defaults)  // or loop
    .merge(&repo_ps)
    .merge_all(&conditional_enforced)
    .merge(&org_enforced_ps)
    .merge(&app_enforced_ps);
```

## References

- [ADR-003-org-level-policy.md](ADR-003-org-level-policy.md)
- [ADR-002-policy-engine.md](ADR-002-policy-engine.md)
- [GitHub Repository Topics API](https://docs.github.com/en/rest/repos/repos#get-all-repository-topics)
- [GitHub Custom Properties API](https://docs.github.com/en/rest/repos/custom-properties)
- Issue #192
