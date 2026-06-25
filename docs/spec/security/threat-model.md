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
