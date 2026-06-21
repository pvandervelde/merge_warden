---
title: "How to configure Renovate stability labels"
description: "Track the Renovate stability period for dependency-update PRs with an automatic label."
---

# How to configure Renovate stability labels

When you use [Renovate](https://docs.renovatebot.com/) to manage dependency updates, Renovate
can be configured to wait a number of days before auto-merging a new package version (the
`stabilityDays` setting). Merge Warden integrates with this mechanism by watching for the
`renovate/stability-days` GitHub commit status and applying a configurable label while the
wait period has not yet elapsed.

This feature is **enabled by default**. No configuration is required to turn it on.

---

## What the feature does

On every PR event Merge Warden checks whether a commit status named
`renovate/stability-days` exists on the PR's head commit:

- If the status is `pending`, `error`, or `failure`, the `pending_stability_label` is applied.
- If the status is `success`, the label is removed.
- If no such status exists (the PR is not a Renovate PR), nothing happens.

The label is purely informational. It does not affect the Merge Warden check result and
cannot block merging on its own.

---

## Default configuration

No configuration file entry is required. The defaults are:

| Field | Default |
| :--- | :--- |
| `enabled` | `true` |
| `pending_stability_label` | `"pr-validation: pending-stability"` |

---

## Disabling the feature

Renovate stability label management can only be disabled at the **application level**
(`MERGE_WARDEN_CONFIG_FILE`). Setting `enabled = false` at the application level prevents
it from being activated by per-repo config.

```toml
# In the application-level config file (MERGE_WARDEN_CONFIG_FILE)
[policies.renovate_stability]
enabled = false
```

> **Note:** A per-repo setting of `enabled = false` has no effect if the application
> level has `enabled = true`. The merge rule is OR: once either tier enables the feature,
> it stays enabled.

---

## Customising the label name

To use a different label name, add the following to `.github/merge-warden.toml`:

```toml
schemaVersion = 1

[policies.pullRequests.renovateStability]
enabled = true
pending_stability_label = "dependency: awaiting-stability"
```

Create the label in the repository first (via GitHub repository settings or the API).
Merge Warden applies labels by name and does not create labels for this feature.

---

## Application-level override

To customise the label globally across all repositories:

```toml
# In the application-level config file
[policies.renovate_stability]
enabled = true
pending_stability_label = "renovate: stability-pending"
```

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestsrenovatestability)
- [Application configuration schema](../reference/app-config.md#policiesrenovate_stability)
- [Configure PR state lifecycle labels](configure-pr-state-labels.md)
