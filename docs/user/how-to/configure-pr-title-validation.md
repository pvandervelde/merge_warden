---
title: "How to configure PR title validation"
description: "Enforce conventional commit format or a custom pattern on pull request titles."
---

# How to configure PR title validation

Add the `[policies.pullRequests.prTitle]` section to `.github/merge-warden.toml` in your
repository.

---

## Minimal configuration

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
```

This enables validation using the built-in conventional commits pattern. Any PR whose title
does not match fails the check.

---

## With a failure label

```toml
[policies.pullRequests.prTitle]
required = true
label_if_missing = "invalid-title-format"
```

When validation fails, Merge Warden applies the label `invalid-title-format` to the pull
request. The label is removed automatically when the title is corrected.

---

## With a custom pattern

```toml
[policies.pullRequests.prTitle]
required = true
pattern = "^(PROJ-\\d+|HOTFIX)\\s.+"
label_if_missing = "invalid-title-format"
```

Set `pattern` to any valid regular expression. The PR title must match the pattern
for the check to pass. `pattern` must be 256 bytes or shorter and must compile within a
fixed 1 MiB compiled-program size budget — both limits are fixed and cannot be raised per
repository. See [If the pattern itself is invalid](#if-the-pattern-itself-is-invalid) below.

---

## Default pattern

When `pattern` is omitted, the built-in conventional commits pattern is used:

```
^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9_-]+\))?!?: .+
```

This requires a type prefix followed by an optional scope in parentheses, a colon, and
a description. Examples:

```
feat: add user authentication
fix(api): handle null response
docs(readme): update installation section
```

---

## Diagnostic messages

When a title fails validation, Merge Warden posts a comment on the pull request explaining
exactly what is wrong. The diagnostics cover the following cases:

| Problem | Example title | Diagnostic |
| :--- | :--- | :--- |
| Missing type prefix | `add feature` | Type prefix required (e.g. `feat:`) |
| Unrecognised type | `update: add feature` | `update` is not a recognised type |
| Uppercase type | `Feat: add feature` | Type must be lowercase |
| Missing colon | `feat add feature` | Expected `:` after type |
| Missing description | `feat:` | Description must not be empty |

---

## If the pattern itself is invalid

If `pattern` (whichever tier set it — this repository's file, an org policy, or the
application default) is longer than 256 bytes or fails to compile, the title check does
**not** fall back to the built-in conventional commits pattern and does **not** pass
silently. It fails with a distinct comment, separate from the ordinary "title doesn't
match" diagnosis:

> The configured `title_pattern` in `.github/merge-warden.toml` is invalid and could not
> be evaluated: *(reason)*. A repository maintainer must correct this pattern before this
> check can run.

To resolve it, find which configuration tier set the broken pattern (check this
repository's `.github/merge-warden.toml` first, then any org policy, then the operator's
application defaults) and fix or remove the `pattern` value there. See
[How Merge Warden works — regex pattern validation](../explanation/how-merge-warden-works.md#regex-pattern-validation)
for how this interacts with each configuration tier.

---

## Disabling title validation

Set `required = false` to disable validation for a repository even if the application-level
defaults have it enabled.

```toml
[policies.pullRequests.prTitle]
required = false
```

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestsprtitle)
- [Set application-level title defaults](set-app-level-defaults.md)
- [Configure bypass rules](configure-bypass-rules.md)
- [How Merge Warden works — regex pattern validation](../explanation/how-merge-warden-works.md#regex-pattern-validation)
