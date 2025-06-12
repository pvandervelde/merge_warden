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
```

### Schema Description

- `schemaVersion` (integer): Version of the configuration schema. Used for backward compatibility.
- `[policies.pullRequests.prTitle]`:
  - `format` (string): PR title format. Supported: `conventional-commits` (default).
- `[policies.pullRequests.workItem]`:
  - `required` (bool): Whether a work item reference is required in the PR description. Default: `true`.
  - `pattern` (string): Regex pattern for work item references. Default: `#\\d+` (e.g., `#123`).

### Default Behavior

If any value is missing, merge-warden applies the following defaults:

- PR title must follow the conventional commit format.
- PR description must contain a work item reference matching `#<number>`.

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
