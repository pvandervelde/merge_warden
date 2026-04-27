---
title: "How to configure WIP detection"
description: "Block merging of pull requests that are still in progress by detecting WIP markers."
---

# How to configure WIP detection

Add the `[policies.pullRequests.wip]` section to `.github/merge-warden.toml` in your
repository. WIP detection is disabled by default.

---

## Minimal configuration

```toml
schemaVersion = 1

[policies.pullRequests.wip]
enforce_wip_blocking = true
```

When enabled, Merge Warden scans every PR title for the default WIP patterns. If a match is
found the check is set to **failure**, preventing the PR from being merged until the markers
are removed.

---

## Default WIP title patterns

When no `wip_title_patterns` are specified, the following substrings are checked
(case-sensitive):

- `WIP`
- `wip:`
- `[wip]`
- `draft:`
- `Draft:`

---

## Full configuration example

```toml
[policies.pullRequests.wip]
# Enable WIP blocking
enforce_wip_blocking = true

# Label applied to WIP pull requests (remove this key to disable labeling)
wip_label = "WIP"

# Substrings to match in the PR TITLE (case-sensitive, any match triggers blocking)
wip_title_patterns = ["WIP", "wip:", "[wip]", "draft:", "Draft:"]

# Substrings to match in the PR DESCRIPTION (empty by default — opt-in)
wip_description_patterns = []
```

---

## Pattern matching rules

- Matching uses substring search (`contains`), not regex.
- Matching is **case-sensitive**.
- If a shorter pattern is already in the list, longer patterns that contain it are
  redundant. For example, `WIP` already matches titles containing `[WIP]` or `WIP:`.
- Description patterns are empty by default. Add entries to `wip_description_patterns`
  to also scan the PR body.

---

## WIP blocking cannot be bypassed

Unlike title or work-item validation, WIP blocking has no bypass mechanism. This is
intentional — a PR marked as WIP must have its markers removed before it can be merged,
regardless of who opened it.

---

## Removing the WIP state

Edit the PR title or description to remove all matching WIP patterns. Merge Warden
re-evaluates the PR on the next webhook event and clears the failure status automatically.

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestswip)
- [Configure PR state lifecycle labels](configure-pr-state-labels.md)
