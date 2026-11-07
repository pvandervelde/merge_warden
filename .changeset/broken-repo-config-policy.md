---
"merge_warden_core": minor
"merge_warden_server": patch
"merge_warden_cli": patch
---

Added an application-level operator policy controlling how Merge Warden reacts
when the `.github/merge-warden.toml` on a repository's **default branch** is
present but cannot be loaded (invalid TOML or an unsupported `schemaVersion`).

**What changed for operators:** a new application-level setting,
`broken_repo_config_behaviour` (in the server's `[policies]` section), selects
one of three reactions to a broken default-branch config:

- `silent_fallback` (default) — unchanged behaviour: Merge Warden silently falls
  back to application defaults. No PR comment is posted and the check conclusion
  is not changed. The existing `config.load.failures_total` metric is still
  recorded.
- `notify` — Merge Warden posts a PR comment explaining that the default-branch
  config could not be loaded and that application defaults are being used, but
  does **not** block the PR.
- `block` — Merge Warden posts the same PR comment **and** forces the GitHub
  check conclusion to `failure`, hard-blocking the PR (overriding the normal
  success/neutral/failure result) until the config is fixed.

This setting is deliberately application-level only: it cannot be set or
overridden in a repository or organisation policy file, so a broken repository
config cannot weaken the operator's chosen reaction to its own brokenness.
Transient fetch failures and absent config files are never treated as "broken"
and never produce a notice.
