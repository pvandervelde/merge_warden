---
title: "How to configure PR size labels"
description: "Automatically label pull requests by size and optionally fail the check for oversized PRs."
---

# How to configure PR size labels

Add the `[policies.pullRequests.prSize]` section to `.github/merge-warden.toml` in your
repository. Size checking is disabled by default.

---

## Minimal configuration

```toml
schemaVersion = 1

[policies.pullRequests.prSize]
enabled = true
```

With only `enabled = true`, Merge Warden labels every pull request with one of the default
size categories and does not fail the check even for very large PRs.

---

## Default size tiers

| Label (default prefix `size/`) | Line count |
| :--- | :--- |
| `size/XS` | 1 – 10 |
| `size/S` | 11 – 50 |
| `size/M` | 51 – 100 |
| `size/L` | 101 – 250 |
| `size/XL` | 251 – 500 |
| `size/XXL` | 501+ |

---

## Full configuration example

```toml
[policies.pullRequests.prSize]
enabled = true

# Fail the check when a PR is XXL (501+ lines)
fail_on_oversized = true

# Exclude generated and documentation files from the line count
excluded_file_patterns = ["package-lock.json", "*.generated.*", "docs/*"]

# Prefix for size labels
label_prefix = "size/"

# Post an educational comment on oversized PRs
add_comment = true
```

---

## Custom size thresholds

Override the default line-count boundaries with a nested `[thresholds]` block:

```toml
[policies.pullRequests.prSize]
enabled = true

[policies.pullRequests.prSize.thresholds]
xs  = 5
s   = 20
m   = 75
l   = 150
xl  = 400
# Anything above xl is automatically XXL
```

Each value is the maximum number of lines for that tier (inclusive). The `xxl` tier
covers everything above `xl` and does not need to be specified.

---

## Excluding files

`excluded_file_patterns` accepts glob-style patterns. Files whose paths match any pattern
are excluded from the line count entirely.

Common patterns:

```toml
excluded_file_patterns = [
  "package-lock.json",   # npm lockfile
  "yarn.lock",           # Yarn lockfile
  "Cargo.lock",          # Cargo lockfile
  "*.generated.*",       # auto-generated files
  "docs/*",              # documentation
  "*.min.js",            # minified assets
]
```

---

## Oversized PR comments

When `add_comment = true` and the PR is XXL, Merge Warden posts a comment explaining the
size category and suggesting the PR be split. The comment is idempotent — it will not be
posted more than once per PR.

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#policiespullrequestsprsize)
- [Configure bypass rules](configure-bypass-rules.md)
