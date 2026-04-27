---
title: "Bypass rules in depth"
description: "How bypass rules work, their intended use cases, security considerations, and limitations."
---

# Bypass rules in depth

Bypass rules let specific GitHub users open pull requests without passing certain policy
checks. This page covers intended use cases, how they work technically, and important
security considerations.

---

## Intended use cases

Bypass rules exist for accounts that legitimately cannot satisfy human-oriented policies:

- **Bot accounts** (`dependabot[bot]`, `renovate[bot]`, `release-bot`) — automated tools
  that create PRs with machine-generated titles and no work item references.
- **Release automation** — scripts that create version-bump PRs as part of a release
  pipeline.
- **Emergency hotfix workflow** — a team may choose to allow senior engineers to bypass
  size checks when a critical fix must ship immediately.

For human-authored PRs, bypass rules are generally not appropriate.

---

## Per-policy granularity

Each policy has its own bypass list. You can allow a bot to skip the work item check
without also skipping the title check:

```toml
[policies.bypassRules.title_convention]
enabled = false          # bots must still follow title format
users   = []

[policies.bypassRules.work_items]
enabled = true           # bots are exempt from work item requirement
users   = ["dependabot[bot]", "renovate[bot]"]

[policies.bypassRules.size]
enabled = false
users   = []
```

---

## What bypass does and does not affect

| What is bypassed | What is not bypassed |
| :--- | :--- |
| Check run failure for that policy | WIP blocking (always enforced) |
| Label application for that policy | Other policies not listed in bypass |
| Comment posting for oversized PRs (size bypass) | Logging — bypass events are always logged |

---

## WIP blocking has no bypass

WIP blocking is intentionally excluded from the bypass mechanism. A PR marked as WIP must
have its markers actively removed before it can be merged. This is a deliberate design
choice: a PR that is explicitly flagged as incomplete should never be silently cleared by a
bypass rule.

---

## Security considerations

### Per-repo bypass lists are controllable by repository maintainers

Bypass rules in `.github/merge-warden.toml` live in the repository itself. Any user with
write access to the default branch can add their own login to a bypass list and bypass
checks on their own future PRs.

If this is a concern for your organisation, define bypass rules in the **application-level
configuration** (`MERGE_WARDEN_CONFIG_FILE`) instead. The application config is controlled
by the operator and cannot be overridden by per-repo configs.

### Bypass is per-login, not per-token

Bypass is based on the GitHub login name of the PR author. Impersonating a bypassed login
requires compromising that account, not merely a token rotation.

### Bypass lists are logged

Every time a bypass rule is applied, an informational log entry is written. In a production
deployment with centralised logging, this provides an audit trail.

---

## Related

- [Configure bypass rules](../how-to/configure-bypass-rules.md)
- [Set application-level policy defaults](../how-to/set-app-level-defaults.md)
- [Configuration precedence](config-precedence.md)
