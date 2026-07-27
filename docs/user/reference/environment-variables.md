---
title: "Environment variables reference"
description: "All environment variables accepted by the Merge Warden server container."
---

# Environment variables reference

All server configuration is supplied via environment variables. The binary fails fast with a
clear error message if a required variable is absent.

---

## Required — GitHub App credentials

| Variable | Description |
| :--- | :--- |
| `MERGE_WARDEN_GITHUB_APP_ID` | Numeric GitHub App ID shown on the App settings page |
| `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY` | Full PEM-encoded private key as an inline string (not a file path) |
| `GITHUB_WEBHOOK_SECRET` | Webhook signing secret configured in the GitHub App webhook settings. **Required only in `webhook` receiver mode.** In `queue` mode this variable is not read — Merge Warden never receives a webhook payload directly in that mode, so it never validates a signature. See [Receiver modes](../explanation/receiver-modes.md). |

---

## Optional — Server behaviour

| Variable | Default | Description |
| :--- | :--- | :--- |
| `MERGE_WARDEN_PORT` | `3000` | TCP port the HTTP server listens on |
| `MERGE_WARDEN_RECEIVER_MODE` | `webhook` | Event receiver mode: `webhook` or `queue`. See [Receiver modes](../explanation/receiver-modes.md). |
| `MERGE_WARDEN_CONFIG_FILE` | *(none)* | Absolute path to a TOML application-level policy config file mounted into the container. See [Set application-level defaults](../how-to/set-app-level-defaults.md). |

---

## Required and optional — Queue mode only

These variables are only read when `MERGE_WARDEN_RECEIVER_MODE=queue`. In `queue` mode,
Merge Warden is a pure queue consumer — a separate service is responsible for receiving
the GitHub webhook, verifying its signature, and enqueueing the message. See
[Receiver modes](../explanation/receiver-modes.md) and
[How to run Merge Warden in queue mode](../how-to/run-in-queue-mode.md).

| Variable | Default | Description |
| :--- | :--- | :--- |
| `MERGE_WARDEN_QUEUE_PROVIDER` | *(required)* | Queue backend to consume from: `azure`, `aws`, or `memory` (in-memory — local testing only, not durable). |
| `MERGE_WARDEN_QUEUE_NAME` | `merge-warden-events` | Name of the queue to consume from. |
| `MERGE_WARDEN_QUEUE_CONCURRENCY` | `4` | Maximum number of in-flight messages processed concurrently. |
| `AZURE_SERVICEBUS_NAMESPACE` | *(required if `MERGE_WARDEN_QUEUE_PROVIDER=azure`)* | Azure Service Bus namespace. Authentication uses the default Azure credential chain (managed identity, `az login`, etc.) — there is no connection-string variable. |
| `AWS_REGION` | `us-east-1` | AWS region for SQS, when `MERGE_WARDEN_QUEUE_PROVIDER=aws`. |
| `AWS_ACCESS_KEY_ID` | *(none)* | Optional static AWS credential, when `MERGE_WARDEN_QUEUE_PROVIDER=aws`. Prefer an IAM role (ECS task role, IRSA, instance profile) in production instead of static keys. |
| `AWS_SECRET_ACCESS_KEY` | *(none)* | Optional static AWS credential, paired with `AWS_ACCESS_KEY_ID`. Same production guidance as above. |

Any `MERGE_WARDEN_QUEUE_PROVIDER` value other than `azure`, `aws`, or `memory` fails
startup with a configuration error. The underlying `queue-runtime` library also supports
NATS and RabbitMQ, but Merge Warden does not yet expose `nats`/`rabbitmq` as provider
values — this is planned, not currently available.

---

## Optional — Telemetry

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info` | Log level filter for the **server container**. Accepted values: `error`, `warn`, `info`, `debug`, `trace`. Can be scoped per module (e.g. `merge_warden=debug`). |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | *(none)* | OTLP HTTP endpoint URL. When set, structured traces **and OTLP metrics** are exported to this collector. When unset, traces are written to stdout only and no OTLP metrics export occurs (the optional Prometheus endpoint below is independent of this variable). The value is used as-is for both signals — this binary does not append `/v1/traces` or `/v1/metrics` for you; include the full path your collector expects (e.g. `http://localhost:4318/v1/traces` and, separately, whatever your collector needs for metrics), or point at a collector that auto-appends the signal path. |
| `OTEL_SERVICE_NAME` | `merge-warden` | Service name reported in traces, spans, and metrics resource attributes. |
| `OTEL_SERVICE_VERSION` | *(binary version)* | Service version reported in traces and metrics. Defaults to the compiled-in binary version. |

---

## Optional — Metrics and health checks

| Variable | Default | Description |
| :--- | :--- | :--- |
| `MERGE_WARDEN_METRICS_ENDPOINT` | *(unset)* | Set to `prometheus` to register `GET /metrics`, which renders all collected metrics in Prometheus text exposition format. Unset, or any value other than `prometheus`, leaves the route unregistered entirely — requests to `GET /metrics` then return `404 Not Found`, not an empty `200`. This is independent of `OTEL_EXPORTER_OTLP_ENDPOINT`; you can enable either, both, or neither. |
| `MERGE_WARDEN_HEALTH_CHECKS` | `basic` | Set to `full` to make `GET /health` actively probe dependencies (GitHub API reachability via an authenticated `get_app()` call, and — queue mode only — queue client connectivity). Any other value, or leaving it unset, keeps `/health` in **basic** (liveness-only) mode: it responds immediately without contacting GitHub or the queue broker, and reports every check `healthy`. See [HTTP endpoints reference](http-endpoints.md) for the full response shape and the "Queue Health Check Limitations" note below. |

See the metric names, types, and labels in
[Monitoring and observability](https://github.com/pvandervelde/merge_warden/blob/master/docs/spec/operations/monitoring.md) for what
`OTEL_EXPORTER_OTLP_ENDPOINT` and `MERGE_WARDEN_METRICS_ENDPOINT=prometheus` export.

### Queue Health Check Limitations

In `full` mode, the `queue` check in `GET /health`'s response reports only whether a queue
client was successfully constructed at startup — it does **not** report an actual message
count. The underlying `queue-runtime` client traits Merge Warden depends on do not expose a
depth-query API, so there is currently no data source for a live queue depth or age-of-oldest-message
value in-process. The health check also deliberately never accepts a queue session to probe
it, to avoid stealing a session lock away from a genuine worker task that may be concurrently
processing a PR. Do not configure alerting or dashboards that expect a `depth` field in the
`/health` response — it is not present.

---

## Notes

- `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY` must be the full PEM content as a multi-line string,
  not a file path. When using shell expansion, use `$(cat /path/to/key.pem)` to inline
  the file.
- The **CLI binary** uses `MERGE_WARDEN_LOG` instead of `RUST_LOG` for its log level.
  All other environment variables above apply only to the server container.
- Setting `RUST_LOG=debug` or `RUST_LOG=trace` significantly increases log volume. Use these
  levels only for troubleshooting.
- When `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, no external trace or metrics export occurs
  even if other `OTEL_*` variables are present.
- If `MERGE_WARDEN_HEALTH_CHECKS=full`, be careful about pointing both a liveness probe and a
  readiness/load-balancer health check at `/health`. A transient GitHub API outage will make
  `/health` return `503`, which is appropriate for taking the instance out of load-balancer
  rotation (readiness) but may cause an orchestrator to needlessly restart an otherwise-healthy
  container on a liveness probe, since restarting does not fix an external outage. Consider
  using `basic` mode (the default) for liveness probes and reserving `full` mode for
  readiness/traffic-routing decisions, or accept the restart behaviour if that trade-off is
  acceptable for your deployment.
- Three of the eleven OTLP metrics — `ingress.queue.enqueue_duration_ms`, `ingress.queue.depth`,
  and `ingress.queue.age_oldest_message_secs` — are defined and exported as instruments, but
  nothing in this binary currently calls the code that records values for them: in queue mode,
  Merge Warden is a pure queue *consumer* (see
  [Webhook vs queue receiver modes](../explanation/receiver-modes.md)), so only a separate,
  out-of-repo receiver service observes enqueue latency, and the queue provider SDK exposes no
  depth-query API. Do not configure alerts that expect these three to ever report a non-zero
  or non-default value against the current release.

---

## Related

- [HTTP endpoints](http-endpoints.md)
- [Deploy on Azure](../how-to/deploy-on-azure.md)
- [Deploy on AWS](../how-to/deploy-on-aws.md)
- [Set application-level defaults](../how-to/set-app-level-defaults.md)
- [Run Merge Warden in queue mode](../how-to/run-in-queue-mode.md)
- [Webhook vs queue receiver modes](../explanation/receiver-modes.md)
- [Monitoring and observability](https://github.com/pvandervelde/merge_warden/blob/master/docs/spec/operations/monitoring.md)
