# merge_warden

Merge Warden is a GitHub Action and Azure Function designed to enforce pull request rules and automate workflows based on
repository configuration. It supports features like PR title validation, work item references, and more.

## Features

* **Pull Request Title Validation**: Enforces conventional commit formats for PR titles.
* **Work Item References**: Ensures PR descriptions include references to work items (e.g., issue numbers).

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
```

### Schema Description

* `schemaVersion` (integer): Version of the configuration schema. Used for backward compatibility.
* `[policies.pullRequests.prTitle]`:
  * `format` (string): PR title format. Supported: `conventional-commits` (default).
* `[policies.pullRequests.workItem]`:
  * `required` (bool): Whether a work item reference is required in the PR description. Default: `true`.
  * `pattern` (string): Regex pattern for work item references. Default: `#\\d+` (e.g., `#123`).

### Default Behavior

* PR title must follow the conventional commit format.
* PR description must contain a work item reference matching `#<number>`.

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
5. **Deployment & Binary Upload:** The creation of the Git tag triggers the `deploy.yml`
    workflow. This workflow:
    * Builds the `az_handler` Azure Function package.
    * Builds the `merge-warden` CLI binaries for multiple platforms.
    * Uploads the CLI binaries to the GitHub release.
    * Deploys the `az_handler` package to Azure Functions.

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
