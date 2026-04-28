---
title: "How to configure PR state lifecycle labels"
description: "Apply a single label that tracks whether a PR is in draft, awaiting review, or approved."
---

# How to configure PR state lifecycle labels

Add the `[policies.pullRequests.prState]` section to `.github/merge-warden.toml` in your
repository. State labels are disabled by default.

---

## Minimal configuration

```toml
schemaVersion = 1

[policies.pullRequests.prState]
enabled = true
```

With only `enabled = true`, no labels are applied because none are specified. Define
at least one of the three state label keys to see labels appear.

---

## Full configuration example

```toml
[policies.pullRequests.prState]
enabled = true

# Label applied while the PR is in draft mode
draft_label = "status: draft"

# Label applied when the PR is ready for review but not yet approved
review_label = "status: in-review"

# Label applied when the PR has at least one approving review
approved_label = "status: approved"
```

---

## Lifecycle transitions

Merge Warden manages exactly **one** state label at a time. On every PR event the current
state is determined and:

1. The appropriate state label is applied.
2. The other two state labels (if present on the PR) are removed.

| PR state | Label applied |
| :--- | :--- |
| Draft | `draft_label` |
| Ready for review, no approvals yet | `review_label` |
| At least one approving review | `approved_label` |

State transitions happen automatically as the PR progresses — no manual label changes are
needed.

---

## Omitting a label

Remove a key entirely (or leave its value empty) to skip labeling for that state. For
example, to label only draft and approved states:

```toml
[policies.pullRequests.prState]
enabled = true
draft_label    = "status: draft"
approved_label = "status: approved"
# review_label omitted — no label for in-review state
```

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestsprstate)
- [Configure WIP detection](configure-wip-detection.md)
