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
| `OTEL_EXPORTER_OTLP_ENDPOINT` | *(none)* | OTLP HTTP endpoint URL. When set, structured traces are exported to this collector. When unset, traces are written to stdout only. |
| `OTEL_SERVICE_NAME` | `merge-warden` | Service name reported in traces and spans. |
| `OTEL_SERVICE_VERSION` | *(binary version)* | Service version reported in traces. Defaults to the compiled-in binary version. |

---

## Notes

- `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY` must be the full PEM content as a multi-line string,
  not a file path. When using shell expansion, use `$(cat /path/to/key.pem)` to inline
  the file.
- The **CLI binary** uses `MERGE_WARDEN_LOG` instead of `RUST_LOG` for its log level.
  All other environment variables above apply only to the server container.
- Setting `RUST_LOG=debug` or `RUST_LOG=trace` significantly increases log volume. Use these
  levels only for troubleshooting.
- When `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, no external trace export occurs even if
  other `OTEL_*` variables are present.

---

## Related

- [HTTP endpoints](http-endpoints.md)
- [Deploy on Azure](../how-to/deploy-on-azure.md)
- [Deploy on AWS](../how-to/deploy-on-aws.md)
- [Set application-level defaults](../how-to/set-app-level-defaults.md)
- [Run Merge Warden in queue mode](../how-to/run-in-queue-mode.md)
- [Webhook vs queue receiver modes](../explanation/receiver-modes.md)
