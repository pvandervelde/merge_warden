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

A tracker connection (`base_url` plus the credential that authenticates against it)
is a fundamentally different trust category from the project-scoping identifiers
(organization/project/project key, etc.) that select which work items *within*
that connection a repository cares about. Letting a repository's own
`.github/merge-warden.toml` set both of these together would let a repository
maintainer — or anyone whose config change merges — point the server's outbound
request at an attacker-controlled host while also naming which environment
variable (potentially holding an unrelated secret, such as the app's own private
key or webhook secret) to send as a bearer credential to that host. This is
credential exfiltration via a repository-controlled outbound destination plus
credential pointer, and it is strictly worse than classic SSRF: SSRF alone lets an
attacker make the server *visit* an arbitrary host; this additionally hands that
host a secret. The design below separates these two trust tiers structurally
rather than relying on review discipline to keep them apart.

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
   unresolvable — and only where the tracker's reference syntax can express it.**
   A cross-project reference is a property of the *matched pattern text*
   (`WorkItemReference.cross_project`), not of every tracker. Today only GitHub's
   `owner/repo#N` syntax (and, if a maintainer's custom pattern captures it,
   GitLab's equivalent) can produce one; Jira keys, Azure DevOps IDs, and Linear
   identifiers have no cross-project notion in this design, so
   `WorkItemReference.cross_project` is always `None` for those tracker kinds —
   this is a correctness invariant of the pattern that produces the reference, not
   a per-request runtime condition. If an adapter whose kind cannot produce a
   cross-project reference is nonetheless called with a non-`None`
   `cross_project` (which should be structurally impossible), it treats the
   reference defensively as `WorkItemResolution::NotFound` rather than panicking
   or silently ignoring the cross-project hint.

   When a cross-project reference *is* present, Merge Warden attempts to resolve
   it against the tracker using the connection the reference's own syntax
   implies. If the tracker denies access (the installation/credential cannot see
   that repository/project) or the item does not exist, the reference is treated
   as **not resolved** — which, under `verify_exists`, fails the check exactly
   like any other unresolved reference. Verification is never silently skipped
   for cross-repo references. This mirrors the existing, already-shipped
   cross-repo behaviour in issue-metadata propagation (resolve when reachable,
   no-op/fail when not) rather than inventing a third behaviour just for this
   feature.

   This interacts with `TrackerConfig::GitHubRepository { owner, repo }` (see
   point 5) as follows: `GitHubRepository` names the "home" repository for a
   repo's work items when its own reference syntax carries no explicit
   cross-project prefix (`cross_project: None`) — resolution targets the
   `owner`/`repo` the tracker config names, not the PR's own repository. When the
   matched text *does* carry an explicit `owner/repo#N` prefix
   (`cross_project: Some(_)`), that explicit target always wins over
   `TrackerConfig::GitHubRepository`'s target: a maintainer writing
   `fixes other-org/other-repo#42` is unambiguously naming a third location,
   distinct from both the PR's own repository and the configured
   `GitHubRepository` target.
5. **Tracker connection coordinates are operator-defined; repository/org config
   selects a profile and supplies only project-scoping identifiers.** A
   repository selects its tracker via an optional
   `[policies.pullRequests.workItem.tracker]` table. Omitting it keeps the
   default: the GitHub tracker, using the same installation credentials and the
   PR's own repository — zero additional configuration for the common case.

   For every other tracker kind, the repository/org config supplies only `kind`
   and **project-scoping identifiers** that select which project/org/team within
   an already-fixed connection to query — `organization`/`project` (Azure
   DevOps), `project_key` (Jira), `project_path` (GitLab), `team_key` (Linear) —
   plus a `profile` field naming an **operator-defined tracker profile**. A
   repository or org config file can no longer supply `base_url` or
   `auth_token_env` directly; those fields exist only on `TrackerProfile`, a new
   type nested under `ApplicationDefaults.tracker_profiles: HashMap<String,
   TrackerProfile>` (`[policies.tracker_profiles.<name>]` in the operator's
   application configuration file) — the same trust tier
   `broken_repo_config_behaviour` (ADR-005) and `repository_scope` already live
   in, named consistently with those existing `ApplicationDefaults` fields. This
   is the same precedent ADR-005 and ADR-006 already establish: **the entity
   whose input is being bounded must not control the bound** — here, the entity
   whose outbound request target and credential pointer are being fixed (a
   repository's config) must not be the one who fixes them.

   Profile lookup happens at config-resolution time, after `PolicySet::merge`
   produces the effective `TrackerConfig`: the named `profile` is looked up in
   `ApplicationDefaults.tracker_profiles`, and the profile's own `kind()` must
   match the `TrackerConfig` variant's kind. An unknown profile name, a missing
   profile reference where one is required (any non-`GitHub`/`GitHubRepository`
   kind), or a kind mismatch between the `TrackerConfig` variant and the named
   profile is a config validation error — fail-closed and maintainer-visible,
   consistent with every other validation decision in this design (ADR-005,
   ADR-006). It is never resolved by guessing or silently falling back to a
   different profile.

   Because the project-scoping identifiers are still repository-controlled and
   get concatenated by the adapter into a request path/URL, every non-GitHub
   adapter validates them against a restrictive, tracker-appropriate charset
   before use and rejects scheme delimiters, `@`, `:`, or any character that
   could change the effective request target. See
   [issue-tracker.md](../spec/interfaces/issue-tracker.md) for the exact shape,
   the charset table, and the outbound-request hardening requirements
   (HTTPS-only, no automatic redirect following, post-DNS-resolution blocking of
   loopback/private/link-local/metadata address ranges, per-request timeout,
   response-size cap) every non-GitHub adapter must implement.
6. **`resolve()` results are cached briefly, in-memory, per process.** A fixed
   60-second TTL cache keyed by `(tracker identity, WorkItemReference)` avoids
   re-hitting the tracker API for a reference that has not changed across
   repeated webhook events against the same PR (e.g. several `synchronize`
   events as fixup commits land). This is the same spirit as the compiled-pattern
   cache (ADR-006): an in-memory, per-process optimisation, not a persistence
   requirement, and not shared across instances in a multi-instance deployment.
   Failed resolutions (`Err`) are never cached, so a transient tracker outage
   does not suppress a retry on the very next event for the same reference. See
   [issue-tracker.md §8](../spec/interfaces/issue-tracker.md#8-resolution-caching)
   for the concrete type.
7. **A fixed cap bounds how many distinct references one PR event resolves.** A
   PR body crafted with many reference-shaped strings must not be able to
   trigger unbounded outbound tracker API calls for a single webhook event.
   `MAX_WORK_ITEM_REFERENCES_PER_EVENT` (20) bounds the number of distinct
   `WorkItemReference` values resolved per event; references beyond the cap are
   not verified — they are treated exactly as `Ok(WorkItemResolution::NotFound)`,
   not as an error, so behaviour is deterministic regardless of how many
   reference-shaped strings a PR body contains. See
   [issue-tracker.md §10](../spec/interfaces/issue-tracker.md#10-per-event-reference-count-cap).
8. **A tracker outage fails the check closed, not neutrally.** When
   `IssueTrackerProvider::resolve` returns
   `Err(IssueTrackerError::RequestFailed(_))` (outage, timeout, rate limit), the
   work-item check **fails** — it does not pass neutrally — with a message
   explicitly distinguishable from both "reference not found" and "reference
   resolved to closed" (e.g. "could not verify work item reference: issue
   tracker is currently unreachable; this check will be re-evaluated if the pull
   request is updated or the check is re-run"). This is a deliberate trade-off,
   consistent with every other fail-closed decision in this design: a tracker
   outage can block merges while verification is enabled. A maintainer who hits
   this routinely can turn `verify_exists` off, since it is opt-in.

## Consequences

- **Enables:** opt-in traceability verification for any of five trackers at
  launch, and a clear seam (one new adapter module plus one tracker-config
  variant) for adding more later without touching `core`'s check logic.
- **Enables:** reuse of the GitHub Issues read access the app already holds
  (`Issues: Read`, already granted for milestone/project propagation) for the
  default, zero-config tracker case.
- **Enables:** operator control over every outbound tracker connection's host
  and credential, independent of what any repository or org config file
  contains — closing the credential-exfiltration vector a repo-controlled
  `base_url` + `auth_token_env` pair would otherwise open.
- **Forbids:** `core` branching on tracker kind. All tracker-specific state
  mapping lives in the adapter.
- **Forbids:** a repository or org config file setting `base_url` or
  `auth_token_env` directly for any tracker kind. Those fields exist only on
  operator-defined `TrackerProfile` values.
- **Requires:** the three work-item-check failure messages (not-found, closed,
  tracker-unreachable) to remain a required, user-visible distinction in every
  implementation and in every test suite that exercises this check — see the
  integration test matrix already specified in
  [issue-tracker.md §9.2](../spec/interfaces/issue-tracker.md#92-core-integration-tests-cratescoresrclib_testsrs).
- **Trade-offs accepted:**
  - A reference to a closed work item is indistinguishable, from the check's
    perspective, from a reference to a non-existent one. Both read as "work item
    check failed, reference does not resolve to an open work item." Operators who
    need to distinguish these in failure messaging can do so from the structured
    verification result even though the pass/fail outcome is identical.
  - On-premises tracker support (Azure DevOps Server, self-hosted GitLab/Jira)
    means Merge Warden makes outbound HTTP calls to operator-configured base URLs.
    This is a new category of server-side request and is treated as a
    threat-model concern (see
    [threat-model.md](../spec/security/threat-model.md#tracker-connection-configuration-and-outbound-request-risk)),
    not just a feature-completeness one — mitigated by keeping the base URL
    operator-only (point 5) and by the outbound-request hardening requirements
    in [issue-tracker.md §6](../spec/interfaces/issue-tracker.md#6-outbound-request-hardening-non-github-adapters).
  - Enabling `verify_exists` trades availability for traceability: a tracker
    outage fails the work-item check for every PR relying on it until the
    tracker recovers or an operator disables `verify_exists`. There is no
    neutral/pass-through fallback.

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
- **Allow a repository or org config file to set `base_url` and `auth_token_env`
  directly on `TrackerConfig` (the original shape of this ADR):** rejected. A
  repository's own config is not a trusted boundary for either value: a
  maintainer (or anyone whose config change merges) could point `base_url` at an
  attacker-controlled host while pointing `auth_token_env` at the name of an
  environment variable holding an unrelated secret, and the server would send
  that secret to that host. This is strictly worse than SSRF (host control
  alone) because it also hands over a credential. Moving both fields to an
  operator-only `TrackerProfile` (point 5) closes this while leaving the
  repository exactly the control it actually needs — which project/org/team,
  within an already-fixed connection, to query.
- **Allow a per-repository base-URL override even with operator profiles in
  place (e.g. letting a repo override a profile's `base_url` for a one-off
  self-hosted instance):** rejected. Any override path reopens the same
  credential-exfiltration vector the profile model exists to close; a repository
  that needs a different connection needs a different operator-defined profile,
  not an override field.

## Implementation notes

- `IssueTrackerProvider`, `TrackerProfile`, and their supporting types live in
  the new `crates/issue_trackers` crate. See
  [issue-tracker.md](../spec/interfaces/issue-tracker.md) for the trait, the
  `WorkItemReference` / `WorkItemState` / `TrackerConfig` / `TrackerProfile`
  types, and per-tracker mapping notes.
- `WorkItemPolicyConfig` gains `verify_exists: bool` (default `false`) and
  `tracker: Option<TrackerConfig>` (default `None` → GitHub, same repository).
  Both participate in `PolicySet::merge` using the same rules as other
  activation/override fields (see
  [policy-engine.md](../spec/interfaces/policy-engine.md)).
- `ApplicationDefaults` gains `tracker_profiles: HashMap<String, TrackerProfile>`
  (default empty map), read only from the operator's application configuration
  file, never part of the repo/org `PolicySet` merge chain — the same placement
  pattern as `broken_repo_config_behaviour` and `repository_scope`.
- Verification runs only after the existing syntactic pattern check has already
  found a reference — it is an additional, later gate, never a replacement for
  pattern matching.
- Config validation rejects a `WorkItemPolicyConfig` whose `tracker.kind()` is
  not the default `GitHub` while its effective `pattern` is still exactly the
  built-in default pattern constant — a non-GitHub tracker selected with a
  GitHub-shaped pattern can never match that tracker's reference syntax, so
  `verify_exists` would always fail. See
  [core-config-validation.md](../spec/interfaces/core-config-validation.md#new-validation-trackerprofile-resolution-and-pattern-alignment)
  and
  [issue-tracker.md §3.7](../spec/interfaces/issue-tracker.md#37-trackerpattern-alignment-validation).

## Examples

```toml
# Operator's application configuration — [policies] maps to ApplicationDefaults.
# Never present in a repository's or org's own config file.
[policies.tracker_profiles.jira_support]
kind = "jira"
base_url = "https://my-team.atlassian.net"
auth_token_env = "JIRA_API_TOKEN"   # name of an environment variable; never the token itself
```

```toml
# Repository .github/merge-warden.toml — selects a tracker kind, names an
# operator-defined profile, and supplies only project-scoping identifiers.
[policies.pullRequests.workItem]
required = true
pattern = "PROJ-\\d+"   # must match this tracker's reference syntax, not the
                         # GitHub-shaped default — see decision point 5 and the
                         # pattern-alignment validation it implies
verify_exists = true     # opt-in; default false preserves syntactic-only behaviour

[policies.pullRequests.workItem.tracker]
kind = "jira"
profile = "jira_support"   # must name a profile defined in the operator's application config
project_key = "PROJ"
# No base_url, no auth_token_env here — those fields do not exist on this
# table; supplying either is rejected as an unknown field.
```

## References

- Issues #346, #357, #358
- [docs/spec/interfaces/issue-tracker.md](../spec/interfaces/issue-tracker.md)
- [docs/spec/design/configuration-system.md](../spec/design/configuration-system.md#work-item-reference-verification)
- [docs/spec/design/issue-metadata-propagation.md](../spec/design/issue-metadata-propagation.md)
- [docs/spec/security/threat-model.md](../spec/security/threat-model.md#tracker-connection-configuration-and-outbound-request-risk)
- ADR-001 (hexagonal architecture / port-adapter dependency direction)
- ADR-005 (application-level-only policy precedent: the entity whose input is
  being bounded must not control the bound)
- ADR-006 (same precedent applied to pattern-validation limits; shares the
  in-memory, per-process, TTL/LRU-bounded cache pattern this ADR reuses for
  `resolve()` results)
