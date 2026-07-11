# Test Coverage: FR-009 Repository Scope Filtering

Status: **GREEN** — implementation complete; all tests passing (`cargo test --workspace`: 1250
passed, 4 ignored, zero regressions). `RepositoryScope`, `ApplicationDefaults.repository_scope`,
`is_repository_in_scope`, `validate_repository_scope_patterns`, and
`ConfigLoadError::InvalidRepositoryScopePattern` are all implemented; see
`crates/core/src/config.rs` and `crates/core/src/errors.rs`.

## Scope

Task #001 — Repository Scope Filtering (FR-009). See
`docs/spec/requirements/functional-requirements.md#fr-009-repository-scope-filtering`,
`docs/spec/design/configuration-system.md#repository-scope-filtering`,
`docs/spec/interfaces/core-config-validation.md#repository-scope-filtering-additions`,
`docs/spec/architecture/event-processing.md#repository-scope-filtering`,
`docs/spec/security/threat-model.md#all-repositories-installation-scope`.

## Files

- `crates/core/src/config_tests.rs` — `RepositoryScope`, `is_repository_in_scope`,
  `validate_repository_scope_patterns`, `ApplicationDefaults` TOML round-trip,
  `PolicySet::from_application_defaults` non-interference.
- `crates/server/src/config_tests.rs` — `load_config()` reading/validating
  `[policies.repository_scope]`.
- `crates/server/src/webhook_tests.rs` — `MergeWardenWebhookHandler::handle_event` gate
  (integration/Tier 3).

## Tier 1 — Specification tests

### `crates/core/src/config.rs`

- [x] `RepositoryScope::default()` → both pattern lists empty
- [x] `ApplicationDefaults::default().repository_scope` → `None`
- [x] TOML: absent `[repository_scope]` → `None`
- [x] TOML: `[repository_scope]` with both `include_patterns` and `exclude_patterns` → both
      populated
- [x] TOML: `[repository_scope]` with only `include_patterns` → `exclude_patterns` defaults to
      `[]`
- [x] `PolicySet::from_application_defaults` output is identical regardless of
      `repository_scope` value (catalog slice: not part of the `PolicySet` merge chain)
- [x] `is_repository_in_scope(&None, _)` → `true` (unconditional)
- [x] `is_repository_in_scope` empty `include_patterns` → `false`
- [x] `is_repository_in_scope` empty `include_patterns` wins even with a wildcard exclude list
- [x] `*` wildcard: any-sequence match / non-match
- [x] `?` wildcard: exactly-one-char match / non-match
- [x] exclude beats include (wildcard include + specific exclude)
- [x] exclude beats include (identical pattern in both lists)
- [x] case-insensitive include matching
- [x] case-insensitive exclude matching
- [x] no-match → `false`
- [x] `validate_repository_scope_patterns(&None)` → `Ok(())`
- [x] `validate_repository_scope_patterns` all-valid patterns → `Ok(())`
- [x] `validate_repository_scope_patterns` empty pattern lists → `Ok(())` (not a pattern error)
- [x] `validate_repository_scope_patterns` invalid include pattern → `Err(InvalidRepositoryScopePattern(_))`
- [x] `validate_repository_scope_patterns` invalid exclude pattern → `Err(InvalidRepositoryScopePattern(_))`
- [x] `validate_repository_scope_patterns` reports the **first** invalid pattern (fail fast)

### `crates/server/src/config.rs` — `load_config()`

- [x] `repository_scope` defaults to `None` when no config file is set
- [x] valid `[policies.repository_scope]` is read into `application_defaults.repository_scope`
- [x] invalid include pattern → `Err(ServerError::ConfigError(_))`
- [x] invalid exclude pattern → `Err(ServerError::ConfigError(_))`
- [x] explicitly empty `include_patterns = []` is a **valid** startup config (not an error)

### `crates/server/src/webhook.rs` — `MergeWardenWebhookHandler::handle_event`

- [x] out-of-scope repo, `pull_request` event → `Ok(())`
- [x] in-scope repo, `pull_request` event → falls through to existing dispatch (offline
      discriminator: reaches the pre-existing "missing installation ID" `ProcessingError`)
- [x] out-of-scope repo, `status` event → `Ok(())`
- [x] in-scope repo, `status` event → falls through to existing dispatch
- [x] missing `repository.name` in raw payload → `Ok(())` (fail-closed), independent of whether
      `repository_scope` is configured
- [x] empty-string `repository.name` → `Ok(())` (fail-closed)
- [x] non-string `repository.name` (JSON number) → `Ok(())` (fail-closed)

## Tier 2 — Adversarial / boundary tests

- [x] `?` boundary: zero characters after the literal prefix does not satisfy one `?`
      (N-1)
- [x] `??` boundary: exactly two characters required, one or three do not match (N-1/N/N+1)
- [x] multiple `include_patterns` combine with OR semantics
- [x] multiple `exclude_patterns` combine with OR semantics
- [x] pattern matching is fully anchored (not a substring search) — `"app"` does not match
      `"myapp"` or `"app2"`
- [x] `.` in a pattern is a literal character, not "any character" (kills a naive
      unescaped-glob-to-regex stub)
- [x] hardcoded `true` stub killed by the empty-include test
- [x] hardcoded `false` stub killed by the `None`-scope test
- [x] empty repo name vs. non-empty pattern → `false`; empty repo name vs. bare `*` → `true`
- [x] end-to-end: exclude overrides include through the full `handle_event` gate
- [x] end-to-end: case-insensitive matching through the full `handle_event` gate
- [x] end-to-end: empty `include_patterns` blocks every repository through the full gate
- [x] end-to-end: an out-of-scope repo short-circuits even when the payload is missing BOTH
      `installation.id` and `pull_request.number` (scope check precedes further payload parsing)
- [x] regression: `repository_scope = None` leaves `pull_request` dispatch unaffected
- [x] regression: `repository_scope = None` leaves `status` dispatch unaffected
- [x] regression: a non-renovate `status` context is still ignored when the repo is in scope

## Tier 3 — Property-based tests (`proptest`) and integration tests

### `crates/core/src/config_tests.rs`

- [x] `is_repository_in_scope(&None, repo_name)` is always `true` for generated repo-name-like
      strings
- [x] empty `include_patterns` is always `false` regardless of generated `exclude_patterns`
      content
- [x] a literal pattern equal to a generated repo name (any case) always matches
      (case-insensitivity as a property, not just an example)
- [x] identical pattern in both `include_patterns` and `exclude_patterns` always excludes
- [x] `is_repository_in_scope` never panics across a restricted glob-safe alphabet
      (`[a-zA-Z0-9*?._-]`)
- [x] `validate_repository_scope_patterns` never panics on fully arbitrary input (`.*`) — this
      is its core safety contract, since it exists to safely reject untrusted operator input

### `crates/server/src/webhook_tests.rs` (integration — the request-processing gate)

- [x] table-driven cross-check: for a set of repository names, `handle_event`'s observable
      outcome (filtered vs. processed) matches `is_repository_in_scope` exactly, for both
      `pull_request` and `status` events

## Known gaps / deferred items (see final report to Tech Lead)

1. **Metrics assertions not included.** `crates/server/src/telemetry.rs` has no counter/metrics
   facility today (only a `tracing` subscriber). The spec requires incrementing
   `merge_warden.webhook.filtered_total{reason=...}`; no test asserts this counter because there
   is nothing to hook into yet. Flagged for the Coder/architect.
2. **`EventEnvelope` repo-name extraction path is a documented assumption, not a verified fact.**
   No local or cached source for `github-bot-sdk` v0.2.0 was reachable from this environment.
   Tests assume `envelope.payload.raw()["repository"]["name"]` per the explicit pseudocode in
   `docs/spec/architecture/event-processing.md#repository-scope-filtering`, and are constructed
   so that using the structured `envelope.repository.name` field instead would produce a
   different, informative test failure rather than a silent pass.
3. **"Invalid glob pattern" fixture is a documented assumption.** An unmatched `[` is used as
   the fixture for `validate_repository_scope_patterns` rejecting a pattern. Given the
   codebase's existing `pattern_matches` precedent (`regex::escape` + substitute), a fully
   spec-compliant (fully-escaping) translator might treat `[` as literal and never fail on any
   input. If the Coder's implementation makes this fixture pass trivially (never invalid), this
   must be escalated to the architect to define a concrete "invalid pattern" example in
   `assertions.md`.
4. Exact statement-level ordering ("scope check is the literal first statement in
   `handle_event`, before `if envelope.event_type == "status"`") is verified behaviourally
   (offline discriminators prove the check precedes any further payload parsing or API call) but
   not via source-line inspection — that is a code-review / QA concern, not a black-box test
   concern.

---

## QA Audit Report (post-implementation)

Status: implementation complete (commits `720b307`, `f5fa9e1`, `af82f24`, `59a18e2` on
`task/001-repository-scope-filtering`). This section records the post-GREEN adversarial audit
per the Tier 4/5 mandate (Tier 6 — Kani — is not applicable; this module is classified
domain-logic, not safety-critical). `cargo-mutants`, `cargo-fuzz`, and `kani` are **not
configured** in this repository (verified: absent from every `Cargo.toml`, no `fuzz/` directory,
no `#[cfg(kani)]` harnesses anywhere in the workspace) and were not installed or invoked, per
audit scope. Manual technique substitutes were used instead, as directed.

### Tier 4 substitute — manual mutation analysis

Method: for each function, plausible mutants were enumerated by hand, then **empirically
verified** by temporarily applying the mutation to the real source file, running the full
existing `repository_scope`/`handle_event` test suite, observing whether it still passed
(survivor) or failed (killed), then reverting the mutation before writing any kill test.
Confirmed-clean reverts were checked with `git diff --stat` after each round (production files
show zero net diff at the end of this audit — only test files changed).

| Function | Mutants enumerated | Survived initial review (confirmed empirically) | Killed by new tests |
|---|---|---|---|
| `compile_repository_scope_pattern` (`crates/core/src/config.rs`) | 9 | 1 | 1 |
| `matches_any_repository_scope_pattern` (`crates/core/src/config.rs`) | 3 | 1 | 1 |
| `is_repository_in_scope` (`crates/core/src/config.rs`) | 6 | 0 | 0 |
| `validate_repository_scope_patterns` (`crates/core/src/config.rs`) | 5 | 1 | 1 |
| `handle_event` scope-gate block (`crates/server/src/webhook.rs`) | 7 | 1 | 1 |
| **Total** | **30** | **4** | **4** |

**Mutation "score" for this audit round:** 26/30 (87%) of hand-enumerated mutants were already
caught by the RED-phase Tester's suite; the remaining 13% (4 mutants) were confirmed survivors
and are now killed by the 4 new targeted tests below. Combined with the pre-existing suite, all
30 enumerated mutants are now killed — 100% of the hand-enumerated set, exceeding the 85%
domain-logic mutation target. (This is a manual-audit substitute metric, not a tool-computed
score; a real `cargo-mutants` run may surface additional mutants beyond hand enumeration — see
Blocking Issues / recommendations.)

#### Survivor 1: character allow-list guard bypass in `compile_repository_scope_pattern`

- **File:** `crates/core/src/config.rs:2392`
- **Mutation:** `c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => translated.push(c)`
  weakened to always match (equivalent to `c if true || ... => translated.push(c)`), so every
  character is accepted into the translated regex instead of only the documented allow-list.
- **Why it survived:** every existing "invalid pattern" fixture (`"payments-["`) uses `[`, which
  independently produces a syntactically invalid regex even when pushed through unescaped —
  so `compile_repository_scope_pattern` still returns `Err(())` via the final `regex::Regex`
  build failure, masking the fact that the character-allow-list check itself was bypassed. No
  test exercised a disallowed character that is regex-harmless on its own (e.g. a space or `@`),
  which is the only way to distinguish "the allow-list rejected this" from "the allow-list was
  bypassed but the resulting regex happened to fail anyway."
- **Kill tests:** `test_validate_repository_scope_patterns_rejects_space_character`,
  `test_validate_repository_scope_patterns_rejects_at_sign_character`
  (`crates/core/src/config_tests.rs`)
- **Resolution:** confirmed killed — yes (re-applied the mutation, both new tests failed with
  `Ok(())` instead of the expected `Err`; reverted, both pass).

#### Survivor 2: `unwrap_or(false)` → `unwrap_or(true)` in `matches_any_repository_scope_pattern`

- **File:** `crates/core/src/config.rs:2415`
- **Mutation:** the fallback for a pattern that fails to compile was flipped from "treat as
  non-match" to "treat as match."
- **Why it survived:** no existing test ever placed an uncompilable pattern into a
  `RepositoryScope` that was then passed directly to `is_repository_in_scope`. All
  `validate_repository_scope_patterns` tests check the *rejection* path in isolation; none
  followed up by feeding the same invalid pattern into `is_repository_in_scope` to check its
  documented "matches nothing" fallback contract.
- **Kill test:** `test_is_repository_in_scope_unparseable_pattern_never_matches`
  (`crates/core/src/config_tests.rs`)
- **Resolution:** confirmed killed — yes.

#### Survivor 3: `include_patterns`/`exclude_patterns` iteration order swap in `validate_repository_scope_patterns`

- **File:** `crates/core/src/config.rs:2500-2504`
- **Mutation:** `.include_patterns.iter().chain(scope.exclude_patterns.iter())` reversed to
  `.exclude_patterns.iter().chain(scope.include_patterns.iter())`.
- **Why it survived:** the existing "reports first invalid pattern (fail fast)" test only puts
  multiple invalid patterns inside `include_patterns` (`exclude_patterns` is empty), so swapping
  which list is checked first is unobservable — with only one list populated, order is
  irrelevant. The doc comment explicitly promises "in that order" (include before exclude), but
  this was never pinned down with both lists simultaneously invalid.
- **Kill test:** `test_validate_repository_scope_patterns_include_violation_reported_before_exclude`
  (`crates/core/src/config_tests.rs`)
- **Resolution:** confirmed killed — yes.

#### Survivor 4: scope-gate reads structured `envelope.repository.name` instead of raw JSON

- **File:** `crates/server/src/webhook.rs:480`
- **Mutation:** `is_repository_in_scope(&self.policies.repository_scope, repo_name)` changed to
  `is_repository_in_scope(&self.policies.repository_scope, envelope.repository.name.as_str())` —
  i.e. the scope *decision* uses the SDK-populated structured field instead of the
  already-extracted raw-JSON `repo_name` variable.
- **Why it survived:** this contradicts "Known gap #2" in the RED-phase report above, which
  claimed tests were constructed so this exact substitution would produce a different, visible
  failure. In practice every fixture in `webhook_tests.rs` that exercises the scope gate
  (`make_pull_request_envelope`, `make_status_envelope_with_scope_fixtures`) sets the structured
  `Repository.name` and the raw JSON `repository.name` to the *same* value, so the two data
  sources are indistinguishable to any existing assertion. The only tests using genuinely
  different raw-vs-structured values (`make_envelope_with_raw_repository_name`) are all
  *malformed-name* fixtures (missing/empty/non-string), which only probe the extraction/parsing
  branch, not the subsequent `is_repository_in_scope` call that consumes the extracted value.
- **Kill test:** `handle_event_scope_decision_uses_raw_repository_name_not_structured_field`
  (`crates/server/src/webhook_tests.rs`)
- **Resolution:** confirmed killed — yes (re-applied the mutation; the new test failed with
  `Err(ProcessingError("Missing installation ID in webhook payload"))` instead of the expected
  `Ok(())`; reverted, test passes).

### Tier 5 substitute — adversarial input probing

No `cargo-fuzz` harness exists in this repo; probes below were run as targeted, timed unit/
integration tests against the real implementation (not committed as mutated code — the
implementation was never altered for this section, only exercised).

| Adversarial input | Target | Result |
|---|---|---|
| 30× chained `a*` wildcards + literal terminal char, matched against a 60-char non-matching haystack sharing the prefix (classic backtracking-regex ReDoS shape) | `is_repository_in_scope` | Clean — matched in < 500ms budget (actual: sub-millisecond); `regex` crate's automata-based engine does not exhibit catastrophic backtracking. No hang. |
| Pathological pattern: 2,000× `a?` + trailing `*`, matched against a 5,000-char haystack | `is_repository_in_scope` | Clean — completed well under the 1s budget. No hang, no panic. |
| RTL override (`\u{202E}`), RTL mark (`\u{200F}`), combining diacritic (`\u{0301}`), emoji (`\u{1F4A5}`), embedded NUL (`\u{0000}`) injected into repository names, matched against a plain-ASCII pattern | `is_repository_in_scope` | Clean — no panic; anchored full-string match correctly rejects all variants as non-matches (extra/foreign code points break the literal match). |
| Same adversarial Unicode set used as **patterns** (operator-authored config) | `validate_repository_scope_patterns` | Clean — all rejected with `Err(InvalidRepositoryScopePattern(_))`, since none are in the ASCII allow-list; no panic. |
| `repository.name` is a JSON object (`{"nested": "value"}`) | `handle_event` | Clean — `Value::as_str()` returns `None` uniformly for non-string JSON types; treated as malformed, `Ok(())`, no panic. Was previously untested (only the JSON-number case existed). |
| `repository.name` is a JSON array (`["a", "b"]`) | `handle_event` | Clean — same as above, `Ok(())`, no panic. Previously untested. |
| `repository.name` is JSON `null` | `handle_event` | Clean — `Ok(())`, no panic. Previously untested. |
| `repository` itself is JSON `null` (not just its `name` field) | `handle_event` | Clean — indexing `serde_json::Value::Null["name"]` returns a static `Null` rather than panicking; `Ok(())`. Previously untested. |
| Embedded NUL byte inside an otherwise well-formed `repository.name` (`"payments-\u{0000}api"`) | `handle_event` | Clean — no panic; the NUL byte is matched literally by the `.*` wildcard translation of `payments-*`, so the event is correctly treated as in-scope (verified via the offline "Missing installation ID" discriminator, proving the match succeeded and dispatch proceeded normally). |

**No crashes, hangs, or incorrect results were found.** All adversarial probes above were added
as permanent regression tests (they are cheap, deterministic, and offline) rather than being
discarded after a clean run, since manual re-derivation of this exact adversarial input set would
otherwise be lost.

### Tier 6 — Formal verification

Not applicable. This module is classified **domain-logic** (mutation target 85%), not
safety-critical (STO/brake/Safety MCU FSM tiers). `kani` is not configured in this repository and
was not invoked, per the audit brief.

### New tests added

| File | Count | Purpose |
|---|---|---|
| `crates/core/src/config_tests.rs` | 4 | Mutation kill tests (Tier 4) |
| `crates/core/src/config_tests.rs` | 4 | Adversarial input probes (Tier 5) |
| `crates/server/src/webhook_tests.rs` | 1 | Mutation kill test (Tier 4) |
| `crates/server/src/webhook_tests.rs` | 5 | Adversarial input probes (Tier 5) |
| **Total** | **14** | |

Full workspace suite: 1250 passed, 4 ignored (was 1236 passed before this audit — exactly +14,
matching the new test count; zero regressions).

### Blocking issues

None. The 4 confirmed mutation survivors were all killed within this audit session; no
implementation defect was found (all four were genuine test-suite gaps, not incorrect production
behaviour — the real implementation already does the documented-correct thing in every case;
only the *tests* failed to pin it down). All Tier 5 adversarial probes passed cleanly against the
unmodified real implementation.

Non-blocking observations for the record:

- "Known gap #2" in the RED-phase section of this document (claiming the structured-vs-raw
  `repository.name` substitution would be caught) was empirically incorrect; it is now closed by
  the new `handle_event_scope_decision_uses_raw_repository_name_not_structured_field` test.
- This audit's mutant enumeration was manual and necessarily non-exhaustive. If `cargo-mutants`
  is added to this repository's toolchain in the future, a project-wide or `merge_warden_core`-
  scoped run is recommended to surface any mutants outside the hand-enumerated set (e.g. in
  code paths not touched by this task).

### Verdict

**CLEAR**

Commit(s): see `git log` on `task/001-repository-scope-filtering` for the `test(mutation):` /
`test(audit):` commits accompanying this report.
