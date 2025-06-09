# merge_warden

This repository contains the Merge Warden project.

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
