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
   built-in default). The cache holds at most **256 entries** (fixed capacity,
   evicted LRU) — comfortably above the number of distinct patterns any realistic
   deployment uses, since most repositories share the small set of built-in
   defaults. Cached entries are `Arc<Regex>`: cloning a pattern out of the cache for
   use by a check is an O(1) reference-count bump, never a recompile or a deep copy
   of the compiled program.
4. When a pattern fails validation on the config path that actually governs PR
   evaluation (the merged, effective config — as opposed to the PR-diff informational
   check, which already has its own non-blocking comment mechanism from FR-007), the
   specific check that depends on it (title or work item) deterministically **fails
   closed**: it reports a failure distinct from "the title/body did not match the
   pattern," naming the configuration problem, and that failure is surfaced through
   the same maintainer-visible channel (check-run output / PR comment) already used
   for that check's ordinary pass/fail feedback. It is never silently disabled and
   never silently falls back to a different configuration tier's pattern.
5. **A pattern-validation failure is not escalated to
   `BrokenRepoConfigBehaviour` (ADR-005).** `BrokenRepoConfigBehaviour` governs
   whether the repository config tier is used *at all* — it reacts to structural
   failures (malformed TOML, an unsupported `schemaVersion`) that prevent the
   whole tier from loading. A bad pattern is a narrower, local failure: it is
   handled entirely within the specific dependent check (title or work item) that
   consults that pattern. Every other setting in the same repository config tier
   — labels, bypass rules, size thresholds, the *other* pattern, and every other
   policy — remains in effect normally. The two mechanisms never interact: a
   malformed pattern never trips the whole-tier `BrokenRepoConfigBehaviour`
   fallback, and `BrokenRepoConfigBehaviour` never suppresses or substitutes for
   a pattern-validation failure.

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
  - The fixed limits (`MAX_PATTERN_LENGTH = 256` **bytes** — measured via
    `str::len()`, not Unicode scalar values or chars, so a pattern using
    multi-byte characters is bounded by its UTF-8 byte length, not its character
    count — and `PATTERN_COMPILE_SIZE_LIMIT = 1 MiB`) are chosen to comfortably
    accommodate every realistic title/work-item pattern while remaining far below
    the point where compilation cost becomes noticeable. They are implementation
    constants, not policy, and can be revisited if a legitimate pattern is ever
    rejected.
  - `PATTERN_COMPILE_SIZE_LIMIT` maps to `RegexBuilder::size_limit` — the budget
    for the size of the compiled program produced at build time. The `regex`
    crate's separate, smaller `dfa_size_limit` (which bounds the *lazy* DFA's
    runtime cache during matching, not compile-time cost) is deliberately left at
    the crate's own default rather than also fixed by this ADR: it bounds a
    different resource (a small, evictable, bounded runtime cache) than the
    compile-time cost this ADR's budget is about, and the crate's default for it
    is already conservative.
  - The process-wide compiled-pattern cache is bounded to **256 entries**
    (fixed-capacity LRU) so that an operator running many repositories with many
    distinct custom patterns cannot cause unbounded cache growth. Cached entries
    are `Arc<Regex>`, so retrieving one is an O(1) refcount bump rather than a
    recompile or a deep copy.

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
# Repository .github/merge-warden.toml with a pattern that demonstrates the
# *compile-size-budget* rejection path specifically — NOT the length-cap path.
#
# `(#\d+){1,10000}` is only 15 bytes long, nowhere near MAX_PATTERN_LENGTH
# (256 bytes); it passes the length check. It fails only because expanding a
# bounded repetition of {1,10000} produces a compiled program that exceeds
# PATTERN_COMPILE_SIZE_LIMIT. Verified experimentally against `regex` 1.13
# (the same major version this repo pins, "1.11", in Cargo.toml):
# `RegexBuilder::new(r"(#\d+){1,10000}").size_limit(1 << 20).build()` returns
# `Err("Compiled regex exceeds size limit of 1048576 bytes.")`, while the same
# pattern compiles successfully with no size limit applied.
[policies.pullRequests.workItem]
pattern = "(#\\d+){1,10000}"   # passes MAX_PATTERN_LENGTH; fails PATTERN_COMPILE_SIZE_LIMIT only

# Resulting title/work-item check-run output (not a silent fallback):
# ⚠️ Work item check cannot run: the configured `work_item_reference_pattern`
#    in .github/merge-warden.toml is invalid (failed to compile within the
#    configured compile-size budget). A repository maintainer must fix this
#    pattern before this check can evaluate pull requests.
```

## References

- [docs/spec/design/configuration-system.md](../spec/design/configuration-system.md#pattern-validation-and-compile-time-safety)
- [docs/spec/interfaces/core-config-validation.md](../spec/interfaces/core-config-validation.md#regex-pattern-validation-and-compilation)
- [docs/spec/security/threat-model.md](../spec/security/threat-model.md#regex-pattern-compilation-cost-and-compile-failure)
- [docs/spec/requirements/functional-requirements.md](../spec/requirements/functional-requirements.md#fr-011-regex-pattern-validation-and-deterministic-failure-handling)
- ADR-005 (application-level-only policy precedent for a repo's own brokenness)
