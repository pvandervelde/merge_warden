---
title: "HTTP endpoints reference"
description: "HTTP endpoints exposed by the Merge Warden server."
---

# HTTP endpoints reference

The Merge Warden server exposes up to three HTTP endpoints on the configured port (default
`3000`), depending on `MERGE_WARDEN_RECEIVER_MODE` and `MERGE_WARDEN_METRICS_ENDPOINT`.

- In **`webhook` mode** (the default), `GET /health` and `POST /api/github/webhook` are both
  registered.
- In **`queue` mode**, only `GET /health` is registered from the pair above.
  `POST /api/github/webhook` does not exist in this mode — Merge Warden is a pure queue
  consumer and never receives a webhook payload directly. See
  [Webhook vs queue receiver modes](../explanation/receiver-modes.md).
- **`GET /metrics`** is registered in *either* receiver mode, but only when
  `MERGE_WARDEN_METRICS_ENDPOINT=prometheus` is set. See
  [Environment variables reference](environment-variables.md).

---

## `GET /health` — Health check

Returns structured JSON describing overall server health and the health of individual
dependencies. Use this endpoint for load balancer health probes, container liveness/readiness
checks, and manual verification.

Behaviour depends on `MERGE_WARDEN_HEALTH_CHECKS` (default `basic`):

- **`basic` (default):** Responds immediately without contacting GitHub or the queue broker.
  Every check is reported `healthy`. Use this for fast liveness probes.
- **`full`:** Actively probes GitHub API reachability (an authenticated `GET /app` call,
  bounded by a 5-second timeout) and, in queue mode, checks that a queue client was
  constructed at startup. `config` is always reported `healthy` in both modes — configuration
  is fully validated at startup, before the HTTP server exists, so reaching this handler at
  all already proves it is valid.

**Request:** No headers or body required.

**Response body:**

```json
{
  "status": "healthy",
  "timestamp": "2026-07-30T12:00:00Z",
  "checks": {
    "config": { "status": "healthy" },
    "github_api": { "status": "healthy", "latency_ms": 42 },
    "queue": { "status": "healthy" }
  }
}
```

- `status` and each `checks.*.status` are one of `"healthy"`, `"degraded"`, or `"unhealthy"`.
  The overall `status` is the least-healthy value across all checks.
- `timestamp` is the UTC time (RFC 3339) at which this response was generated. Every request
  runs its checks fresh — nothing is cached — so this is only useful for detecting a stale
  *copy* of a response body (e.g. one saved or forwarded by another system), not for detecting
  staleness of the live endpoint itself.
- `checks.queue` is present **only** when the server is running in queue mode
  (`MERGE_WARDEN_RECEIVER_MODE=queue`).
- `latency_ms` appears only on `github_api`, only in `full` mode, and only once the probe has
  completed (it measures the round trip, whether the probe succeeded or failed).
- A `message` field (short, non-sensitive diagnostic text — never a raw upstream error body)
  appears on a check whenever that check is not `healthy`, **and** on `queue` in `full` mode
  even while `healthy` — it discloses that the `queue` check cannot do a live reachability/depth
  probe (see the next bullet), so `"healthy"` there means only "a queue client was constructed
  at startup", not "the broker was just contacted successfully".
- **No check ever includes a `depth` field.** The `queue` check reports connectivity only —
  see [Queue Health Check Limitations](environment-variables.md#queue-health-check-limitations)
  in the environment variables reference for why a live message count is not available.
- Keys in `checks` are always: `config`, `github_api`, and (queue mode only) `queue` — no
  other component keys are ever present today.

**HTTP status codes:**

| Status | Overall `status` | Meaning |
| :--- | :--- | :--- |
| `200 OK` | `healthy` | All checks passed |
| `207 Multi-Status` | `degraded` | Reserved for a future soft-dependency check; no check in this release currently produces `degraded` |
| `503 Service Unavailable` | `unhealthy` | At least one check (currently: only `github_api`, in `full` mode) failed |

**Example — basic mode (default), webhook mode:**

```bash
curl -i http://localhost:3000/health
# HTTP/1.1 200 OK
# {"status":"healthy","timestamp":"2026-07-30T12:00:00Z","checks":{"config":{"status":"healthy"},"github_api":{"status":"healthy"}}}
```

**Example — full mode, queue mode, GitHub reachable:**

```bash
curl -i http://localhost:3000/health
# HTTP/1.1 200 OK
# {"status":"healthy","timestamp":"2026-07-30T12:00:00Z","checks":{"config":{"status":"healthy"},"github_api":{"status":"healthy","latency_ms":42},"queue":{"status":"healthy","message":"Reports only that a queue client was constructed at startup; queue-runtime 0.2.1 exposes no depth/reachability query API, so this is not a live probe"}}}
```

**Example — full mode, GitHub API unreachable or credentials invalid:**

```bash
curl -i http://localhost:3000/health
# HTTP/1.1 503 Service Unavailable
# {"status":"unhealthy","timestamp":"2026-07-30T12:00:00Z","checks":{"config":{"status":"healthy"},"github_api":{"status":"unhealthy","latency_ms":5003,"message":"GitHub API request timed out"}}}
```

---

## `GET /metrics` — Prometheus metrics

Renders every metric Merge Warden has collected since startup in
[Prometheus text exposition format](https://prometheus.io/docs/instrumenting/exposition_formats/).
Intended for a Prometheus (or Prometheus-compatible, e.g. Grafana Agent, VictoriaMetrics)
scraper to pull on an interval.

**Only registered when `MERGE_WARDEN_METRICS_ENDPOINT=prometheus`.** With any other value (or
unset, the default), this route does not exist and `GET /metrics` returns `404 Not Found`
rather than an empty `200`.

This endpoint is independent of `OTEL_EXPORTER_OTLP_ENDPOINT` — you can run OTLP push export,
the Prometheus pull endpoint, both, or neither. See
[Environment variables reference](environment-variables.md) for the metric names, and
[Monitoring and observability](https://github.com/pvandervelde/merge_warden/blob/master/docs/spec/operations/monitoring.md) for alert-threshold
guidance.

**Request:** No headers or body required.

**Response:**

| Status | Meaning |
| :--- | :--- |
| `200 OK` | Metrics rendered successfully (including an empty body if nothing has been recorded yet) |
| `500 Internal Server Error` | The Prometheus text encoder failed — logged server-side; not expected in normal operation |

The response `Content-Type` is set by the framework's default string response handling
(`text/plain; charset=utf-8`), **not** the `text/plain; version=0.0.4; charset=utf-8` media
type some Prometheus tooling emits by convention. Standard Prometheus scrapers accept plain
`text/plain` for the classic text exposition format, but if you use a scraper or sidecar that
strictly requires the versioned media type, verify compatibility before relying on this
endpoint.

<!-- TODO: confirm with the maintainer whether metrics_handler should explicitly set
     `Content-Type: text/plain; version=0.0.4; charset=utf-8` to match Prometheus convention
     exactly, rather than relying on axum's default `String` response content type. -->

**Example response body (abridged):**

```text
# HELP ingress_webhook_requests_total Total webhook requests received
# TYPE ingress_webhook_requests_total counter
ingress_webhook_requests_total{event_type="pull_request",result="accepted"} 12
# HELP pr_validation_duration_ms Time taken to run all validation rules for a single PR
# TYPE pr_validation_duration_ms histogram
pr_validation_duration_ms_bucket{le="5"} 0
pr_validation_duration_ms_bucket{le="+Inf"} 12
pr_validation_duration_ms_sum 934.2
pr_validation_duration_ms_count 12
# HELP pr_bypass_activations_total Bypass rule activations
# TYPE pr_bypass_activations_total counter
pr_bypass_activations_total{bypass_type="admin_override"} 1
```

Metric names use underscores rather than the dotted form used for OTLP export (e.g.
`ingress_webhook_requests_total` here vs. `ingress.webhook.requests_total` over OTLP) —
this follows standard Prometheus naming convention, not a difference in what is measured.

---

## `POST /api/github/webhook` — GitHub webhook receiver

Receives and processes GitHub webhook events. This is the URL to configure as the
**Webhook URL** in your GitHub App settings.

**Required request headers:**

| Header | Description |
| :--- | :--- |
| `Content-Type` | Must be `application/json` |
| `X-GitHub-Event` | GitHub event type (e.g. `pull_request`, `pull_request_review`) |
| `X-GitHub-Delivery` | Unique delivery ID assigned by GitHub |
| `X-Hub-Signature-256` | HMAC-SHA256 signature of the request body using the webhook secret |

**Request body:** JSON payload as delivered by GitHub. The shape varies by event type and is
defined by the [GitHub webhooks documentation](https://docs.github.com/en/webhooks/webhook-events-and-payloads).

**Response:**

| Status | Meaning |
| :--- | :--- |
| `202 Accepted` | Payload received and queued or processed |
| `400 Bad Request` | Missing required headers or malformed JSON |
| `401 Unauthorized` | HMAC signature verification failed |

**Signature verification:**

Every incoming request is verified using HMAC-SHA256 with the value of
`GITHUB_WEBHOOK_SECRET`. Requests with a missing or invalid signature are rejected with
`401 Unauthorized`. This verification cannot be disabled in production mode.

**Processed events:**

Only `pull_request` and `pull_request_review` event types trigger policy evaluation.
All other event types are acknowledged with `202 Accepted` and discarded.

For `pull_request` events, only the following actions trigger processing:
`opened`, `edited`, `ready_for_review`, `reopened`, `unlocked`, `synchronize`.

If `[policies.repository_scope]` is configured in the application-level config, events for
repositories outside the configured scope are also acknowledged without any further
processing — regardless of event type or action — and without any GitHub API call being
made on behalf of that repository. See
[Configure repository scope filtering](../how-to/configure-repository-scope.md).

---

## Related

- [Environment variables reference](environment-variables.md)
- [GitHub App permissions](github-app-permissions.md)
- [Webhook vs queue receiver modes](../explanation/receiver-modes.md)
- [Configure repository scope filtering](../how-to/configure-repository-scope.md)
- [Monitoring and observability](https://github.com/pvandervelde/merge_warden/blob/master/docs/spec/operations/monitoring.md)
