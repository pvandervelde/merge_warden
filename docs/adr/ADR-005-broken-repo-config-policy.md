# ADR-005: Application-Level Policy for a Broken Default-Branch Config

Status: Accepted
Date: 2026-11-11
Owners: merge_warden team

## Context

When `.github/merge-warden.toml` is present on a repository's **default branch** but
cannot be loaded (malformed TOML or an unsupported `schemaVersion`), Merge Warden
falls back to application defaults when evaluating a PR. Historically this fallback
was silent: no PR comment, no check signal. Repository owners could be unaware that
their intended policy was not being applied and that PRs were being evaluated against
application defaults instead.

Issue #336 asks for an operator-controllable reaction to this situation. The central
design question is **which configuration tier owns the setting**. Merge Warden already
has a four-tier resolution chain (ADR-002, ADR-003): application defaults → org defaults
→ repo config → org/app enforcement. The obvious temptation is to make the broken-config
reaction a normal policy field that flows through that chain. That is exactly the wrong
place for it: the setting governs how we react to a repository config that is itself
broken, so a broken (or malicious) repo config must not be able to influence — least of
all weaken — the operator's chosen reaction to its own brokenness.

## Decision

The broken-default-branch-config reaction is an **application-level setting only**.
It is exposed as `broken_repo_config_behaviour` on `ApplicationDefaults` (the server
`[policies]` table) with three values: `silent_fallback` (default, unchanged behaviour),
`notify` (post a PR comment, do not block), and `block` (post a PR comment and force the
check conclusion to `failure`). Do **not** add this field to `RepositoryProvidedConfig`,
`OrgPolicy`, `PoliciesConfig`, or `PolicySet`, and do **not** route it through
`PolicySet::merge`. It is read directly from `ApplicationDefaults` by
`resolve_pull_request_config`, which attaches a `BrokenRepoConfigNotice` to the resolved
`CurrentPullRequestValidationConfiguration` after the merge chain has run — never as part
of it.

## Consequences

- **Enables:** operators to make broken default-branch configs visible (or hard-blocking)
  without any per-repo opt-in, and without a broken repo config being able to suppress the
  signal.
- **Forbids:** per-repo and per-org overrides of the reaction. A repository cannot set
  `broken_repo_config_behaviour` in its own config file — the key is silently ignored there
  (the repo config types have no such field and no `deny_unknown_fields`).
- **Trade-offs accepted:**
  - Only malformed-TOML / unsupported-schema failures count as "broken". Transient fetch
    failures (`NotFound`/`Io`) and an absent config file are deliberately **not** treated
    as broken, so infrastructure blips cannot spuriously notify or block.
  - `block` forces `failure` even for draft PRs, overriding the usual draft→`neutral`
    softening, because the blocker is an operator-level config problem the PR author cannot
    resolve by finishing the draft.
  - The notice comment uses its own marker (`<!-- MERGE_WARDEN_BROKEN_CONFIG_CHECK -->`),
    distinct from the PR-diff config-check marker, so the two comment lifecycles never
    interfere. Untrusted error text is sanitised and rendered inside a fenced block before
    being embedded in the comment.

## Alternatives considered

- **Make it a normal policy field in the merge chain:** rejected. It would let a broken or
  malicious repo config weaken the operator's reaction to its own brokenness — the precise
  failure mode this feature exists to prevent.
- **Infer the reaction from existing enforcement flags:** rejected. The broken-config
  reaction is orthogonal to title/work-item/size enforcement and needs its own explicit,
  operator-owned control.

## Implementation notes

- `resolve_pull_request_config` classifies the repo-tier `ConfigLoadError`; only
  `Toml`/`ParseError`/`UnsupportedSchemaVersion` with a non-`SilentFallback` behaviour
  produce a `BrokenRepoConfigNotice`.
- `MergeWarden::process_pull_request` reconciles the notice comment via the shared
  `reconcile_marker_comment` helper (post on `Some`, clean up on `None`), and when the
  notice is `Block` it both forces the `failure` conclusion and names the broken config as
  the cause in the check summary/text.
- `silent_fallback` preserves the prior functional behaviour: no comment, no conclusion
  change; the existing `config.load.failures_total` metric is still recorded.

## Examples

```toml
# Server application config — [policies] maps to ApplicationDefaults
[policies]
broken_repo_config_behaviour = "silent_fallback"  # default; or "notify" or "block"
```

## References

- Issue #336, PR #374
- ADR-002 (policy engine / `PolicySet`), ADR-003 (org-level policy)
- `docs/spec/design/configuration-system.md` — "Broken Default-Branch Configuration Policy"
- `docs/spec/requirements/functional-requirements.md` — FR-010
