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
    todo!("See docs/spec/interfaces/core-config-validation.md")
}
```

### Testing Requirements for `RenovateStabilityConfig`

#### Config unit tests for `RenovateStabilityConfig` (`crates/core/src/config_tests.rs`)

- `RenovateStabilityConfig` defaults: `enabled = true`, label equals `RENOVATE_STABILITY_LABEL`
- TOML round-trip: a config with `renovateStability` block parses correctly
- Merge: repo config with `enabled = false` overrides app default of `enabled = true`
  (activation bool rule: `base || over` — a repo can only enable, not disable, when
  the merge rule applies; document which direction enforcement overrides apply for this field)
- Merge: repo config with custom `pending_stability_label` wins over default

#### Integration unit tests for renovate stability (`crates/core/src/lib_tests.rs`)

- `process_pull_request` when `renovate/stability-days` status is `pending` → label applied
- `process_pull_request` when `renovate/stability-days` status is `success` → label removed
- `process_pull_request` when no `renovate/stability-days` status → no-op
- `process_pull_request` when `renovate_stability.enabled = false` → no label operations
- `communicate_renovate_stability_status` error from provider → logged at warn; processing continues
- Check conclusion is unaffected by all of the above scenarios

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
