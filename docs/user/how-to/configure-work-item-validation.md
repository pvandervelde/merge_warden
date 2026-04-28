---
title: "How to configure work item validation"
description: "Require pull request descriptions to reference an issue or work item."
---

# How to configure work item validation

Add the `[policies.pullRequests.workItem]` section to `.github/merge-warden.toml` in your
repository.

---

## Minimal configuration

```toml
schemaVersion = 1

[policies.pullRequests.workItem]
required = true
```

This enables work item validation using the built-in pattern. Any PR whose description does
not contain a recognised issue reference fails the check.

---

## With a failure label

```toml
[policies.pullRequests.workItem]
required = true
label_if_missing = "missing-work-item"
```

When validation fails, Merge Warden applies the label `missing-work-item` to the pull
request. The label is removed when the description is updated to include a valid reference.

---

## With a custom pattern

```toml
[policies.pullRequests.workItem]
required = true
pattern = "JIRA-\\d+"
label_if_missing = "missing-work-item"
```

Set `pattern` to any valid regular expression. The PR description must match the pattern
for the check to pass.

---

## Default pattern

When `pattern` is omitted, the built-in pattern matches the following formats:

| Format | Example |
| :--- | :--- |
| Short reference | `fixes #123` |
| GH-prefix reference | `closes GH-456` |
| Full GitHub URL | `resolves https://github.com/owner/repo/issues/789` |
| Cross-repo reference | `references owner/other-repo#42` |

Supported closing keywords (case-insensitive): `fixes`, `closes`, `resolves`,
`references`, `relates to`.

---

## Disabling work item validation

Set `required = false` to disable validation for a repository even if the application-level
defaults have it enabled.

```toml
[policies.pullRequests.workItem]
required = false
```

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestsworkitem)
- [Configure issue metadata propagation](configure-issue-propagation.md)
- [Configure bypass rules](configure-bypass-rules.md)
