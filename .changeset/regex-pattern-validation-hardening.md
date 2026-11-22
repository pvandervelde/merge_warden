---
"merge_warden_core": minor
"merge_warden_server": minor
---

Hardened how Merge Warden validates and compiles operator/maintainer-supplied
`title_pattern` and `work_item_reference_pattern` regex patterns (set in application
defaults, an org policy file, or a repository's `.github/merge-warden.toml`).

**What changed for everyone:** every pattern is now capped at 256 bytes and must compile
within a fixed 1 MiB compiled-program size budget before it is accepted. Both limits are
fixed application constants — they cannot be raised per repository or per organisation.
Validated/compiled patterns are reused across pull-request events via a process-wide cache
(`merge_warden_core::config::COMPILED_PATTERN_CACHE`), so identical patterns are no longer
recompiled on every webhook event; this part is a pure performance improvement with no
behavioural change.

**What changed for operators:** the server now validates `default_title_pattern` and
`default_work_item_pattern` at startup, before accepting any connections. If either is too
long or fails to compile, the server logs the error and **exits without starting** instead
of running with validation silently disabled. An invalid `title_pattern`/
`work_item_reference_pattern` in an org policy's `[enforced]`/`[defaults]` section is now
caught at org-policy load time and routed through the existing `fail_if_unreachable`
behaviour (warn-and-fall-back by default, or abort PR processing if `true`), and recorded
under `config_load_failures_total{failure_reason="invalid_pattern"}`.

**What changed for maintainers:** if the pattern governing a pull request's title or
work-item check is invalid (via any configuration tier, including a repository's own
`.github/merge-warden.toml`), that check now fails closed with a distinct, dedicated PR
comment naming the problem — never the ordinary "title doesn't match"/"no work item
reference found" message, and never silently skipped or fallen back to the built-in
default. This failure is scoped to the one check that depends on the broken pattern; every
other setting in the same repository config (labels, bypass rules, size thresholds, the
*other* pattern) continues to apply normally, and it never escalates to
`broken_repo_config_behaviour`.

`ValidationResult` gained a new `config_error: Option<String>` field, populated only in
this failure case — an additive change, not a breaking one.

See [How Merge Warden works — regex pattern validation](docs/user/explanation/how-merge-warden-works.md#regex-pattern-validation),
[Set application-level defaults — startup pattern validation](docs/user/how-to/set-app-level-defaults.md#startup-validation-of-default-patterns),
and [ADR-006](docs/adr/ADR-006-regex-pattern-validation.md) for full details.
