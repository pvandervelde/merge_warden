# Interface Specification: Issue Tracker Abstraction

**Version:** 1.0
**ADR reference:** [ADR-007-issue-tracker-abstraction.md](../../adr/ADR-007-issue-tracker-abstraction.md)

---

## Overview

This document specifies the pluggable issue-tracker abstraction that backs optional
work-item reference verification (`verify_exists`). It defines the port trait
(`IssueTrackerProvider`), the tracker-agnostic data model, per-tracker configuration,
and the mapping each tracker adapter is responsible for between its own state model
and the two-value concept `core` understands: **open** or **closed**.

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
    NotFound,
}
```

### 1.4 `IssueTrackerError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum IssueTrackerError {
    /// Transient failure (network, rate limit, 5xx). Callers should treat
    /// `verify_exists` as unable to evaluate and fail the check closed with a
    /// message distinct from "reference not found" (see
    /// [configuration-system.md](../design/configuration-system.md#work-item-reference-verification)).
    #[error("issue tracker request failed: {0}")]
    RequestFailed(String),

    /// The credential presented was rejected by the tracker (expired/invalid
    /// token, insufficient scope).
    #[error("issue tracker authentication failed: {0}")]
    AuthenticationFailed(String),

    /// The tracker configuration itself is invalid (e.g. malformed `base_url`,
    /// missing required field for the selected `kind`). Surfaced at config-load
    /// time, analogous to pattern validation (ADR-006) — never deferred to the
    /// first PR that needs verification.
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
    ///   the referenced project/repository.
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

### 3.1 `TrackerKind` and `TrackerConfig`

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

/// Per-repository tracker selection and connection details.
///
/// `#[serde(tag = "kind")]` so the TOML `kind` field selects which connection
/// fields are required; unused fields for the selected kind are rejected by
/// normal `serde` deny-unknown-fields behaviour at the enum-variant level.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrackerConfig {
    /// Default when `tracker` is omitted entirely. Uses the PR's own repository
    /// and the GitHub App installation's existing `Issues: Read` permission —
    /// no additional fields, no additional credential.
    GitHub,

    /// A different GitHub repository than the one hosting the PR (e.g. an
    /// organisation-wide issue-tracking repository). Still uses the same
    /// installation credential; the installation must have `Issues: Read`
    /// access to `owner/repo`.
    GitHubRepository { owner: String, repo: String },

    /// Azure DevOps Services (cloud) or Server (on-premises) — `base_url`
    /// distinguishes them (`https://dev.azure.com` for Services; an operator
    /// URL for Server).
    AzureDevOps {
        base_url: String,
        organization: String,
        project: String,
        /// Name of an environment variable holding a PAT with work-item read
        /// access. Never a raw secret in configuration.
        auth_token_env: String,
    },

    /// Jira Cloud or Jira Server/Data Center — `base_url` distinguishes them.
    Jira {
        base_url: String,
        project_key: String,
        auth_token_env: String,
    },

    /// GitLab.com or self-managed GitLab — `base_url` distinguishes them.
    GitLab {
        base_url: String,
        /// `"group/project"` path.
        project_path: String,
        auth_token_env: String,
    },

    /// Linear (cloud-only; Linear has no on-premises offering).
    Linear {
        team_key: String,
        auth_token_env: String,
    },
}

impl Default for TrackerConfig {
    fn default() -> Self {
        TrackerConfig::GitHub
    }
}
```

### 3.2 Placement in `WorkItemPolicyConfig`

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

### 3.3 `PolicySet` merge rules (additions to `WorkItemPolicyConfig::merge`)

| Field | Rule |
| :--- | :--- |
| `verify_exists` | `base.verify_exists \|\| over.verify_exists` — same OR/activation semantics as `required`; once enabled by either tier, stays enabled |
| `tracker` | `over.tracker` if it is not `TrackerConfig::default()`; otherwise `base.tracker` |

This mirrors the existing `pattern` merge rule (non-default `over` value wins) and
keeps `WorkItemPolicyConfig::merge` the single place work-item merge semantics are
defined. See [policy-engine.md](./policy-engine.md).

### 3.4 Sample TOML

```toml
[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"
verify_exists = true

[policies.pullRequests.workItem.tracker]
kind = "azure_devops"
base_url = "https://my-ado-server.internal.example.com"   # on-prem Server
organization = "my-collection"
project = "MyProject"
auth_token_env = "ADO_PAT"
```

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

See [ADR-007](../../adr/ADR-007-issue-tracker-abstraction.md) for the policy
decision (resolve; fail closed if unresolvable). Mechanically:

1. `core` parses `WorkItemReference.cross_project` from the matched pattern text
   using tracker-agnostic heuristics defined alongside the pattern match (reusing,
   for the GitHub case, the same `owner/repo#N` recognition already implemented for
   issue-metadata propagation's `IssueReference::CrossRepo`).
2. `IssueTrackerProvider::resolve` is called with the full `WorkItemReference`,
   including `cross_project` when present. Each adapter is responsible for routing
   the lookup to the referenced project/repository rather than the one implied by
   its own `TrackerConfig` connection details.
3. If the adapter's credential cannot reach the cross-project target, `resolve`
   returns `Ok(WorkItemResolution::NotFound)` (not an error) — from `core`'s
   perspective, an inaccessible cross-project reference is indistinguishable from
   a non-existent one, and both fail the check identically under `verify_exists`.

---

## 6. Testing Requirements

### 6.1 Trait-level contract tests (`crates/issue_trackers/src/lib_tests.rs`, per adapter)

- `resolve` on an existing, open work item → `Ok(Found { state: Open })`
- `resolve` on an existing, closed work item → `Ok(Found { state: Closed })`
- `resolve` on a non-existent key → `Ok(NotFound)`
- `resolve` on a cross-project reference the credential cannot access →
  `Ok(NotFound)`, not `Err`
- `resolve` on a transient tracker failure (simulated 5xx/timeout) → `Err(RequestFailed)`
- `resolve` with an invalid/expired credential → `Err(AuthenticationFailed)`
- Each tracker's state-mapping table (section 4) is exercised for every listed
  native state value, both directions (open-mapped and closed-mapped)

### 6.2 `core` integration tests (`crates/core/src/lib_tests.rs`)

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

---

## 7. Dependency Map

```
issue_trackers (new crate)
  └── IssueTrackerProvider (trait)
  └── WorkItemReference, CrossProjectHint, WorkItemState, WorkItemResolution
  └── IssueTrackerError
  └── adapters: github, azure_devops, jira, gitlab, linear

core/src/config.rs
  └── WorkItemPolicyConfig
        ├── verify_exists: bool        (new field)
        └── tracker: TrackerConfig     (new field)
  └── WorkItemPolicyConfig::merge      (extended — see policy-engine.md)

core/src/checks.rs (or equivalent)
  └── check_work_item_reference
        └── calls IssueTrackerProvider::resolve only when verify_exists == true
```
