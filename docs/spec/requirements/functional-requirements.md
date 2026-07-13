# Functional Requirements

Detailed functional requirements for Merge Warden, defining the core capabilities, user interactions, and system behaviors expected from the application.

## Overview

This document specifies the functional requirements that define what Merge Warden must do to fulfill its purpose as a pull request validation and management system. These requirements drive the system design, implementation priorities, and acceptance criteria.

## Core Functional Requirements

### FR-001: Pull Request Validation

**Requirement:** The system shall validate pull requests against configurable policies and rules.

**Description:**

- Validate PR titles against conventional commit format
- Check for required work item references in PR descriptions
- Assess PR size and categorize appropriately
- Apply repository-specific validation rules
- Generate validation reports with actionable feedback

**Acceptance Criteria:**

- ✅ PR title validation with clear error messages
- ✅ Work item reference detection and validation
- ✅ PR size calculation and categorization
- ✅ Configuration-driven rule application
- ✅ Comprehensive validation reporting

**Priority:** Critical
**Dependencies:** FR-002 (Configuration Management)

### FR-002: Configuration Management

**Requirement:** The system shall support flexible, repository-specific configuration management.

**Description:**

- Load configuration from `.github/merge-warden.toml` files
- Support centralized configuration through Azure App Configuration
- Validate configuration syntax and semantics
- Provide fallback to default configuration
- Enable runtime configuration updates

**Acceptance Criteria:**

- ✅ TOML configuration file parsing
- ✅ Configuration schema validation
- ✅ Default value application
- ✅ Error handling for invalid configuration
- ✅ Configuration caching and refresh

**Priority:** Critical
**Dependencies:** None

### FR-003: Intelligent Labeling

**Requirement:** The system shall automatically apply appropriate labels to pull requests based on content analysis.

**Description:**

- Analyze commit messages for conventional commit types
- Map commit types to repository-specific labels
- Apply size-based labels (XS, S, M, L, XL, XXL)
- Create fallback labels when repository labels don't exist
- Support custom labeling strategies

**Acceptance Criteria:**

- ✅ Conventional commit type detection
- ✅ Repository label mapping and matching
- ✅ Size-based label application
- ✅ Fallback label creation with appropriate colors
- ✅ Custom labeling rule support

**Priority:** High
**Dependencies:** FR-001 (Pull Request Validation)

### FR-004: Bypass Mechanisms

**Requirement:** The system shall provide secure bypass mechanisms for authorized users and specific scenarios.

**Description:**

- Support user-based bypass rules for specific policies
- Enable emergency bypass capabilities
- Log all bypass activations with full audit trail
- Require appropriate authorization for bypass activation
- Support temporary and permanent bypass configurations

**Acceptance Criteria:**

- ✅ User authorization verification
- ✅ Policy-specific bypass rules
- ✅ Comprehensive audit logging
- ✅ Emergency bypass procedures
- ✅ Bypass configuration management

**Priority:** High
**Dependencies:** FR-005 (Security and Authorization)

### FR-005: Security and Authorization

**Requirement:** The system shall implement robust security measures and authorization controls.

**Description:**

- Authenticate GitHub App installations securely
- Validate webhook signatures and origins
- Implement role-based access control for bypasses
- Protect sensitive configuration and credentials
- Audit all security-relevant operations

**Acceptance Criteria:**

- ✅ GitHub App authentication and token management
- ✅ Webhook signature validation
- ✅ User permission verification
- ✅ Secure credential storage and access
- ✅ Security event logging and monitoring

**Priority:** Critical
**Dependencies:** None

### FR-006: Multi-Platform Deployment

**Requirement:** The system shall support deployment across multiple platforms and environments.

**Description:**

- Deploy as a container for cloud-based webhook processing (Azure Container Apps, AWS ECS, etc.)
- Provide CLI tool for local and CI/CD environment usage
- Support container-based deployments
- Enable cross-platform compatibility
- Maintain feature parity across deployment targets

**Acceptance Criteria:**

- ✅ Container deployment and operation (Azure Container Apps, AWS ECS)
- ✅ CLI tool functionality and distribution
- ✅ Container image creation and deployment
- ✅ Cross-platform binary compilation
- ✅ Feature consistency across platforms

**Priority:** High
**Dependencies:** FR-002 (Configuration Management)

### FR-007: Configuration Change Validation

**Requirement:** The system shall validate `.github/merge-warden.toml` configuration changes included in
a pull request and post an informational comment on that PR reporting whether the new configuration
is valid.

**Description:**

- Detect when a PR's changed files include `.github/merge-warden.toml`
- Fetch the proposed version of the file from the PR head ref
- Parse and validate it against the `RepositoryProvidedConfig` schema
- Post a comment summarising the validation outcome (pass or structured error list)
- Remove or replace a previous config-validation comment on subsequent pushes to the same PR
- Never block merging based on this check — the comment is informational only
- Apply regardless of whether the PR author has bypass permissions for other rules

**Acceptance Criteria:**

- ✅ Changed-file scan detects `.github/merge-warden.toml` in the PR diff
- ✅ Proposed config is fetched from the PR head SHA, not the default branch
- ✅ Valid config produces a single success comment identified by `CONFIG_COMMENT_MARKER`
- ✅ Invalid config produces a single failure comment with a structured error list
- ✅ Comment is updated idempotently (one comment per PR at most)
- ✅ Stale config comment is removed when the PR no longer touches the config file
- ✅ Check conclusion (`success`/`failure`/`neutral`) is unaffected by config validation result

**Priority:** High
**Dependencies:** FR-002 (Configuration Management), FR-001 (Pull Request Validation)

### FR-008: Renovate Stability Label Management

**Requirement:** The system shall reflect the state of the Renovate `stability-days` check
as a label on pull requests, providing at-a-glance visibility of whether the Renovate
stability period has elapsed.

**Description:**

- Listen for GitHub `status` webhook events where `context == "renovate/stability-days"`
- Apply the configured `pending_stability_label` (default: `pr-validation: pending-stability`)
  when the status is `pending`, `error`, or `failure`
- Remove the label when the status is `success`
- Re-evaluate the label on every `pull_request` event by inspecting the current HEAD commit
  statuses, ensuring the label is always accurate regardless of event ordering
- When the `renovate/stability-days` context is absent from the HEAD commit's statuses,
  take no action
- Auto-create the label in the repository if it does not exist (color `#986ee2`,
  description "PR is waiting for Renovate stability period")
- Never block merging - this feature is observability-only and must not contribute to
  the commit-status check conclusion

**Acceptance Criteria:**

- ✅ `status` event with `context == "renovate/stability-days"` and state `pending`/`error`/`failure` applies `pending_stability_label`
- ✅ `status` event with `context == "renovate/stability-days"` and state `success` removes `pending_stability_label`
- ✅ `status` event with any other `context` is a no-op
- ✅ `pull_request` event re-evaluates the label against the current HEAD commit statuses
- ✅ Label operations are idempotent (adding a present label and removing an absent label are both safe)
- ✅ Check conclusion (`success`/`failure`/`neutral`) is unaffected by this feature
- ✅ Feature is disabled entirely when `enabled = false` in configuration

**Priority:** Medium
**Dependencies:** FR-001 (Pull Request Validation), FR-002 (Configuration Management)

### FR-009: Repository Scope Filtering

**Requirement:** The system shall allow operators to restrict which repositories it actively
processes, independent of which repositories the GitHub App installation can technically
access.

**Description:**

- Support organisations large enough that GitHub only offers "All repositories" as an
  install-scope option, forcing the GitHub App installation to receive webhooks for
  repositories it was never intended to act on
- Allow operators to declare an allow-list of repository name glob patterns
  (`include_patterns`) and an optional deny-list (`exclude_patterns`) in application-level
  configuration
- Evaluate the scope check as the first step of event processing — before the `event_type`
  dispatch branches and before any repository-specific data (config file, org policy,
  topics, custom properties) is fetched
- Default to processing every repository when no scope configuration is present, preserving
  full backward compatibility with deployments that predate this feature

**Acceptance Criteria:**

- ✅ Absent `[repository_scope]` configuration processes events for every repository (no filtering)
- ✅ `[repository_scope]` present with an empty `include_patterns` list processes no repositories (fail-closed "pause everything" lever), regardless of `exclude_patterns`
- ✅ Glob wildcards `*` (any sequence) and `?` (single character) are supported in both `include_patterns` and `exclude_patterns`
- ✅ A repository name matching `exclude_patterns` is excluded even when it also matches `include_patterns` (exclude takes precedence over include)
- ✅ Repository name matching is case-insensitive
- ✅ A webhook payload with a missing or malformed `repository.name` field is treated as out of scope (fail-closed)
- ✅ Filtered events (out of scope or unparseable payload) are acknowledged (`ack.complete()`) without further processing and without returning an error
- ✅ No `.github/merge-warden.toml`, org-policy, topics, or custom-properties fetch occurs for an out-of-scope repository
- ✅ No GitHub API call is made on behalf of a filtered repository (the scope check runs before `installation_by_id`)

**Priority:** High
**Dependencies:** FR-002 (Configuration Management), FR-001 (Pull Request Validation)

## Detailed Functional Specifications

### Pull Request Title Validation

**Conventional Commit Format Support:**

- `feat: add new feature` (Features)
- `fix: resolve bug in validation` (Bug fixes)
- `docs: update API documentation` (Documentation)
- `style: improve code formatting` (Code style)
- `refactor: restructure validation logic` (Refactoring)
- `perf: optimize webhook processing` (Performance)
- `test: add integration tests` (Testing)
- `chore: update dependencies` (Maintenance)
- `ci: improve build pipeline` (CI/CD)
- `build: update compilation settings` (Build system)
- `revert: undo previous changes` (Reverts)

**Validation Rules:**

- Type must be from approved list
- Scope is optional but must be alphanumeric if present
- Description must be present and descriptive
- Breaking changes must be indicated with `!` or `BREAKING CHANGE:`
- Maximum title length enforcement (configurable)

### Work Item Reference Detection

**Supported Patterns:**

- GitHub Issues: `#123`, `fixes #123`, `closes #123`
- Azure DevOps: `AB#123`, `fixes AB#123`
- Jira: `PROJ-123`, `fixes PROJ-123`
- Custom patterns via configuration

**Validation Logic:**

- Pattern matching against PR description
- Multiple work item reference support
- Required vs. optional reference configuration
- Link validation for supported platforms

### Size-Based Categorization

**Size Calculation:**

- Line additions and deletions count
- File-based exclusion patterns
- Binary file handling
- Generated code exclusion

**Size Categories:**

```
XS:  1-10 lines
S:   11-100 lines
M:   101-300 lines
L:   301-1000 lines
XL:  1001-3000 lines
XXL: 3000+ lines
```

**Size-Based Actions:**

- Automatic label application
- Educational comments for large PRs
- Optional validation failure for oversized PRs
- Size trend tracking and reporting

### Label Management

**Label Discovery:**

- Repository label enumeration
- Fuzzy matching for similar labels
- Case-insensitive matching
- Description-based matching

**Label Creation:**

- Fallback label generation
- Consistent color schemes
- Repository-specific customization
- Batch label creation capabilities

### Configuration Schema

**Repository Configuration (`.github/merge-warden.toml`):**

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
format = "conventional-commits"
maxLength = 72

[policies.pullRequests.workItem]
required = true
pattern = "#\\d+"
platforms = ["github", "azure-devops"]

[policies.pullRequests.prSize]
enabled = true
fail_on_oversized = false
excluded_file_patterns = ["*.md", "docs/**"]

[policies.pullRequests.prSize.thresholds]
xs = 10
s = 100
m = 300
l = 1000
xl = 3000
xxl = 10000

[change_type_labels]
enabled = true

[change_type_labels.conventional_commit_mappings]
feat = ["enhancement", "feature"]
fix = ["bug", "bugfix"]
docs = ["documentation"]
```

## Integration Requirements

### GitHub Integration

**Required Capabilities:**

- GitHub App installation and configuration
- Webhook event processing (pull_request, pull_request_review)
- GitHub API interactions (comments, labels, status checks)
- Repository and organization permission management
- Rate limiting and error handling

**GitHub App Permissions:**

Repository permissions:

- **Pull requests**: Read/Write (for status checks and comments)
- **Issues**: Read (for work item validation)
- **Metadata**: Read (for repository information)
- **Contents**: Read (for configuration file access)
- **Commit statuses**: Read (required for `get_commit_statuses`; needed by FR-008)

Webhook event subscriptions:

- **pull_request** (required for PR validation — FR-001 through FR-007)
- **status** (commit statuses; required for Renovate stability label management — FR-008)

Organisation permissions:

- **Projects**: Read & Write (required for Projects v2 propagation — reading which projects an issue belongs to, and adding the pull request to those projects; this must be the *organisation-level* Projects permission, not the repository-level one)

> **Limitation:** `sync_project_from_issue` only works for repositories owned by a GitHub organisation. Personal (user-owned) repositories are not supported because the GitHub API does not allow GitHub App tokens to write to user-scoped Projects v2.

### Azure Services Integration

**Azure Container Apps:**

- HTTP ingress for webhook processing
- Health-check endpoint (`GET /health`) for liveness/readiness probes
- Request-driven autoscaling
- OTLP telemetry export to Application Insights via the Azure Monitor OpenTelemetry Distro collector

**Azure App Configuration:**

- Centralized configuration management
- Environment-specific settings
- Configuration versioning and audit
- Real-time configuration updates

**Azure Key Vault:**

- Secure credential storage
- GitHub App private key management
- API key and secret management
- Managed identity integration

## Performance Requirements

### Response Time Requirements

**Webhook Processing:**

- Initial response: < 2 seconds
- Complete processing: < 10 seconds
- Large PR processing (XXL): < 30 seconds

**Configuration Loading:**

- Cache hit: < 100ms
- Cache miss: < 2 seconds
- Fallback configuration: < 500ms

**GitHub API Operations:**

- Comment creation: < 3 seconds
- Label application: < 2 seconds
- Status check updates: < 1 second

### Throughput Requirements

**Webhook Processing:**

- Support 100 concurrent webhook requests
- Process 1000 PRs per hour per deployment
- Handle burst traffic up to 5x normal load

**CLI Operations:**

- Process local repository in < 5 seconds
- Support batch processing of multiple repositories
- Maintain performance with large monorepos

### Scalability Requirements

**Azure Functions:**

- Auto-scale based on queue depth and request rate
- Support multiple deployment regions
- Handle traffic spikes during peak development hours

**Configuration Management:**

- Support 1000+ repositories per deployment
- Cache configuration for 10+ minutes
- Handle configuration updates without service interruption

## Error Handling Requirements

### Error Recovery

**Transient Failures:**

- Automatic retry with exponential backoff
- Circuit breaker pattern for external services
- Graceful degradation when dependencies unavailable

**Permanent Failures:**

- Clear error messages with actionable guidance
- Fallback to default behavior when possible
- Comprehensive error logging for debugging

### User Experience

**Error Communication:**

- User-friendly error messages in PR comments
- Technical details in application logs
- Links to documentation for common issues
- Escalation paths for unresolved problems

## Monitoring and Observability Requirements

### Metrics Collection

**Business Metrics:**

- PR processing rates and success rates
- Policy compliance percentages
- Feature adoption and usage patterns
- User engagement and satisfaction

**Technical Metrics:**

- Response times and throughput
- Error rates and types
- Resource utilization and costs
- Dependency health and availability

### Alerting Requirements

**Critical Alerts:**

- Service unavailability or high error rates
- Security incidents or unauthorized access
- Data loss or corruption events
- Critical dependency failures

**Performance Alerts:**

- Response time degradation
- Throughput reduction
- Resource exhaustion warnings
- Capacity planning triggers

## Data Requirements

### Data Storage

**Configuration Data:**

- Repository-specific settings
- User preferences and permissions
- Audit logs and event history
- Performance metrics and analytics

**Data Retention:**

- Configuration: Indefinite (until manually deleted)
- Audit logs: 7 years (compliance requirement)
- Performance metrics: 1 year (operational analysis)
- Error logs: 90 days (troubleshooting)

### Data Protection

**Privacy Requirements:**

- No storage of sensitive code content
- Minimal user data collection
- GDPR compliance for EU users
- Secure data transmission and storage

**Backup and Recovery:**

- Configuration backup and restoration
- Audit log preservation
- Point-in-time recovery capabilities
- Disaster recovery procedures

## Related Documents

- **[Platform Requirements](./platform-requirements.md)**: Technical platform specifications
- **[Performance Requirements](./performance-requirements.md)**: Detailed performance criteria
- **[Compliance Requirements](./compliance-requirements.md)**: Legal and regulatory requirements
- **[Architecture Overview](../architecture/README.md)**: System architecture supporting these requirements
