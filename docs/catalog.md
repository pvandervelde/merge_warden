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

## `merge_warden_core` — config / policy

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `PolicySet` | type | `merge_warden_core::config` | Resolved, merged set of PR validation policies (title, work-item, size, WIP, PR-state, issue-propagation, change-type labels, bypass rules). Use `from_application_defaults` + `from_repository_config` + `merge` to compose the effective policy for a PR evaluation cycle. Derives `Default`. | config, policy, merge |
| `PolicySet::from_application_defaults` | fn | `merge_warden_core::config` | Constructs a `PolicySet` seeded from `ApplicationDefaults`; enforcement-override flags (`enable_title_validation` etc.) are intentionally NOT applied here — apply them after `merge`. | config, policy, merge |
| `PolicySet::from_repository_config` | fn | `merge_warden_core::config` | Constructs a `PolicySet` seeded from a `RepositoryProvidedConfig`; absent optional fields become typed defaults so they register as "unconfigured" during merge. | config, policy, merge |
| `PolicySet::from_org_section` | fn | `merge_warden_core::config` | Constructs a `PolicySet` from one section (enforced or defaults) of an `OrgPolicySectionRaw`; bypass rules are fully supported and parsed from `[*.policies.bypassRules.*]` when present. | config, policy, org-policy |
| `BypassRulesConfig::to_bypass_rules` | fn | `merge_warden_core::config` | Converts a `&BypassRulesConfig` into a `BypassRules` value; absent sub-rules become `BypassRule::default()`. Pair with `Option::map` and `unwrap_or_default` when the config section may be absent. `pub(crate)`. | config, bypass-rules, conversion |
| `PolicySet::from_app_enforcement_flags` | fn | `merge_warden_core::config` | Constructs a `PolicySet` containing only the app-level enforcement flags (`enable_title_validation`, `enable_work_item_validation`, `pr_size_check.enabled`, `wip_check.enforce_wip_blocking`). Applied as the highest-priority merge tier. | config, policy, org-policy |
| `PolicySet::to_validation_config` | fn | `merge_warden_core::config` | Converts a fully-merged `PolicySet` into a `CurrentPullRequestValidationConfiguration`, threading through `ApplicationDefaults` context fields (bypass rules, bot_mention, etc.). | config, policy, org-policy |
| `PolicySet::merge` | fn | `merge_warden_core::config` | Merges two `PolicySet` values: `self` is the lower-priority base, `other` is the higher-priority override. Returns a new `PolicySet` with each field resolved according to §2 of the policy-engine spec. | config, policy, merge |
| `OrgPolicySource` | type | `merge_warden_core::config` | Coordinates of the org-level policy TOML file (owner, repo, path) plus a `fail_if_unreachable` flag. Set as `ApplicationDefaults::org_policy_source`. | config, org-policy |
| `OrgPolicy` | type | `merge_warden_core::config` | Parsed org-level policy with `enforced: PolicySet` and `defaults: PolicySet` tiers. | config, org-policy |
| `load_org_policy` | fn | `merge_warden_core::config` | Fetches and parses the org policy TOML using a `ConfigFetcher`. Returns `Ok(None)` on missing file or lenient error; `Err(OrgPolicyUnavailable)` on strict error. | config, org-policy |
| `resolve_pull_request_config` | fn | `merge_warden_core::config` | Six-tier PR config orchestrator: app defaults → org defaults → conditional_defaults* → repo → conditional_enforced* → org enforced → app enforcement flags. Accepts `metadata_provider: Option<&dyn RepositoryMetadataProvider>` for conditional policy evaluation; pass `None` to skip conditional tiers. Primary entry point for platform handlers. | config, org-policy, conditional-policy |
| `CurrentPullRequestValidationConfiguration::from_app_defaults` | fn | `merge_warden_core::config` | Constructs a `CurrentPullRequestValidationConfiguration` directly from `ApplicationDefaults` without loading any files. Used as the degraded fallback in platform handlers. | config, fallback |
| `PolicyCondition` | type | `merge_warden_core::config` | Parsed condition block for a conditional policy entry: `has_any_topic: Vec<String>` (OR semantics, case-insensitive) and `has_custom_property: HashMap<String,String>` (AND+case-sensitive). `matches(&RepositoryContext) -> bool` evaluates the condition. | config, conditional-policy |
| `ConditionalPolicy` | type | `merge_warden_core::config` | Conditional policy entry containing a `PolicyCondition` plus `defaults: PolicySet` and `enforced: PolicySet` tiers, applied only when `condition.matches()` is true for a repository's context. | config, conditional-policy |
| `RepositoryScope` | type | `merge_warden_core::config` | Repository allow/deny scope filter (FR-009): `include_patterns: Vec<String>` (empty = fail-closed, no repos in scope) and `exclude_patterns: Vec<String>` (`#[serde(default)]`). Set as `ApplicationDefaults::repository_scope`; deliberately NOT part of the `PolicySet` merge chain — gates whether an event is processed at all, not how it is validated. | config, repository-scope, ingress |
| `is_repository_in_scope` | fn | `merge_warden_core::config` | `(scope: &Option<RepositoryScope>, repo_name: &str) -> bool`. `None` → always true. Empty `include_patterns` → always false. Otherwise: matches ≥1 include pattern AND 0 exclude patterns (exclude wins). Glob patterns (`*`/`?` wildcards, literal `.`) compiled case-insensitive and anchored; panic-free — an uncompilable pattern is treated as a non-match. | config, repository-scope, glob |
| `matches_any_repository_scope_pattern` | fn | `merge_warden_core::config` | `pub(crate)`-private `(patterns: &[String], repo_name: &str) -> bool`. Shared "does any pattern match" helper used by `is_repository_in_scope` for both its include and exclude checks; compiles each pattern via the same restricted glob translator as `compile_repository_scope_pattern` (not `pattern_matches` — see that entry's note on why the two glob matchers are not unified). | config, repository-scope, glob |
| `validate_repository_scope_patterns` | fn | `merge_warden_core::config` | `(scope: &Option<RepositoryScope>) -> Result<(), ConfigLoadError>`. Compiles every include then exclude pattern (fail-fast on first invalid one) using the same glob translator as `is_repository_in_scope`. Call once at startup (`crates/server/src/config.rs::load_config`) so malformed patterns fail fast rather than silently matching nothing/everything at webhook time. | config, repository-scope, validation |

## `merge_warden_developer_platforms` — models / traits

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `RepositoryContext` | type | `merge_warden_developer_platforms::models` | Runtime metadata for a repository: `topics: Vec<String>` and `custom_properties: HashMap<String,String>`. Derives `Debug, Clone, Default, PartialEq, Eq`. Used by `PolicyCondition::matches`. | metadata, models, conditional-policy |
| `RepositoryMetadataProvider` | trait | `merge_warden_developer_platforms` | Async port trait for fetching repository metadata. Single method: `get_repository_context(owner, name) -> Result<RepositoryContext, Error>`. Implemented by `GitHubProvider`; pass `None` in tests or callers that don't need conditional policies. | metadata, trait, port |
| `CommitStatus` | struct | `merge_warden_developer_platforms::models` | A single GitHub commit status entry with `context: String`, `state: String`, and `description: Option<String>`. Derives `Debug, Clone, Serialize, Deserialize`. Maps from `GET /repos/{owner}/{repo}/commits/{sha}/statuses`; GitHub returns newest-first so callers use the first occurrence per context. | models, commit-status, GitHub |

## `merge_warden_developer_platforms` — tracing / observability

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `api_error_status_code` | fn | `merge_warden_developer_platforms::github` | Best-effort mapping from an `ApiError` to the HTTP status code most closely associated with it (404/401/403/429/504/400; `0` for errors with no HTTP equivalent). Used to populate the `status` tracing span field on `GitHubProvider` methods that call through SDK wrappers not exposing the raw `reqwest::Response`. | tracing, error-handling, github |
| `record_status` | fn | `merge_warden_developer_platforms::github` | `(field: &str, value: u16)` — records an HTTP status code onto a named field (conventionally `status`, `pr_fetch_status`, `properties_status`, or `projects_status`) of the *current* tracing span. Call after every GitHub API call inside a `#[instrument]`-annotated `GitHubProvider` method that declared the field as `tracing::field::Empty`. Consolidates ~55 previously-duplicated `tracing::Span::current().record(...)` call sites. | tracing, span, github |

## `merge_warden_core` — observability

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `MetricsRecorder` | trait | `merge_warden_core` | Port trait (`Send + Sync + Debug`) for reporting `pr.validation.duration_ms` and `pr.bypass.activations_total` without introducing an OTel/infrastructure dependency into `crates/core`. Inject a real implementation via `MergeWarden::with_metrics_recorder`; see `merge_warden_server::metrics::CoreMetricsRecorder` for the production adapter. | metrics, trait, port, observability |
| `NoopMetricsRecorder` | type | `merge_warden_core` | No-op `MetricsRecorder` implementation (`Debug, Default, Clone, Copy`); used when `MergeWarden` is constructed without `with_metrics_recorder`. | metrics, noop, observability |

## `merge_warden_server` — observability (metrics / health)

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `Metrics` | struct | `merge_warden_server::metrics` | Cheap-to-`Clone` facade over every OTel instrument merge-warden emits (11 counters/histograms/gauges — webhook, queue, PR-validation, bypass signals). Build once via `Metrics::new(&meter)` from the provider returned by `init_metrics`, thread through `AppState`. Has a self-contained `Default` impl (inert, reader-less provider) for callers constructed before a real provider exists. | metrics, otel, facade |
| `MetricsConfig` | type | `merge_warden_server::metrics` | `MetricsConfig::from_env()` reads `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`/`OTEL_SERVICE_VERSION`, and `MERGE_WARDEN_METRICS_ENDPOINT` (`"prometheus"` enables the scrape endpoint). Never fails. | metrics, config, env |
| `init_metrics` | fn | `merge_warden_server::metrics` | Builds the `SdkMeterProvider`: attaches an OTLP `PeriodicReader` when `otlp_endpoint` is set (mirrors `telemetry::init_telemetry`'s trace pipeline) and/or an `opentelemetry-prometheus` reader when `prometheus_enabled`. Call once at startup. | metrics, otel, startup |
| `metrics_handler` | fn (axum handler) | `merge_warden_server::metrics` | `GET /metrics` — renders the module-private Prometheus registry via `prometheus::TextEncoder`. Only registered on the router when `MetricsConfig::prometheus_enabled`. | metrics, prometheus, axum |
| `CoreMetricsRecorder` | type | `merge_warden_server::metrics` | Production adapter implementing `merge_warden_core::MetricsRecorder` on top of `Metrics`, so `crates/core` stays OTel-free. Inject via `MergeWarden::with_metrics_recorder(Arc::new(CoreMetricsRecorder::new(metrics)))`. | metrics, adapter, core-bridge |
| `HealthState` | enum | `merge_warden_server::health` | `Healthy`/`Degraded`/`Unhealthy`; `.http_status()` maps to `200`/`207`/`503`. Serializes lowercase. Ordered by `severity()` (higher = worse) for aggregation. | health, enum |
| `ComponentHealth` / `HealthReport` | struct | `merge_warden_server::health` | Per-dependency (`ComponentHealth`: status/latency_ms/depth/message) and overall (`HealthReport`: status + `BTreeMap<String, ComponentHealth>`) shapes for `GET /health`'s structured JSON body. | health, json, struct |
| `HealthCheckConfig` | type | `merge_warden_server::health` | `HealthCheckConfig::from_env()` reads `MERGE_WARDEN_HEALTH_CHECKS` (`"full"` enables real dependency probing; anything else is liveness-only). | health, config, env |
| `with_metrics_route` | fn | `merge_warden_server::webhook` | `(router, state) -> Router` — conditionally registers `GET /metrics` on a router when `state.metrics_config.prometheus_enabled`, otherwise returns the router unchanged. Shared by `build_router` and `build_queue_router`. | routing, axum, metrics |
| `otel_service_metadata_from_env` | fn | `merge_warden_server::telemetry` | `pub(crate)`. Reads the three env vars shared by every OTel exporter config in this crate — `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME` (default `"merge-warden"`), `OTEL_SERVICE_VERSION` (default: crate version) — as a `(Option<String>, String, String)` tuple. Backs both `TelemetryConfig::from_env` and `MetricsConfig::from_env`; never fails. | config, env, otel |
| `build_otel_resource` | fn | `merge_warden_server::telemetry` | `pub(crate) (service_name: &str, service_version: &str) -> opentelemetry_sdk::Resource`. Builds the `service.name`/`service.version` OTel resource shared by the trace pipeline (`init_telemetry`) and the metrics pipeline (`metrics::init_metrics`). | otel, resource, startup |

## `merge_warden_server` — test fixtures (`#[cfg(test)]` only)

| Name | Kind | Location | Description | Tags |
|------|------|----------|-------------|------|
| `test_support::github_client_for` | fn | `merge_warden_server::test_support` | `pub(crate)`, test-only. `(base_url: &str) -> GitHubClient`. Builds a test `GitHubClient` whose App auth targets `base_url` — pass `"https://api.github.com"` for routing/wiring tests that never dial out, or an unroutable address (e.g. `"http://127.0.0.1:1"`) for tests that need every call to fail fast. | testing, fixture, github |
| `test_support::test_app_state` / `TestAppStateOptions` | fn / struct | `merge_warden_server::test_support` | `pub(crate)`, test-only. Builds an `Arc<AppState>` for router/handler tests from a `TestAppStateOptions` (`github_base_url`, `prometheus_enabled`, `full_checks_enabled`, `queue`, each defaulted to an inert "webhook mode, nothing enabled" baseline via `Default`). Replaces the near-identical `AppState`-literal helpers previously duplicated across `webhook_tests.rs` and `health_tests.rs`. | testing, fixture, app-state |
