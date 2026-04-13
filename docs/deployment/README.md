# Merge Warden Deployment Guide

Merge Warden is distributed as an OCI container image published to GitHub Container
Registry (GHCR). The same image runs on any OCI-compatible container host.

## Container Image

```
ghcr.io/pvandervelde/merge-warden-server:latest
ghcr.io/pvandervelde/merge-warden-server:<version>
```

Images are built from `crates/server/Dockerfile` and published automatically on each
release via the [publish-release](.../../.github/workflows/publish-release.yml)
workflow. Both `linux/amd64` and `linux/arm64` manifests are published in the same
multi-arch image so the same tag works on x86-64 servers, AWS Graviton, and Apple
Silicon dev containers.

The runtime image is based on `gcr.io/distroless/cc-debian12` — it contains only the
compiled binary and the minimal shared-library set (libc, libgcc). There is no shell
or package manager.

---

## Environment Variables

All configuration is injected via environment variables. The binary fails fast with a
clear error message if any required variable is absent.

### Required — GitHub App secrets

| Variable | Description |
|---|---|
| `GITHUB_APP_ID` | Numeric GitHub App ID |
| `GITHUB_APP_PRIVATE_KEY` | Full PEM-encoded private key (inline string, not a file path) |
| `GITHUB_WEBHOOK_SECRET` | Webhook signing secret configured in GitHub |

### Optional — Server behaviour

| Variable | Default | Description |
|---|---|---|
| `MERGE_WARDEN_PORT` | `3000` | TCP port the HTTP server listens on |
| `MERGE_WARDEN_RECEIVER_MODE` | `webhook` | `webhook` or `queue` |
| `MERGE_WARDEN_CONFIG_FILE` | *(none)* | Path to a TOML policy config file (mounted volume) |

### Optional — Telemetry

| Variable | Default | Description |
|---|---|---|
| `RUST_LOG` | `info` | Log level filter (`error`, `warn`, `info`, `debug`, `trace`) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | *(none)* | OTLP HTTP endpoint — enables structured trace export |
| `OTEL_SERVICE_NAME` | `merge-warden` | Service name reported in traces |
| `OTEL_SERVICE_VERSION` | binary version | Service version reported in traces |

When `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, traces are written to stdout only.

---

## HTTP Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/merge_warden` | Health check — returns `200 OK` |
| `POST` | `/api/merge_warden` | GitHub webhook receiver |

Configure your GitHub App webhook URL to `https://<host>/api/merge_warden` and set
the content type to `application/json`.

---

## Policy Configuration

Application-level policy defaults are loaded from a TOML file. Mount the file into
the container and point `MERGE_WARDEN_CONFIG_FILE` at it.

```toml
# merge-warden.toml  (see samples/merge-warden.sample.toml for full reference)

[policies]
# enforce_conventional_commits = true
```

If no config file is provided, compiled-in defaults apply. Individual repositories can
always override defaults with their own `.github/merge-warden.toml` file.

---

## Quick Start — Docker

```bash
docker run --rm \
  -e GITHUB_APP_ID=12345 \
  -e GITHUB_APP_PRIVATE_KEY="$(cat /path/to/private-key.pem)" \
  -e GITHUB_WEBHOOK_SECRET=supersecret \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

Verify the server is healthy:

```bash
curl http://localhost:3000/api/merge_warden
# HTTP 200
```

---

## Platform-Specific Guides

| Platform | Guide |
|---|---|
| Azure Container Apps | [docs/deployment/azure/README.md](azure/README.md) |
| AWS ECS / Fargate | [docs/deployment/aws/README.md](aws/README.md) |

---

## Receiver Mode

### `webhook` (default)

GitHub sends a webhook POST directly to the server. The HMAC signature is verified
and the event is processed inline before the response is returned.

```
GitHub → POST /api/merge_warden → verify HMAC → process PR → 202 Accepted
```

### `queue`

*(Task 3.0 — not yet implemented)*

The server enqueues the webhook payload for asynchronous processing by a separate
consumer task. Useful when event processing latency exceeds GitHub's 10-second
webhook timeout.

---

## Building the Image Locally

```bash
# From the workspace root
docker build -f crates/server/Dockerfile -t merge_warden_server:local .

# Run with local secrets
docker run --rm \
  -e GITHUB_APP_ID=12345 \
  -e GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e GITHUB_WEBHOOK_SECRET=supersecret \
  -p 3000:3000 \
  merge_warden_server:local
```
