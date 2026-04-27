---
title: "Per-repository configuration schema"
description: "Complete field reference for .github/merge-warden.toml."
---

# Per-repository configuration schema

Place this file at `.github/merge-warden.toml` on the **default branch** of any repository
managed by Merge Warden. The server fetches it via the GitHub API on every webhook event —
no server restart is needed when you update it.

If the file is absent or malformed, the server falls back to application-level defaults.
With compiled-in defaults, all validation is disabled.

The top-level `schemaVersion` field is required.

```toml
schemaVersion = 1
```

---

## `[policies.pullRequests.prTitle]`

Controls pull request title validation.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `required` | bool | `false` | When `true`, the PR title must match the pattern. |
| `pattern` | string | *(conventional commits)* | Regular expression the PR title must match. Omit to use the built-in conventional commits pattern. |
| `label_if_missing` | string | *(none)* | Label applied to the PR when the title is invalid. Removed when the title passes. Omit to disable labeling. |

**Built-in default pattern:**

```
^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+
```

---

## `[policies.pullRequests.workItem]`

Controls work item reference validation.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `required` | bool | `false` | When `true`, the PR description must contain a matching work item reference. |
| `pattern` | string | *(GitHub issue patterns)* | Regular expression applied to the PR description. Omit to use the built-in pattern. |
| `label_if_missing` | string | *(none)* | Label applied when no work item reference is found. Removed when a valid reference is added. |

**Built-in default pattern** matches:
`fixes #123`, `closes GH-456`, `resolves https://github.com/owner/repo/issues/789`,
`references owner/repo#42`.

---

## `[policies.pullRequests.prSize]`

Controls automatic PR size labeling.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | When `true`, size labels are applied on every PR event. |
| `fail_on_oversized` | bool | `false` | When `true`, the check fails for XXL PRs (above the `xl` threshold). |
| `excluded_file_patterns` | array of strings | `[]` | Glob patterns for files to exclude from the line count. |
| `label_prefix` | string | `"size/"` | Prefix prepended to size tier names to form the label (e.g. `size/XS`). |
| `add_comment` | bool | `true` | When `true`, an educational comment is posted on XXL PRs. |

### `[policies.pullRequests.prSize.thresholds]`

Optional. Override the default line-count boundaries for each size tier.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `xs` | integer | `10` | Maximum line count for the XS tier (1 – `xs`). |
| `s` | integer | `50` | Maximum line count for the S tier (`xs+1` – `s`). |
| `m` | integer | `100` | Maximum line count for the M tier. |
| `l` | integer | `250` | Maximum line count for the L tier. |
| `xl` | integer | `500` | Maximum line count for the XL tier. Above this is XXL. |

---

## `[policies.pullRequests.wip]`

Controls WIP (Work In Progress) detection.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enforce_wip_blocking` | bool | `false` | When `true`, PRs whose title or description match a WIP pattern have their check set to failure. |
| `wip_label` | string | *(none)* | Label applied to WIP pull requests. Omit or leave empty to disable WIP labeling. |
| `wip_title_patterns` | array of strings | `["WIP", "wip:", "[wip]", "draft:", "Draft:"]` | Case-sensitive substrings searched in the PR title. |
| `wip_description_patterns` | array of strings | `[]` | Case-sensitive substrings searched in the PR description. Empty by default. |

> **WIP blocking cannot be bypassed.** Unlike title or work-item checks, there is no bypass
> mechanism for WIP blocking.

---

## `[policies.pullRequests.prState]`

Controls PR state lifecycle label management.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | When `true`, exactly one state label is maintained on the PR at all times. |
| `draft_label` | string | *(none)* | Label applied when the PR is in draft mode. Omit to skip labeling for this state. |
| `review_label` | string | *(none)* | Label applied when the PR is ready for review but not yet approved. |
| `approved_label` | string | *(none)* | Label applied when the PR has at least one approving review. |

---

## `[policies.pullRequests.issuePropagation]`

Controls propagation of issue metadata onto pull requests.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `sync_milestone_from_issue` | bool | `false` | When `true`, copies the milestone from the first closing-keyword issue reference in the PR body onto the PR. |
| `sync_project_from_issue` | bool | `false` | When `true`, adds the PR to every Projects v2 project linked to the referenced issue. Requires a GitHub organisation. |

---

## `[policies.bypassRules.*]`

Each bypass section has the same shape. Three bypass policies are available:

| Section key | What it bypasses |
| :--- | :--- |
| `title_convention` | PR title format validation |
| `work_items` | Work item reference requirement |
| `size` | PR size `fail_on_oversized` check |

**Fields (same for all three):**

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | When `true`, users listed in `users` bypass this policy check. |
| `users` | array of strings | `[]` | GitHub login names that bypass this check. |

**Example:**

```toml
[policies.bypassRules.title_convention]
enabled = true
users   = ["release-bot", "dependabot[bot]"]
```

---

## `[change_type_labels]`

Controls automatic change-type label detection and application.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | When `true`, Merge Warden maps the PR title's commit type to a repository label. |

### `[change_type_labels.conventional_commit_mappings]`

Maps each conventional commit type to a list of candidate label names. Merge Warden
searches the repository's existing labels for a match.

| Key | Candidate labels (built-in) |
| :--- | :--- |
| `feat` | `enhancement`, `feature`, `new feature` |
| `fix` | `bug`, `bugfix`, `fix` |
| `docs` | `documentation`, `docs` |
| `style` | `style`, `formatting` |
| `refactor` | `refactor`, `refactoring`, `code quality` |
| `perf` | `performance`, `optimization` |
| `test` | `test`, `tests`, `testing` |
| `chore` | `chore`, `maintenance`, `housekeeping` |
| `ci` | `ci`, `continuous integration`, `build` |
| `build` | `build`, `dependencies` |
| `revert` | `revert` |

Override any entry by specifying a new list of strings:

```toml
[change_type_labels.conventional_commit_mappings]
feat = ["new-feature", "enhancement"]
```

### `[change_type_labels.fallback_label_settings]`

Controls label creation when no existing label matches.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `name_format` | string | `"type: {change_type}"` | Template for the created label name. Use `{change_type}` as a placeholder for the commit type. |
| `create_if_missing` | bool | `true` | When `true`, a new label is created in the repository if no existing label matches. |

### `[change_type_labels.fallback_label_settings.color_scheme]`

Hex colour codes used when creating fallback labels. One entry per commit type.

### `[change_type_labels.detection_strategy]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `exact_match` | bool | `true` | Match label name exactly against candidate values. |
| `prefix_match` | bool | `true` | Match label name that starts with a common prefix and the candidate (e.g. `type: feat`). |
| `description_match` | bool | `true` | Match label whose description contains one of the candidate values. |
| `common_prefixes` | array of strings | `["type:", "kind:", "category:"]` | Prefixes used when `prefix_match` is enabled. |

---

## Complete example

See [`samples/merge-warden.sample.toml`](../../../samples/merge-warden.sample.toml) in the
repository for a fully annotated example configuration.

---

## Related

- [Application configuration schema](app-config.md) — server-wide defaults
- [Configuration precedence](../explanation/config-precedence.md)
- [Tutorial: Enforce your first PR policy](../tutorials/02-add-first-policy.md)
