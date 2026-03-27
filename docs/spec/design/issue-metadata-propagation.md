# Issue Metadata Propagation

**Version:** 1.0
**Last Updated:** March 27, 2026

## Overview

When a pull request references a GitHub issue (via `Fixes #123`, `Closes owner/repo#456`, etc.),
Merge Warden can optionally copy the issue's **milestone** and/or **project membership** onto the
pull request. This keeps project tracking boards and milestone dashboards accurate without
requiring authors to manually duplicate metadata from the issue to the PR.

Both features are **disabled by default** and fully opt-in via configuration flags. Users who do
not use GitHub Issues, do not use GitHub Milestones or Projects v2, or who track work in an
external system (Jira, Linear, Azure DevOps, etc.) experience zero behaviour change.

## Design Principles

### Opt-in Only

All propagation behaviour is gated behind explicit boolean config flags
(`sync_milestone_from_issue`, `sync_project_from_issue`). When both are `false` (the default),
no additional GitHub API calls are made beyond what the existing work-item reference check already
performs.

### Separation of Concerns

Issue metadata lookup is a separate concern from pull request validation. A new
`IssueMetadataProvider` trait is introduced in `developer_platforms` alongside the existing
`PullRequestProvider` and `ConfigFetcher` traits. `core` depends only on the trait abstraction —
it never imports concrete GitHub types.

### First Closing Reference Wins

When the PR body contains multiple issue references, the **first closing-keyword reference**
(`fixes`, `closes`, `resolves`) is used as the metadata source. Informational references
(`references`, `relates to`) are ignored for propagation purposes but still satisfy the
work-item reference check.

### Non-destructive No-op

If the referenced issue has no milestone (or no projects), the PR is left unchanged. Propagation
never **removes** a milestone or project from a PR — it only adds or replaces.

### Overwrite on Conflict

If the PR already has a milestone or project membership that differs from the issue's, the issue's
value takes precedence and **overwrites** the existing PR value. This is logged at `info` level
for auditability. Teams that want to preserve manually-set PR metadata should leave the flags
disabled.

---

## Issue Reference Parsing

Issue references are extracted from the PR body using the same base patterns already recognised
by the work-item reference check (`WORK_ITEM_REGEX`). For propagation purposes, only
**closing-keyword** references are considered:

| Keyword      | Closes issue on merge? | Used for propagation? |
| :----------- | :--------------------: | :-------------------: |
| `fixes`      | ✅                     | ✅                    |
| `closes`     | ✅                     | ✅                    |
| `resolves`   | ✅                     | ✅                    |
| `references` | ❌                     | ❌                    |
| `relates to` | ❌                     | ❌                    |

### Supported Reference Formats

```text
fixes #123                                                  → same-repo issue 123
Closes GH-456                                               → same-repo issue 456
RESOLVES #999                                               → same-repo issue 999
fixes owner/repo#42                                         → cross-repo owner/repo issue 42
closes https://github.com/owner/repo/issues/789             → cross-repo owner/repo issue 789
```

### Parsed Output Type

```rust
/// A parsed issue reference extracted from a pull request body.
///
/// Carries enough information to fetch the issue from the appropriate repository,
/// which may differ from the repository the PR lives in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueReference {
    /// Issue in the same repository as the PR.
    SameRepo {
        /// Issue number.
        issue_number: u64,
    },
    /// Issue in a different repository.
    CrossRepo {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// Issue number.
        issue_number: u64,
    },
}

impl IssueReference {
    /// Returns the issue number regardless of reference kind.
    pub fn issue_number(&self) -> u64 {
        match self {
            Self::SameRepo { issue_number } | Self::CrossRepo { issue_number, .. } => *issue_number,
        }
    }
}
```

### Parser Function

```rust
/// Extracts the first closing-keyword issue reference from a pull request body.
///
/// Scans `body` for `fixes / closes / resolves` references in all supported
/// formats. Returns the first match found, or `None` if no closing reference
/// is present.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::extract_closing_issue_reference;
///
/// assert_eq!(
///     extract_closing_issue_reference("fixes #42"),
///     Some(IssueReference::SameRepo { issue_number: 42 }),
/// );
///
/// assert_eq!(
///     extract_closing_issue_reference("relates to #99"),
///     None, // informational keyword — not a closing reference
/// );
/// ```
pub fn extract_closing_issue_reference(body: &str) -> Option<IssueReference>;
```

---

## IssueMetadataProvider Trait

### Location

`crates/developer_platforms/src/lib.rs` — alongside `PullRequestProvider` and `ConfigFetcher`.

### Definition

```rust
/// Provides read access to issue metadata for propagation to pull requests.
///
/// Implementations retrieve milestone and project information from an issue so
/// that `merge_warden_core` can copy that information onto the associated pull
/// request.
///
/// # Platform Support
///
/// The default implementation is [`github::GitHubIssueMetadataProvider`].
/// Teams using external issue trackers (Jira, Linear, etc.) that do not wish
/// to implement this trait can simply leave both propagation flags disabled in
/// their configuration, which is the default.
#[async_trait]
pub trait IssueMetadataProvider: Sync + Send {
    /// Fetch milestone and project metadata for a single issue.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - Owner of the repository where the issue lives.
    /// * `repo_name`  - Name of the repository where the issue lives.
    /// * `issue_number` - Issue number within that repository.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(metadata))` — issue exists and metadata was fetched.
    /// - `Ok(None)` — issue does not exist (404).
    /// - `Err(e)` — transient or permission error.
    async fn get_issue_metadata(
        &self,
        repo_owner: &str,
        repo_name: &str,
        issue_number: u64,
    ) -> Result<Option<IssueMetadata>, Error>;

    /// Set the milestone on a pull request.
    ///
    /// Overwrites any existing milestone on the PR. Pass `milestone_number: None`
    /// to clear — but note that the propagation logic never calls this with `None`.
    ///
    /// # Arguments
    ///
    /// * `repo_owner`       - Owner of the repository containing the PR.
    /// * `repo_name`        - Name of that repository.
    /// * `pr_number`        - Pull request number.
    /// * `milestone_number` - Milestone number to apply, or `None` to clear.
    async fn set_pull_request_milestone(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        milestone_number: Option<u64>,
    ) -> Result<(), Error>;

    /// Add a pull request to a GitHub Projects v2 project.
    ///
    /// Uses the PR's node ID and the project's node ID to call the
    /// `addProjectV2ItemById` GraphQL mutation.
    ///
    /// # Arguments
    ///
    /// * `repo_owner`       - Owner of the repository containing the PR.
    /// * `repo_name`        - Name of that repository.
    /// * `pr_number`        - Pull request number.
    /// * `project_node_id`  - GraphQL node ID of the target project.
    async fn add_pull_request_to_project(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        project_node_id: &str,
    ) -> Result<(), Error>;
}
```

---

## Data Models

### Location

`crates/developer_platforms/src/models.rs`

### Types

```rust
/// Metadata fetched from a referenced issue for propagation to a pull request.
#[derive(Debug, Clone)]
pub struct IssueMetadata {
    /// Milestone on the issue, if any.
    pub milestone: Option<IssueMilestone>,

    /// Projects v2 the issue belongs to.
    ///
    /// Empty when the issue has no linked projects, or when project
    /// propagation is not supported by the provider implementation.
    pub projects: Vec<IssueProject>,
}

/// Milestone information from a referenced issue.
#[derive(Debug, Clone)]
pub struct IssueMilestone {
    /// Milestone number (repository-scoped, used to set PR milestone via REST).
    pub number: u64,

    /// Human-readable milestone title (used in log messages).
    pub title: String,
}

/// Projects v2 project information from a referenced issue.
#[derive(Debug, Clone)]
pub struct IssueProject {
    /// GraphQL node ID of the project (used in `addProjectV2ItemById` mutation).
    pub node_id: String,

    /// Human-readable project title (used in log messages).
    pub title: String,
}
```

---

## Configuration

### Config Struct

```rust
/// Configuration for propagating issue metadata to pull requests.
///
/// Both flags default to `false`. Teams that do not use GitHub Milestones or
/// Projects v2, or that track issues in an external system, should leave both
/// flags disabled (or omit `[policies.pullRequests.issuePropagation]` entirely).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssuePropagationConfig {
    /// When `true`, copy the milestone from the first closing-keyword issue
    /// reference in the PR body onto the pull request.
    ///
    /// No-op when the referenced issue has no milestone. Overwrites an
    /// existing PR milestone if it differs from the issue's.
    #[serde(default = "IssuePropagationConfig::default_false")]
    pub sync_milestone_from_issue: bool,

    /// When `true`, add the pull request to every Projects v2 project that
    /// the referenced issue belongs to.
    ///
    /// No-op when the referenced issue has no linked projects. This feature
    /// requires github-bot-sdk support for `get_issue_linked_projects` and
    /// `add_item_to_project` (both GraphQL-backed); see the SDK issue filed
    /// against pvandervelde/github-bot-sdk.
    #[serde(default = "IssuePropagationConfig::default_false")]
    pub sync_project_from_issue: bool,
}

impl IssuePropagationConfig {
    fn default_false() -> bool {
        false
    }
}

impl Default for IssuePropagationConfig {
    fn default() -> Self {
        Self {
            sync_milestone_from_issue: false,
            sync_project_from_issue: false,
        }
    }
}
```

### Placement in Existing Config Hierarchy

`IssuePropagationConfig` is added as a field on `PullRequestsPoliciesConfig`:

```rust
pub struct PullRequestsPoliciesConfig {
    // ... existing fields ...

    /// Configuration for issue metadata propagation.
    #[serde(default, rename = "issuePropagation")]
    pub issue_propagation: IssuePropagationConfig,
}
```

And forwarded into `CurrentPullRequestValidationConfiguration`:

```rust
pub struct CurrentPullRequestValidationConfiguration {
    // ... existing fields ...

    /// Configuration for issue metadata propagation.
    pub issue_propagation: IssuePropagationConfig,
}
```

### Sample TOML

```toml
[policies.pullRequests.issuePropagation]
# Copy the milestone from the referenced issue onto the PR.
# Default: false
sync_milestone_from_issue = true

# Add the PR to every Projects v2 project the referenced issue belongs to.
# Default: false. Requires github-bot-sdk GraphQL project support.
sync_project_from_issue = false
```

---

## Integration with `process_pull_request()`

Propagation runs **after** all validation checks and labelling, immediately before the final
`update_pr_check_status` call. It is unconditional on validation outcome — a PR with an invalid
title still has its milestone synced if the feature is enabled.

```
PR received
  │
  ├─ communicate_pr_state_labels()
  ├─ [draft early return]
  ├─ check_wip_status()
  ├─ check_title()
  ├─ check_work_item_reference()
  ├─ check_pr_size()
  ├─ communicate_pr_title_validity_status()
  ├─ communicate_pr_work_item_validity_status()
  ├─ communicate_pr_size_status()
  ├─ determine_labels()
  ├─ propagate_issue_metadata()    ← NEW
  └─ update_pr_check_status()
```

### `propagate_issue_metadata()` Logic

```rust
/// Propagates milestone and project metadata from the referenced issue to the PR.
///
/// Runs only when at least one propagation flag is enabled. Extracts the first
/// closing-keyword issue reference from the PR body, fetches its metadata, and
/// applies milestone/project updates as configured.
///
/// Failures are logged at `warn` level and do not affect the check status outcome.
async fn propagate_issue_metadata(
    &self,
    repo_owner: &str,
    repo_name: &str,
    pr: &PullRequest,
    issue_provider: &dyn IssueMetadataProvider,
) {
    let config = &self.config.issue_propagation;

    if !config.sync_milestone_from_issue && !config.sync_project_from_issue {
        return;
    }

    let body = match &pr.body {
        Some(b) => b,
        None => return,
    };

    let reference = match extract_closing_issue_reference(body) {
        Some(r) => r,
        None => return,
    };

    let (issue_owner, issue_repo) = match &reference {
        IssueReference::SameRepo { .. } => (repo_owner, repo_name),
        IssueReference::CrossRepo { owner, repo, .. } => (owner.as_str(), repo.as_str()),
    };

    let metadata = match issue_provider
        .get_issue_metadata(issue_owner, issue_repo, reference.issue_number())
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            debug!(issue = reference.issue_number(), "Referenced issue not found; skipping propagation");
            return;
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch issue metadata; skipping propagation");
            return;
        }
    };

    if config.sync_milestone_from_issue {
        self.sync_milestone(repo_owner, repo_name, pr, &metadata, issue_provider).await;
    }

    if config.sync_project_from_issue {
        self.sync_projects(repo_owner, repo_name, pr, &metadata, issue_provider).await;
    }
}
```

---

## GitHub Implementation

### Location

`crates/developer_platforms/src/github.rs` — as a new `impl IssueMetadataProvider for GitHubProvider` block.

### Milestone (REST)

- `get_issue_metadata` calls `client.get_issue(owner, repo, issue_number)` (existing SDK method)
  and maps `Issue.milestone` → `IssueMilestone { number, title }`.
- `set_pull_request_milestone` calls `client.set_pull_request_milestone(owner, repo, pr_number, milestone_number)` (existing SDK method).

### Projects v2 (GraphQL)

- `get_issue_metadata` also calls `client.get_issue_linked_projects(owner, repo, issue_number)`
  (new SDK method requested in github-bot-sdk issue) and maps results to `Vec<IssueProject>`.
- `add_pull_request_to_project` calls `client.add_item_to_project(owner, project_number, pr_node_id)` and requires the PR's `node_id`.

> **SDK dependency**: Both `get_issue_linked_projects` and `add_item_to_project` are
> currently unimplemented in `github-bot-sdk`. Task 6.6 (project propagation) is
> **blocked** until those SDK methods are delivered. Task 6.5 (milestone propagation)
> can proceed independently — all required SDK methods are already implemented.

### Cross-repo References

For cross-repo issue references, the `github-bot-sdk` installation client must have read
permission on the referenced repository's issues. If the referenced repo is in a different
GitHub organisation or the app is not installed there, `get_issue` will return 404, which is
handled as a no-op (the PR is left unchanged, a `debug` log entry is emitted).

---

## Conflict Behaviour

| Scenario | Milestone action | Project action |
| :------- | :--------------- | :------------- |
| PR has no milestone, issue has milestone | Set PR milestone | — |
| PR has milestone A, issue has milestone A | No-op (already correct) | — |
| PR has milestone A, issue has milestone B | **Overwrite → milestone B** (logged at `info`) | — |
| PR has no milestone, issue has no milestone | No-op | — |
| PR not in project X, issue is in project X | — | Add PR to project X |
| PR already in project X, issue is in project X | — | No-op (SDK handles idempotency) |
| Issue has no projects | — | No-op |

---

## Behavioral Assertions

These assertions drive the tests in tasks 6.2–6.9.

### Issue Reference Parser (task 6.2)

1. `extract_closing_issue_reference("fixes #42")` → `SameRepo { issue_number: 42 }`
2. `extract_closing_issue_reference("closes GH-100")` → `SameRepo { issue_number: 100 }`
3. `extract_closing_issue_reference("RESOLVES #7")` → `SameRepo { issue_number: 7 }` (case-insensitive)
4. `extract_closing_issue_reference("closes owner/repo#55")` → `CrossRepo { owner: "owner", repo: "repo", issue_number: 55 }`
5. `extract_closing_issue_reference("closes https://github.com/owner/repo/issues/88")` → `CrossRepo { owner: "owner", repo: "repo", issue_number: 88 }`
6. `extract_closing_issue_reference("references #42")` → `None` (informational keyword)
7. `extract_closing_issue_reference("relates to #42")` → `None` (informational keyword)
8. `extract_closing_issue_reference("")` → `None`
9. `extract_closing_issue_reference("no references here")` → `None`
10. When body contains `"references #10\nfixes #20"`, result is `SameRepo { issue_number: 20 }` (first *closing* reference wins)
11. When body contains `"fixes #10\nfixes #20"`, result is `SameRepo { issue_number: 10 }` (first match wins overall)

### Milestone Propagation (task 6.5)

1. When `sync_milestone_from_issue = true`, referenced issue has milestone 5, PR has no milestone → `set_pull_request_milestone(pr, Some(5))` is called.
2. When `sync_milestone_from_issue = true`, referenced issue has no milestone → `set_pull_request_milestone` is **not** called.
3. When `sync_milestone_from_issue = false` → `set_pull_request_milestone` is **never** called regardless of issue state.
4. When referenced issue returns 404 → no-op, no error surfaced to check status.
5. When `set_pull_request_milestone` fails → logged at `warn`, check status outcome unaffected.
6. PR with milestone A, issue with milestone B, flag enabled → `set_pull_request_milestone(pr, Some(B))` is called (overwrite).
7. PR with milestone A, issue with milestone A, flag enabled → `set_pull_request_milestone` is **not** called (no-op).

### Project Propagation (task 6.6)

1. When `sync_project_from_issue = true`, issue belongs to project P1 → `add_pull_request_to_project(pr_node_id, P1.node_id)` is called.
2. When `sync_project_from_issue = true`, issue belongs to projects P1 and P2 → both `add_pull_request_to_project` calls are made.
3. When `sync_project_from_issue = true`, issue has no projects → `add_pull_request_to_project` is **never** called.
4. When `sync_project_from_issue = false` → `add_pull_request_to_project` is **never** called.
5. When `add_pull_request_to_project` fails → logged at `warn`, check status outcome unaffected.

### Cross-repo References (task 6.2 / 6.5)

1. Cross-repo reference (`owner/repo#42`) fetches issue from `owner/repo`, not from the PR's repository.
2. When the app has no access to the cross-repo issue repository (404) → no-op.

### Config Defaults (task 6.7)

1. `IssuePropagationConfig::default()` has both flags `false`.
2. Omitting `[policies.pullRequests.issuePropagation]` from TOML produces `IssuePropagationConfig::default()`.
3. Setting `sync_milestone_from_issue = true` in TOML is reflected in the parsed config.
