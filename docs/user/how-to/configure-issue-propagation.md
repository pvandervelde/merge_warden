---
title: "How to configure issue metadata propagation"
description: "Automatically copy milestones and Projects v2 links from issues onto pull requests."
---

# How to configure issue metadata propagation

Add the `[policies.pullRequests.issuePropagation]` section to `.github/merge-warden.toml`
in your repository. Both options default to disabled.

---

## Prerequisites

- The PR description must contain a work item reference with a closing keyword so that
  Merge Warden can identify which issue to read from. See
  [Configure work item validation](configure-work-item-validation.md).
- For `sync_project_from_issue`: the repository must belong to a **GitHub organisation**.
  Projects v2 propagation is not available for repositories owned by individual users.

---

## Configuration

```toml
schemaVersion = 1

[policies.pullRequests.issuePropagation]

# Copy the milestone from the referenced issue onto the PR.
sync_milestone_from_issue = true

# Add the PR to every Projects v2 project the referenced issue belongs to.
# Requires a GitHub organisation — not available for personal repositories.
sync_project_from_issue = true
```

---

## How milestone sync works

1. Merge Warden parses the PR description for the first closing-keyword issue reference.
2. It reads the milestone set on that issue.
3. If the issue has a milestone and the PR does not already have the same milestone, the
   PR milestone is set to match.

If the referenced issue has no milestone, no action is taken and the PR milestone is left
unchanged.

---

## How project sync works

1. Merge Warden reads all Projects v2 projects linked to the referenced issue.
2. For each project, the PR is added to that project.
3. Already-linked projects are not duplicated.

If the referenced issue has no linked projects, no action is taken.

---

## Supported closing keywords

The following keywords (case-insensitive) are recognised when scanning the PR description:

- `fixes`
- `closes`
- `resolves`
- `references`
- `relates to`

Only the **first** matching issue reference is used for propagation.

---

## Cross-repository references

Cross-repository references are supported:

```
fixes owner/other-repo#42
```

Merge Warden will read the milestone and projects from the issue in the referenced
repository (provided the GitHub App is installed there).

---

## No-op behaviour

Propagation is always no-op safe:

- If the referenced issue has no milestone, the PR milestone is left unchanged.
- If the PR already has the correct milestone, no API call is made.
- If the PR is already in a project, it is not added again.
- If no closing-keyword reference is found in the PR description, propagation is skipped
  entirely.

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestsissuepropagation)
- [Configure work item validation](configure-work-item-validation.md)
- [GitHub App permissions](../reference/github-app-permissions.md)
