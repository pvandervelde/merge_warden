---
title: "How to configure an organisation-level policy"
description: "Store shared PR policies in a central repository and enforce them across all repositories."
---

# How to configure an organisation-level policy

An organisation-level policy file lets a platform team define PR policies in one central
place and have them applied to every repository the Merge Warden server processes — without
each repository needing to maintain a `.github/merge-warden.toml`.

The org policy adds a fourth tier to the configuration resolution chain (1 = highest priority):

| Priority | Source | Overridable? |
| :---: | :--- | :--- |
| 1 | Org-enforced policies — `[enforced]` section | No — beats everything below |
| 2 | Per-repository `.github/merge-warden.toml` | — |
| 3 | Org-default policies — `[defaults]` section | Yes — per-repo config overrides these |
| 4 | Application defaults — `MERGE_WARDEN_CONFIG_FILE` | Yes |
| 5 (lowest) | Compiled-in defaults | Yes |

---

## Step 1: Create the org policy repository

Create (or designate) a repository in your organisation to hold the policy file. The
repository must be accessible to the GitHub App installation.

A common convention is a repository named `platform-configs` or `.github`.

---

## Step 2: Create the org policy file

Create a file in the policy repository. Any path works; `merge-warden/org-policy.toml`
is a readable convention.

The file must begin with `schemaVersion = 1` and may include an `[enforced]` section,
a `[defaults]` section, and any number of `[[conditional_policies]]` blocks.

### Minimal example

```toml
schemaVersion = 1

# Settings that CANNOT be overridden by per-repo config.
[enforced.policies.pullRequests.prTitle]
required = true
label_if_missing = "pr-issue: invalid-title-format"
```

### Full example

See
[`samples/merge-warden-org-policy.sample.toml`](https://github.com/pvandervelde/merge_warden/blob/master/samples/merge-warden-org-policy.sample.toml)
for an annotated example covering enforced, default, and conditional policy sections.

---

## Step 3: Point the server at the org policy file

Add `[org_policy_source]` to the application-level configuration file
(`MERGE_WARDEN_CONFIG_FILE`):

```toml
[org_policy_source]
owner = "my-org"
repo  = "platform-configs"
path  = "merge-warden/org-policy.toml"

# Optional — set to true to fail PR processing if the policy file is unreachable.
# Default: false (degrade gracefully to three-tier resolution when unreachable).
# fail_if_unreachable = false
```

The server fetches the org policy file on every PR event using the same GitHub App
credentials configured for the server instance.

> **Required permission:** The GitHub App must have **`Contents: Read`** permission
> on the policy repository. Without it, every fetch will fail and PR processing will
> either be aborted (if `fail_if_unreachable = true`) or degrade to three-tier
> resolution (the default).

---

## Enforced vs default sections

| Section | When to use | Can be overridden by per-repo config? |
| :--- | :--- | :--- |
| `[enforced.*]` | Non-negotiable requirements | No — org enforced beats per-repo config |
| `[defaults.*]` | Sensible org defaults | Yes — per-repo config can override them |

Use the `[enforced]` section for compliance requirements that must hold regardless of
what individual teams want. Use `[defaults]` for policies that most teams should follow
but that individual teams may need to adjust.

---

## Conditional policies

A `[[conditional_policies]]` block applies only to repositories that match its condition.
Conditions evaluate against repository **topics** and **custom properties**.

```toml
[[conditional_policies]]
[conditional_policies.condition]
# Applies to any repository with the "payments" or "finance" topic.
has_any_topic = ["payments", "finance"]

[conditional_policies.enforced.policies.pullRequests.prTitle]
required = true
pattern = "^PAY-[0-9]+: .+"
label_if_missing = "pr-issue: invalid-title-format"
```

### Condition semantics

| Criterion | Match logic |
| :--- | :--- |
| `has_any_topic` | At least one listed topic must be present (OR, case-insensitive) |
| `has_custom_property` | All listed key-value pairs must be present (AND, values case-sensitive) |
| Both criteria together | Both must match simultaneously (AND) |
| Empty condition | Matches every repository |

### Custom properties

Custom properties require a GitHub Enterprise plan and the
`org_custom_property: read` permission on the GitHub App. On non-enterprise GitHub
plans the custom properties API returns a 403/404 response; Merge Warden treats this as
an empty map and evaluates topic conditions normally.

---

## Behaviour when the org policy file is unreachable

| `fail_if_unreachable` | File status | Behaviour |
| :--- | :--- | :--- |
| `false` (default) | Fetch error or parse error | Warn and fall back to three-tier resolution |
| `false` (default) | File does not exist (`404`) | Warn and fall back to three-tier resolution |
| `true` | Fetch error or parse error | Return an error; PR processing aborted |
| `true` | File does not exist (`404`) | Warn and fall back (file absence is always a graceful fallback) |

The rationale for the `404` exception: a missing file is the expected bootstrap state
when the policy file has not yet been created. A fetch error, by contrast, indicates an
infrastructure problem that the operator may want to surface explicitly.

---

## Related

- [Application configuration schema — org_policy_source](../reference/app-config.md#org_policy_source)
- [Configuration precedence](../explanation/config-precedence.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
- [Per-repository configuration schema](../reference/per-repo-config.md)
