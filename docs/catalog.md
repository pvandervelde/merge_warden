# Catalog (what exists / reuse map)

Purpose: prevent reinventing utilities, modules, patterns, and "hidden" features.

Add to this whenever a reusable component becomes "the standard way".

## Crate Structure

## `merge_warden_core` — labels

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `is_keyword_negated` | fn | `merge_warden_core::labels` | Returns true when a negation word in the 5-word clause-scoped window before a regex match span indicates the keyword is negated | negation, keyword, detection |
| `parse_suppressed_labels` | fn | `merge_warden_core::labels` | Scans PR comments for `<bot_mention> suppress: <label>` commands; returns HashMap of label→commenter login; skips bot's own explanation comments | suppression, labels, comments |
| `build_keyword_label_comment` | fn | `merge_warden_core::labels` | Builds a per-label HTML-marker explanation comment body with human-readable text and copy-pasteable suppress command | comments, keyword, labels |
| `KEYWORD_LABEL_COMMENT_MARKER` | const | `merge_warden_core::config` | HTML comment prefix `"<!-- MERGE_WARDEN_KEYWORD_LABEL:"` used as a unique per-label marker for idempotent comment management | marker, comments, labels |
| `CONFIG_COMMENT_MARKER` | const | `merge_warden_core::config` | HTML comment marker `"<!-- MERGE_WARDEN_CONFIG_CHECK -->"` used to find/replace/delete the config-file validity comment idempotently | marker, comments, config |
| `ConfigValidationOutcome` | type | `merge_warden_core::config` | Result of validating a TOML config file: `{ valid: bool, errors: Vec<String> }` — derives `Debug, Clone, PartialEq` | config, validation |
| `validate_config_content` | fn | `merge_warden_core::config` | Parses TOML config content and checks `schemaVersion == 1`; returns `ConfigValidationOutcome` — purely informational, never affects check conclusion | config, validation |
| `fetch_config_at_ref` | trait method | `merge_warden_developer_platforms::ConfigFetcher` | Fetches a file from a repo at a specific git ref (e.g. PR head SHA); returns `Ok(Some(content))`, `Ok(None)` when absent, or `Err` | git, config, fetch |
| `head_sha` | field | `merge_warden_developer_platforms::models::PullRequest` | The HEAD commit SHA of the PR's source branch (`#[serde(default)]`); used to fetch config at the exact revision being reviewed | pull-request, git |
| `NEGATION_SINGLE_WORDS` | const | `merge_warden_core::labels` | Conservative list of single-word negation tokens used by `is_keyword_negated`; excludes ambiguous words like "eliminates" | negation, constants |
| `set_pull_request_labels_with_config` | fn | `merge_warden_core::labels` | Applies change-type + keyword labels to a PR; supports negation-aware detection, comment-based suppression, explanation comment lifecycle, and smart label detection via `LabelManager` | labels, detection, negation, suppression |
| `manage_size_labels` | fn | `merge_warden_core::labels` | Applies the correct size label to a PR using smart discovery; falls back to `format!("{}{}", label_prefix, category)` when no repo labels are found — takes `label_prefix: &str` from `PrSizeCheckConfig` | size, labels |
