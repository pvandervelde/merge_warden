# ADR-007: Pluggable Issue Tracker Abstraction and Work-Item Verification Semantics

Status: Accepted
Date: 2026-11-19
Owners: merge_warden team

## Context

Work-item reference validation currently checks only that *something matching*
`work_item_reference_pattern` is present in the PR title/body. `#99999` passes as
readily as a real, open issue — this is a formatting/hygiene control, not a
traceability control. Several requests converge on closing that gap:

- Live verification that a reference actually resolves against the tracker.
- A definition of "valid" that also considers whether the referenced work item is
  still open, not merely whether it once existed.
- Support for work-item trackers other than GitHub Issues — Azure DevOps (both
  cloud and on-premises/Server), Jira, GitLab, and Linear at minimum — because the
  repository a PR lives in and the system a team tracks work items in are
  independent choices. A team can host its code on GitHub while tracking work in
  Jira or Azure DevOps.

This is a different bounded context from the existing `developer_platforms` crate.
`developer_platforms` abstracts *where the pull request lives* (today, exclusively
GitHub). The new abstraction must express *where the referenced work item lives*,
which is orthogonal — a GitHub-hosted PR can reference a Jira ticket, and (in
principle, for future multi-platform PR hosting) a GitLab-hosted MR could reference
a GitHub issue. Conflating the two abstractions would force every future PR-hosting
platform to also know about every issue-tracker API, and vice versa.

## Decision

1. **New crate boundary.** Introduce `crates/issue_trackers`, a sibling to
   `developer_platforms`, hosting the `IssueTrackerProvider` port trait and one
   adapter module per supported tracker (`github`, `azure_devops`, `jira`,
   `gitlab`, `linear`). `core` depends only on the `IssueTrackerProvider` trait,
   never on a concrete tracker SDK or HTTP client — the same dependency direction
   already established for `developer_platforms` (ADR-001).
2. **Tracker-agnostic verification contract.** `IssueTrackerProvider` exposes a
   single method that resolves a work-item reference to an existence/open-state
   result. Each tracker adapter is solely responsible for mapping its own
   status/state model onto the two-value `WorkItemState::{Open, Closed}` concept
   `core` cares about (e.g. GitHub `open`/`closed`; Azure DevOps `New`/`Active`/
   `Resolved`/`Closed`/`Removed` mapped per process template; Jira status category
   `"Done"` vs. not; GitLab `opened`/`closed`; Linear workflow-state `type` of
   `completed`/`canceled` vs. not). `core` never special-cases a tracker kind.
3. **`verify_exists` is a single toggle meaning "exists AND is open."** Issue #357
   asks what "valid" means once a reference is verified; the chosen answer is that
   there is one opt-in boolean, `verify_exists` (default `false`, preserving today's
   syntactic-only behaviour when unset), and when `true` a reference is **valid**
   only if the tracker reports the work item exists **and** its state maps to
   `Open`. A reference to a closed work item behaves identically to a reference
   that does not resolve at all: the work-item check fails.
   - Rejected alternative: two independent flags (`verify_exists` /
     `require_open`). A workflow that wants "exists but can be closed" is not a
     use case either source issue motivates, and a second flag would let a
     reference to an already-closed, already-resolved ticket silently pass a
     check whose entire purpose is confirming the PR is doing currently-open work.
     If a real need for that finer granularity emerges, it can be added as a
     second, narrower flag later without breaking the single-flag default.
4. **Cross-repository / cross-project reference policy: resolve, fail closed if
   unresolvable.** When a reference points at a work item outside the PR's own
   repository/project (GitHub `owner/repo#N`; the equivalent constructs on other
   trackers if a maintainer's `work_item_reference_pattern` captures them), Merge
   Warden attempts to resolve it against the tracker using the same tracker
   connection the reference's own syntax implies. If the tracker denies access
   (the installation/credential cannot see that repository/project) or the item
   does not exist, the reference is treated as **not resolved** — which, under
   `verify_exists`, fails the check exactly like any other unresolved reference.
   Verification is never silently skipped for cross-repo references. This mirrors
   the existing, already-shipped cross-repo behaviour in issue-metadata
   propagation (resolve when reachable, no-op/fail when not) rather than inventing
   a third behaviour just for this feature.
5. **Per-repository tracker configuration.** A repository selects its tracker via
   an optional `[policies.pullRequests.workItem.tracker]` table. Omitting it keeps
   the default: the GitHub tracker, using the same installation credentials and
   the PR's own repository — zero additional configuration for the common case.
   Selecting a non-GitHub tracker (or a self-hosted GitHub Enterprise Server /
   Azure DevOps Server instance) requires the tracker's connection details (base
   URL, project/org identifier, and a reference to a credential — never a raw
   secret — in configuration). See
   [issue-tracker.md](../spec/interfaces/issue-tracker.md) for the exact shape.

## Consequences

- **Enables:** opt-in traceability verification for any of five trackers at
  launch, and a clear seam (one new adapter module plus one tracker-config
  variant) for adding more later without touching `core`'s check logic.
- **Enables:** reuse of the GitHub Issues read access the app already holds
  (`Issues: Read`, already granted for milestone/project propagation) for the
  default, zero-config tracker case.
- **Forbids:** `core` branching on tracker kind. All tracker-specific state
  mapping lives in the adapter.
- **Trade-offs accepted:**
  - A reference to a closed work item is indistinguishable, from the check's
    perspective, from a reference to a non-existent one. Both read as "work item
    check failed, reference does not resolve to an open work item." Operators who
    need to distinguish these in failure messaging can do so from the structured
    verification result even though the pass/fail outcome is identical.
  - On-premises tracker support (Azure DevOps Server, self-hosted GitLab/Jira)
    means Merge Warden makes outbound HTTP calls to operator-configured base URLs.
    This is a new category of server-side request and is treated as a threat-model
    concern (see [threat-model.md](../spec/security/threat-model.md#tracker-connection-configuration-and-outbound-request-risk)),
    not just a feature-completeness one.

## Alternatives considered

- **Extend the existing `IssueMetadataProvider` trait (from issue-metadata
  propagation) instead of introducing a new one:** rejected. That trait is
  deliberately GitHub-specific (its contract includes Projects v2 and milestone
  mutation methods that have no equivalent on Jira/Linear/GitLab/ADO) and is wired
  for a different purpose (copying metadata onto the PR, not pass/fail
  verification). Reusing it would force every tracker adapter to implement
  methods that only make sense for GitHub.
- **Put the new trait inside `developer_platforms`:** rejected. That crate's name
  and existing contents (`PullRequestProvider`, `ConfigFetcher`,
  `IssueMetadataProvider`) are specifically about the platform hosting the PR.
  Issue trackers are an independent axis; keeping them in the same crate would
  blur a boundary that is easy to keep clean today and progressively harder to
  separate later.
- **Two separate flags for exists/open:** see point 3 above.
- **Ignore cross-repo references for verification purposes:** rejected. Silently
  treating a cross-repo reference as always-valid (unverified) would reintroduce
  exactly the gap `verify_exists` exists to close, just for a subset of
  references.
- **Fail all cross-repo references unconditionally (never attempt to resolve):**
  rejected as unnecessarily strict — most cross-repo references are legitimate
  (e.g., a monorepo-adjacent tracking repository) and the installation frequently
  does have access.

## Implementation notes

- `IssueTrackerProvider` and its supporting types live in the new
  `crates/issue_trackers` crate. See
  [issue-tracker.md](../spec/interfaces/issue-tracker.md) for the trait, the
  `WorkItemReference` / `WorkItemState` / `TrackerConfig` types, and per-tracker
  mapping notes.
- `WorkItemPolicyConfig` gains `verify_exists: bool` (default `false`) and
  `tracker: Option<TrackerConfig>` (default `None` → GitHub, same repository).
  Both participate in `PolicySet::merge` using the same rules as other
  activation/override fields (see
  [policy-engine.md](../spec/interfaces/policy-engine.md)).
- Verification runs only after the existing syntactic pattern check has already
  found a reference — it is an additional, later gate, never a replacement for
  pattern matching.

## Examples

```toml
[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"
verify_exists = true   # opt-in; default false preserves syntactic-only behaviour

# Omit [policies.pullRequests.workItem.tracker] to use GitHub + this repository.

[policies.pullRequests.workItem.tracker]
kind = "jira"
base_url = "https://my-team.atlassian.net"
project_key = "PROJ"
auth_token_env = "JIRA_API_TOKEN"
```

## References

- Issues #346, #357, #358
- [docs/spec/interfaces/issue-tracker.md](../spec/interfaces/issue-tracker.md)
- [docs/spec/design/configuration-system.md](../spec/design/configuration-system.md#work-item-reference-verification)
- [docs/spec/design/issue-metadata-propagation.md](../spec/design/issue-metadata-propagation.md)
- [docs/spec/security/threat-model.md](../spec/security/threat-model.md#tracker-connection-configuration-and-outbound-request-risk)
- ADR-001 (hexagonal architecture / port-adapter dependency direction)
