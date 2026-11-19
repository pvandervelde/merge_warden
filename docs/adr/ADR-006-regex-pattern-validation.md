# ADR-006: Config-Time Validation and Deterministic Failure Handling for Operator-Supplied Regex Patterns

Status: Accepted
Date: 2026-11-19
Owners: merge_warden team

## Context

`title_pattern` and `work_item_reference_pattern` are maintainer-supplied regular
expressions, read from `.github/merge-warden.toml`, an org policy file, or
application defaults, and evaluated against attacker-influenceable input (the PR
title and body of any contributor who can open a pull request). Merge Warden uses
the `regex` crate, whose matching engine is a Thompson NFA with linear-time
matching guarantees — classic catastrophic-backtracking ReDoS (the dominant risk
with backtracking engines such as PCRE) does not apply here, because `regex` never
backtracks.

The residual risk is at the *compilation* boundary, not the matching boundary: a
pathological pattern can still be expensive, or outright fail, to compile into the
NFA/DFA program the engine executes. Today the only control on that boundary is
the `regex` crate's own internal default size limit, which is invisible to
operators, not tunable from application-level policy, and — more importantly —
produces no distinct, documented behaviour when a pattern simply fails to compile.
A repository could believe a strict pattern is being enforced while the check
silently never runs, which is worse than having no pattern validation at all.

## Decision

1. Every pattern string that enters the system (`ApplicationDefaults` title/work-item
   defaults, org policy `[enforced]`/`[defaults]` sections, repository
   `.github/merge-warden.toml`) is run through one shared validation entry point,
   `validate_pattern`, before it is accepted:
   - Reject patterns longer than a fixed maximum (`MAX_PATTERN_LENGTH`) without
     attempting to compile them.
   - Compile the pattern with an explicit, fixed compiled-program size budget
     (`PATTERN_COMPILE_SIZE_LIMIT`, passed to `RegexBuilder::size_limit`) well below
     the crate's own default, so a pattern that is syntactically valid but
     pathologically expensive to compile is rejected deterministically and quickly
     rather than consuming unbounded memory/CPU during compilation.
   - Both limits are **fixed application constants, not configurable per repository
     or org**. A repository or org policy file cannot raise its own compile budget,
     for the same reason `broken_repo_config_behaviour` (ADR-005) is application-level
     only: the entity whose input is being bounded must not control the bound.
2. Validation happens once, at the point each tier's config is parsed/merged
   (`load_config` at server startup for application defaults, `load_org_policy`,
   `load_merge_warden_config` / `resolve_pull_request_config` for the repository
   tier, and `validate_config_content` for the PR-diff informational check) — never
   lazily on first use during webhook processing.
3. A successfully validated pattern is compiled exactly once per config resolution
   and the compiled form is reused for every check that consults it during that
   resolution. A process-wide cache keyed by the pattern string additionally avoids
   recompiling byte-identical patterns across webhook events and across
   repositories that happen to share a pattern (most do, since most repos use the
   built-in default).
4. When a pattern fails validation on the config path that actually governs PR
   evaluation (the merged, effective config — as opposed to the PR-diff informational
   check, which already has its own non-blocking comment mechanism from FR-007), the
   specific check that depends on it (title or work item) deterministically **fails
   closed**: it reports a failure distinct from "the title/body did not match the
   pattern," naming the configuration problem, and that failure is surfaced through
   the same maintainer-visible channel (check-run output / PR comment) already used
   for that check's ordinary pass/fail feedback. It is never silently disabled and
   never silently falls back to a different configuration tier's pattern.

## Consequences

- **Enables:** deterministic, visible behaviour for every malformed or oversized
  pattern, at every tier, closing the "silently disabled" failure mode that the
  issue explicitly calls out as the worst outcome.
- **Enables:** a measurable reduction in regex compilation cost per webhook event,
  since identical patterns are compiled once and reused rather than recompiled on
  every PR event.
- **Forbids:** per-repository or per-org override of the length/compile-budget
  limits.
- **Forbids:** treating a compile failure as equivalent to "pattern did not match" —
  the two must always be distinguishable in both the message text and the
  underlying result type.
- **Trade-offs accepted:**
  - The fixed limits (`MAX_PATTERN_LENGTH = 256` characters,
    `PATTERN_COMPILE_SIZE_LIMIT = 1 MiB`) are chosen to comfortably accommodate every
    realistic title/work-item pattern while remaining far below the point where
    compilation cost becomes noticeable. They are implementation constants, not
    policy, and can be revisited if a legitimate pattern is ever rejected.
  - The process-wide compiled-pattern cache is bounded (LRU, capped entry count) so
    that an operator running many repositories with many distinct custom patterns
    cannot cause unbounded cache growth.

## Alternatives considered

- **Rely solely on the `regex` crate's default `size_limit`:** rejected. It is not
  app-level policy, is not surfaced to operators or maintainers, and gives no
  distinct signal when it trips versus any other compile error.
- **Make the compile-failure reaction configurable (mirroring
  `broken_repo_config_behaviour`'s `silent_fallback`/`notify`/`block`):** rejected
  for this specific failure mode. Issue #347 is explicit that silent fallback is the
  worst option for a pattern a maintainer believes is being enforced; unlike a
  malformed TOML file (where falling back to defaults is a reasonable degraded
  state), there is no safe default behaviour for "the title/work-item check cannot
  be evaluated at all" other than failing that check.
- **Treat compile failure as a generic "broken repo config" case
  (`BrokenRepoConfigBehaviour`):** rejected as the sole mechanism. That policy is
  about whether to use the repo tier *at all* and defaults to silent; a bad pattern
  specifically needs its own always-on, maintainer-visible failure path scoped to
  the affected check.

## Implementation notes

- `validate_pattern(pattern: &str) -> Result<Regex, PatternValidationError>` is the
  single entry point; every TOML deserialisation path that produces a
  `PullRequestsTitlePolicyConfig.pattern` or `WorkItemPolicyConfig.pattern` value
  routes through it before that value is considered part of a valid configuration.
- See [core-config-validation.md](../spec/interfaces/core-config-validation.md#regex-pattern-validation-and-compilation)
  for concrete types and signatures.
- See [configuration-system.md](../spec/design/configuration-system.md#pattern-validation-and-compile-time-safety)
  for the end-to-end behavioural description.

## Examples

```text
# Repository .github/merge-warden.toml with an oversized pattern
[policies.pullRequests.workItem]
pattern = "(#\\d+){1,10000}"   # exceeds MAX_PATTERN_LENGTH or compile budget

# Resulting title/work-item check-run output (not a silent fallback):
# ⚠️ Work item check cannot run: the configured `work_item_reference_pattern`
#    in .github/merge-warden.toml is invalid (pattern too long / failed to
#    compile). A repository maintainer must fix this pattern before this
#    check can evaluate pull requests.
```

## References

- [docs/spec/design/configuration-system.md](../spec/design/configuration-system.md#pattern-validation-and-compile-time-safety)
- [docs/spec/interfaces/core-config-validation.md](../spec/interfaces/core-config-validation.md#regex-pattern-validation-and-compilation)
- [docs/spec/security/threat-model.md](../spec/security/threat-model.md#regex-pattern-compilation-cost-and-compile-failure)
- [docs/spec/requirements/functional-requirements.md](../spec/requirements/functional-requirements.md#fr-011-regex-pattern-validation-and-deterministic-failure-handling)
- ADR-005 (application-level-only policy precedent for a repo's own brokenness)
