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

## Regex Pattern Compilation Cost and Compile Failure

### Threat

`title_pattern` and `work_item_reference_pattern` are maintainer-supplied regular
expressions, read from `.github/merge-warden.toml`, an org policy file, or
application defaults, and evaluated against attacker-influenceable input: the
title and body of a pull request opened by any contributor who can open a PR
against the repository, regardless of whether that contributor can edit the
configuration file itself.

Merge Warden's regex engine (`regex` crate) compiles patterns to a Thompson NFA
and matches in time linear in the length of the input — it never backtracks.
**Classic catastrophic-backtracking ReDoS, the dominant pattern-matching DoS risk
against backtracking engines (PCRE-family), does not apply here.** This threat
model previously had no explicit entry for pattern risk; the absence should not
be read as "regex is unexamined," but the engine choice should also not be
over-credited: linear-time matching closes the backtracking-ReDoS class, but it
does not address the compilation boundary.

The residual risk sits at **compile time**, not match time:

1. A pattern that is syntactically valid but pathologically large/complex can be
   expensive to compile into the engine's internal program, independent of how
   cheap matching against it would be once compiled.
2. A pattern can simply fail to compile — invalid syntax, or valid syntax that
   exceeds a compiled-program size limit — and today there is no distinct,
   documented behaviour for that case. The only existing guard is the `regex`
   crate's own internal default `size_limit`, which is not application-level
   policy, is invisible to operators and maintainers, and (absent dedicated
   handling) risks being conflated with "the title/body simply didn't match,"
   which would let a repository believe a check is enforced when it is in fact
   silently inert.

### Severity

**Low-to-Medium.** The matching engine's linear-time guarantee removes the
highest-severity version of this risk (unbounded CPU consumption from matching).
What remains is a bounded, compile-time cost per configuration load (not per PR
event, once mitigated — see below) and a correctness/trust risk if a compile
failure is silently absorbed rather than surfaced.

### Required Mitigations

1. **Fixed, application-level length cap and compile-size budget** on every
   pattern at every configuration tier, enforced at config-load/validation time
   — never deferred to first use during webhook processing. Neither limit is
   configurable per repository or org, for the same reason
   `broken_repo_config_behaviour` (ADR-005) is application-level only: the
   config whose cost is being bounded must not be able to raise its own bound.
2. **Compile once, reuse the compiled form** for the life of a configuration
   resolution, with a process-wide cache keyed by pattern string, so identical
   patterns (most repositories use at least one built-in default) are not
   recompiled on every webhook event. The cache itself is bounded (fixed-capacity
   LRU) to prevent unbounded memory growth from many distinct operator/repository
   patterns.
3. **Deterministic, maintainer-visible fail-closed behaviour on compile failure**
   — never a silent pass, never a silent disablement, never a silent fallback to
   a different configuration tier's pattern. The dependent check (title or work
   item) fails with a message that explicitly names the configuration problem,
   surfaced through the same check-run/PR-comment channel already used for that
   check's ordinary feedback.

See [ADR-006](../../adr/ADR-006-regex-pattern-validation.md),
[configuration-system.md](../design/configuration-system.md#pattern-validation-and-compile-time-safety),
and
[core-config-validation.md](../interfaces/core-config-validation.md#regex-pattern-validation-and-compilation)
for the full mechanism.

### Related

- [ADR-006](../../adr/ADR-006-regex-pattern-validation.md)
- [Pattern Validation and Compile-Time Safety](../design/configuration-system.md#pattern-validation-and-compile-time-safety)
- [Functional Requirements FR-011](../requirements/functional-requirements.md#fr-011-regex-pattern-validation-and-deterministic-failure-handling)

## Tracker Connection Configuration and Outbound Request Risk

### Threat

Optional work-item verification (`verify_exists`) and the pluggable issue-tracker
abstraction introduce a new category of server-initiated outbound HTTP request:
Merge Warden calls a tracker's API at a `base_url`. For cloud trackers
(`dev.azure.com`, `atlassian.net`, `gitlab.com`, Linear's API) this base URL is
effectively fixed by convention, but on-premises deployments — Azure DevOps
Server and self-hosted Jira/GitLab — require an operator-supplied base URL.

If a repository's or org's own configuration could set `base_url` and the name
of the environment variable supplying the credential (`auth_token_env`)
together, this would be a **credential-exfiltration** vector, not merely SSRF: a
maliciously or carelessly configured `base_url` pointed at an attacker-controlled
host, paired with an `auth_token_env` naming an unrelated secret the process
happens to hold (e.g. the application's own GitHub private key or webhook
secret), would cause Merge Warden to send that secret to the attacker's host —
strictly worse than classic SSRF, which only lets an attacker make the server
*visit* an arbitrary address. This is the design this threat model entry
documents the mitigation for; see "Required Mitigations" below for how the
connection target and the credential pointer are kept out of repository/org
control entirely.

### Severity

**High.** A repository-controlled (or org-policy-controlled) `base_url` paired
with a repository-controlled `auth_token_env` would allow any actor whose config
change merges — not just an operator — to exfiltrate any secret available as an
environment variable to the process, to a host of their choosing. The mitigation
below removes this path structurally (the connection coordinates are no longer
settable outside the operator's own application configuration), but the severity
rating reflects the risk the design must close, not the residual risk once
closed.

### Required Mitigations

1. **Tracker connection coordinates are operator-defined only.** `base_url` and
   `auth_token_env` exist exclusively on `TrackerProfile`, defined in
   `ApplicationDefaults.tracker_profiles` — the operator's own application
   configuration file, never a repository's or org's config file. A repository
   or org selects a tracker `kind`, names a `profile` by key, and supplies only
   project-scoping identifiers (`organization`/`project`, `project_key`,
   `project_path`, `team_key`) that select which project/org/team *within* the
   already-fixed connection to query. This is the same trust-tier precedent as
   `broken_repo_config_behaviour` (ADR-005) and the pattern-validation limits
   (ADR-006): the entity whose outbound request target and credential pointer
   are being fixed must not be the one who fixes them. See
   [ADR-007](../../adr/ADR-007-issue-tracker-abstraction.md) and
   [issue-tracker.md §3](../interfaces/issue-tracker.md#3-tracker-configuration).
2. **Credentials are never stored in configuration.** Every `TrackerProfile`
   references a credential by the *name* of an environment variable
   (`auth_token_env`), never the secret value itself — consistent with the
   project's existing secrets-handling posture (environment variables / Key
   Vault, never repository-committed secrets).
3. **Project-scoping identifiers are validated against a restrictive charset.**
   Because `organization`/`project`/`project_key`/`project_path`/`team_key`
   remain repository-controlled and are concatenated into a request path/URL by
   the adapter, each is validated against a charset that excludes scheme
   delimiters, `@`, `:`, and whitespace before use — see
   [issue-tracker.md §3.5](../interfaces/issue-tracker.md#35-project-scoping-identifier-validation).
   This prevents a scoping identifier from being crafted to change the effective
   request target, even though it does not control the connection host.
4. **Outbound request hardening on every non-GitHub adapter:** HTTPS-only
   (reject non-`https://` profile URLs); no automatic redirect following, or
   re-validation of the resolved target after any redirect; blocking of
   loopback, link-local, private, and cloud-metadata IP ranges **after DNS
   resolution** (not by inspecting the hostname string alone, since DNS
   rebinding — a name that resolves to a public address at validation time and a
   private/metadata address at request time — bypasses a hostname-only check); a
   per-request timeout; and a response-size cap. See
   [issue-tracker.md §6](../interfaces/issue-tracker.md#6-outbound-request-hardening-non-github-adapters)
   for the full, itemised requirement set every `azure_devops`/`jira`/`gitlab`
   adapter must implement.
5. **Least-privilege tracker credentials.** Operators should provision read-only
   tokens/PATs for every non-GitHub tracker where the tracker vendor supports
   scoping a token to read-only, work-item-read access — mirroring the existing
   GitHub App permission model (`Issues: Read`, not `Issues: Write`, is all
   `verify_exists` requires for the GitHub tracker).
6. **Operator-level network egress allow-listing remains recommended** for
   on-premises trackers, as defence in depth alongside the code-level controls
   above — restrict egress at the network layer (firewall/proxy egress
   allow-list) to the known set of tracker hosts the organisation actually uses.
   Merge Warden **can** determine which hosts are legitimate for a given
   deployment: the host is fixed by the operator's own `TrackerProfile`, not by
   repository input, so "legitimate" means "a profile the operator themselves
   defined." Network-layer allow-listing is still recommended as defence in
   depth against a mistake in that same operator-authored profile, not because
   the application is unable to express the boundary.
7. **Tracker configuration is validated at config-load time**, not lazily on
   first use — a malformed `base_url`, an unknown or kind-mismatched `profile`
   reference, an invalid project-scoping identifier, or a missing required field
   for the selected `kind` is rejected the same way an invalid pattern is (see
   `IssueTrackerError::InvalidConfiguration` in
   [issue-tracker.md](../interfaces/issue-tracker.md)), so a broken tracker
   configuration is visible before it affects any PR.

### Related

- [issue-tracker.md](../interfaces/issue-tracker.md) — `TrackerProfile` / `TrackerConfig` split, outbound-request hardening, project-scoping identifier validation
- [ADR-007](../../adr/ADR-007-issue-tracker-abstraction.md)
- [Functional Requirements FR-013](../requirements/functional-requirements.md#fr-013-pluggable-issue-tracker-support)
