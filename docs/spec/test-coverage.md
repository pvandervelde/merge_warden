# Test Coverage: FR-009 Repository Scope Filtering

Status: **RED** — test suite committed before implementation. Compiles with `cargo check
--tests` producing only "item does not exist" errors against the not-yet-implemented interface
(`RepositoryScope`, `ApplicationDefaults.repository_scope`, `is_repository_in_scope`,
`validate_repository_scope_patterns`, `ConfigLoadError::InvalidRepositoryScopePattern`).

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
