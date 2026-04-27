---
title: "How to configure change-type labels"
description: "Automatically apply labels that reflect the change type derived from the PR title's conventional commit prefix."
---

# How to configure change-type labels

Add the `[change_type_labels]` section to `.github/merge-warden.toml`. Merge Warden reads
the conventional commit type from the PR title and applies the matching label from your
repository's label list.

---

## Minimal configuration

```toml
schemaVersion = 1

[change_type_labels]
enabled = true
```

With only `enabled = true`, Merge Warden uses the built-in type-to-label mappings and
will create fallback labels automatically if none are found.

---

## How label detection works

For a PR titled `feat: add dark mode`, Merge Warden:

1. Extracts the type `feat` from the title.
2. Looks up the `feat` entry in `conventional_commit_mappings`.
3. Searches the repository's existing labels for a match using the configured detection
   strategy (exact name, prefix, or description).
4. Applies the first matching label.
5. If no match is found and `create_if_missing = true`, creates a fallback label using the
   `name_format` pattern (e.g. `type: feat`).

---

## Full configuration example

```toml
[change_type_labels]
enabled = true

# Maps each commit type to candidate label names to search for in the repository
[change_type_labels.conventional_commit_mappings]
feat     = ["enhancement", "feature", "new feature"]
fix      = ["bug", "bugfix", "fix"]
docs     = ["documentation", "docs"]
style    = ["style", "formatting"]
refactor = ["refactor", "refactoring", "code quality"]
perf     = ["performance", "optimization"]
test     = ["test", "tests", "testing"]
chore    = ["chore", "maintenance", "housekeeping"]
ci       = ["ci", "continuous integration", "build"]
build    = ["build", "dependencies"]
revert   = ["revert"]

# Settings for labels that are created when no existing label matches
[change_type_labels.fallback_label_settings]
# Label name format — {change_type} is replaced with the commit type
name_format      = "type: {change_type}"
# Create the label if none of the candidate names are found in the repository
create_if_missing = true

# Hex colours for auto-created labels
[change_type_labels.fallback_label_settings.color_scheme]
feat     = "#0075ca"
fix      = "#d73a4a"
docs     = "#0052cc"
style    = "#f9d0c4"
refactor = "#fef2c0"
perf     = "#a2eeef"
test     = "#d4edda"
chore    = "#e1e4e8"
ci       = "#fbca04"
build    = "#c5def5"
revert   = "#b60205"

# Detection strategy when searching for matching labels in the repository
[change_type_labels.detection_strategy]
# Match label names exactly (e.g. label named "feat")
exact_match       = true
# Match labels whose name starts with a common prefix (e.g. "type: feat")
prefix_match      = true
# Match labels whose description contains the type (e.g. "(type: feat)")
description_match = true
# Prefixes to check when prefix_match is enabled
common_prefixes   = ["type:", "kind:", "category:"]
```

---

## Label detection strategy

| Strategy | What it matches |
| :--- | :--- |
| `exact_match` | Label name equals one of the candidate values exactly |
| `prefix_match` | Label name starts with a common prefix followed by the candidate |
| `description_match` | Label description contains the candidate value |

All three are enabled by default. Disable any strategy by setting it to `false`.

---

## Related

- [Full per-repo config schema](../reference/per-repo-config.md#change_type_labels)
- [Configure PR title validation](configure-pr-title-validation.md)
