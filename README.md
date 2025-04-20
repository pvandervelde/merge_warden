# merge_warden

This repository contains the Merge Warden project.

## Release Process

This project uses [release-please](https://github.com/googleapis/release-please) to automate
releases based on [Conventional Commits](https://www.conventionalcommits.org/).

1. **Development:** Changes are made on feature branches and merged into the `master` branch.
    Commit messages should follow the Conventional Commits specification.
2. **Release PR:** When commits that warrant a release (e.g., `feat:`, `fix:`) are pushed to
    `master`, the `release-please.yml` workflow runs. It automatically creates or updates a
    "Release PR". This PR contains the updated version in `Cargo.toml` and an updated
    `CHANGELOG.md`.
3. **Merging:** Review and merge the Release PR.
4. **Release Creation:** Merging the Release PR triggers `release-please` (via the same workflow)
    to create a GitHub tag (e.g., `v0.2.0`) and a corresponding GitHub release.
5. **Deployment & Binary Upload:** The creation of the GitHub release triggers the `deploy.yml`
    workflow. This workflow:
    * Builds the `az_handler` Azure Function package.
    * Builds the `merge-warden` CLI binaries for multiple platforms.
    * Uploads the CLI binaries to the GitHub release.
    * Deploys the `az_handler` package to Azure Functions.
