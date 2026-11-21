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
for the check to pass. `pattern` must be 256 bytes or shorter and must compile within a
fixed 1 MiB compiled-program size budget — both limits are fixed and cannot be raised per
repository. See [If the pattern itself is invalid](#if-the-pattern-itself-is-invalid) below.

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

## If the pattern itself is invalid

If `pattern` (whichever tier set it — this repository's file, an org policy, or the
application default) is longer than 256 bytes or fails to compile, the work item check
does **not** fall back to the built-in pattern and does **not** pass silently. It fails
with a distinct comment, separate from the ordinary "no work item reference found"
message:

> The configured `work_item_reference_pattern` in `.github/merge-warden.toml` is invalid
> and could not be evaluated: *(reason)*. A repository maintainer must correct this
> pattern before this check can run.

To resolve it, find which configuration tier set the broken pattern (check this
repository's `.github/merge-warden.toml` first, then any org policy, then the operator's
application defaults) and fix or remove the `pattern` value there. See
[How Merge Warden works — regex pattern validation](../explanation/how-merge-warden-works.md#regex-pattern-validation)
for how this interacts with each configuration tier.

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
- [How Merge Warden works — regex pattern validation](../explanation/how-merge-warden-works.md#regex-pattern-validation)
