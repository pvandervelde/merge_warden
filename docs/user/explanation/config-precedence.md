---
title: "Configuration precedence"
description: "How compiled-in defaults, application config, and per-repository config interact."
---

# Configuration precedence

Merge Warden resolves settings from three layers. Higher layers override lower ones.

```
Per-repository .github/merge-warden.toml    ← highest priority
        ↓  overrides
MERGE_WARDEN_CONFIG_FILE (application defaults)
        ↓  overrides
Compiled-in defaults                         ← lowest priority
```

---

## The three layers

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

### Layer 3 — Per-repository config (`.github/merge-warden.toml`)

A TOML file committed to the default branch of each repository. It controls policies for
that repository only.

When present, it overrides the corresponding fields from the application-level config.
Fields not specified in the per-repo file continue to use the application-level value.

See [Per-repository configuration schema](../reference/per-repo-config.md).

---

## Field-level override

Precedence operates at the field level, not the file level. If `enforceTitleValidation` is
set in the application config but `.github/merge-warden.toml` does not include a
`[policies.pullRequests.prTitle]` section, the application-level setting applies.

If `.github/merge-warden.toml` does include that section and sets `required = false`, the
per-repo value overrides the application-level value for that field.

---

## Example

Assume the application config sets:

```toml
[policies]
enforceTitleValidation    = true
enforceWorkItemValidation = true
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
- [Application configuration schema](../reference/app-config.md)
- [Per-repository configuration schema](../reference/per-repo-config.md)
