# Interface Specification: Issue Tracker Abstraction

**Version:** 1.1
**ADR reference:** [ADR-007-issue-tracker-abstraction.md](../../adr/ADR-007-issue-tracker-abstraction.md)

---

## Overview

This document specifies the pluggable issue-tracker abstraction that backs optional
work-item reference verification (`verify_exists`). It defines the port trait
(`IssueTrackerProvider`), the tracker-agnostic data model, per-tracker configuration
(including the operator/repository trust split between `TrackerProfile` and
`TrackerConfig`), outbound-request hardening requirements, resolution caching, and
the mapping each tracker adapter is responsible for between its own state model and
the two-value concept `core` understands: **open** or **closed**.

This abstraction is intentionally separate from `IssueMetadataProvider`
(`crates/developer_platforms`, documented in
[issue-metadata-propagation.md](../design/issue-metadata-propagation.md)), which copies
milestone/project metadata between a GitHub issue and a GitHub-hosted PR and has no
tracker-agnostic equivalent. `IssueTrackerProvider` answers one narrower question —
"does this reference resolve to an open work item?" — for any supported tracker.

All items below live in a new crate, `crates/issue_trackers`, unless stated otherwise.
`merge_warden_core` depends only on the trait and the tracker-agnostic types in this
document; it never imports a concrete tracker's SDK or HTTP client.

---

## 1. Tracker-Agnostic Types

### 1.1 `WorkItemReference`

```rust
/// A work-item reference extracted from a PR title/body by the existing pattern
/// match, re-expressed in a form an `IssueTrackerProvider` can resolve.
///
/// Distinct from `developer_platforms::IssueReference` (used by issue-metadata
/// propagation), which is GitHub-specific and only recognises closing keywords.
/// `WorkItemReference` carries whatever the configured tracker's reference syntax
/// implies, which may or may not include cross-project information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemReference {
    /// The tracker-native key, exactly as it should be looked up — e.g. `"123"`
    /// for GitHub/GitLab, `"456"` for Azure DevOps, `"PROJ-789"` for Jira,
    /// `"ENG-12"` for Linear.
    pub key: String,

    /// Present when the reference points outside the PR's own repository/project
    /// (e.g. GitHub `owner/repo#42`). `None` means "resolve against the tracker
    /// configured for this PR's own repository."
    ///
    /// # Correctness invariant
    ///
    /// Only tracker kinds whose reference syntax can express a cross-project
    /// prefix may ever produce `Some(_)` here. Today that is GitHub's
    /// `owner/repo#N` (and, if a maintainer's custom pattern captures it,
    /// GitLab's equivalent). For Jira, Azure DevOps, and Linear this field is
    /// always `None` — there is no cross-project reference concept for those
    /// adapters in this design. This is a property of the pattern that produced
    /// the reference, not a per-request runtime condition; see
    /// [§5](#5-cross-project-reference-resolution) for the defensive handling
    /// an adapter must apply if this invariant is ever violated.
    pub cross_project: Option<CrossProjectHint>,
}

/// Coordinates of a different repository/project than the one the PR lives in,
/// parsed from the reference text itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossProjectHint {
    /// Tracker-specific project/repository identifier (e.g. `"owner/repo"` for
    /// GitHub/GitLab, a project name for Azure DevOps/Jira).
    pub project: String,
}
```

### 1.2 `WorkItemState`

```rust
/// The tracker-agnostic open/closed concept every adapter must map its own
/// status model onto. There is no third state: an adapter must resolve every
/// status its tracker can report to one of these two variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkItemState {
    Open,
    Closed,
}
```

### 1.3 `WorkItemResolution`

```rust
/// The outcome of attempting to resolve a `WorkItemReference` against a tracker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkItemResolution {
    /// The work item exists; its tracker-native status mapped to `state`.
    Found { state: WorkItemState },

    /// The work item does not exist, or the configured credential cannot see
    /// the project/repository the reference points at (cross-project reference
    /// resolved to "not accessible"). These two cases are intentionally not
    /// distinguished at this level — both mean "cannot confirm this reference
    /// is valid" — but adapters should log the distinction at `debug`/`warn`
    /// for operator diagnosis.
    ///
    /// Also returned (never a panic, never silent success) when an adapter
    /// whose tracker kind cannot express cross-project references receives a
    /// `WorkItemReference` with `cross_project: Some(_)` — see
    /// [§5](#5-cross-project-reference-resolution).
    ///
    /// Also returned for every reference beyond
    /// [`MAX_WORK_ITEM_REFERENCES_PER_EVENT`](#10-per-event-reference-count-cap)
    /// in a single PR event — treated identically to "not accessible," never as
    /// an error.
    NotFound,
}
```

### 1.4 `IssueTrackerError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum IssueTrackerError {
    /// Transient failure (network, rate limit, 5xx, timeout, oversized
    /// response). Callers must treat `verify_exists` as unable to evaluate and
    /// fail the check closed with a message distinct from "reference not
    /// found" and from "reference resolved to closed" (see
    /// [configuration-system.md](../design/configuration-system.md#work-item-reference-verification)
    /// and [ADR-007 decision point 8](../../adr/ADR-007-issue-tracker-abstraction.md)).
    #[error("issue tracker request failed: {0}")]
    RequestFailed(String),

    /// The credential presented was rejected by the tracker (expired/invalid
    /// token, insufficient scope).
    #[error("issue tracker authentication failed: {0}")]
    AuthenticationFailed(String),

    /// The tracker configuration itself is invalid — e.g. a `TrackerProfile`
    /// with a non-`https://` `base_url`; an unknown `profile` name; a `profile`
    /// whose `TrackerProfile::kind()` does not match the referencing
    /// `TrackerConfig` variant's kind; a project-scoping identifier
    /// (`organization`, `project`, `project_key`, `project_path`, `team_key`)
    /// containing a character outside its permitted charset (see
    /// [§3.5](#35-project-scoping-identifier-validation)); or a missing
    /// required field for the selected `kind`. Surfaced at config-load time,
    /// analogous to pattern validation (ADR-006) — never deferred to the first
    /// PR that needs verification.
    #[error("issue tracker configuration is invalid: {0}")]
    InvalidConfiguration(String),
}
```

---

## 2. `IssueTrackerProvider` Trait

```rust
/// Resolves work-item references against a configured issue tracker.
///
/// One implementation per supported tracker (`github`, `azure_devops`, `jira`,
/// `gitlab`, `linear`). `core` holds a `Box<dyn IssueTrackerProvider>` selected at
/// config-resolution time from the repository's `WorkItemPolicyConfig.tracker`
/// field; it never matches on tracker kind itself.
#[async_trait]
pub trait IssueTrackerProvider: Send + Sync {
    /// Resolves a single work-item reference.
    ///
    /// # Returns
    ///
    /// - `Ok(WorkItemResolution::Found { state })` — the work item exists; `state`
    ///   is this adapter's mapping of the tracker's native status.
    /// - `Ok(WorkItemResolution::NotFound)` — the work item does not exist, or
    ///   (for a cross-project reference) the configured credential cannot access
    ///   the referenced project/repository, or the reference was beyond the
    ///   per-event cap (§10), or (defensively) the adapter's kind cannot
    ///   express cross-project references but received one anyway (§5).
    /// - `Err(IssueTrackerError)` — the resolution attempt itself failed
    ///   (transient error, auth failure, bad configuration). Distinct from
    ///   `NotFound` so callers can fail the check with an accurate message
    ///   ("could not verify" vs. "does not exist").
    async fn resolve(
        &self,
        reference: &WorkItemReference,
    ) -> Result<WorkItemResolution, IssueTrackerError>;

    /// Identifies which tracker kind this implementation serves, for logging and
    /// error messages only. Never used for `core`-side branching.
    fn kind(&self) -> TrackerKind;
}
```

---

## 3. Tracker Configuration

Tracker configuration is deliberately split across two trust tiers:

- **`TrackerProfile`** — operator-defined only. Carries the actual outbound
  connection: `base_url` (for self-hostable trackers) and `auth_token_env` (the
  *name* of an environment variable, never a raw secret). Lives in
  `ApplicationDefaults.tracker_profiles`, the operator's application
  configuration file — the same trust tier as `broken_repo_config_behaviour`
  (ADR-005) and `repository_scope`. A repository or org config file cannot set
  or override any field of a `TrackerProfile`.
- **`TrackerConfig`** — repository/org-controlled. Selects a tracker `kind`,
  names which operator-defined `profile` to use (for every kind except
  `GitHub`/`GitHubRepository`, which need none), and supplies only
  project-scoping identifiers (`organization`/`project`, `project_key`,
  `project_path`, `team_key`) that select which project/org/team *within* the
  profile's already-fixed connection to query.

This split exists because a repository's own config is not a trusted boundary
for an outbound connection target and a credential pointer together: see
[ADR-007](../../adr/ADR-007-issue-tracker-abstraction.md) and the threat-model
entry on
[tracker connection configuration](../security/threat-model.md#tracker-connection-configuration-and-outbound-request-risk)
for the credential-exfiltration risk this closes.

`TrackerProfile.auth_token_env` is operator-authored, which carries materially
lower risk than a repository-authored value — but a typo could still point a
tracker profile at the wrong environment variable. Operators **should** name
these variables with a dedicated prefix (e.g. `MERGE_WARDEN_TRACKER_*`) so a
tracker credential can never collide, by name, with an unrelated secret (the
app's private key, the webhook secret) the process also holds. This is a
documented operational convention, not a value the server parses or enforces —
enforcing a prefix programmatically would require the server to know which
other env vars exist and why, which is out of scope here.

### 3.1 `TrackerProfile` (operator-defined)

```rust
/// An operator-defined tracker connection: the only place `base_url` and
/// `auth_token_env` may be set. Defined exclusively in the operator's
/// application configuration (`ApplicationDefaults.tracker_profiles`), never in
/// a repository's or org's own config file — analogous in trust tier to
/// [`BrokenRepoConfigBehaviour`] (ADR-005): the entity whose outbound request
/// target and credential pointer are being fixed must not be the one who fixes
/// them.
///
/// `#[serde(tag = "kind")]` so the TOML `kind` field selects which connection
/// fields are required.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrackerProfile {
    /// Azure DevOps Services (cloud) or Server (on-premises) — `base_url`
    /// distinguishes them (`https://dev.azure.com` for Services; an operator
    /// URL for Server). Must be `https://`; see
    /// [§6](#6-outbound-request-hardening-non-github-adapters).
    AzureDevOps {
        base_url: String,
        /// Name of an environment variable holding a PAT with work-item read
        /// access. Never a raw secret in configuration.
        auth_token_env: String,
    },

    /// Jira Cloud or Jira Server/Data Center — `base_url` distinguishes them.
    Jira {
        base_url: String,
        auth_token_env: String,
    },

    /// GitLab.com or self-managed GitLab — `base_url` distinguishes them.
    GitLab {
        base_url: String,
        auth_token_env: String,
    },

    /// Linear (cloud-only; Linear has no on-premises offering, hence no
    /// `base_url` field — the adapter uses Linear's fixed API endpoint).
    Linear {
        auth_token_env: String,
    },
}

impl TrackerProfile {
    /// The tracker kind this profile connects to. Used at config-resolution
    /// time to validate that a `TrackerConfig`'s referenced `profile` is of a
    /// compatible kind — a mismatch is `IssueTrackerError::InvalidConfiguration`.
    pub fn kind(&self) -> TrackerKind {
        match self {
            TrackerProfile::AzureDevOps { .. } => TrackerKind::AzureDevOps,
            TrackerProfile::Jira { .. } => TrackerKind::Jira,
            TrackerProfile::GitLab { .. } => TrackerKind::GitLab,
            TrackerProfile::Linear { .. } => TrackerKind::Linear,
        }
    }
}
```

### 3.2 `TrackerKind` and `TrackerConfig` (repository/org-controlled)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackerKind {
    GitHub,
    AzureDevOps,
    Jira,
    GitLab,
    Linear,
}

/// Per-repository tracker selection and **project-scoping-only** identifiers.
/// Never carries a connection coordinate (`base_url`, `auth_token_env`) for any
/// variant — those live exclusively on [`TrackerProfile`] (§3.1).
///
/// `#[serde(tag = "kind")]` so the TOML `kind` field selects which fields are
/// required; unused fields for the selected kind are rejected by normal
/// `serde` deny-unknown-fields behaviour at the enum-variant level.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrackerConfig {
    /// Default when `tracker` is omitted entirely. Uses the PR's own repository
    /// and the GitHub App installation's existing `Issues: Read` permission —
    /// no additional fields, no additional credential, no profile.
    GitHub,

    /// A different GitHub repository than the one hosting the PR (e.g. an
    /// organisation-wide issue-tracking repository). Still uses the same
    /// installation credential (no profile needed); the installation must have
    /// `Issues: Read` access to `owner/repo`. When the matched reference text
    /// carries its own explicit cross-project prefix
    /// (`WorkItemReference.cross_project: Some(_)`), that explicit target wins
    /// over this variant's `owner`/`repo` — see
    /// [§5](#5-cross-project-reference-resolution) and
    /// [ADR-007 decision point 4](../../adr/ADR-007-issue-tracker-abstraction.md).
    GitHubRepository { owner: String, repo: String },

    /// Azure DevOps — connection details (`base_url`, credential) come from
    /// the named operator-defined profile.
    AzureDevOps {
        /// Name of a key in `ApplicationDefaults.tracker_profiles`. Must refer
        /// to a `TrackerProfile::AzureDevOps` entry; an unknown name or a
        /// kind mismatch is a config validation error.
        profile: String,
        /// Project-scoping identifiers only — validated against the charset
        /// in [§3.5](#35-project-scoping-identifier-validation).
        organization: String,
        project: String,
    },

    /// Jira — connection details come from the named operator-defined profile.
    Jira {
        profile: String,
        project_key: String,
    },

    /// GitLab — connection details come from the named operator-defined profile.
    GitLab {
        profile: String,
        /// `"group/project"` path. Project-scoping only; validated against the
        /// charset in §3.5.
        project_path: String,
    },

    /// Linear — connection details come from the named operator-defined profile.
    Linear {
        profile: String,
        team_key: String,
    },
}

impl Default for TrackerConfig {
    fn default() -> Self {
        TrackerConfig::GitHub
    }
}
```

### 3.3 Placement in `WorkItemPolicyConfig`

```rust
pub struct WorkItemPolicyConfig {
    pub required: bool,
    pub pattern: String,
    pub label_if_missing: Option<String>,

    /// Opt-in live verification. Default `false`: the check remains syntactic
    /// only, matching pre-existing behaviour exactly. `true` means "exists and
    /// is open" — see [ADR-007](../../adr/ADR-007-issue-tracker-abstraction.md).
    #[serde(default)]
    pub verify_exists: bool,

    /// Which tracker to verify against. Defaults to `TrackerConfig::GitHub`
    /// (this repository) when omitted.
    #[serde(default)]
    pub tracker: TrackerConfig,
}
```

### 3.4 `PolicySet` merge rules (additions to `WorkItemPolicyConfig::merge`)

| Field | Rule |
| :--- | :--- |
| `verify_exists` | `base.verify_exists \|\| over.verify_exists` — same OR/activation semantics as `required`; once enabled by either tier, stays enabled |
| `tracker` | `over.tracker` if it is not `TrackerConfig::default()`; otherwise `base.tracker` |

This mirrors the existing `pattern` merge rule (non-default `over` value wins) and
keeps `WorkItemPolicyConfig::merge` the single place work-item merge semantics are
defined. See [policy-engine.md](./policy-engine.md).

Merge never resolves a `profile` name or validates kind compatibility — it only
selects which tier's `TrackerConfig` value wins. Profile lookup, kind-compatibility
checking, and the pattern/tracker-kind alignment check in §3.7 all happen once, at
config-resolution time, after the merge chain has produced the final
`WorkItemPolicyConfig`.

### 3.5 Project-Scoping Identifier Validation

`organization`/`project` (Azure DevOps), `project_key` (Jira), `project_path`
(GitLab), and `team_key` (Linear) remain repository-controlled, and each adapter
concatenates its value into a request path/URL against the connection the
profile already fixes. Every adapter **must** validate its scoping identifier(s)
against a restrictive charset before use — rejecting at config-load time rather
than passing an unvalidated string to an HTTP client:

| Field | Permitted charset |
| :--- | :--- |
| `organization`, `project` (Azure DevOps) | ASCII letters, digits, `-`, `_`, `.` |
| `project_key` (Jira) | ASCII letters, digits, `-`, `_` |
| `project_path` (GitLab) | ASCII letters, digits, `-`, `_`, `.`, `/` (one or more `group[/subgroup]/project` segments; no leading/trailing `/`, no `//`) |
| `team_key` (Linear) | ASCII letters, digits, `-`, `_` |

A value containing a scheme delimiter (`://` or a bare `//`), `@`, `:`,
whitespace, or any character outside the permitted charset is rejected with
`IssueTrackerError::InvalidConfiguration` and never reaches the adapter's HTTP
client. This guards against a project-scoping identifier being crafted to change
the effective request target (e.g. smuggling a second host via `@`, or a
protocol-relative prefix), even though `base_url` itself is operator-controlled,
not repository-controlled (§3.1).

### 3.6 Sample TOML

Operator-side (application configuration file — never a repository or org
config file):

```toml
# Server application config — [policies] maps to ApplicationDefaults
[policies.tracker_profiles.jira_support]
kind = "jira"
base_url = "https://my-team.atlassian.net"
auth_token_env = "JIRA_API_TOKEN"

[policies.tracker_profiles.ado_internal]
kind = "azure_devops"
base_url = "https://my-ado-server.internal.example.com"   # on-prem Server
auth_token_env = "ADO_PAT"
```

Repository-side (`.github/merge-warden.toml`) — references a profile by name
and supplies only project-scoping identifiers:

```toml
[policies.pullRequests.workItem]
required = true
pattern = "AB#\\d+"             # must match this tracker's reference syntax — see §3.7
verify_exists = true

[policies.pullRequests.workItem.tracker]
kind = "azure_devops"
profile = "ado_internal"        # must name a profile defined in the operator's application config
organization = "my-collection"
project = "MyProject"
```

### 3.7 Tracker/Pattern Alignment Validation

Selecting a non-default `TrackerConfig` (anything other than `TrackerConfig::GitHub`)
while leaving `work_item_reference_pattern` at its built-in, GitHub-shaped default
means that tracker's references (Jira keys, Azure DevOps IDs, Linear identifiers)
can never match the pattern, so `verify_exists` would always fail. Config
validation rejects this combination with a clear, maintainer-visible error naming
the mismatch — it does not attempt to infer or auto-correct a pattern for the
selected tracker. See
[core-config-validation.md](./core-config-validation.md) for the concrete
validation function and
[ADR-007 implementation notes](../../adr/ADR-007-issue-tracker-abstraction.md)
for the rationale.

---

## 4. Per-Tracker State Mapping

Each adapter owns its own mapping; this table documents the intended mapping so
reviewers can audit adapter implementations against the authoritative contract.

| Tracker | Native state source | Maps to `WorkItemState::Open` | Maps to `WorkItemState::Closed` |
| :--- | :--- | :--- | :--- |
| GitHub | `issue.state` | `"open"` | `"closed"` |
| Azure DevOps | `System.State`, interpreted via the work item's process template category | Any state in the `"Proposed"`/`"InProgress"` categories | Any state in the `"Resolved"`/`"Completed"`/`"Removed"` categories |
| Jira | `status.statusCategory.key` | `"new"`, `"indeterminate"` | `"done"` |
| GitLab | `issue.state` | `"opened"` | `"closed"` |
| Linear | `issue.state.type` (workflow state type) | `"triage"`, `"backlog"`, `"unstarted"`, `"started"` | `"completed"`, `"cancelled"` |

Using status *categories* (Jira's `statusCategory`, Azure DevOps's process-template
category, Linear's workflow-state `type`) rather than literal status names is
deliberate: these trackers allow custom workflow state names per project, and the
category/type field is the only stable, tracker-guaranteed signal for "is this
conceptually still open work."

---

## 5. Cross-Project Reference Resolution

See [ADR-007 decision point 4](../../adr/ADR-007-issue-tracker-abstraction.md) for
the policy decision (resolve; fail closed if unresolvable; only meaningful for
tracker kinds whose reference syntax can express it). Mechanically:

1. `core` parses `WorkItemReference.cross_project` from the matched pattern text
   using tracker-agnostic heuristics defined alongside the pattern match (reusing,
   for the GitHub case, the same `owner/repo#N` recognition already implemented for
   issue-metadata propagation's `IssueReference::CrossRepo`). Only GitHub (and,
   where a maintainer's custom pattern captures it, GitLab) can produce
   `cross_project: Some(_)`; for Jira, Azure DevOps, and Linear it is always
   `None` — this is an invariant of the pattern, not a runtime branch.
2. `IssueTrackerProvider::resolve` is called with the full `WorkItemReference`,
   including `cross_project` when present. Each adapter that supports it is
   responsible for routing the lookup to the referenced project/repository
   rather than the one implied by its own `TrackerConfig`/`TrackerProfile`
   connection details.
3. If the adapter's credential cannot reach the cross-project target, `resolve`
   returns `Ok(WorkItemResolution::NotFound)` (not an error) — from `core`'s
   perspective, an inaccessible cross-project reference is indistinguishable from
   a non-existent one, and both fail the check identically under `verify_exists`.
4. **Defensive handling for trackers that cannot express cross-project
   references.** The Jira, Azure DevOps, and Linear adapters must never receive
   a `WorkItemReference` with `cross_project: Some(_)` in correct operation
   (point 1's invariant). If one is received anyway (e.g. a future pattern
   change violates the invariant), the adapter treats it as
   `Ok(WorkItemResolution::NotFound)` rather than panicking or silently ignoring
   the cross-project hint and resolving against its own configured project. This
   is a defensive correctness backstop, not a supported runtime code path, and
   is covered by the contract tests in [§9.1](#91-trait-level-contract-tests-cratesissue_trackerssrclib_testsrs-per-adapter).
5. **Interaction with `TrackerConfig::GitHubRepository`.** When the configured
   tracker is `TrackerConfig::GitHubRepository { owner, repo }` and the matched
   reference has no explicit cross-project prefix (`cross_project: None`),
   resolution targets the `owner`/`repo` the tracker config names — that is the
   purpose of the variant: this repository's work items live elsewhere. When the
   reference text *does* carry an explicit `owner/repo#N` prefix
   (`cross_project: Some(_)`), that explicit target always wins over
   `TrackerConfig::GitHubRepository`'s target: a maintainer writing
   `fixes other-org/other-repo#42` is unambiguously naming a third location.

---

## 6. Outbound Request Hardening (Non-GitHub Adapters)

Every adapter other than `github` makes outbound HTTP requests to an
operator-defined `base_url` (via `TrackerProfile`, §3.1). Keeping that `base_url`
operator-only closes the credential-exfiltration risk a repository-controlled
value would create (ADR-007), but an operator-supplied value can still be
misconfigured, and the adapters that consume it must not reopen a weaker version
of the same class of risk through a careless HTTP client default. Every
non-GitHub adapter (`azure_devops`, `jira`, `gitlab`, `linear`) **must**:

1. **HTTPS-only.** Reject any `TrackerProfile.base_url` that is not an
   `https://` URL. This is validated **eagerly, for every entry in
   `ApplicationDefaults.tracker_profiles`, when the operator's application
   configuration is loaded at server startup** — not deferred until some
   repository's config first names that profile. A malformed operator profile
   must fail server startup, the same way a malformed `ApplicationDefaults`
   value does today; it must never surface for the first time as a runtime
   error on an arbitrary PR event. Linear has no on-premises variant and no
   operator-supplied `base_url`, but its fixed cloud endpoint is still accessed
   over HTTPS only.
2. **No automatic redirect following** — or, if the underlying HTTP client
   cannot disable redirects outright, re-validate the resolved final target
   against the same scheme/host constraints after every redirect hop, before
   the request completes.
3. **Block loopback, link-local, private, and cloud-metadata IP ranges after DNS
   resolution** — not by inspecting the hostname string alone. A hostname-only
   check is bypassable by DNS rebinding (a name that resolves to a public
   address at validation time and a private/metadata address at request time).
   At minimum this blocks `127.0.0.0/8`, `169.254.0.0/16` (covers the
   `169.254.169.254` cloud metadata address), `10.0.0.0/8`, `172.16.0.0/12`,
   `192.168.0.0/16`, and their IPv6 equivalents (`::1`, `fe80::/10`,
   `fc00::/7`).
4. **Per-request timeout.** Every outbound call is bounded; a tracker that never
   responds must not hang the webhook-processing path indefinitely.
5. **Response-size cap.** Every outbound response body is read under a fixed
   maximum size; a response exceeding it is treated as
   `IssueTrackerError::RequestFailed`.

The `github` adapter is exempt **only** from points 1 and 3 — HTTPS-only and
IP-range blocking are meaningless when the target host is the fixed, compiled-in
`api.github.com` and never an operator- or repository-supplied value (a future
GitHub Enterprise Server host would itself be an operator/profile-level value
subject to the same controls, not a repository-supplied one). Point 2 (no
automatic redirect following) still applies to `github`: without it, a redirect
response could carry the installation token's `Authorization` header to a
different host than `api.github.com`, which is exactly the exfiltration shape
points 1–3 exist to prevent — a fixed initial host does not make that safe.
Points 2, 4, and 5 apply to every adapter without exception, `github` included.

---

## 7. Dependency Map

```text
issue_trackers (new crate)
  └── IssueTrackerProvider (trait)
  └── WorkItemReference, CrossProjectHint, WorkItemState, WorkItemResolution
  └── IssueTrackerError
  └── TrackerProfile, TrackerConfig, TrackerKind
  └── adapters: github, azure_devops, jira, gitlab, linear

core/src/config.rs
  └── WorkItemPolicyConfig
        ├── verify_exists: bool        (new field)
        └── tracker: TrackerConfig     (new field)
  └── WorkItemPolicyConfig::merge      (extended — see policy-engine.md)
  └── ApplicationDefaults
        └── tracker_profiles: HashMap<String, TrackerProfile>   (new field, operator-only)
  └── MAX_WORK_ITEM_REFERENCES_PER_EVENT (new constant — §10)

core/src/checks.rs (or equivalent)
  └── check_work_item_reference
        └── calls IssueTrackerProvider::resolve only when verify_exists == true
        └── bounded by MAX_WORK_ITEM_REFERENCES_PER_EVENT and WorkItemResolutionCache
```

(This section is placed ahead of §8–§10 for historical reasons — see §6, §9, and
§10 for the outbound-hardening, caching, and per-event-cap mechanisms that extend
the shape above.)

---

## 8. Resolution Caching

`IssueTrackerProvider::resolve` results are cached in-memory, per process, for a
fixed TTL of **60 seconds**, keyed by `(tracker identity, WorkItemReference)`.
This avoids re-hitting the tracker API for a reference that has not changed
across repeated webhook events against the same PR (e.g. several `synchronize`
events as a contributor pushes fixup commits without touching the work-item
reference).

```rust
/// In-memory, per-process cache of recent `resolve()` outcomes. Not a
/// persistence requirement — lost on process restart, same spirit as
/// `CompiledPatternCache` (ADR-006) — and not shared across processes in a
/// multi-instance deployment.
pub struct WorkItemResolutionCache { /* internal, TTL = 60s, not part of the public API shape */ }

impl WorkItemResolutionCache {
    /// Returns a cached `Ok` resolution younger than the TTL, or calls
    /// `provider.resolve(reference)` and caches a successful result before
    /// returning it.
    ///
    /// `Err(IssueTrackerError)` results are never cached: a transient tracker
    /// failure must not suppress a retry on the very next event for the same
    /// reference.
    pub async fn get_or_resolve(
        &self,
        tracker_identity: &str,
        provider: &dyn IssueTrackerProvider,
        reference: &WorkItemReference,
    ) -> Result<WorkItemResolution, IssueTrackerError> {
        todo!("See ADR-007 decision point 6 and this section")
    }
}
```

`tracker_identity` distinguishes two repositories configured with different
`TrackerConfig`/profile combinations so a cache hit never crosses tracker
connections — it is derived from the resolved `TrackerConfig` (profile name plus
scoping identifiers) **and the named profile's own `base_url`**, not merely the
tracker kind or the profile name alone. Including `base_url` in the key means
that if an operator edits a profile's `base_url` and reloads application
configuration, any entries cached under the old connection expire naturally
(their key no longer matches) instead of serving a result resolved against the
connection that is no longer configured — without this, a profile edit at
runtime could otherwise return a stale result from the previous host for up to
the 60-second TTL.

---

## 9. Testing Requirements

### 9.1 Trait-level contract tests (`crates/issue_trackers/src/lib_tests.rs`, per adapter)

- `resolve` on an existing, open work item → `Ok(Found { state: Open })`
- `resolve` on an existing, closed work item → `Ok(Found { state: Closed })`
- `resolve` on a non-existent key → `Ok(NotFound)`
- `resolve` on a cross-project reference the credential cannot access →
  `Ok(NotFound)`, not `Err`
- For `jira`, `azure_devops`, `linear` only: `resolve` called (defensively) with
  `cross_project: Some(_)` → `Ok(NotFound)`, not a panic and not a lookup against
  the adapter's own configured project
- `resolve` on a transient tracker failure (simulated 5xx/timeout) → `Err(RequestFailed)`
- `resolve` with an invalid/expired credential → `Err(AuthenticationFailed)`
- Each tracker's state-mapping table (section 4) is exercised for every listed
  native state value, both directions (open-mapped and closed-mapped)
- `TrackerProfile` with a non-`https://` `base_url` → rejected with
  `InvalidConfiguration` at config-load time, for every self-hostable tracker kind
- A project-scoping identifier containing `@`, `:`, `://`, or whitespace →
  rejected with `InvalidConfiguration`, for every tracker kind that has one
- An adapter configured with a redirect-returning endpoint (test double) either
  never follows the redirect, or re-validates the resolved target before
  completing the request
- An adapter configured with a `base_url` that resolves (via a test DNS double)
  to a private/loopback/link-local/metadata address → request is blocked
- A request exceeding the per-request timeout → `Err(RequestFailed)`
- A response exceeding the response-size cap → `Err(RequestFailed)`

### 9.2 `core` integration tests (`crates/core/src/lib_tests.rs`)

- `verify_exists = false` (default) → `IssueTrackerProvider::resolve` is never called
- `verify_exists = true`, reference resolves to `Open` → work-item check passes
- `verify_exists = true`, reference resolves to `Closed` → work-item check fails
  with a message distinguishable from "no reference found"
- `verify_exists = true`, reference resolves to `NotFound` → work-item check fails
- `verify_exists = true`, tracker returns `Err` → work-item check fails with a
  message distinguishable from both of the above ("could not verify" vs. "does
  not exist" vs. "is closed")
- `tracker` omitted → GitHub, same repository, is used
- `tracker` set to a non-default kind → the corresponding adapter is selected and
  the GitHub default adapter is never constructed
- `tracker` set to a non-default kind naming an unknown `profile` → config
  validation fails with a clear, maintainer-visible error; the work-item check is
  never evaluated against a partially-resolved tracker
- `tracker` set to a non-default kind naming a `profile` of a different kind
  (e.g. `kind = "jira"` naming an Azure DevOps profile) → config validation fails
- `tracker` set to a non-default kind while `pattern` is left at the built-in
  default → config validation fails naming the tracker/pattern mismatch (§3.7)
- A PR body containing more than `MAX_WORK_ITEM_REFERENCES_PER_EVENT`
  reference-shaped strings → references beyond the cap are treated as
  `NotFound`, not resolved against the tracker, and the event completes
  deterministically
- Two webhook events against the same PR within the cache TTL, same reference →
  `IssueTrackerProvider::resolve` is called at most once (observable via a
  call-counting test double)
- A cached `Err` result is never reused — the very next event retries the tracker

---

## 10. Per-Event Reference Count Cap

A PR body crafted with many reference-shaped strings must not be able to trigger
unbounded outbound tracker API calls for a single webhook event.
`MAX_WORK_ITEM_REFERENCES_PER_EVENT` (**20**) bounds the number of distinct
`WorkItemReference` values resolved per PR event.

References beyond the cap are **not verified** — they are treated exactly as if
`IssueTrackerProvider::resolve` had returned `Ok(WorkItemResolution::NotFound)`
for them, never as an `Err` and never silently skipped in a way that would make
behaviour depend on reference ordering or count in an undocumented way. This
keeps behaviour deterministic: a PR with 19 references and one with 21 both
complete the event the same way, with only the latter's 20th-and-beyond
references failing verification.

See [core-config-validation.md](./core-config-validation.md) for the constant's
definition site (`crates/core/src/config.rs`, alongside `MAX_PATTERN_LENGTH`).
