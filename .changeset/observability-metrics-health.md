---
"merge_warden_server": minor
---

Added an OTLP metrics pipeline, an optional Prometheus scrape endpoint, and a structured,
dependency-aware `GET /health` endpoint, alongside new `tracing` spans on GitHub API calls
and PR validation rules.

**What operators get:**

- Eleven OpenTelemetry metrics (webhook request counts and durations, queue processing
  duration, DLQ count, worker errors, processing success rate, PR validation duration, and
  bypass activation counts) are now exported over OTLP whenever `OTEL_EXPORTER_OTLP_ENDPOINT`
  is set — the same variable that already enabled trace export. Three queue-specific gauges
  (`ingress.queue.enqueue_duration_ms`, `ingress.queue.depth`,
  `ingress.queue.age_oldest_message_secs`) are defined for forward compatibility but are not
  yet populated by this binary — see the docs for why.
- A new optional Prometheus-format scrape endpoint at `GET /metrics`, enabled by setting
  `MERGE_WARDEN_METRICS_ENDPOINT=prometheus`. Off by default; the route does not exist
  (`404`) unless explicitly enabled.
- `GET /health` now returns structured JSON with a per-dependency breakdown
  (`github_api`, `config`, and — queue mode only — `queue`) instead of a bare `200`. HTTP
  status now reflects overall health: `200` healthy, `207` degraded, `503` unhealthy. By
  default (`MERGE_WARDEN_HEALTH_CHECKS=basic`), behaviour is unchanged from before — a fast,
  dependency-free liveness check. Set `MERGE_WARDEN_HEALTH_CHECKS=full` to make `/health`
  actively probe GitHub API reachability and queue client connectivity.
- GitHub API calls and PR validation rule evaluation are now individually traced with
  `#[tracing::instrument]` spans (including `endpoint`/status and `rule_name` attributes), so
  OTLP traces from a webhook-to-validation flow are no longer empty.

**How to turn it on:** set `MERGE_WARDEN_METRICS_ENDPOINT=prometheus` for the Prometheus
endpoint and/or `MERGE_WARDEN_HEALTH_CHECKS=full` for dependency-probing health checks. Both
are opt-in and default to their previous (off / basic) behaviour, so existing deployments are
unaffected until these variables are set.

See [Environment variables reference](docs/user/reference/environment-variables.md) and
[HTTP endpoints reference](docs/user/reference/http-endpoints.md) for full details.
