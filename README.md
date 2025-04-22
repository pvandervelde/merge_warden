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
