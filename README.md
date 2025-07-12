# merge_warden

Merge Warden is a GitHub Action and Azure Function designed to enforce pull request rules and automate workflows based on
repository configuration. It supports features like PR title validation, work item references, and more.

## Features

* **Pull Request Title Validation**: Enforces conventional commit formats for PR titles.
* **Work Item References**: Ensures PR descriptions include references to work items (e.g., issue numbers).
* **Automatic PR Size Labeling**: Automatically labels PRs based on the number of lines changed, with optional check failure for oversized PRs.

## Configuration: merge-warden Rules

merge-warden supports repository-specific configuration of pull request rules via a TOML file.

### Configuration File Location

* The configuration file must be named `.github/merge-warden.toml` and reside in the default branch.

### Purpose

This file allows you to specify which rules merge-warden should enforce for pull requests, such as PR title format and
work item requirements. If the file is missing or malformed, merge-warden will fall back to default settings and log a
warning.

### Example Configuration

```toml
schemaVersion = 1

# Define the pull request policies pertaining to the pull request title.
[policies.pullRequests.prTitle]
# Indicate if the pull request title should follow a specific format.
required = true
# The regular expression pattern that the pull request title must match. By default it follows the conventional commit
# specification.
pattern = "^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\\([a-z0-9_-]+\\))?!?: .+"
# Define the label that will be applied to the pull request if the title does not match the specified pattern.
# If the label is not specified, no label will be applied.
label_if_missing = "invalid-title-format"

[policies.pullRequests.workItem]
# Indicate if the pull request description should contain a work item reference.
required = true
# The regular expression pattern that the pull request description must match to reference a work item.
# By default, it matches issue references like `#123`, `GH-123`, or full URLs to GitHub issues.
pattern = "(?i)(fixes|closes|resolves|references|relates to)\\s+(#\\d+|GH-\\d+|https://github\\.com/[^/]+/[^/]+/issues/\\d+|[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+#\\d+)"
# Define the label that will be applied to the pull request if it does not contain a work item reference.
# If the label is not specified, no label will be applied.
label_if_missing = "missing-work-item"

# Define the pull request size checking policies.
[policies.pullRequests.prSize]
# Enable or disable PR size checking.
enabled = true
# Define custom size thresholds (optional). If not specified, defaults are used.
# [policies.pullRequests.prSize.thresholds]
# xs = 10    # 1-10 lines
# s = 50     # 11-50 lines
# m = 100    # 51-100 lines
# l = 250    # 101-250 lines
# xl = 500   # 251-500 lines
# xxl = 501  # 501+ lines (oversized)
# Whether to fail the check for oversized PRs (XXL category).
fail_on_oversized = false
# File patterns to exclude from size calculation (e.g., generated files).
excluded_file_patterns = ["package-lock.json", "yarn.lock", "*.generated.*"]
# Prefix for size labels applied to PRs.
label_prefix = "size/"
# Whether to add educational comments for oversized PRs.
add_comment = true
```

### Schema Description

* `schemaVersion` (integer): Version of the configuration schema. Used for backward compatibility.
* `[policies.pullRequests.prTitle]`:
  * `required` (bool): Whether PR title validation is enforced. Default: `true`.
  * `pattern` (string): Regex pattern for PR title format. Default: conventional commits pattern.
  * `label_if_missing` (string): Label applied when title is invalid. Optional.
* `[policies.pullRequests.workItem]`:
  * `required` (bool): Whether a work item reference is required in the PR description. Default: `true`.
  * `pattern` (string): Regex pattern for work item references. Default: `#\\d+` (e.g., `#123`).
  * `label_if_missing` (string): Label applied when work item is missing. Optional.
* `[policies.pullRequests.prSize]`:
  * `enabled` (bool): Whether PR size checking is enabled. Default: `false`.
  * `fail_on_oversized` (bool): Whether to fail check for oversized PRs. Default: `false`.
  * `excluded_file_patterns` (array): File patterns to exclude from size calculation. Default: `[]`.
  * `label_prefix` (string): Prefix for size labels. Default: `"size/"`.
  * `add_comment` (bool): Whether to add comments for oversized PRs. Default: `true`.
  * `[policies.pullRequests.prSize.thresholds]`: Custom size thresholds (optional).
    * `xs`, `s`, `m`, `l`, `xl`, `xxl` (integers): Line count thresholds for each size category.

### Default Behavior

* PR title must follow the conventional commit format.
* PR description must contain a work item reference matching `#<number>`.
* PR size checking is disabled by default for backward compatibility.

### Notes

* Only the default branch is checked for the configuration file.
* If the configuration file is missing, malformed, or has an unsupported schema version, merge-warden logs a warning and
  uses defaults.
* The schema is designed to be extensible for future rules. Always specify `schemaVersion`.

## Release Process

This project uses a custom workflow leveraging [knope](https://knope.tech/) and
[git-cliff](https://git-cliff.org/) to automate releases based on
[Conventional Commits](https://www.conventionalcommits.org/).

1. **Development:** Changes are made on feature branches and merged into the `master` branch.
    Commit messages should follow the Conventional Commits specification.
2. **Prepare Release PR:** When commits that warrant a release (e.g., `feat:`, `fix:`) are pushed
    to `master`, the `prepare-release.yml` workflow runs. It automatically:
    * Calculates the next semantic version using `knope`.
    * Cleans up any stale release branches/PRs.
    * Creates or updates a `release/X.Y.Z` branch.
    * Updates the root `Cargo.toml` version using `knope`.
    * Updates `CHANGELOG.md` using `git-cliff`.
    * Creates or updates a "Release PR" targeting `master`.
3. **Merging:** Review and merge the Release PR.
4. **Tagging & Release:** Merging the Release PR triggers the `publish-release.yml` workflow. It:
    * Creates an annotated Git tag (e.g., `0.2.0`) based on the version in `Cargo.toml`.
    * Pushes the tag.
    * Creates a corresponding GitHub release using notes extracted by `git-cliff`.
5. **Artifact Creation & Upload:** The creation of the Git tag triggers the `deploy.yml`
    workflow. This workflow:
    * Builds the `az_handler` Azure Function package.
    * Builds the `merge-warden` CLI binaries for multiple platforms.
    * Creates deployment artifacts with checksums.
    * Uploads all artifacts to the GitHub release for distribution.

## Deployment

Merge Warden is distributed as pre-built artifacts that can be deployed to various cloud platforms. The source repository provides the code and build artifacts, while deployment infrastructure is maintained separately.

### Available Deployment Options

* **Azure Functions**: Deploy using the `azure-function-package.zip` artifact from GitHub releases
* **CLI Tool**: Download platform-specific binaries from GitHub releases for local or server use

### Getting Started with Deployment

1. **Download artifacts** from the [latest GitHub release](https://github.com/pvandervelde/merge_warden/releases/latest)
2. **Review the deployment documentation** in the [`docs/deployment/`](docs/deployment/) directory
3. **Use the provided samples** in [`samples/terraform/`](samples/terraform/) for infrastructure setup
4. **Leverage CI/CD templates** from [`samples/ci-cd/`](samples/ci-cd/) for automated deployment

### Deployment Resources

* 📚 **[Azure Deployment Guide](docs/deployment/azure/)** - Complete guide for Azure Functions deployment
* 🏗️ **[Terraform Samples](samples/terraform/azure/)** - Infrastructure-as-code templates
* ⚡ **[CI/CD Workflows](samples/ci-cd/)** - GitHub Actions templates for automated deployment
* 🛠️ **[Helper Scripts](samples/scripts/)** - Deployment automation scripts

For detailed deployment instructions, see the [deployment documentation](docs/deployment/).

## Development Environment Setup for GitHub Action Testing

To test Merge Warden as a GitHub Action or webhook locally, follow these steps:

### 1. Prerequisites

* Install [Rust](https://www.rust-lang.org/tools/install) (ensure `cargo` is available)
* Install [Node.js](https://nodejs.org/) if you plan to use any JavaScript-based tooling
* Ensure you have a GitHub account and a test repository for integration

### 2. Build the CLI

Clone this repository and build the CLI:

```sh
cargo build --workspace --all-targets
```

### 3. Configure the CLI

Run the CLI to set up authentication and configuration:

```sh
cargo run --bin merge-warden -- auth github
```

Follow the prompts to provide your GitHub token or App credentials.

### 4. Set Up a Local Webhook Receiver

You can run the Azure Function or the CLI locally to receive webhook events. For local testing,
you may use a tool like [smee](https://smee.io/) to expose your local port to the internet:

```sh
smee http 3100
```

Note the public URL provided by smee (e.g., `https://abcd1234.smee.io`).

### 5. Configure GitHub Webhooks

In your test repository on GitHub:

* Go to **Settings > Webhooks**
* Click **Add webhook**
* Set the **Payload URL** to your smee URL (e.g., `https://abcd1234.smee.io/api/webhook`)
* Set **Content type** to `application/json`
* Select events you want to trigger the webhook (e.g., Pull requests)
* Save the webhook

### 6. Run the CLI or Azure Function

Start the webhook receiver locally. For the CLI:

```sh
cargo run --bin merge-warden -- serve
```

Or run the Azure Function host if testing the Azure deployment.

### 7. Test the Integration

* Create or update pull requests in your test repository
* Observe the CLI or Azure Function logs for webhook events and validation results

### 8. Troubleshooting

* Ensure your webhook secret matches between GitHub and your local config
* Check firewall or network settings if webhooks are not received
* Use `RUST_LOG=debug` for more verbose output

### Smart Change Type Label Detection

merge-warden includes intelligent change type label detection that automatically applies appropriate labels based on conventional commit types in PR titles. This feature uses a sophisticated detection strategy to find and use existing repository labels instead of creating duplicate labels.

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
