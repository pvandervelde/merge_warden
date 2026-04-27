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

> **Important:** This file uses different field names to the per-repository config. The
> table in each section below links the equivalent per-repo field. Pointing
> `MERGE_WARDEN_CONFIG_FILE` at a per-repo sample file will silently produce no enforcement.

See [`samples/app-config.sample.toml`](../../../samples/app-config.sample.toml) for a
fully annotated example.

---

## `[policies]`

Top-level policy defaults.

| Field | Type | Default | Per-repo equivalent |
| :--- | :--- | :--- | :--- |
| `enforceTitleValidation` | bool | `false` | `[prTitle] required` |
| `titlePattern` | string | *(conventional commits)* | `[prTitle] pattern` |
| `labelIfTitleInvalid` | string | *(none)* | `[prTitle] label_if_missing` |
| `enforceWorkItemValidation` | bool | `false` | `[workItem] required` |
| `workItemPattern` | string | *(GitHub issue patterns)* | `[workItem] pattern` |
| `labelIfWorkItemMissing` | string | *(none)* | `[workItem] label_if_missing` |

---

## `[policies.prSize]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable PR size labeling for all repositories. |
| `fail_on_oversized` | bool | `false` | Fail the check for XXL PRs. |
| `excluded_file_patterns` | array of strings | `[]` | Glob patterns excluded from line counts. |
| `label_prefix` | string | `"size/"` | Label prefix (e.g. `size/XS`). |
| `add_comment` | bool | `true` | Post a comment on oversized PRs. |

---

## `[policies.wip]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enforce_wip_blocking` | bool | `false` | Block merging of WIP-marked PRs. |
| `wip_label` | string | *(none)* | Label applied to WIP PRs. |
| `wip_title_patterns` | array of strings | `["WIP", "wip:", "[wip]", "draft:", "Draft:"]` | Substrings searched in the PR title. |
| `wip_description_patterns` | array of strings | `[]` | Substrings searched in the PR description. |

---

## `[policies.prState]`

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable PR state lifecycle label management. |
| `draft_label` | string | *(none)* | Label for draft PRs. |
| `review_label` | string | *(none)* | Label for PRs awaiting review. |
| `approved_label` | string | *(none)* | Label for approved PRs. |

---

## `[policies.bypassRules.*]`

Three bypass sections are available: `title_convention`, `work_items`, `size`.

Each section has:

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Activate the bypass list. |
| `users` | array of strings | `[]` | GitHub login names that bypass this check. |

Bypass rules in the application config apply across all repositories and cannot be
overridden by per-repo configs.

---

## Complete example

```toml
[policies]
enforceTitleValidation    = true
labelIfTitleInvalid       = "pr-issue: invalid-title-format"

enforceWorkItemValidation = true
labelIfWorkItemMissing    = "pr-issue: missing-work-item"

[policies.prSize]
enabled          = false
fail_on_oversized = false
label_prefix     = "size/"
add_comment      = true

[policies.wip]
enforce_wip_blocking  = true
wip_label             = "WIP"
wip_title_patterns    = ["WIP", "wip:", "[wip]", "draft:", "Draft:"]
wip_description_patterns = []

[policies.prState]
enabled        = true
draft_label    = "status: draft"
review_label   = "status: in-review"
approved_label = "status: approved"

[policies.bypassRules.title_convention]
enabled = false
users   = []

[policies.bypassRules.work_items]
enabled = false
users   = []

[policies.bypassRules.size]
enabled = false
users   = []
```

---

## Related

- [Per-repository configuration schema](per-repo-config.md)
- [Set application-level defaults](../how-to/set-app-level-defaults.md)
- [Configuration precedence](../explanation/config-precedence.md)
- [Why there are two configuration files](../explanation/two-config-files.md)
