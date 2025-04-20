# RFC: Migrate from release-plz to release-please

## 1. Problem Description

The current release process uses `release-plz`. While functional, it has limitations, notably its
expectation of publishing to crates.io, which is not desired for this project. The goal is to
replace `release-plz` and `GitVersion` with `release-please` to manage versioning, changelog
generation, release pull requests, and GitHub release creation (including uploading pre-built
binaries) without interacting with crates.io, using the release PR strategy.

## 2. Surrounding Context

* **Project Structure:** Rust workspace (monorepo).
* **Main Artifact:** `az_handler` (Azure Function) in `crates/azure-functions`.
* **Dependencies:** `az_handler` aggregates changes from `merge_warden_core`,
    `merge_warden_developer_platforms`, and `merge_warden_cli`.
* **Commits:** Conventional Commits standard is used.
* **Current Version:** `0.1.0` (defined in root `Cargo.toml`).
* **Binaries:** `merge-warden` CLI built for multiple targets, uploaded to GitHub releases.
* **Deployment:** `az_handler` deployed to Azure Functions.
* **Current Automation:**
  * `.github/workflows/release-plz.yml`: Runs `release-plz release-pr` and `release-plz release`
        on `master` push. Uses GitHub App token.
  * `.github/workflows/deploy.yml`: Triggered on release `published`. Builds artifacts, uploads
        CLI binaries, deploys Azure function. Uses GitVersion.
  * `.github/workflows/ci.yml`: Runs tests, builds `az_handler`. Uses GitVersion.

## 3. Proposed Solution

Replace `release-plz` and `GitVersion` with `release-please` and modify GitHub Actions workflows
to adopt the `release-please` release PR strategy.

### Design Goals

* Automate version bumping based on Conventional Commits.
* Automate `CHANGELOG.md` updates.
* Automate the creation/update of a "Release PR".
* Upon merging the Release PR, automatically create a GitHub tag and release.
* Automatically build and upload `merge-warden` CLI binaries to the GitHub release.
* Continue deploying `az_handler` to Azure upon release.
* Do *not* publish any crates to crates.io.
* Remove dependency on `release-plz` and `GitVersion`.

### Design Constraints

* Must work with the existing Rust workspace structure.
* Must use the existing GitHub App token for permissions if possible.
* Must upload the specified `merge-warden` binaries.

### Design Decisions

* Use `google-github-actions/release-please-action`.
* Configure `release-please` for a `rust` project type, managing the workspace version in the
    root `Cargo.toml`.
* Modify `deploy.yml` to trigger on tag push (created by `release-please`) instead of release
    publish.
* Reuse existing binary build/upload steps from `deploy.yml`.
* Remove `GitVersion` usage from all workflows.

### Alternatives Considered

* **Manual Releases:** Too much overhead, error-prone.
* **Keeping `release-plz`:** Does not fully meet the desired workflow (avoiding crates.io
    expectation, release PR strategy preference).
* **Other tools (e.g., `cargo-release`):** `release-please` is well-suited for the Conventional
    Commit -> Release PR -> GitHub Release flow.

## 4. Design

### Architecture

The core change involves replacing the release orchestration tool (`release-plz` + `GitVersion`)
with `release-please`. Workflows will be adjusted to fit `release-please`'s event model
(PR creation -> merge -> tag push -> build/deploy).

### Data Flow / Workflow Diagram

```mermaid
graph TD
    A[Push to master] --> B(release-please.yml: Run release-please-action);
    B -- Changes Detected --> C[Create/Update Release PR<br>(Updates Cargo.toml, CHANGELOG.md)];
    C --> D{Review & Merge Release PR};
    D -- Tag Push by release-please (e.g., v0.2.0) --> E[deploy.yml: Trigger on Tag Push];
    subgraph deploy.yml
        E --> F[Build az_handler];
        E --> G[Build merge-warden CLI (multi-target)];
        F --> H[Deploy az_handler to Azure];
        G --> I[Upload merge-warden binaries to GitHub Release];
    end
    subgraph ci.yml (No GitVersion)
        J[Push/PR] --> K[Run Tests, Build Checks];
    end
```

### Module Breakdown (Files to Change/Create)

1. **Create `release-please-config.json`:** Configure `release-please`.
2. **Create `.release-please-manifest.json`:** Initialize manifest.
3. **Create `.github/workflows/release-please.yml`:** New workflow for `release-please-action`.
4. **Modify `.github/workflows/deploy.yml`:** Change trigger, remove GitVersion.
5. **Modify `.github/workflows/ci.yml`:** Remove GitVersion.
6. **Delete `release-plz.toml`:** Remove old config.
7. **Delete `.github/workflows/release-plz.yml`:** Remove old workflow.
8. **Modify `README.md` (or similar):** Update release process documentation.
9. **Update Memory Bank:** Log decision and progress.

## 5. Conclusion

This RFC outlines the plan to migrate from `release-plz` and `GitVersion` to `release-please`,
adopting a release PR strategy focused on GitHub Releases and avoiding crates.io interaction.
