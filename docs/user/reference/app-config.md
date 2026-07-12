---
title: "Application configuration schema"
description: "Complete field reference for the MERGE_WARDEN_CONFIG_FILE application-level defaults file."
---

# Application configuration schema

The application-level configuration file is loaded by the server via the
`MERGE_WARDEN_CONFIG_FILE` environment variable. It defines policy defaults that apply to
**every repository** processed by the server instance.

Per-repository `.github/merge-warden.toml` files take precedence over these defaults.
See [Configuration precedence](../explanation/config-precedence.md).

> **Note:** This file uses the same snake_case field name convention as the per-repository
> config. Pointing `MERGE_WARDEN_CONFIG_FILE` at a per-repo sample file will silently
> produce no enforcement because the top-level structures differ.

See [`samples/app-config.sample.toml`](https://github.com/pvandervelde/merge_warden/blob/master/samples/app-config.sample.toml)
for a fully annotated example.

---

## `[policies]`

Top-level policy defaults.

| Field | Type | Default | Per-repo equivalent |
| :--- | :--- | :--- | :--- |
| `enable_title_validation` | bool | `false` | `[prTitle] required` |
| `default_title_pattern` | string | *(conventional commits)* | `[prTitle] pattern` |
| `default_invalid_title_label` | string | *(none)* | `[prTitle] label_if_missing` |
| `enable_work_item_validation` | bool | `false` | `[workItem] required` |
| `default_work_item_pattern` | string | *(GitHub issue patterns)* | `[workItem] pattern` |
| `default_missing_work_item_label` | string | *(none)* | `[workItem] label_if_missing` |
| `bot_mention` | string | `"@merge-warden"` | *(none — app-level only)* |

### `bot_mention`

The mention prefix that PR participants use to issue bot commands in PR comments.
The only current command is label suppression:

```
@merge-warden suppress: <label-name>
```

If your GitHub App is installed under a different name (for example because you host
your own fork), set `bot_mention` to match your App's mention handle:

```toml
[policies]
bot_mention = "@acme-merge-warden[bot]"
```

This field has no per-repo equivalent. It is controlled solely by the operator.

---

## `[policies.pr_size_check]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable PR size labeling for all repositories. |
| `fail_on_oversized` | bool | `false` | Fail the check for XXL PRs. |
| `excluded_file_patterns` | array of strings | `[]` | Glob patterns excluded from line counts. |
| `ignore_deletions` | bool | `false` | When `true`, only additions are counted; deleted lines do not contribute to the PR size. |
| `label_prefix` | string | `"size/"` | Label prefix (e.g. `size/XS`). |
| `add_comment` | bool | `true` | Post a comment on oversized PRs. |

---

## `[policies.wip_check]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enforce_wip_blocking` | bool | `false` | Block merging of WIP-marked PRs. |
| `wip_label` | string | *(none)* | Label applied to WIP PRs. |
| `wip_title_patterns` | array of strings | `["WIP", "wip:", "[wip]", "draft:", "Draft:"]` | Substrings searched in the PR title. |
| `wip_description_patterns` | array of strings | `[]` | Substrings searched in the PR description. |

---

## `[policies.pr_state_labels]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable PR state lifecycle label management. |
| `draft_label` | string | *(none)* | Label for draft PRs. |
| `review_label` | string | *(none)* | Label for PRs awaiting review. |
| `approved_label` | string | *(none)* | Label for approved PRs. |

---

## Issue propagation

Issue propagation (`sync_milestone_from_issue`, `sync_project_from_issue`) has no
application-level equivalent. These settings are only configurable in the
per-repository `.github/merge-warden.toml` file under
`[policies.pullRequests.issuePropagation]`. There is no server-wide default for
issue propagation.

See [Per-repository configuration schema](per-repo-config.md#policiespullrequestsissuepropagation)
for details.

---

## `[policies.renovate_stability]`

Controls the Renovate stability-days label management feature at the application level.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `true` | When `true`, the `pending_stability_label` is applied to a PR while its `renovate/stability-days` commit status is `pending`, `error`, or `failure`. Removed when the status becomes `success`. |
| `pending_stability_label` | string | `"pr-validation: pending-stability"` | Name of the label applied during the stability wait period. |

`enabled` merges via OR rather than plain override — see
[Configuration precedence — the Renovate-stability `enabled` merge rule](../explanation/config-precedence.md#exception-the-renovate-stability-enabled-merge-rule-is-or-not-override)
for why setting `enabled = false` here does not disable the feature for repositories that
have their own `.github/merge-warden.toml`.

**Example:**

```toml
[policies.renovate_stability]
enabled = true
pending_stability_label = "renovate: stability-pending"
```

See [Configure Renovate stability labels](../how-to/configure-renovate-stability.md)
and [Per-repository configuration schema — renovateStability](per-repo-config.md#policiespullrequestsrenovatestability).

---

## `[org_policy_source]`

Optional. When present, the server fetches a central org-level policy TOML file on
every PR event and inserts it into the configuration resolution chain.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `owner` | string | *(required)* | GitHub organisation or user name that owns the policy repository. |
| `repo` | string | *(required)* | Name of the repository that holds the org policy file. |
| `path` | string | *(required)* | Path to the org policy TOML file within the repository, relative to the repository root. |
| `fail_if_unreachable` | bool | `false` | When `true`, an unreachable or unparseable org policy file causes PR processing to abort with an error. When `false`, failures degrade gracefully to three-tier resolution. A missing file (`404`) always degrades gracefully regardless of this setting. |

**Example:**

```toml
[org_policy_source]
owner               = "my-org"
repo                = "platform-configs"
path                = "merge-warden/org-policy.toml"
fail_if_unreachable = false
```

The GitHub App must have `Contents: Read` permission on the policy repository.

See [Configure an organisation-level policy](../how-to/configure-org-policy.md) for
setup instructions and the org policy file format.

---

## `[policies.bypass_rules.*]`

Three bypass sections are available: `title_convention`, `work_items`, `size`.

Each section has:

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Activate the bypass list. |
| `users` | array of strings | `[]` | GitHub login names that bypass this check. |

Bypass rules in the application config apply across all repositories and cannot be
overridden by per-repo configs.

---

## `[policies.repository_scope]`

Optional. Restricts which repositories Merge Warden actively processes, independent of
which repositories the GitHub App installation can technically access. Has no per-repo
equivalent — this is an operator-only, application-level setting.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `include_patterns` | array of strings | *(section omitted)* | Glob patterns (`*`, `?`) matched case-insensitively against the bare repository name. A repository must match at least one entry to be processed. An explicit empty list (`[]`) processes **no** repositories — a fail-closed "pause everything" lever. |
| `exclude_patterns` | array of strings | `[]` | Glob patterns that take precedence over `include_patterns`. A repository matching an exclude pattern is never processed, even if it also matches an include pattern. |

**Behaviour when the section is absent:** every repository the GitHub App is installed on
is processed — full backward compatibility with deployments that predate this feature.

**Example:**

```toml
[policies.repository_scope]
include_patterns = ["payments-*", "checkout", "billing-?"]
exclude_patterns = ["payments-legacy"]
```

This is evaluated as a webhook-ingress-level gate **before** the configuration resolution
chain runs — it is not one of the tiers described in
[Configuration precedence](../explanation/config-precedence.md), and a repository excluded
here cannot re-include itself via its own `.github/merge-warden.toml` (that file is never
fetched for an out-of-scope repository).

See [Configure repository scope filtering](../how-to/configure-repository-scope.md) for
worked examples and pattern syntax details.

---

## Complete example

```toml
# Optional — enable org-level policy from a central repository.
# [org_policy_source]
# owner               = "my-org"
# repo                = "platform-configs"
# path                = "merge-warden/org-policy.toml"
# fail_if_unreachable = false

[policies]
enable_title_validation       = true
default_invalid_title_label   = "pr-issue: invalid-title-format"

enable_work_item_validation      = true
default_missing_work_item_label  = "pr-issue: missing-work-item"

# Bot mention prefix for label-suppression commands posted in PR comments.
# Change this if your GitHub App is installed under a different name.
# bot_mention = "@merge-warden"

[policies.pr_size_check]
enabled           = false
fail_on_oversized = false
label_prefix      = "size/"
add_comment       = true
ignore_deletions  = false

[policies.wip_check]
enforce_wip_blocking     = true
wip_label                = "WIP"
wip_title_patterns       = ["WIP", "wip:", "[wip]", "draft:", "Draft:"]
wip_description_patterns = []

[policies.pr_state_labels]
enabled        = true
draft_label    = "status: draft"
review_label   = "status: in-review"
approved_label = "status: approved"

[policies.renovate_stability]
enabled                = true
pending_stability_label = "pr-validation: pending-stability"

[policies.bypass_rules.title_convention]
enabled = false
users   = []

[policies.bypass_rules.work_items]
enabled = false
users   = []

[policies.bypass_rules.size]
enabled = false
users   = []

# Optional — restrict which repositories Merge Warden actively processes.
# Omit this section entirely to process every repository the GitHub App is
# installed on (the pre-existing, backward-compatible behaviour).
# [policies.repository_scope]
# include_patterns = ["payments-*", "checkout", "billing-?"]
# exclude_patterns = ["payments-legacy"]
```

---

## Related

- [Per-repository configuration schema](per-repo-config.md)
- [Set application-level defaults](../how-to/set-app-level-defaults.md)
- [Configure repository scope filtering](../how-to/configure-repository-scope.md)
- [Configuration precedence](../explanation/config-precedence.md)
- [Why there are two configuration files](../explanation/two-config-files.md)
