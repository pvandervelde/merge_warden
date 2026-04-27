# merge_warden

Merge Warden is a GitHub webhook server that enforces pull request policies and automates
PR workflows. Run it as a container, point your GitHub App's webhook at it, and it
automatically checks every pull request against rules you configure per repository.

📖 **[Full documentation](https://pvandervelde.github.io/merge_warden/)**

## Features

- **PR title validation** — enforces conventional commit format (or your own regex)
- **Work-item references** — requires a linked issue in the PR description
- **PR size labels** — automatically labels XS / S / M / L / XL / XXL by lines changed
- **WIP detection** — blocks merge when the PR title or description contains WIP markers
- **PR state labels** — applies a single label reflecting draft / in-review / approved state
- **Issue propagation** — copies milestone and Projects v2 membership from the linked issue
- **Change-type labels** — maps conventional commit types to repository labels
- **Bypass rules** — per-policy lists of users who can skip validation

All policies are configured with a TOML file committed to each repository at
`.github/merge-warden.toml`. A server-level config file sets the defaults that apply when
no per-repo file exists.

## Quick start

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID=<your-app-id> \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e GITHUB_WEBHOOK_SECRET=<your-webhook-secret> \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

Then verify:

```bash
curl http://localhost:3000/api/merge_warden
# HTTP 200 OK
```

See the [getting-started tutorial](https://pvandervelde.github.io/merge_warden/tutorials/01-getting-started/)
for the complete walkthrough including GitHub App setup.

## Deployment

Merge Warden runs as an OCI container on any container host:

- [Azure Container Apps](https://pvandervelde.github.io/merge_warden/how-to/deploy-on-azure/)
- [AWS ECS / Fargate](https://pvandervelde.github.io/merge_warden/how-to/deploy-on-aws/)
- [Local development with Docker](https://pvandervelde.github.io/merge_warden/how-to/run-locally/)

## Configuration

Add `.github/merge-warden.toml` to the default branch of any repository you want Merge
Warden to monitor. The server reads this file via the GitHub API on every webhook event —
no restart needed when you change it.

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
label_if_missing = "invalid-title-format"

[policies.pullRequests.workItem]
required = true
label_if_missing = "missing-work-item"

[policies.pullRequests.prSize]
enabled = true
fail_on_oversized = false
```

Full schema reference: [per-repo config](https://pvandervelde.github.io/merge_warden/reference/per-repo-config/)

## Contributing

Issues and pull requests are welcome. Please open an issue before starting work on a
significant change.

## License

[MIT](LICENSE)
* Use `RUST_LOG=debug` for more verbose output

### Change Type Label Detection

merge-warden includes change type label detection that automatically applies appropriate
labels based on conventional commit types in PR titles. This feature uses a strategy
to find and use existing repository labels instead of creating duplicate labels.

#### Detection Strategy

The system uses a three-tier approach to find appropriate labels:

1. **Exact Match**: Searches for labels that exactly match conventional commit types or their common aliases
2. **Prefix Match**: Searches for labels with prefixes like `type:`, `kind:`, or `category:`
3. **Description Match**: Searches for labels with descriptions ending with `(type: <CHANGE_TYPE>)`

#### Configuration

##### Repository-Level Configuration (.github/merge-warden.toml)

```toml
[change_type_labels]
enabled = true

# Define mappings from conventional commit types to repository labels
[change_type_labels.conventional_commit_mappings]
feat = ["enhancement", "feature", "new feature"]
fix = ["bug", "bugfix", "fix"]
docs = ["documentation", "docs"]
style = ["style", "formatting"]
refactor = ["refactor", "refactoring", "code quality"]
perf = ["performance", "optimization"]
test = ["test", "tests", "testing"]
chore = ["chore", "maintenance", "housekeeping"]
ci = ["ci", "continuous integration", "build"]
build = ["build", "dependencies"]
revert = ["revert"]

# Fallback label settings when no existing labels match
[change_type_labels.fallback_label_settings]
name_format = "type: {change_type}"
create_if_missing = true

# Color scheme for fallback labels (hex colors)
[change_type_labels.fallback_label_settings.color_scheme]
feat = "#0075ca"
fix = "#d73a4a"
docs = "#0052cc"
style = "#f9d0c4"
refactor = "#fef2c0"
perf = "#a2eeef"
test = "#d4edda"
chore = "#e1e4e8"
ci = "#fbca04"
build = "#c5def5"
revert = "#b60205"

# Detection strategy configuration
[change_type_labels.detection_strategy]
exact_match = true
prefix_match = true
description_match = true
common_prefixes = ["type:", "kind:", "category:"]
```

**Repository Configuration Options:**

| Configuration Section | Key | Type | Default | Description |
|----------------------|-----|------|---------|-------------|
| `[change_type_labels]` | `enabled` | Boolean | `true` | Enable/disable smart label detection for this repository |
| `[change_type_labels.conventional_commit_mappings]` | `feat` | Array | `["enhancement", "feature", "new feature"]` | Repository labels to search for `feat` commits |
| | `fix` | Array | `["bug", "bugfix", "fix"]` | Repository labels to search for `fix` commits |
| | `docs` | Array | `["documentation", "docs"]` | Repository labels to search for `docs` commits |
| | `style` | Array | `["style", "formatting"]` | Repository labels to search for `style` commits |
| | `refactor` | Array | `["refactor", "refactoring", "code quality"]` | Repository labels to search for `refactor` commits |
| | `perf` | Array | `["performance", "optimization"]` | Repository labels to search for `perf` commits |
| | `test` | Array | `["test", "tests", "testing"]` | Repository labels to search for `test` commits |
| | `chore` | Array | `["chore", "maintenance", "housekeeping"]` | Repository labels to search for `chore` commits |
| | `ci` | Array | `["ci", "continuous integration", "build"]` | Repository labels to search for `ci` commits |
| | `build` | Array | `["build", "dependencies"]` | Repository labels to search for `build` commits |
| | `revert` | Array | `["revert"]` | Repository labels to search for `revert` commits |
| `[change_type_labels.fallback_label_settings]` | `name_format` | String | `"type: {change_type}"` | Format for creating fallback labels (`{change_type}` is replaced with the commit type) |
| | `create_if_missing` | Boolean | `true` | Whether to create labels when none exist in the repository |
| `[change_type_labels.fallback_label_settings.color_scheme]` | `feat` | String | `"#0075ca"` | Hex color for `feat` fallback labels |
| | `fix` | String | `"#d73a4a"` | Hex color for `fix` fallback labels |
| | `docs` | String | `"#0052cc"` | Hex color for `docs` fallback labels |
| | `style` | String | `"#f9d0c4"` | Hex color for `style` fallback labels |
| | `refactor` | String | `"#fef2c0"` | Hex color for `refactor` fallback labels |
| | `perf` | String | `"#a2eeef"` | Hex color for `perf` fallback labels |
| | `test` | String | `"#d4edda"` | Hex color for `test` fallback labels |
| | `chore` | String | `"#e1e4e8"` | Hex color for `chore` fallback labels |
| | `ci` | String | `"#fbca04"` | Hex color for `ci` fallback labels |
| | `build` | String | `"#c5def5"` | Hex color for `build` fallback labels |
| | `revert` | String | `"#b60205"` | Hex color for `revert` fallback labels |
| `[change_type_labels.detection_strategy]` | `exact_match` | Boolean | `true` | Enable exact name matching against repository labels |
| | `prefix_match` | Boolean | `true` | Enable prefix matching (e.g., `type:feat` matches `feat`) |
| | `description_match` | Boolean | `true` | Enable description matching (e.g., label description ending with `(type: feat)`) |
| | `common_prefixes` | Array | `["type:", "kind:", "category:"]` | Prefixes to check during prefix matching |

##### Azure Function Configuration

The Azure Function can be configured via Azure App Configuration to control change type label detection behavior at the application level. This provides centralized configuration management across all repositories that use the Azure Function.

**Azure App Configuration Keys:**

| Configuration Key | Type | Default | Description |
|------------------|------|---------|-------------|
| `change_type_labels:enabled` | Boolean | `true` | Enable/disable smart label detection |
| `change_type_labels:mappings:feat` | JSON Array | `["enhancement", "feature", "new feature"]` | Repository labels to search for `feat` commits |
| `change_type_labels:mappings:fix` | JSON Array | `["bug", "bugfix", "fix"]` | Repository labels to search for `fix` commits |
| `change_type_labels:mappings:docs` | JSON Array | `["documentation", "docs"]` | Repository labels to search for `docs` commits |
| `change_type_labels:fallback:name_format` | String | `"type: {change_type}"` | Format for creating fallback labels |
| `change_type_labels:fallback:create_if_missing` | Boolean | `true` | Whether to create labels when none exist |
| `change_type_labels:colors:feat` | String | `"#0075ca"` | Color for `feat` fallback labels |
| `change_type_labels:colors:fix` | String | `"#d73a4a"` | Color for `fix` fallback labels |
| `change_type_labels:detection:exact_match` | Boolean | `true` | Enable exact name matching |
| `change_type_labels:detection:prefix_match` | Boolean | `true` | Enable prefix matching (e.g., `type:`) |
| `change_type_labels:detection:description_match` | Boolean | `true` | Enable description matching |
| `change_type_labels:detection:common_prefixes` | JSON Array | `["type:", "kind:", "category:"]` | Prefixes to check for prefix matching |

**Example Azure App Configuration:**

```json
{
  "change_type_labels:enabled": "true",
  "change_type_labels:mappings:feat": ["enhancement", "feature", "new feature"],
  "change_type_labels:mappings:fix": ["bug", "bugfix", "fix"],
  "change_type_labels:fallback:name_format": "type: {change_type}",
  "change_type_labels:fallback:create_if_missing": "true",
  "change_type_labels:colors:feat": "#0075ca",
  "change_type_labels:colors:fix": "#d73a4a",
  "change_type_labels:detection:exact_match": "true",
  "change_type_labels:detection:prefix_match": "true",
  "change_type_labels:detection:description_match": "true",
  "change_type_labels:detection:common_prefixes": ["type:", "kind:", "category:"]
}
```

**Configuration Priority:**

1. **Repository Configuration**: `.github/merge-warden.toml` (highest priority)
2. **Azure App Configuration**: Application-level defaults
3. **Hardcoded Defaults**: Built-in fallback values

The Azure Function automatically:

* Loads Azure App Configuration settings on startup
* Merges repository configuration when available
* Falls back gracefully to Azure App Configuration when repository config is missing or invalid
* Uses hardcoded defaults if Azure App Configuration is unavailable

#### Behavior

* **Repository-first**: Uses existing repository labels when possible
* **Consistent styling**: Respects repository's existing label colors and naming
* **Fallback creation**: Creates standardized labels only when none exist
* **Configurable**: Both application and repository-level configuration
* **Non-blocking**: Label failures don't block PR processing

#### Repository Configuration Options

| Configuration Section | Key | Type | Default | Description |
|----------------------|-----|------|---------|-------------|
| `[change_type_labels]` | `enabled` | Boolean | `true` | Enable/disable smart label detection for this repository |
| `[change_type_labels.conventional_commit_mappings]` | `feat` | Array | `["enhancement", "feature", "new feature"]` | Repository labels to search for `feat` commits |
| | `fix` | Array | `["bug", "bugfix", "fix"]` | Repository labels to search for `fix` commits |
| | `docs` | Array | `["documentation", "docs"]` | Repository labels to search for `docs` commits |
| | `style` | Array | `["style", "formatting"]` | Repository labels to search for `style` commits |
| | `refactor` | Array | `["refactor", "refactoring", "code quality"]` | Repository labels to search for `refactor` commits |
| | `perf` | Array | `["performance", "optimization"]` | Repository labels to search for `perf` commits |
| | `test` | Array | `["test", "tests", "testing"]` | Repository labels to search for `test` commits |
| | `chore` | Array | `["chore", "maintenance", "housekeeping"]` | Repository labels to search for `chore` commits |
| | `ci` | Array | `["ci", "continuous integration", "build"]` | Repository labels to search for `ci` commits |
| | `build` | Array | `["build", "dependencies"]` | Repository labels to search for `build` commits |
| | `revert` | Array | `["revert"]` | Repository labels to search for `revert` commits |
| `[change_type_labels.fallback_label_settings]` | `name_format` | String | `"type: {change_type}"` | Format for creating fallback labels (`{change_type}` is replaced with the commit type) |
| | `create_if_missing` | Boolean | `true` | Whether to create labels when none exist in the repository |
| `[change_type_labels.fallback_label_settings.color_scheme]` | `feat` | String | `"#0075ca"` | Hex color for `feat` fallback labels |
| | `fix` | String | `"#d73a4a"` | Hex color for `fix` fallback labels |
| | `docs` | String | `"#0052cc"` | Hex color for `docs` fallback labels |
| | `style` | String | `"#f9d0c4"` | Hex color for `style` fallback labels |
| | `refactor` | String | `"#fef2c0"` | Hex color for `refactor` fallback labels |
| | `perf` | String | `"#a2eeef"` | Hex color for `perf` fallback labels |
| | `test` | String | `"#d4edda"` | Hex color for `test` fallback labels |
| | `chore` | String | `"#e1e4e8"` | Hex color for `chore` fallback labels |
| | `ci` | String | `"#fbca04"` | Hex color for `ci` fallback labels |
| | `build` | String | `"#c5def5"` | Hex color for `build` fallback labels |
| | `revert` | String | `"#b60205"` | Hex color for `revert` fallback labels |
| `[change_type_labels.detection_strategy]` | `exact_match` | Boolean | `true` | Enable exact name matching against repository labels |
| | `prefix_match` | Boolean | `true` | Enable prefix matching (e.g., `type:feat` matches `feat`) |
| | `description_match` | Boolean | `true` | Enable description matching (e.g., label description ending with `(type: feat)`) |
| | `common_prefixes` | Array | `["type:", "kind:", "category:"]` | Prefixes to check during prefix matching |

## Troubleshooting

If you encounter issues with Merge Warden, consider the following troubleshooting steps:

1. **Check Configuration**: Ensure your `.github/merge-warden.toml` file is correctly formatted and valid.
2. **Review Logs**: Examine the logs from the Merge Warden action or Azure Function for error messages or warnings.
3. **Test Regex Patterns**: Use a regex testing tool to validate your PR title and work item reference patterns.
4. **Validate Webhook Setup**: Ensure GitHub webhooks are correctly configured and the payload URL is reachable.
5. **Inspect Labels**: Check that the expected labels exist in the repository and match the configured patterns.
6. **Debug Locally**: Use the development environment setup to debug issues locally with real webhook events.
7. **Consult Documentation**: Refer to this documentation for guidance on configuration options and features.
8. **Seek Support**: If problems persist, consider seeking support from the repository maintainers or community.

## Contributing

Contributions to Merge Warden are welcome! To contribute:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes and commit them with descriptive messages.
4. Push your branch to your forked repository.
5. Submit a pull request describing your changes.

Please ensure your code follows the project's coding standards and includes appropriate tests.

## License

Merge Warden is licensed under the MIT License. See the `LICENSE` file for details.
