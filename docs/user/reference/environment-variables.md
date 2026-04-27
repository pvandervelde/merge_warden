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
| `GITHUB_WEBHOOK_SECRET` | Webhook signing secret configured in the GitHub App webhook settings |

---

## Optional — Server behaviour

| Variable | Default | Description |
| :--- | :--- | :--- |
| `MERGE_WARDEN_PORT` | `3000` | TCP port the HTTP server listens on |
| `MERGE_WARDEN_RECEIVER_MODE` | `webhook` | Event receiver mode: `webhook` or `queue`. See [Receiver modes](../explanation/receiver-modes.md). |
| `MERGE_WARDEN_CONFIG_FILE` | *(none)* | Absolute path to a TOML application-level policy config file mounted into the container. See [Set application-level defaults](../how-to/set-app-level-defaults.md). |

---

## Optional — Telemetry

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info` | Log level filter. Accepted values: `error`, `warn`, `info`, `debug`, `trace`. Can be scoped per module (e.g. `merge_warden=debug`). |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | *(none)* | OTLP HTTP endpoint URL. When set, structured traces are exported to this collector. When unset, traces are written to stdout only. |
| `OTEL_SERVICE_NAME` | `merge-warden` | Service name reported in traces and spans. |
| `OTEL_SERVICE_VERSION` | *(binary version)* | Service version reported in traces. Defaults to the compiled-in binary version. |

---

## Notes

- `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY` must be the full PEM content as a multi-line string,
  not a file path. When using shell expansion, use `$(cat /path/to/key.pem)` to inline
  the file.
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
