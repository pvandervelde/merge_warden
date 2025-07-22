# Release Management

Comprehensive release management strategy for Merge Warden, covering version control, automated release workflows, and deployment coordination across all targets.

## Overview

This document defines the release management processes, automation workflows, and coordination strategies for delivering Merge Warden updates across Azure Functions, CLI distributions, and future deployment targets. It encompasses version control, changelog generation, release automation, and quality assurance procedures.

## Release Strategy

### Semantic Versioning

Merge Warden follows [Semantic Versioning 2.0.0](https://semver.org/) with the following interpretation:

**Version Format:** `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes to public APIs or configuration schema
- **MINOR**: New features and functionality (backward compatible)
- **PATCH**: Bug fixes and minor improvements (backward compatible)

**Pre-release Versions:**

- **Alpha**: `1.2.3-alpha.1` - Early development builds
- **Beta**: `1.2.3-beta.1` - Feature-complete pre-releases
- **Release Candidate**: `1.2.3-rc.1` - Final testing before release

### Release Cadence

**Regular Releases:**

- **Major releases**: Quarterly (as needed for breaking changes)
- **Minor releases**: Monthly (new features and enhancements)
- **Patch releases**: Bi-weekly or as needed (bug fixes)

**Emergency Releases:**

- **Hotfixes**: As needed for critical security or stability issues
- **Security patches**: Immediate for security vulnerabilities

## Version Management

### Centralized Versioning

**Workspace Configuration:**

```toml
# Root Cargo.toml
[workspace]
members = ["crates/*"]

[workspace.package]
version = "1.2.3"
edition = "2021"

[workspace.dependencies]
# Shared dependencies
```

**Crate Configuration:**

```toml
# crates/*/Cargo.toml
[package]
name = "merge-warden-core"
version.workspace = true
edition.workspace = true
```

### Version Calculation

**Conventional Commits Integration:**

- `feat:` commits trigger MINOR version bump
- `fix:` commits trigger PATCH version bump
- `BREAKING CHANGE:` footer triggers MAJOR version bump
- Other commit types (docs, style, etc.) don't affect version

**Tools:**

- **knope**: Version calculation based on conventional commits
- **git-cliff**: Changelog generation from commit history
- **Custom workflows**: GitHub Actions orchestration

## Automated Release Workflows

### Release Preparation Workflow

**Trigger:** Push to `master` branch (excluding release commits)

**Process:**

```mermaid
graph TD
    A[Push to master] --> B{Check commit message}
    B -->|Not release commit| C[Setup tools]
    B -->|Release commit| X[Skip workflow]
    C --> D[Calculate next version]
    D --> E[Clean up stale release branches]
    E --> F[Generate changelog section]
    F --> G[Update Cargo.toml version]
    G --> H[Commit changes]
    H --> I[Create/update release branch]
    I --> J[Create/update release PR]
```

**Implementation:**

```yaml
name: Prepare Release
on:
  push:
    branches: [master]

jobs:
  prepare-release:
    if: "!startsWith(github.event.head_commit.message, 'chore(release):')"
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install tools
        run: |
          cargo install knope
          cargo install git-cliff

      - name: Calculate next version
        id: version
        run: |
          NEXT_VERSION=$(knope --dry-run | grep "Next version:" | cut -d' ' -f3)
          echo "next_version=$NEXT_VERSION" >> $GITHUB_OUTPUT

      - name: Clean stale release branches
        run: ./scripts/cleanup-stale-releases.sh ${{ steps.version.outputs.next_version }}

      - name: Generate changelog
        run: |
          git-cliff --tag v${{ steps.version.outputs.next_version }} --prepend CHANGELOG.md

      - name: Update version
        run: |
          knope prepare-release --version ${{ steps.version.outputs.next_version }}

      - name: Create release PR
        uses: peter-evans/create-pull-request@v5
        with:
          branch: release/v${{ steps.version.outputs.next_version }}
          title: "chore(release): v${{ steps.version.outputs.next_version }}"
          body-path: .github/pull_request_template.md
```

### Release Publication Workflow

**Trigger:** Release PR merged to `master`

**Process:**

```mermaid
graph TD
    A[Release PR merged] --> B[Checkout master]
    B --> C[Extract version from Cargo.toml]
    C --> D[Create Git tag]
    D --> E[Push tag to origin]
    E --> F[Extract release notes]
    F --> G[Create GitHub release]
    G --> H[Trigger deployment workflows]
```

**Implementation:**

```yaml
name: Publish Release
on:
  pull_request:
    types: [closed]
    branches: [master]

jobs:
  publish-release:
    if: github.event.pull_request.merged == true && startsWith(github.head_ref, 'release/v')
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Extract version
        id: version
        run: |
          VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
          echo "version=$VERSION" >> $GITHUB_OUTPUT

      - name: Create and push tag
        run: |
          git tag -a v${{ steps.version.outputs.version }} -m "Release v${{ steps.version.outputs.version }}"
          git push origin v${{ steps.version.outputs.version }}

      - name: Extract release notes
        id: notes
        run: |
          ./scripts/extract-release-notes.sh ${{ steps.version.outputs.version }} > release-notes.md

      - name: Create GitHub release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: v${{ steps.version.outputs.version }}
          name: Release v${{ steps.version.outputs.version }}
          body_path: release-notes.md
          draft: false
          prerelease: false
```

## Release Coordination

### Multi-Target Deployment

**Deployment Sequence:**

1. **Tag Creation**: Automated via release workflow
2. **Azure Functions**: Triggered by tag push
3. **CLI Binaries**: Built and published to GitHub Releases
4. **Container Images**: Updated and pushed to registry
5. **Documentation**: Updated and deployed

**Coordination Workflow:**

```yaml
name: Coordinated Deployment
on:
  push:
    tags: ['v*']

jobs:
  deploy-azure-functions:
    uses: ./.github/workflows/deploy-azure-functions.yml
    with:
      version: ${{ github.ref_name }}

  build-cli-binaries:
    uses: ./.github/workflows/build-cli.yml
    with:
      version: ${{ github.ref_name }}

  update-documentation:
    uses: ./.github/workflows/update-docs.yml
    with:
      version: ${{ github.ref_name }}
```

### Release Validation

**Pre-release Checks:**

- Automated test suite execution
- Security vulnerability scanning
- Performance regression testing
- Documentation validation

**Post-release Validation:**

- Deployment health checks
- Integration testing with real repositories
- Performance monitoring
- User acceptance validation

## Changelog Management

### Automated Generation

**Configuration (cliff.toml):**

```toml
[changelog]
header = """
# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
"""

body = """
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | striptags | trim | upper_first }}
    {% for commit in commits %}
        - {{ commit.message | upper_first }}
    {% endfor %}
{% endfor %}
"""

[git]
conventional_commits = true
filter_unconventional = true
split_commits = false
commit_preprocessors = [
    { pattern = '\((\w+\s)?#([0-9]+)\)', replace = "([#${2}](https://github.com/pvandervelde/merge_warden/issues/${2}))"}
]
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^doc", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactoring" },
    { message = "^style", group = "Styling" },
    { message = "^test", group = "Testing" },
    { message = "^chore\\(release\\):", skip = true },
    { message = "^chore", group = "Miscellaneous Tasks" },
    { body = ".*security", group = "Security" },
]
```

### Release Notes Structure

**Standard Format:**

```markdown
## [1.2.3] - 2025-07-22

### Features
- Add support for custom validation rules
- Implement repository-specific configuration overrides

### Bug Fixes
- Fix issue with large pull request processing
- Resolve configuration loading timeout

### Documentation
- Update deployment guide
- Add troubleshooting section

### Performance
- Optimize webhook processing pipeline
- Reduce memory usage in CLI operations
```

## Hotfix Management

### Emergency Release Process

**Hotfix Workflow:**

1. **Create hotfix branch** from latest release tag
2. **Apply minimal fix** with thorough testing
3. **Create hotfix PR** with expedited review
4. **Emergency release** with patch version bump
5. **Backport to master** if applicable

**Hotfix Branch Strategy:**

```bash
# Create hotfix branch from release tag
git checkout v1.2.3
git checkout -b hotfix/v1.2.4

# Apply fix and test
git commit -m "fix: critical security vulnerability"

# Create emergency release
git tag v1.2.4
git push origin v1.2.4

# Backport to master
git checkout master
git cherry-pick <hotfix-commit>
```

### Security Patches

**Security Release Process:**

1. **Private disclosure** and assessment
2. **Coordinated fix development**
3. **Security advisory preparation**
4. **Expedited release and disclosure**
5. **User notification and guidance**

## Release Quality Assurance

### Testing Strategy

**Pre-release Testing:**

- Unit test execution (100% pass rate required)
- Integration test validation
- End-to-end scenario testing
- Performance regression testing
- Security vulnerability scanning

**Release Candidate Process:**

- Release candidate deployment to staging
- Extended testing period (48-72 hours)
- Community feedback collection
- Performance monitoring and analysis

### Release Criteria

**Release Gates:**

- All automated tests passing
- Security scan clean results
- Performance benchmarks met
- Documentation updated
- Breaking changes documented

**Quality Metrics:**

- Test coverage ≥ 90%
- No critical or high severity security issues
- Performance within 5% of previous release
- Documentation completeness score ≥ 95%

## Rollback Procedures

### Release Rollback

**Automated Rollback Triggers:**

- Critical bug discovery within 24 hours
- Security vulnerability exploitation
- Performance degradation > 20%
- High error rates (> 5%)

**Rollback Process:**

1. **Immediate action**: Revert to previous stable version
2. **Communication**: Notify users and stakeholders
3. **Investigation**: Root cause analysis
4. **Resolution**: Fix development and testing
5. **Re-release**: Updated version with fixes

### Version Management During Rollback

**Tag Management:**

- Rolled-back versions marked as deprecated
- Clear communication about version status
- Updated release notes with rollback information

## Release Communication

### Stakeholder Notification

**Release Announcements:**

- GitHub release notes
- Repository README updates
- Documentation site updates
- Email notifications (for major releases)

**Breaking Change Communication:**

- Migration guides for breaking changes
- Deprecation notices with timelines
- Compatibility matrices
- Support for previous versions

### Community Engagement

**Pre-release Communication:**

- Beta release announcements
- Feature previews and demos
- Community feedback collection
- Documentation updates

**Post-release Follow-up:**

- Usage analytics review
- Community feedback analysis
- Issue tracking and resolution
- Performance monitoring

## Related Documents

- **[Deployment](./deployment.md)**: Deployment procedures and coordination
- **[Configuration Management](./configuration-management.md)**: Configuration versioning
- **[Monitoring](./monitoring.md)**: Release monitoring and health checks
- **[Testing](../testing/README.md)**: Release testing strategies
