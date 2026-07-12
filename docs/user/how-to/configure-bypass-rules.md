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

Per-repo bypass lists are editable by anyone with write access to the repository's default
branch. For a full discussion of the security tradeoffs and why you might prefer
application-level or org-level bypass lists instead, see
[Bypass rules in depth](../explanation/bypass-rules.md#security-considerations).

Bypass rules **can also be set in the org-level policy file** (`merge-warden-org-policy.toml`), giving platform teams a central place to configure standard bot bypasses:

- **`[defaults.policies.bypassRules.*]`** — org-wide defaults that individual repositories *can* override with their own `.github/merge-warden.toml`.
- **`[enforced.policies.bypassRules.*]`** — org-wide bypass rules that repositories *cannot* remove. Use this to ensure automation bots are never blocked by policy checks across all repositories.

> **Opt-out caveat:** A repository cannot opt out of an org-default bypass by setting `enabled = false` with an empty `users` list — the merge engine treats that combination as "unconfigured" and the org default flows through. To explicitly clear a bypass list inherited from org defaults, set `enabled = true` with an empty `users` list.

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiesbypassrules)
- [Bypass rules in depth](../explanation/bypass-rules.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
