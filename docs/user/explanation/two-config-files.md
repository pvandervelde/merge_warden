---
title: "Why there are two configuration files"
description: "An explanation of why the application-level and per-repository configs use different schemas."
---

# Why there are two configuration files

Merge Warden has two separate TOML configuration files with different schemas. This is a
common source of confusion. This page explains why they exist and how they differ.

---

## The two files at a glance

| | Application config | Per-repository config |
| :--- | :--- | :--- |
| **Path** | set via `MERGE_WARDEN_CONFIG_FILE` | `.github/merge-warden.toml` |
| **Scope** | All repositories on the server | One specific repository |
| **Who controls it** | Operator (platform team) | Repository maintainer |
| **Schema** | `ApplicationDefaults` | `policies.*` |
| **Example field** | `enforceTitleValidation` | `[prTitle] required` |

---

## Why they have different schemas

The two files were designed for different audiences with different mental models.

**Operators** think in terms of broad organisational defaults: "I want all repositories to
have title validation enabled unless a team explicitly opts out." The `ApplicationDefaults`
schema reflects this with simple boolean flags at the top level.

**Repository maintainers** think in terms of specific policies with detailed settings: "I
want title validation with this pattern and this failure label." The `policies.*` schema
allows finer-grained control per policy type.

The schemas evolved to serve these different needs rather than being forced into a single
shape that would be awkward for both audiences.

---

## Practical consequences

### Pointing the wrong file at the wrong variable

If you set `MERGE_WARDEN_CONFIG_FILE` to a per-repo sample file, the server will load the
file without TOML errors but will silently apply no policy enforcement. The `policies.*`
keys are not recognised by the `ApplicationDefaults` parser and are ignored.

Always use:

- `samples/app-config.sample.toml` as the base for `MERGE_WARDEN_CONFIG_FILE`
- `samples/merge-warden.sample.toml` as the base for `.github/merge-warden.toml`

### Field name mapping

| Application config field | Per-repo equivalent |
| :--- | :--- |
| `enforceTitleValidation` | `[prTitle] required` |
| `enforceWorkItemValidation` | `[workItem] required` |
| `titlePattern` | `[prTitle] pattern` |
| `labelIfTitleInvalid` | `[prTitle] label_if_missing` |
| `workItemPattern` | `[workItem] pattern` |
| `labelIfWorkItemMissing` | `[workItem] label_if_missing` |

For the complete mapping, see [Application configuration schema](../reference/app-config.md).

---

## Related

- [Configuration precedence](config-precedence.md)
- [Application configuration schema](../reference/app-config.md)
- [Per-repository configuration schema](../reference/per-repo-config.md)
- [Set application-level defaults](../how-to/set-app-level-defaults.md)
