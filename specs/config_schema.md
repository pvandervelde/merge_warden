# Design: merge-warden Configuration Schema

## Title

merge-warden Configuration Schema and Rule Extensibility

## Problem Description

Currently, merge-warden enforces a fixed set of pull request rules. Different repositories may require different rules
or policies. There is a need for repository-specific configuration to allow flexible rule enforcement.

## Surrounding Context

- merge-warden is used as a GitHub Action and Azure Function to enforce PR rules.
- Rules are currently hardcoded and not configurable per repository.

## Proposed Solution

Introduce a configuration file `.github/merge-warden.toml` in the default branch of each repository. This file will
define which rules to enforce for pull requests, using a versioned TOML schema.

### Alternatives Considered

- Using YAML or JSON instead of TOML (TOML chosen for readability and convention in GitHub workflows).
- Storing configuration outside the repository (less transparent and harder to audit).

## Design

### Configuration File Location

- `.github/merge-warden.toml` in the default branch.

### Example Configuration

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
format = "conventional-commits"

[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"

[policies.pullRequests.prSize]
enabled = true
fail_on_oversized = false

[change_type_labels]
enabled = true

[change_type_labels.conventional_commit_mappings]
feat = ["enhancement", "feature"]
fix = ["bug", "bugfix"]
docs = ["documentation"]

[change_type_labels.fallback_label_settings]
name_format = "type: {change_type}"
create_if_missing = true

[change_type_labels.detection_strategy]
exact_match = true
prefix_match = true
description_match = true
```

### Schema Description

- `schemaVersion` (integer): Version of the configuration schema. Used for backward compatibility.
- `[policies.pullRequests.prTitle]`:
  - `format` (string): PR title format. Supported: `conventional-commits` (default).
- `[policies.pullRequests.workItem]`:
  - `required` (bool): Whether a work item reference is required in the PR description. Default: `true`.
  - `pattern` (string): Regex pattern for work item references. Default: `#\\d+` (e.g., `#123`).

- `[policies.pullRequests.prSize]`:
  - `enabled` (bool): Whether PR size checking and labeling is enabled. Default: `false`.
  - `fail_on_oversized` (bool): Whether to fail the check for oversized PRs (XXL category). Default: `false`.
  - `excluded_file_patterns` (array): File patterns to exclude from size calculations. Default: `[]`.
  - `label_prefix` (string): Prefix for size labels. Default: `"size/"`.
  - `add_comment` (bool): Whether to add educational comments for oversized PRs. Default: `true`.
  - `[policies.pullRequests.prSize.thresholds]`: Custom size thresholds (optional).
    - `xs`, `s`, `m`, `l`, `xl`, `xxl` (integers): Line count thresholds for each size category.

- `[change_type_labels]`:
  - `enabled` (bool): Whether smart change type label detection is enabled. Default: `true`.
  - `[change_type_labels.conventional_commit_mappings]`: Mappings from conventional commit types to repository-specific labels.
    - `feat` (array): Labels to search for `feat` commits. Default: `["enhancement", "feature", "new feature"]`.
    - `fix` (array): Labels to search for `fix` commits. Default: `["bug", "bugfix", "fix"]`.
    - `docs` (array): Labels to search for `docs` commits. Default: `["documentation", "docs"]`.
    - `style` (array): Labels to search for `style` commits. Default: `["style", "formatting"]`.
    - `refactor` (array): Labels to search for `refactor` commits. Default: `["refactor", "refactoring", "code quality"]`.
    - `perf` (array): Labels to search for `perf` commits. Default: `["performance", "optimization"]`.
    - `test` (array): Labels to search for `test` commits. Default: `["test", "tests", "testing"]`.
    - `chore` (array): Labels to search for `chore` commits. Default: `["chore", "maintenance", "housekeeping"]`.
    - `ci` (array): Labels to search for `ci` commits. Default: `["ci", "continuous integration", "build"]`.
    - `build` (array): Labels to search for `build` commits. Default: `["build", "dependencies"]`.
    - `revert` (array): Labels to search for `revert` commits. Default: `["revert"]`.
  - `[change_type_labels.fallback_label_settings]`: Settings for creating fallback labels when repository labels are not found.
    - `name_format` (string): Format for creating new label names. Default: `"type: {change_type}"`.
    - `create_if_missing` (bool): Whether to create fallback labels if none are found. Default: `true`.
    - `[change_type_labels.fallback_label_settings.color_scheme]`: Colors for fallback labels (hex format).
      - `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`, `revert` (strings): Hex color codes for each commit type.
  - `[change_type_labels.detection_strategy]`: Configuration for label detection strategy.
    - `exact_match` (bool): Enable exact name matching. Default: `true`.
    - `prefix_match` (bool): Enable prefix matching (e.g., `type:`, `kind:`). Default: `true`.
    - `description_match` (bool): Enable description matching. Default: `true`.
    - `common_prefixes` (array): Common prefixes to check for prefix matching. Default: `["type:", "kind:", "category:"]`.

### Default Behavior

If any value is missing, merge-warden applies the following defaults:

- PR title must follow the conventional commit format.
- PR description must contain a work item reference matching `#<number>`.
- PR size checking is disabled for backward compatibility.
- Smart change type label detection is enabled and uses intelligent repository-specific label mapping.

### Backward Compatibility

- The `schemaVersion` field is mandatory.
- If the configuration file is missing, malformed, or has an unsupported schema version, merge-warden logs a warning
  and uses defaults.
- The schema is designed to be extensible for future rules.

## Other Relevant Details

- Only the default branch is checked for the configuration file.
- The configuration is loaded at runtime and can be changed by updating the file in the repository.

## Conclusion

This design enables repository-specific, extensible, and versioned configuration of pull request rules in merge-warden,
supporting current and future policy needs.
