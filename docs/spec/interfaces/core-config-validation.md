# Interface Spec: core — Configuration Change Validation

**Source**: `crates/core/src/`
**Spec**: `docs/spec/design/configuration-system.md#configuration-change-validation`
**Requirement**: `docs/spec/requirements/functional-requirements.md#fr-007-configuration-change-validation`

---

## Summary

FR-007 adds inline validation of `.github/merge-warden.toml` when that file appears in
a PR's changed-file list. The following types, constants, and functions are added to the
`core` crate's public surface. `MergeWarden<P>` gains a new trait bound but no new public
methods beyond the existing `process_pull_request` entry point.

---

## New Constant: `CONFIG_COMMENT_MARKER`

Location: `crates/core/src/config.rs`

```rust
/// HTML comment sentinel used to identify bot comments that report
/// `.github/merge-warden.toml` validation results.
///
/// Every config-validation comment begins with this marker so that
/// `communicate_config_validity_status` can locate, compare, and replace
/// it idempotently without disturbing unrelated PR comments.
pub const CONFIG_COMMENT_MARKER: &str = "<!-- MERGE_WARDEN_CONFIG_CHECK -->";
```

---

## New Type: `ConfigValidationOutcome`

Location: `crates/core/src/config.rs`

```rust
/// The result of validating raw `.github/merge-warden.toml` content.
///
/// Produced by [`validate_config_content`] and consumed by
/// `MergeWarden::communicate_config_validity_status`.
///
/// # Examples
///
/// ```rust
/// use merge_warden_core::config::{validate_config_content, ConfigValidationOutcome};
///
/// let toml = r#"schemaVersion = 1"#;
/// let outcome = validate_config_content(toml);
/// assert!(outcome.valid);
/// assert!(outcome.errors.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValidationOutcome {
    /// `true` when the content parsed successfully and `schema_version == 1`.
    pub valid: bool,

    /// Human-readable error descriptions; non-empty only when `valid` is `false`.
    ///
    /// Each entry is a standalone sentence suitable for display in a GitHub comment.
    pub errors: Vec<String>,
}
```

---

## New Function: `validate_config_content`

Location: `crates/core/src/config.rs`

```rust
/// Validates raw TOML content as a `RepositoryProvidedConfig`.
///
/// Runs the same parsing and schema-version check that
/// [`load_merge_warden_config`] performs internally, but operates on an
/// already-fetched string rather than fetching the file itself.  This
/// allows the validation to be called with content from the PR head ref
/// without re-fetching from the default branch.
///
/// # Arguments
///
/// * `content` — Raw TOML text to validate.
///
/// # Returns
///
/// A [`ConfigValidationOutcome`] indicating whether the content is valid
/// and, when invalid, a list of human-readable error descriptions.
///
/// # Errors
///
/// This function is infallible. All failure modes are represented as a
/// `ConfigValidationOutcome` with `valid = false` and a populated `errors`
/// field.  No panics, no `Result` wrapping.
///
/// # Validation Rules Applied
///
/// 1. The content must be parseable as `RepositoryProvidedConfig` via
///    `toml::from_str`.  Serde errors (unknown fields, type mismatches)
///    produce one error entry each.
/// 2. `schema_version` must equal `1`.  Any other value produces a
///    dedicated error entry.
/// 3. `title_pattern` and `work_item_reference_pattern`, if present, are each
///    passed through [`validate_pattern`]; a `PatternValidationError` produces
///    a dedicated error entry naming the offending field (see
///    "Regex Pattern Validation and Compilation" below).
pub fn validate_config_content(content: &str) -> ConfigValidationOutcome {
    todo!("See docs/spec/interfaces/core-config-validation.md")
}
```

---

## Trait Bound Change: `MergeWarden<P>`

Location: `crates/core/src/lib.rs`

### Before

```rust
impl<P: PullRequestProvider + std::fmt::Debug> MergeWarden<P> { ... }
```

### After

```rust
impl<P: PullRequestProvider + ConfigFetcher + std::fmt::Debug> MergeWarden<P> { ... }
```

The `ConfigFetcher` bound is required so that `process_pull_request` can call
`self.provider.fetch_config_at_ref(...)` to read the proposed config from the PR head SHA.

`GitHubProvider` already implements both traits; no construction-site changes are needed
in the `server`, `cli`, or integration-test crates.

### Struct Definition (unchanged)

```rust
pub struct MergeWarden<P> {
    provider: P,
    config: CurrentPullRequestValidationConfiguration,
    issue_provider: Option<Box<dyn IssueMetadataProvider>>,
}
```

---

## New Private Method: `communicate_config_validity_status`

Location: `crates/core/src/lib.rs`

This method is not public API; it is called only from `process_pull_request`.
It is documented here so that implementors can understand the full behaviour
contract without reading through `lib.rs`.

```rust
/// Adds, updates, or removes the config-validation comment on a pull request.
///
/// Follows the same idempotency pattern used by all other `communicate_*`
/// methods:
///
/// 1. List existing comments and collect those whose body starts with
///    [`CONFIG_COMMENT_MARKER`].
/// 2. When `outcome.valid` is `true`: delete every collected comment (clean up
///    a previous failure comment when the author has since fixed the config).
/// 3. When `outcome.valid` is `false`:
///    a. Build the new comment body from `outcome.errors`.
///    b. If exactly one existing comment has an identical body, skip (no-op).
///    c. Otherwise delete all existing copies, then post one fresh comment.
///
/// Failures to add, update, or delete comments are logged at `warn` level and
/// do not propagate — config comment management must never block the main
/// validation pipeline.
///
/// # Arguments
///
/// * `repo_owner` — Repository owner.
/// * `repo_name`  — Repository name.
/// * `pr_number`  — Pull request number.
/// * `outcome`    — Result produced by [`validate_config_content`].
#[instrument]
async fn communicate_config_validity_status(
    &self,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    outcome: &ConfigValidationOutcome,
) {
    todo!("See docs/spec/interfaces/core-config-validation.md")
}
```

---

## Changes to `process_pull_request`

`process_pull_request` gains one new step between size checking and change-type labeling.

```
// Existing: fetch PR file list (moved out of size-check guard; now always fetched)
let pr_files = self.provider
    .get_pull_request_files(repo_owner, repo_name, pr_number)
    .await
    .map_err(|e| MergeWardenError::GitProviderError(...))?;

// New: config file validity check
const CONFIG_FILE_PATH: &str = ".github/merge-warden.toml";
if pr_files.iter().any(|f| f.filename == CONFIG_FILE_PATH) {
    match self.provider
        .fetch_config_at_ref(repo_owner, repo_name, CONFIG_FILE_PATH, &pr.head_sha)
        .await
    {
        Ok(Some(content)) => {
            let outcome = validate_config_content(&content);
            self.communicate_config_validity_status(
                repo_owner, repo_name, pr_number, &outcome,
            ).await;
        }
        Ok(None) => {
            // File absent at head SHA — treat as if not changed; no comment.
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch config file for validation; skipping comment");
        }
    }
}
```

The `pr_files` vector is still passed to the size-check logic unchanged; the only
structural change is that it is fetched unconditionally (previously it was only
fetched when `pr_size_check.enabled` was true).

---

## Comment Format

### Valid configuration

No comment is posted (or any previous failure comment is deleted).

### Invalid configuration

```
<!-- MERGE_WARDEN_CONFIG_CHECK -->
⚠️ **Invalid merge-warden configuration**

The `.github/merge-warden.toml` in this PR contains errors and will be ignored
at runtime — the application defaults will be used instead.

**Errors found:**

- <error 1>
- <error 2>

Please fix these errors before merging so that the intended policy takes effect.
```

---

## Error Handling

| Scenario | Behaviour |
|---|---|
| `get_pull_request_files` fails | Propagates as `MergeWardenError::GitProviderError` (existing behaviour) |
| `fetch_config_at_ref` fails | Logged at `warn`; config check skipped; processing continues |
| File absent at head SHA (`Ok(None)`) | Config check skipped; no comment; processing continues |
| `add_comment` / `delete_comment` fails | Logged at `warn`; processing continues |

---

## Renovate Stability Config Additions

The following types and constants are added to `crates/core/src/config.rs` as part of
[FR-008 (Renovate Stability Label Management)](../requirements/functional-requirements.md#fr-008-renovate-stability-label-management).

### New Constants

```rust
/// Context string identifying the Renovate stability check in GitHub commit statuses.
pub const RENOVATE_STABILITY_CHECK_CONTEXT: &str = "renovate/stability-days";

/// Default label name applied while the Renovate stability period has not elapsed.
pub const RENOVATE_STABILITY_LABEL: &str = "pr-validation: pending-stability";
```

### New Type: `RenovateStabilityConfig`

```rust
/// Configuration for Renovate stability-days label management.
///
/// When enabled, `pending_stability_label` is applied to the PR while the
/// `renovate/stability-days` commit status is `pending`, `error`, or `failure`,
/// and removed when the status is `success`.
///
/// This feature is observability-only: it never influences the commit-status
/// check conclusion and never prevents merging.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenovateStabilityConfig {
    /// Whether Renovate stability label management is enabled.
    ///
    /// Defaults to `true`.
    #[serde(default = "RenovateStabilityConfig::default_enabled")]
    pub enabled: bool,

    /// Label applied while the Renovate stability period has not elapsed.
    ///
    /// Defaults to [`RENOVATE_STABILITY_LABEL`].
    #[serde(default = "RenovateStabilityConfig::default_label")]
    pub pending_stability_label: String,
}

impl Default for RenovateStabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pending_stability_label: RENOVATE_STABILITY_LABEL.to_string(),
        }
    }
}
```

### Integration Points

`RenovateStabilityConfig` is threaded through the configuration hierarchy as follows:

| Struct | Field added | Notes |
| --- | --- | --- |
| `PullRequestsPoliciesConfig` | `renovate_stability: RenovateStabilityConfig` | TOML key `[policies.pullRequests.renovateStability]` |
| `PolicySet` | `renovate_stability: RenovateStabilityConfig` | Participates in `PolicySet::merge` |
| `CurrentPullRequestValidationConfiguration` | `renovate_stability: RenovateStabilityConfig` | Read by `communicate_renovate_stability_status` |
| `ApplicationDefaults` | `renovate_stability: RenovateStabilityConfig` | Server-level default; `enabled = true` |
| `RepositoryProvidedConfig` | via `PullRequestsPoliciesConfig` | Repository-level override |

#### Merge semantics for `RenovateStabilityConfig`

| Field | Rule |
| --- | --- |
| `enabled` | `base \|\| over` — once enabled in either tier, stays enabled |
| `pending_stability_label` | `over` wins if non-empty and differs from `default()` |

### New Private Method: `communicate_renovate_stability_status`

Location: `crates/core/src/lib.rs`

Called unconditionally early in `process_pull_request`, immediately after the PR is
fetched and before the draft check. Errors are logged at `warn` and do not propagate.

```rust
/// Applies or removes the Renovate stability label for the current PR HEAD.
///
/// Delegates to `manage_renovate_stability_label` in `crates/core/src/labels.rs`.
/// Errors are logged at `warn` level and never propagate — this step must not
/// block the main validation pipeline or affect the check conclusion.
///
/// # Arguments
///
/// * `repo_owner` — Repository owner.
/// * `repo_name`  — Repository name.
/// * `pr_number`  — Pull request number.
/// * `head_sha`   — HEAD commit SHA of the pull request.
#[instrument]
async fn communicate_renovate_stability_status(
    &self,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    head_sha: &str,
) {
    todo!("See docs/spec/design/labeling-system.md#renovate-stability-days-labeling")
}
```

### Testing Requirements for `RenovateStabilityConfig`

#### Config unit tests for `RenovateStabilityConfig` (`crates/core/src/config_tests.rs`)

- `RenovateStabilityConfig` defaults: `enabled = true`, label equals `RENOVATE_STABILITY_LABEL`
- TOML round-trip: a config with `renovateStability` block parses correctly
- Merge: repo config with `enabled = false` does NOT override app default of `enabled = true`;
  result is `true` (activation bool rule: `base || over` — once either tier enables the feature,
  the merged value is always `true`; a repo can only enable, not disable)
- Merge: repo config with custom `pending_stability_label` wins over default

#### Integration unit tests for renovate stability (`crates/core/src/lib_tests.rs`)

- `process_pull_request` when `renovate/stability-days` status is `pending` → label applied
- `process_pull_request` when `renovate/stability-days` status is `success` → label removed
- `process_pull_request` when no `renovate/stability-days` status → no-op
- `process_pull_request` when `renovate_stability.enabled = false` → no label operations
- `communicate_renovate_stability_status` error from provider → logged at warn; processing continues
- Check conclusion is unaffected by all of the above scenarios

---

## Repository Scope Filtering Additions

The following types, constants, and functions are added to `crates/core/src/config.rs` as part of
[FR-009 (Repository Scope Filtering)](../requirements/functional-requirements.md#fr-009-repository-scope-filtering).

### New Type: `RepositoryScope`

Location: `crates/core/src/config.rs`

```rust
/// Declares which repositories Merge Warden actively processes, independent of
/// which repositories the GitHub App installation can access.
///
/// Needed because large organisations may be forced to install the GitHub App
/// with "All repositories" scope (install-scope limits prevent selecting
/// individual repositories once an org exceeds a certain size), which means
/// the service receives webhooks for repositories it should not act on.
///
/// # Matching semantics
///
/// See [`is_repository_in_scope`] for the full precedence contract. In short:
/// exclude always wins over include, and an explicitly empty `include_patterns`
/// matches nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RepositoryScope {
    /// Glob patterns (`*` = any sequence, `?` = single character) matched
    /// case-insensitively against the bare repository name (not `owner/repo`).
    pub include_patterns: Vec<String>,

    /// Glob patterns that take precedence over `include_patterns` when matched.
    ///
    /// Defaults to an empty list (no exclusions) when the key is omitted from TOML.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}
```

### Extension to `ApplicationDefaults`

```rust
pub struct ApplicationDefaults {
    // ... existing fields unchanged ...

    /// Optional repository scope filter.
    ///
    /// When `None`, every repository the installation receives webhooks for
    /// is processed — identical to pre-FR-009 behaviour.
    #[serde(default)]
    pub repository_scope: Option<RepositoryScope>,
}
```

### New Function: `is_repository_in_scope`

```rust
/// Determines whether `repo_name` should be processed, given the configured
/// `RepositoryScope`.
///
/// # Contract
///
/// - `scope` is `None` → `true` (no scope configured; process everything).
/// - `scope.include_patterns` is empty → `false` (explicit "pause everything" lever),
///   regardless of `exclude_patterns`.
/// - Otherwise → `true` iff `repo_name` matches at least one pattern in
///   `include_patterns` AND does not match any pattern in `exclude_patterns`.
///
/// Matching is case-insensitive (`repo_name` and each pattern are lowercased
/// before comparison, consistent with the topic-matching convention in
/// [conditional-policies.md](./conditional-policies.md)) and is performed
/// against the bare repository name only — `owner/repo` matching is not supported.
///
/// # Arguments
///
/// * `scope` — the configured repository scope, or `None`.
/// * `repo_name` — bare repository name extracted from the webhook payload.
pub fn is_repository_in_scope(scope: &Option<RepositoryScope>, repo_name: &str) -> bool {
    todo!("See docs/spec/architecture/event-processing.md#repository-scope-filtering")
}
```

### New Function: `validate_repository_scope_patterns`

```rust
/// Compiles every pattern in `include_patterns` and `exclude_patterns` to confirm
/// each one translates to a valid anchored regex, without retaining the compiled form.
///
/// Called once by `load_config()` at startup (see
/// [server-config.md](./server-config.md#load_config)) so that a typo in an
/// operator-authored pattern fails fast at process start rather than silently
/// matching nothing (or everything) at webhook-handling time.
///
/// # Errors
///
/// Returns `ConfigurationError::InvalidRepositoryScopePattern(String)` on the
/// first invalid pattern encountered, where the `String` is the offending
/// pattern text. Validation stops at the first failure (fail fast).
pub fn validate_repository_scope_patterns(
    scope: &Option<RepositoryScope>,
) -> Result<(), ConfigurationError> {
    todo!("See docs/spec/design/configuration-system.md#repository-scope-filtering")
}
```

### New Variant: `ConfigurationError::InvalidRepositoryScopePattern`

```rust
pub enum ConfigurationError {
    // ... existing variants unchanged ...

    /// A pattern in `repository_scope.include_patterns` or
    /// `repository_scope.exclude_patterns` is not a valid glob pattern.
    ///
    /// The inner `String` is the offending pattern text, surfaced verbatim so
    /// operators can locate it in their TOML file.
    InvalidRepositoryScopePattern(String),
}
```

### Testing Requirements for `RepositoryScope` and `is_repository_in_scope`

#### Config unit tests (`crates/core/src/config_tests.rs`)

- `RepositoryScope` default: `include_patterns` empty, `exclude_patterns` empty
- TOML round-trip: a `[repository_scope]` block with both `include_patterns` and `exclude_patterns` parses correctly
- TOML round-trip: `[repository_scope]` with `include_patterns` only (no `exclude_patterns` key) parses with `exclude_patterns = []`
- `is_repository_in_scope`: `scope = None` → `true` for any `repo_name` (absent scope)
- `is_repository_in_scope`: `scope = Some(RepositoryScope { include_patterns: vec![], .. })` → `false`, regardless of `exclude_patterns` (empty include)
- `is_repository_in_scope`: `include_patterns = ["payments-*"]` → `"payments-api"` matches, `"checkout"` does not (wildcard match)
- `is_repository_in_scope`: `include_patterns = ["billing-?"]` → `"billing-1"` matches, `"billing-12"` does not (single-char wildcard)
- `is_repository_in_scope`: `include_patterns = ["*"]`, `exclude_patterns = ["payments-legacy"]` → `"payments-legacy"` is excluded; all other names match (exclude overrides include)
- `is_repository_in_scope`: repo name matches both an include and an exclude pattern → excluded (exclude takes precedence)
- `is_repository_in_scope`: matching is case-insensitive (`"Payments-API"` matches `include_patterns = ["payments-*"]`)
- `validate_repository_scope_patterns`: all valid patterns → `Ok(())`
- `validate_repository_scope_patterns`: one invalid pattern in `include_patterns` → `Err(ConfigurationError::InvalidRepositoryScopePattern(_))` (invalid pattern rejected at load)
- `validate_repository_scope_patterns`: one invalid pattern in `exclude_patterns` → `Err(ConfigurationError::InvalidRepositoryScopePattern(_))`
- `validate_repository_scope_patterns`: `scope = None` → `Ok(())`

---

## Broken Default-Branch Configuration Policy Additions

The following types, constants, and functions are added to `crates/core/src/config.rs`
(plus one new private helper and call-site wiring in `crates/core/src/lib.rs`) as part of
[FR-010 (Broken Default-Branch Configuration Policy)](../requirements/functional-requirements.md#fr-010-broken-default-branch-configuration-policy).

### New Constant: `BROKEN_CONFIG_COMMENT_MARKER`

Location: `crates/core/src/config.rs`

```rust
/// HTML comment marker identifying the broken-default-branch-config notice comment.
///
/// Deliberately distinct from [`CONFIG_COMMENT_MARKER`]: that marker flags a malformed
/// configuration file introduced *within the PR diff*; this one flags a malformed
/// `.github/merge-warden.toml` already present on the repository's **default branch**.
/// Keeping the markers separate lets each comment lifecycle be reconciled independently.
pub const BROKEN_CONFIG_COMMENT_MARKER: &str = "<!-- MERGE_WARDEN_BROKEN_CONFIG_CHECK -->";
```

### New Type: `BrokenRepoConfigBehaviour`

Location: `crates/core/src/config.rs`

```rust
/// Application-level operator policy controlling how Merge Warden reacts when the
/// `.github/merge-warden.toml` on a repository's default branch is present but cannot be
/// loaded (malformed TOML or an unsupported `schemaVersion`).
///
/// Application-level only — not settable at the organization or repository level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BrokenRepoConfigBehaviour {
    /// Silently fall back to application defaults — no PR comment, check conclusion
    /// unchanged. `config.load.failures_total` is still recorded.
    #[default]
    SilentFallback,
    /// Post a PR comment explaining the broken default-branch config; do not block.
    Notify,
    /// Post the same PR comment and force the check conclusion to `"failure"`.
    Block,
}
```

### New Type: `BrokenRepoConfigNotice`

Location: `crates/core/src/config.rs`

```rust
/// A populated directive describing that a broken default-branch config was detected and
/// the operator wants it surfaced on the PR. Produced by [`resolve_pull_request_config`];
/// `Some` only when the repo-tier load failed with a "broken" error (not a fetch failure or
/// absent file) and `broken_repo_config_behaviour` is `Notify` or `Block`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenRepoConfigNotice {
    /// Always `Notify` or `Block` — never `SilentFallback`.
    pub behaviour: BrokenRepoConfigBehaviour,
    /// Human-readable explanation, derived from the `ConfigLoadError`'s `Display` output.
    pub error_detail: String,
}
```

### Second Extension to `ApplicationDefaults`

```rust
pub struct ApplicationDefaults {
    // ... existing fields unchanged ...

    /// Application-level operator policy for a broken default-branch config.
    /// Not part of the org/repo policy merge chain and cannot be set in a repository or
    /// org policy file.
    #[serde(default)]
    pub broken_repo_config_behaviour: BrokenRepoConfigBehaviour,
}
```

### Extension to `CurrentPullRequestValidationConfiguration`

```rust
pub struct CurrentPullRequestValidationConfiguration {
    // ... existing fields unchanged ...

    /// `Some` only when the repo-tier config was broken and the application behaviour is
    /// `Notify` or `Block`. Populated by [`resolve_pull_request_config`] after the merge
    /// chain runs, not by any `PolicySet` conversion.
    pub broken_repo_config: Option<BrokenRepoConfigNotice>,
}
```

### Error Classification in `resolve_pull_request_config`

Only `ConfigLoadError::Toml(_)`, `ConfigLoadError::ParseError(_)`, and
`ConfigLoadError::UnsupportedSchemaVersion(..)` are classified as "broken" and eligible for a
`BrokenRepoConfigNotice`. Fetch failures and an absent file fall back to
`RepositoryProvidedConfig::default()` exactly as before FR-010, and never populate
`broken_repo_config`.

### New Private Function: `sanitize_config_error_detail`

Location: `crates/core/src/config.rs`

```rust
/// Neutralises comment-body breakout / marker-injection vectors in an untrusted
/// config-load error detail before it is embedded in a PR comment's fenced code block.
///
/// `error_detail` originates from `ConfigLoadError`'s `Display` — for a malformed-TOML
/// failure this quotes the repository's own `.github/merge-warden.toml` source lines,
/// which are attacker-influenceable by anyone who can land a config on the default branch.
pub(crate) fn sanitize_config_error_detail(detail: &str) -> String;
```

Substitutions applied: `<!--` → `<! --` and `-->` → `-- >` (split HTML-comment delimiters so
an injected marker token cannot match the `body.contains(marker)` filter used by both
comment reconcilers); and a triple-backtick run → `` ` ` ` `` (so the detail cannot break out
of the fenced block it is rendered inside).

### Check-Conclusion Override in `process_pull_request`

When `broken_repo_config` is `Some(notice)` with `notice.behaviour == Block`, the computed
check conclusion is forced to `"failure"` after the normal `success`/`neutral`/`failure`
determination (including the draft-PR `"neutral"` case) — this is the only way FR-010 can
change the check conclusion. `Notify` and `SilentFallback`/`None` never alter it.

### Testing Requirements (`crates/core/src/config_tests.rs`, `crates/core/src/lib_tests.rs`)

- `BrokenRepoConfigBehaviour` default is `SilentFallback`; TOML round-trip for all three
  snake_case values
- `resolve_pull_request_config`: malformed TOML / unsupported `schemaVersion` on the default
  branch with `behaviour = SilentFallback` → `broken_repo_config: None`
  (byte-for-byte-identical fallback behaviour)
- `resolve_pull_request_config`: same, with `behaviour = Notify` or `Block` →
  `broken_repo_config: Some(BrokenRepoConfigNotice { behaviour, error_detail })`
- `resolve_pull_request_config`: a fetch failure (`NotFound`/`Io`) or absent file never
  populates `broken_repo_config`, regardless of `behaviour`
- `process_pull_request` with `Notify`: notice comment posted; check conclusion unaffected
- `process_pull_request` with `Block`: notice comment posted; check conclusion forced to
  `"failure"` even when validation would otherwise produce `"success"` or the draft
  `"neutral"`
- Notice comment reconciliation: idempotent re-run is a no-op; comment is removed once the
  default-branch config is fixed
- `sanitize_config_error_detail`: `<!--`/`-->` sequences and triple-backtick runs in the
  input are split in the output
- `broken_repo_config_behaviour` is rejected (or ignored, per `serde(default)` on unknown
  fields policy) when present in `RepositoryProvidedConfig` or an org policy file

---

## Regex Pattern Validation and Compilation

The following types, constants, and functions are added to `crates/core/src/config.rs`
as part of
[FR-011 (Regex Pattern Validation and Deterministic Failure Handling)](../requirements/functional-requirements.md#fr-011-regex-pattern-validation-and-deterministic-failure-handling).
See [ADR-006](../../adr/ADR-006-regex-pattern-validation.md) for the rationale.

### New Constants

```rust
/// Maximum accepted length, in characters, for `title_pattern` and
/// `work_item_reference_pattern` at any configuration tier.
///
/// A fixed application constant — not configurable per repository or org, for
/// the same reason `broken_repo_config_behaviour` is application-level only
/// (see ADR-005): the entity whose pattern is being bounded must not control
/// the bound.
pub const MAX_PATTERN_LENGTH: usize = 256;

/// Compiled-program size budget (bytes) passed to `RegexBuilder::size_limit`
/// when validating and compiling a configured pattern. Deliberately far below
/// the `regex` crate's own default, so a pathologically expensive-to-compile
/// pattern is rejected quickly and deterministically rather than consuming
/// unbounded memory/CPU during compilation.
pub const PATTERN_COMPILE_SIZE_LIMIT: usize = 1_048_576; // 1 MiB
```

### New Type: `PatternValidationError`

```rust
/// Why a configured `title_pattern` or `work_item_reference_pattern` was
/// rejected. Always distinct, in both type and rendered message, from "the
/// title/body did not match the pattern" — the two must never be conflated.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternValidationError {
    /// Pattern exceeds `MAX_PATTERN_LENGTH`. Rejected without attempting to
    /// compile it.
    #[error("pattern length {length} exceeds the maximum of {max} characters")]
    TooLong { length: usize, max: usize },

    /// Pattern is within the length limit but failed to compile — either a
    /// genuine syntax error, or a pattern that is syntactically valid but
    /// exceeds `PATTERN_COMPILE_SIZE_LIMIT` when compiled.
    #[error("pattern failed to compile: {message}")]
    CompileFailed { message: String },
}
```

### New Function: `validate_pattern`

```rust
/// The single entry point every pattern string must pass through before it is
/// accepted as part of a valid configuration, at every tier (application
/// defaults, org policy, repository config) and in the PR-diff informational
/// check (`validate_config_content`).
///
/// # Errors
///
/// Returns `PatternValidationError::TooLong` before attempting to compile a
/// pattern longer than [`MAX_PATTERN_LENGTH`]. Returns
/// `PatternValidationError::CompileFailed` for any pattern that is within the
/// length limit but fails `RegexBuilder::new(pattern).size_limit(PATTERN_COMPILE_SIZE_LIMIT).build()`,
/// whether due to invalid regex syntax or exceeding the compile-size budget.
pub fn validate_pattern(pattern: &str) -> Result<regex::Regex, PatternValidationError> {
    todo!("See docs/spec/design/configuration-system.md#pattern-validation-and-compile-time-safety")
}
```

### New Type: `CompiledPatternCache`

```rust
/// Process-wide cache of validated, compiled patterns, keyed by the pattern
/// string. Avoids recompiling byte-identical patterns across webhook events
/// and across repositories that share a pattern (most repositories use the
/// built-in default for at least one of `title_pattern` /
/// `work_item_reference_pattern`).
///
/// Bounded (fixed-capacity LRU) so that operators running many repositories
/// with many distinct custom patterns cannot cause unbounded cache growth.
/// Capacity is an internal implementation constant, not configuration.
pub struct CompiledPatternCache { /* internal LRU, not part of the public API shape */ }

impl CompiledPatternCache {
    /// Returns the cached compiled pattern for `pattern`, or validates and
    /// compiles it via [`validate_pattern`] and caches the result (success or
    /// failure) before returning.
    ///
    /// Caching validation *failures* as well as successes avoids repeatedly
    /// re-running a doomed compile attempt for a config that has not changed.
    pub fn get_or_compile(
        &self,
        pattern: &str,
    ) -> Result<Arc<regex::Regex>, PatternValidationError> {
        todo!("See docs/spec/design/configuration-system.md#pattern-validation-and-compile-time-safety")
    }
}
```

### Call Sites

| Call site | When it runs | Behaviour on `Err` |
|---|---|---|
| `load_config()` (server startup) | Once, for `ApplicationDefaults.default_title_pattern` / `.default_work_item_pattern` | Process fails to start — same fail-fast precedent as `validate_repository_scope_patterns` |
| `load_org_policy` | Once per org-policy fetch, for both `[enforced]` and `[defaults]` sections' patterns | Treated as a parse-equivalent failure of the org policy file (same `Ok(None)`/`Err(OrgPolicyUnavailable)` behaviour as any other org-policy parse error, per `fail_if_unreachable`) |
| `load_merge_warden_config` / `resolve_pull_request_config` | Once per webhook event, for the resolved repository-tier patterns | The merged `CurrentPullRequestValidationConfiguration` carries the error (see below); the dependent check fails closed at evaluation time rather than at config-resolution time, so that the *other* check (title vs. work item) is unaffected |
| `validate_config_content` | PR-diff informational check (FR-007) | Added as additional `ConfigValidationOutcome.errors` entries — non-blocking, identical treatment to any other schema error already produced by this function |

### `CurrentPullRequestValidationConfiguration` — no new fields

`title_pattern: String` and `work_item_reference_pattern: String` are **unchanged** —
they continue to carry the raw pattern string. `check_title` and
`check_work_item_reference` call `CompiledPatternCache::get_or_compile` themselves
at evaluation time rather than the config-resolution step eagerly embedding a
`Result<Regex, _>` into CPVRC. This keeps CPVRC a plain, comparable data structure
and keeps "pattern is invalid" a check-time concern with a single code path,
matching how `check_title` and `check_work_item_reference` already produce their
other failure messages.

### Check-Time Behaviour on `PatternValidationError`

When `CompiledPatternCache::get_or_compile` returns `Err` for the pattern a check
depends on:

- That check (title or work item) is reported as **failed**, with a message that
  names the configuration problem (e.g. *"The configured `work_item_reference_pattern`
  in `.github/merge-warden.toml` is invalid and could not be evaluated:
  pattern failed to compile. A repository maintainer must correct this pattern
  before this check can run."*) — never the ordinary "no reference found" /
  "title does not match" message.
- This message is surfaced through the same channel already used for that
  check's pass/fail feedback (check-run output and/or PR comment), so no new
  comment marker or communication path is introduced.
- The *other* check is unaffected: an invalid `title_pattern` does not prevent
  the work-item check (or any other check) from running normally, and vice versa.
- This failure is deterministic and unconditional — there is no configuration
  option to silence it or fall back to a different tier's pattern (contrast with
  `broken_repo_config_behaviour`, which *is* configurable, because a malformed
  TOML file has a safe degraded state — using application defaults — whereas "the
  configured pattern cannot be evaluated" has no safe default other than failing
  the check it would have governed).

### Testing Requirements for Pattern Validation

#### Unit tests (`crates/core/src/config_tests.rs`)

- `validate_pattern` with a pattern at exactly `MAX_PATTERN_LENGTH` → `Ok(_)`
  (boundary is inclusive)
- `validate_pattern` with a pattern one character over `MAX_PATTERN_LENGTH` →
  `Err(TooLong { length, max })`, pattern is **not** attempted to compile
  (verify via a pattern that is both oversized and syntactically invalid, to
  confirm the length check short-circuits before the compile attempt)
- `validate_pattern` with invalid regex syntax (e.g. unbalanced `(`) →
  `Err(CompileFailed { .. })`
- `validate_pattern` with a syntactically valid pattern engineered to exceed
  `PATTERN_COMPILE_SIZE_LIMIT` → `Err(CompileFailed { .. })`
- `validate_pattern` with the built-in default `title_pattern` / `work_item_reference_pattern`
  → `Ok(_)` (defaults must always pass validation)
- `CompiledPatternCache::get_or_compile` called twice with the same valid
  pattern string → second call returns the same cached `Arc<Regex>` without
  recompiling (observable via a compile-counting test double)
- `CompiledPatternCache::get_or_compile` called twice with the same invalid
  pattern string → the cached failure is returned both times without
  re-attempting compilation

#### Integration tests (`crates/core/src/lib_tests.rs`)

- Repository config with an oversized `title_pattern` → title check fails with
  a configuration-error message distinct from "title does not match"; work-item
  check (with a valid pattern) still runs and evaluates normally
- Repository config with an uncompilable `work_item_reference_pattern` → work-item
  check fails with a configuration-error message distinct from "no reference
  found"; title check is unaffected
- Application defaults with a deliberately invalid `default_title_pattern` →
  `load_config()` returns `Err` at startup (process does not start)
- Org policy `[enforced]` section with an invalid pattern, `fail_if_unreachable = false`
  → `load_org_policy` returns `Ok(None)` with a `warn!` log entry, matching
  existing org-policy parse-error handling
- Org policy `[enforced]` section with an invalid pattern, `fail_if_unreachable = true`
  → `load_org_policy` returns `Err(OrgPolicyUnavailable)`
- `validate_config_content` on a PR-diff config with an oversized pattern →
  `ConfigValidationOutcome.valid == false`, with an error entry naming the
  oversized field; check conclusion unaffected (same non-blocking guarantee as
  every other `validate_config_content` error)

---

## Work Item Verification Additions

The following types and fields are added to `crates/core/src/config.rs` as part of
[FR-012 (Work Item Reference Verification)](../requirements/functional-requirements.md#fr-012-work-item-reference-verification).
Concrete tracker-side types (`IssueTrackerProvider`, `WorkItemReference`,
`WorkItemState`, `TrackerConfig`, `TrackerKind`) are specified in
[issue-tracker.md](./issue-tracker.md); this section covers only the
`WorkItemPolicyConfig` additions and their merge/validation treatment.

### Extension to `WorkItemPolicyConfig`

```rust
pub struct WorkItemPolicyConfig {
    pub required: bool,
    pub pattern: String,
    pub label_if_missing: Option<String>,

    /// Opt-in live verification against the configured issue tracker. `false`
    /// (the default) preserves syntactic-only behaviour exactly — the check
    /// passes on any reference matching `pattern`, regardless of whether it
    /// resolves to anything real. `true` means the reference must resolve to
    /// a work item that both exists and is open (see
    /// [issue-tracker.md](./issue-tracker.md) and
    /// [ADR-007](../../adr/ADR-007-issue-tracker-abstraction.md)).
    #[serde(default)]
    pub verify_exists: bool,

    /// Which tracker to verify against. Defaults to the GitHub tracker, using
    /// this repository, when omitted.
    #[serde(default)]
    pub tracker: issue_trackers::TrackerConfig,
}
```

### Merge rules (extends `WorkItemPolicyConfig::merge`, see
[policy-engine.md](./policy-engine.md))

| Field | Rule |
| :--- | :--- |
| `verify_exists` | `base.verify_exists \|\| over.verify_exists` |
| `tracker` | `over.tracker` if not `TrackerConfig::default()`; otherwise `base.tracker` |

### Testing Requirements

#### Config unit tests (`crates/core/src/config_tests.rs`)

- `WorkItemPolicyConfig::default()` → `verify_exists == false`,
  `tracker == TrackerConfig::GitHub`
- TOML round-trip: `verify_exists = true` with and without a `[tracker]` subtable
- Merge: `base.verify_exists = true`, `over.verify_exists = false` →
  `result.verify_exists == true` (OR semantics, cannot be disabled by a lower
  tier re-asserting default)
- Merge: `over.tracker` set to a non-default kind wins over `base.tracker`
- Merge: `over.tracker` omitted (default) → `base.tracker` is preserved

#### Integration tests (`crates/core/src/lib_tests.rs`)

See [issue-tracker.md §6.2](./issue-tracker.md#62-core-integration-tests-cratescoresrclib_testsrs)
for the full behavioural matrix.

---

## Testing Requirements

### Unit tests (`crates/core/src/config_tests.rs`)

- `validate_config_content` with valid TOML → `ConfigValidationOutcome { valid: true, errors: [] }`
- `validate_config_content` with invalid TOML (parse error) → `valid: false`, non-empty errors
- `validate_config_content` with `schema_version = 0` → `valid: false`, schema-version error entry
- `validate_config_content` with unknown top-level field → `valid: false`, unknown-field error

### Unit tests (`crates/core/src/lib_tests.rs`)

- `process_pull_request` when config file is in diff and valid → no config comment posted
- `process_pull_request` when config file is in diff and invalid → config comment posted
- `process_pull_request` when config file is in diff, invalid, prior identical comment exists → no duplicate
- `process_pull_request` when config file is not in diff → no config comment posted
- `process_pull_request` when config file fetch returns `Ok(None)` → no config comment posted
- `process_pull_request` when config file fetch returns `Err(...)` → no config comment; processing continues
- Check conclusion (`success`/`failure`/`neutral`) is unaffected by config validation result
