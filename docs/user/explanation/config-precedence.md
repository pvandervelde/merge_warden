---
title: "Configuration precedence"
description: "How compiled-in defaults, application config, and per-repository config interact."
---

# Configuration precedence

Merge Warden resolves settings from up to four layers. Higher layers override lower ones.

```
Per-repository .github/merge-warden.toml    ← highest priority (repo can override org defaults)
        ↓  overrides
Org-level policy [defaults] section          ← org defaults (overridable by repo)
        ↑
        │  (per-repo config sits between org defaults and org enforced)
        ↓
Org-level policy [enforced] section          ← org enforced (cannot be overridden by repo)
        ↓  overrides
MERGE_WARDEN_CONFIG_FILE (application defaults)
        ↓  overrides
Compiled-in defaults                         ← lowest priority
```

The org-level policy tier is optional. When `[org_policy_source]` is not configured in
the application config, the system uses the three-tier model (application defaults →
per-repository config → compiled-in defaults).

---

## The four layers

### Layer 1 — Compiled-in defaults

Defaults compiled into the server binary. With only compiled-in defaults, **all validation
is disabled** — the server accepts webhooks and posts check results, but never fails a check
or applies a label.

These defaults exist so the server is safe to run immediately after deployment without any
configuration.

### Layer 2 — Application-level config (`MERGE_WARDEN_CONFIG_FILE`)

A TOML file supplied to the server by the operator. It sets organisation-wide defaults that
apply to every repository the server processes.

Typical use: require title validation and work item references across all repositories
without forcing every repository to maintain its own config file.

See [Application configuration schema](../reference/app-config.md) and
[Set application-level defaults](../how-to/set-app-level-defaults.md).

### Layer 3 — Org-level policy (optional)

When `[org_policy_source]` is configured in the application-level config, the server
fetches a central org policy TOML file on every PR event. This file has two sections:

- **`[defaults]`** — org-wide defaults that individual repositories *can* override with
  their own `.github/merge-warden.toml`. Sits between the application defaults and the
  per-repo config in the resolution chain.
- **`[enforced]`** — policies that *cannot* be overridden by per-repo config. Sits above
  the per-repo config in the chain.

Org policies also support `[[conditional_policies]]` blocks that apply only to repositories
matching specific topic or custom-property conditions.

See [Configure an organisation-level policy](../how-to/configure-org-policy.md).

### Layer 4 — Per-repository config (`.github/merge-warden.toml`)

A TOML file committed to the default branch of each repository. It controls policies for
that repository only.

When present, it overrides the corresponding fields from the application-level config and
any org-level defaults. It cannot override fields set in the org-level `[enforced]` section.

See [Per-repository configuration schema](../reference/per-repo-config.md).

---

## Field-level override

Precedence operates at the field level, not the file level. If `enable_title_validation` is
set in the application config but `.github/merge-warden.toml` does not include a
`[policies.pullRequests.prTitle]` section, the application-level setting applies.

If `.github/merge-warden.toml` does include that section and sets `required = false`, the
per-repo value overrides the application-level value for that field — unless the org-level
`[enforced]` section has already set `required = true` for that field, in which case the
enforced value wins.

---

## Example (three-tier, no org policy)

Assume the application config sets:

```toml
[policies]
enable_title_validation      = true
enable_work_item_validation  = true
```

And a repository's `.github/merge-warden.toml` contains:

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
required = false
```

Effective configuration for that repository:

| Policy | Effective value | Source |
| :--- | :--- | :--- |
| Title validation | disabled | Per-repo overrides app default |
| Work item validation | enabled | App default (per-repo does not set it) |

---

## What happens if no config file is provided

If `MERGE_WARDEN_CONFIG_FILE` is not set and the repository has no
`.github/merge-warden.toml`, all policies come from compiled-in defaults. With compiled-in
defaults, no checks fail and no labels are applied.

---

## Related

- [Why there are two configuration files](two-config-files.md)
- [Configure an organisation-level policy](../how-to/configure-org-policy.md)
- [Application configuration schema](../reference/app-config.md)
- [Per-repository configuration schema](../reference/per-repo-config.md)
