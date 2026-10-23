---
"merge_warden_core": patch
"merge_warden_server": patch
"merge_warden_cli": patch
---

Improved observability when a repository's `.github/merge-warden.toml` (or the
organisation policy file) fails to load.

**What changed for contributors:** when a pull request modifies
`.github/merge-warden.toml` with invalid TOML syntax or an unsupported `schemaVersion`,
the warning comment Merge Warden posts on the PR is now self-contained — it states what
failed, that application defaults are used until the file is fixed, and the exact next
step to take, with the low-level `toml` crate error included as supporting detail rather
than as the headline message. Previously the comment could surface a raw, opaque `toml`
crate error string with no guidance.

**What changed for operators:** every broken-config log line (in the webhook handler, the
queue worker, the `check-pr` CLI command, and the PR-diff config-change check) now
consistently includes `repository_owner`, `repository`, `config_path`, and `error`
fields — several sites previously omitted one or more of these, and two TOML-parse code
paths logged nothing at all. A new counter, `config.load.failures_total` (OTLP) /
`config_load_failures_total` (Prometheus `GET /metrics`), is incremented on every real
config-load failure from the repo/org policy loading paths (fetch error, TOML parse
error, or unsupported schema version) and on invalid-content outcomes from the PR-diff
config-change check (TOML parse error or unsupported schema version), labelled
`repository_owner`, `repository`, and `failure_reason`. It is not incremented when a
config file is simply absent (the normal, supported fallback case), nor for the PR-diff
check's own file-fetch errors, which are a secondary, informational preview feature
distinct from the repo/org config loading paths the counter is primarily scoped to.

See [Per-repository configuration schema](docs/user/reference/per-repo-config.md),
[How Merge Warden works](docs/user/explanation/how-merge-warden-works.md), and
[HTTP endpoints reference](docs/user/reference/http-endpoints.md) for details.
