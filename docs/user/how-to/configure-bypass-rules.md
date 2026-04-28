---
title: "How to configure bypass rules"
description: "Allow specific GitHub users to open pull requests without passing title, work item, or size checks."
---

# How to configure bypass rules

Add bypass rules to `.github/merge-warden.toml` to allow specific GitHub usernames to skip
individual policy checks. Each policy discipline has its own bypass list.

---

## Available bypass policies

| Config key | What it bypasses |
| :--- | :--- |
| `[policies.bypassRules.title_convention]` | PR title format validation |
| `[policies.bypassRules.work_items]` | Work item reference requirement |
| `[policies.bypassRules.size]` | PR size fail-on-oversized check |

WIP blocking has **no bypass mechanism** — it is always enforced regardless of who opened
the PR.

---

## Configuration

```toml
schemaVersion = 1

[policies.bypassRules.title_convention]
enabled = true
users   = ["release-bot", "dependabot[bot]"]

[policies.bypassRules.work_items]
enabled = true
users   = ["dependabot[bot]"]

[policies.bypassRules.size]
enabled = false
users   = []
```

Set `enabled = true` and list GitHub login names in `users`. When a PR is opened by a
listed user, the corresponding check is skipped entirely — no failure and no labels.

---

## Disabling a bypass list

Set `enabled = false` (or omit the section) to disable bypass for that policy. The `users`
list has no effect when `enabled = false`.

---

## Security considerations

Bypass user lists are specified in the **per-repository configuration file** (`.github/merge-warden.toml`),
which is stored in the repository itself. Anyone with write access to the default branch can
add themselves to a bypass list.

For stricter control, define bypass lists in the **application-level configuration file**
(`MERGE_WARDEN_CONFIG_FILE`), which is controlled by the operator and cannot be overridden
by repository maintainers. See [Set application-level policy defaults](set-app-level-defaults.md).

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiesbypassrules)
- [Bypass rules in depth](../explanation/bypass-rules.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
