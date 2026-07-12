---
title: "How to configure repository scope filtering"
description: "Restrict which repositories Merge Warden actively processes, independent of GitHub App installation scope."
---

# How to configure repository scope filtering

Add the `[policies.repository_scope]` section to the application-level configuration file
(`MERGE_WARDEN_CONFIG_FILE`) to restrict which repositories Merge Warden actively processes.
This is an operator-only setting — it cannot be set in a repository's own
`.github/merge-warden.toml`.

This section is optional. Omitting it entirely processes every repository the GitHub App is
installed on, preserving the pre-existing behaviour.

---

## Allow-listing specific repositories

```toml
# In the application-level config file (MERGE_WARDEN_CONFIG_FILE)
[policies.repository_scope]
include_patterns = ["payments-*", "checkout", "billing-?"]
exclude_patterns = ["payments-legacy"]
```

With this configuration:

- `payments-api`, `payments-web`, `checkout`, and `billing-1` are processed (they match an
  `include_patterns` entry).
- `payments-legacy` is **not** processed, even though it matches `payments-*`, because
  `exclude_patterns` takes precedence over `include_patterns`.
- `inventory-service` is not processed — it matches no `include_patterns` entry.

`exclude_patterns` is optional and defaults to an empty list when omitted.

---

## Pausing all processing (fail-closed kill switch)

Set `include_patterns` to an explicit empty list to stop Merge Warden from processing
**any** repository, regardless of `exclude_patterns`:

```toml
[policies.repository_scope]
include_patterns = []
```

This is a deliberate fail-closed lever — useful as an emergency "pause everything" switch
(for example, while investigating an incident) without having to uninstall the GitHub App
or tear down the deployment. Restore normal operation by either removing the
`[policies.repository_scope]` section entirely or by setting `include_patterns` back to a
non-empty list.

---

## Pattern syntax

| Wildcard | Matches |
| :--- | :--- |
| `*` | Any sequence of characters (including none) |
| `?` | Exactly one character |

Every other character is matched literally. Patterns are matched **case-insensitively**
against the bare repository name (not `owner/repo` — only the `repo` part), and the match
is anchored to the full name (a pattern must match the entire repository name, not a
substring of it).

Examples:

| Pattern | Matches | Does not match |
| :--- | :--- | :--- |
| `payments-*` | `payments-api`, `payments-` | `my-payments-api` |
| `billing-?` | `billing-1`, `billing-x` | `billing-12`, `billing-` |
| `checkout` | `checkout`, `CheckOut` (case-insensitive) | `checkout-v2` |

A pattern containing characters other than letters, digits, `-`, `_`, `*`, and `?` is
rejected at server startup with a configuration error — Merge Warden fails fast rather than
silently matching nothing (or everything) at webhook-handling time.

---

## What happens to filtered-out repositories

For a repository outside the configured scope:

- The event is acknowledged (no error, no retry triggered on GitHub's side) but no further
  processing occurs.
- No `.github/merge-warden.toml`, org-policy, topics, or custom-properties fetch happens.
- No GitHub API call is made on behalf of that repository at all.
- A webhook payload with a missing or malformed `repository.name` field is also treated as
  out of scope (fail-closed), regardless of `[policies.repository_scope]` configuration.

See [How Merge Warden works — the event lifecycle](../explanation/how-merge-warden-works.md)
for where this check runs relative to the rest of event processing.

---

## Interaction with per-repository and org-level configuration

Repository scope filtering is evaluated **before** any configuration is resolved — it is
not part of the four-layer configuration precedence chain described in
[Configuration precedence](../explanation/config-precedence.md). This has one important
consequence: a repository excluded by `[policies.repository_scope]` cannot re-include
itself via its own `.github/merge-warden.toml`. That file is never fetched for an
out-of-scope repository, so there is nothing for it to override. See
[Configuration precedence — repository scope filtering runs first](../explanation/config-precedence.md#repository-scope-filtering-runs-before-any-of-this)
for details.

---

## Related

- [Application configuration schema — policies.repository_scope](../reference/app-config.md#policiesrepository_scope)
- [Configuration precedence](../explanation/config-precedence.md)
- [How Merge Warden works](../explanation/how-merge-warden-works.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
