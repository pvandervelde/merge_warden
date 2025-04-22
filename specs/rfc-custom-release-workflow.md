# RFC: Custom Release Workflow

**Status:** Proposed

## Problem Description

The existing `release-please` workflow does not fully meet the project's requirements for updating
specific files (like Cargo workspace versions) and managing changelogs in the desired manner. A custom
workflow is needed to provide more control over the release process.

## Surrounding Context

The project is a Rust workspace using Cargo, with multiple crates under the `crates/` directory. It
utilizes conventional commits for tracking changes and aims for automated semantic versioning and
changelog generation. Previous attempts involved `release-plz`/`GitVersion` and then `release-please`.
This RFC proposes replacing `release-please`.

## Proposed Solution

Implement a custom release process using GitHub Actions, `knope` for version calculation and file
updates, and `git-cliff` for changelog generation. The process involves two main workflows: one to
prepare a release pull request and another to publish the release upon merging that PR.

### Design Goals

* Automate version bumping based on conventional commits.
* Automate `CHANGELOG.md` generation.
* Centralize version definition in the root `Cargo.toml`.
* Maintain a clear release PR for review before tagging.
* Automate Git tagging and GitHub Release creation.
* Prevent accidental release cycles triggered by merging release PRs.
* Clean up stale release branches automatically.

### Design Constraints

* Must work within GitHub Actions environment.
* Relies on consistent use of conventional commits.
* Requires tools (`knope`, `git-cliff`) to be installable in the workflow runner.

### Design Decisions

* Use `knope` for version calculation and `Cargo.toml` update due to its focus on Rust projects
    and conventional commits.
* Use `git-cliff` for changelog generation as specified.
* Use `version.workspace = true` in crate `Cargo.toml` files for centralized versioning.
* Generate a single root `CHANGELOG.md`.
* Use a dedicated `release/vX.Y.Z` branch pattern for release PRs.
* Automatically close PRs and delete branches for stale (superseded) release versions.

### Alternatives Considered

* **Modifying `release-please`:** Deemed insufficient for required file modifications.
* **Custom Scripting:** Considered more complex to maintain than using dedicated tools like `knope`.
* **Re-evaluating `GitVersion`:** Rejected by the user.
* **`cargo-release`:** Focused on Rust releases, might require more scripting for the PR workflow.
* **`semantic-release` (Node.js):** Popular but requires Node.js environment and likely plugins/scripts
    for Rust integration. `knope` appears more integrated for this specific use case.

## Design

### 1. Issue Creation

* A GitHub issue will be created to track the implementation. (`#rule: wf-issue-use #rule: wf-issue-creation`)

### 2. Initial Setup Task

* **Goal:** Centralize version management.
* **Action:** Modify all `crates/*/Cargo.toml` files. Change `version = "..."` to `version.workspace = true`.

### 3. Tooling Configuration

* **`knope`:** Configure via `knope.toml` for conventional commits, version calculation, and root
    `Cargo.toml` update.
* **`git-cliff`:** Configure via `cliff.toml` for `CHANGELOG.md` generation.

### 4. Workflow 1: Release PR Creation/Update (`.github/workflows/prepare-release.yml`)

* **Trigger:** Push to `master` branch.
* **Goal:** Automate creation/update of a release pull request.
* **Steps:**
    1. **Check Trigger:** Skip execution if the triggering commit message matches `chore(release): v*`.
    2. Checkout code.
    3. Set up Rust/Cargo, `knope`, `git-cliff`.
    4. Calculate `NEXT_VERSION` (knope).
    5. **Handle Stale Releases:** Find all existing branches matching `release/v*`. For any branch
        whose version does *not* match `NEXT_VERSION`, close its associated PR (if any) and delete
        the branch.
    6. Check if `release/v${NEXT_VERSION}` branch exists.
    7. Generate changelog section for `NEXT_VERSION` (git-cliff).
    8. Prepend changelog section to `CHANGELOG.md`.
    9. Update root `Cargo.toml` version to `NEXT_VERSION` (knope/script).
    10. Commit `Cargo.toml` and `CHANGELOG.md`.
    11. Create/Update `release/v${NEXT_VERSION}` branch and PR targeting `master`.
        * PR Title: `chore(release): v${NEXT_VERSION}` (`#rule: scm-git-pull-request-title`)
        * Use PR template if available (`#rule: scm-git-pull-request-template`).
* **Diagram:**

    ```mermaid
    graph TD
        A[Push to master] --> B{"Check commit msg != 'chore(release): v*'"}
        B -- No --> X[Skip Workflow];
        B -- Yes --> C{Checkout & Setup};
        C --> D{"Calculate NEXT_VERSION (knope)"};
        D --> E{"Find/Handle Stale release/v* branches (close PRs, delete branches)"};
        E --> F{"Branch release/vNEXT_VERSION exists?"};
        F -- Yes --> G["Generate Changelog (git-cliff)"];
        F -- No --> G;
        G --> H{"Update Cargo.toml & CHANGELOG.md"};
        H --> I{Commit Changes};
        I --> J{"Push to release/vNEXT_VERSION (force?)"};
        J --> K{"Create PR (if new)"};
    ```

### 5. Workflow 2: Tagging and GitHub Release (`.github/workflows/publish-release.yml`)

* **Trigger:** Pull Request closed (merged) where head branch matches `release/v*`.
* **Goal:** Create Git tag and GitHub Release.
* **Steps:**
    1. Checkout `master` branch post-merge.
    2. Set up Rust/Cargo.
    3. Verify merged PR branch was `release/v*`.
    4. Get final `VERSION` from root `Cargo.toml`.
    5. Create annotated tag: `git tag -a v${VERSION} -m "Release v${VERSION}"`.
    6. Push tag: `git push origin v${VERSION}`.
    7. Extract release notes for `v${VERSION}` from `CHANGELOG.md`.
    8. Create GitHub Release using tag `v${VERSION}` and extracted notes (`#rule: wf-release-notes`).
* **Diagram:**

    ```mermaid
    graph TD
        A["Release PR Merged to master"] --> B{"Checkout master & Setup"};
        B --> C{"Verify PR branch was release/v*"};
        C --> D{"Get VERSION from Cargo.toml"};
        D --> E{"Create Git Tag vVERSION"};
        E --> F{Push Git Tag};
        F --> G{"Extract Notes from CHANGELOG.md"};
        G --> H{Create GitHub Release};
    ```

### 6. Cleanup

* Remove old `.github/workflows/release-please.yml`, `release-please-config.json`,
    `.release-please-manifest.json`.
* Update `README.md` release process section.

## Conclusion

This custom workflow provides the necessary control over versioning and changelog generation for the
Rust workspace, integrating `knope` and `git-cliff` within GitHub Actions, while handling potential
edge cases like release cycles and stale release branches.
