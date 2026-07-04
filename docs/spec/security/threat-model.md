# Threat Model

This document describes the security threats and required mitigations for the Merge
Warden system.

## Org Policy Repository Trust

### Threat

The Merge Warden server fetches the org policy TOML from a GitHub repository whose
coordinates (`owner`, `repo`, `path`) are specified in the operator-controlled
application configuration file (`MERGE_WARDEN_CONFIG_FILE`). The server trusts the
content of that file unconditionally — there is no commit-signature verification and
no secondary approval gate beyond what the repository's own branch protection provides.

Any actor who can commit to (or merge into) the designated org policy repository's
default branch can:

1. Add arbitrary GitHub logins to `[enforced.policies.bypassRules.*]`, granting those
   accounts the ability to bypass PR checks across **every repository** watched by the
   Merge Warden installation.
2. Override PR policy enforcement settings (title format, work-item requirements, size
   limits) org-wide, without each individual repository's maintainers being able to
   prevent it.

The blast radius is the entire GitHub organisation, not a single repository.

### Severity

**High** — write access to one repository grants org-wide policy control.

### Required Mitigations (Operator Responsibilities)

The following controls MUST be in place on the org policy repository before enabling
org-level policies in production:

1. **Branch protection:** Require pull request reviews (minimum 2 approvers recommended)
   on the default branch. Prohibit force-push and branch deletion.
2. **Restricted write access:** Limit write (push/merge) permissions to the org policy
   repository to a small, named set of platform engineers. Use a GitHub team with `write`
   access rather than individual grants.
3. **Audit logging:** Enable GitHub's organisation audit log and alert on pushes to the
   org policy repository's default branch.
4. **Change monitoring:** Treat changes to the org policy TOML like changes to
   infrastructure-as-code — subject to the same review and change-management process as
   other privileged configuration.

### Future Code Controls (Planned)

- Commit-signature verification: optionally reject org policy files not signed by a
  configured GPG key.
- Structured bypass-change audit events: emit a structured log entry summarising any
  bypass rule changes detected between successive fetches of the org policy file.

### Related

- [`OrgPolicySource`](../interfaces/org-policy.md) — how the org policy location is configured
- [Bypass rules](../interfaces/policy-engine.md) — merge semantics and bypass evaluation

## All-Repositories Installation Scope

### Threat

In large organisations, GitHub enforces an install-scope limit that prevents an operator from
selecting individual repositories when installing the Merge Warden GitHub App — past that size
threshold, "All repositories" is the only option offered. This means the installation access
token Merge Warden obtains via `installation_by_id` has GitHub API access to **every**
repository in the organisation, regardless of the `repository_scope` configuration described in
[configuration-system.md](../design/configuration-system.md#repository-scope-filtering).

`repository_scope` is a **service-level intent filter**, not a GitHub-permission-level security
boundary. It controls which repositories Merge Warden *chooses* to act on; it does not revoke
or narrow the installation token's actual API access. An operator cannot use `repository_scope`
to restrict what the installation is technically capable of reaching — only GitHub's own
installation-repository selection (where available) or splitting into multiple app
installations can do that.

A misconfigured or accidentally empty `include_patterns` list is the primary operational risk:
if the pattern list is wrong (e.g., a typo that matches no repositories, or an operator
forgetting to add a newly onboarded repository), Merge Warden will silently stop processing PRs
for repositories the operator expected it to cover.

### Severity

**Low-to-Medium** — the check is a processing-scope filter, not an access-control boundary; a
misconfiguration causes missed enforcement (fail-safe from a security perspective, since
`include_patterns = []` fails closed) rather than unauthorised access.

### Required Mitigations

1. **Fail-closed on empty include list:** an explicitly empty `include_patterns` list processes
   zero repositories rather than silently defaulting to unrestricted processing. See
   [FR-009 acceptance criteria](../requirements/functional-requirements.md#fr-009-repository-scope-filtering).
2. **Startup pattern validation:** every pattern in `include_patterns` and `exclude_patterns` is
   compiled and validated by `validate_repository_scope_patterns` at `load_config()` time; an
   invalid pattern fails the process at startup rather than silently matching nothing (or
   everything) at webhook-handling time.
3. **Operator review of onboarding:** treat `repository_scope` changes like any other
   application-level configuration change — review pattern additions and removals whenever
   repositories are onboarded to or decommissioned from the organisation.

### No New Pre-Auth Attack Surface

The repository-scope check runs after HMAC-SHA256 signature verification (performed by the
`github-bot-sdk` `WebhookReceiver` in webhook mode, or by the separate receiver service in queue
mode — see [event-processing.md](../architecture/event-processing.md)). An unauthenticated
caller cannot use the scope check to probe which repositories are configured: no response is
returned to the caller beyond the standard signature-gated 202/401 already in place, and the
scope decision only affects internal processing after the event has already been accepted.

### Related

- [Repository Scope Filtering](../design/configuration-system.md#repository-scope-filtering) — configuration design
- [core-config-validation.md](../interfaces/core-config-validation.md#repository-scope-filtering-additions) — types and validation
- [event-processing.md](../architecture/event-processing.md#repository-scope-filtering) — enforcement point in the pipeline
