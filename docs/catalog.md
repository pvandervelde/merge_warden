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
| `NEGATION_SINGLE_WORDS` | const | `merge_warden_core::labels` | Conservative list of single-word negation tokens used by `is_keyword_negated`; excludes ambiguous words like "eliminates" | negation, constants |
| `set_pull_request_labels_with_config` | fn | `merge_warden_core::labels` | Applies change-type + keyword labels to a PR; supports negation-aware detection, comment-based suppression, explanation comment lifecycle, and smart label detection via `LabelManager` | labels, detection, negation, suppression |
