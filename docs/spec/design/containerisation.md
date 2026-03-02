# Design: Containerisation

## Status

Draft — awaiting implementation (Phase 2.0)

## Context

The current `crates/azure-functions` binary is **already an Axum HTTP server** — it
does not use the Azure Functions worker SDK. However, startup wires two Azure-specific
services:

1. **Azure Key Vault** (`get_azure_secrets`) — retrieves `GithubAppId`,
   `GithubAppPrivateKey`, `GithubWebhookSecret` via Managed Identity.
2. **Azure App Configuration** (`get_application_config`) — retrieves
   `ApplicationDefaults` (default policy values).

These two calls are the only things preventing the binary from running outside Azure.
The containerisation task replaces both with environment-variable injection, renames
the crate to `server`, and adds a `Dockerfile` that publishes to GitHub Container
Registry (`ghcr.io`).

**No HTTP framework change is needed.** Axum remains the host.

---

## Goals

- Binary runs on Azure Container Apps, AWS ECS/Fargate, and any OCI-compatible host
- Secrets injected as environment variables (cloud-native pattern for secrets in
  containers — AWS injects Secrets Manager values as env vars; Azure Container Apps
  supports env var references to Key Vault secrets)
- Configuration injected as environment variables or mounted config file
- No Azure SDK dependencies in the binary for runtime secrets/config (Azure SDK deps
  are acceptable for optional local dev tooling)
- Single `Dockerfile`, published to `ghcr.io/pvandervelde/merge_warden/server`
- CI pipeline builds and pushes the image on release

---

## What Changes

### Crate rename: `azure-functions` → `server`

- Directory: `crates/server/`
- Crate name in `Cargo.toml`: `merge_warden_server`
- CHANGELOG.md carries over; add entry noting the rename
- All CI references to `crates/azure-functions` updated

### Secret injection: Key Vault → environment variables

**Current (`get_azure_secrets`):**

```rust
let app_id  = get_secret_from_keyvault(url, "GithubAppId").await?;
let key     = get_secret_from_keyvault(url, "GithubAppPrivateKey").await?;
let secret  = get_secret_from_keyvault(url, "GithubWebhookSecret").await?;
```

**Replacement:** read directly from environment at startup.

| Secret | Environment variable |
|---|---|
| GitHub App ID | `GITHUB_APP_ID` |
| GitHub App private key (PEM) | `GITHUB_APP_PRIVATE_KEY` (inline PEM string) |
| GitHub webhook secret | `GITHUB_WEBHOOK_SECRET` |

The entire `app_config_client.rs`, `get_secret_from_keyvault`, and associated Azure
SDK dependencies (`azure-identity`, `azure-security-keyvault-secrets`,
`azure-core`) are removed from this crate.

Container platforms inject these as follows:

| Platform | Mechanism |
|---|---|
| Azure Container Apps | Environment variable referencing Key Vault secret |
| AWS ECS | Task definition secrets from Secrets Manager → injected as env vars |
| Local / Docker run | `--env` or `--env-file` |
| GitHub Actions | `env:` block referencing repository secrets |

No abstraction layer is needed — the container contract is simply "these env vars
must be present at startup". If any required variable is absent, the binary prints
a clear error and exits with code 1.

### Configuration injection: App Configuration → environment variables + TOML file

**Current (`get_application_config`):**
Calls Azure App Configuration via Managed Identity to retrieve `ApplicationDefaults`.

**Replacement:**
`ApplicationDefaults` is loaded from, in priority order:

1. A TOML config file at the path given by `MERGE_WARDEN_CONFIG_FILE` (mounted
   volume in container deployments, or CLI `--config` flag)
2. Individual environment variables that override specific fields
3. Compiled-in defaults (identical to current `ApplicationDefaults::default()`)

This matches how `cli` currently loads configuration and reuses the existing
`ConfigSource` / TOML loading path. The `AppConfigClient` module is removed.

| Configuration area | Environment variable prefix |
|---|---|
| Server listen port | `MERGE_WARDEN_PORT` (default: `3000`) |
| Log level | `RUST_LOG` (existing) |
| OTLP endpoint | `OTEL_EXPORTER_OTLP_ENDPOINT` (standard OTLP env var) |
| Policy defaults | `MERGE_WARDEN_*` prefix (details in interface design) |

### Telemetry: add optional OTLP layer

`telemetry.rs` already uses `tracing` with console output — this is cloud-portable.
Add an optional OpenTelemetry OTLP export layer activated by the standard
`OTEL_EXPORTER_OTLP_ENDPOINT` environment variable:

```
tracing subscriber stack:
  console layer (fmt, always on)
  + OTLP/gRPC layer (on when OTEL_EXPORTER_OTLP_ENDPOINT is set)
```

**Azure**: Azure Monitor OpenTelemetry Distro accepts OTLP — no Application
Insights SDK required.
**AWS**: AWS Distro for OpenTelemetry (ADOT) collector accepts OTLP and forwards
to CloudWatch.
**Local**: omit `OTEL_EXPORTER_OTLP_ENDPOINT`; console output only.

Dependencies to add (feature-flagged is acceptable but not required):

- `opentelemetry` + `opentelemetry-otlp`
- `tracing-opentelemetry`

### Receiver mode: env-var-driven at startup

The binary supports two receiver modes (see also task 0.3):

| Mode | `MERGE_WARDEN_RECEIVER_MODE` | Description |
|---|---|---|
| `webhook` (default) | `webhook` | Axum HTTP handler processes events inline |
| `queue` | `queue` | Axum HTTP handler enqueues; separate Tokio task processes |

At most one mode is active per process. The mode is selected once at startup and cannot
change without restart. This keeps a single binary while supporting both deployment
topologies.

---

## What Does Not Change

- Axum as the HTTP server framework
- Route structure: `GET /api/merge_warden` (health), `POST /api/merge_warden` (webhook)
- `core` crate — zero changes
- `developer_platforms` crate — internal SDK migration is task 0.1; public traits unchanged
- `cli` crate — zero changes

---

## Dockerfile

Single-stage build (or multi-stage with builder + distroless/scratch runtime):

```dockerfile
# Build stage
FROM rust:1.90-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p merge_warden_server

# Runtime stage
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/merge_warden_server /merge_warden_server
EXPOSE 3000
ENTRYPOINT ["/merge_warden_server"]
```

**Health check** (for container orchestrators):

```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["/merge_warden_server", "--health-check"]
```

Or rely on the existing `GET /api/merge_warden` → 200 OK as the health probe URL
(preferred — no extra binary flag needed).

---

## CI: GitHub Actions Release Workflow

The repository already has two release workflows:

- `.github/workflows/prepare-release.yml` — runs on push to `master`; calculates
  the next version, generates the changelog, and opens a release PR.
- `.github/workflows/publish-release.yml` — runs when that release PR is merged;
  creates the git tag and GitHub Release.

The Docker image build and push is added to **`publish-release.yml`**, after the
GitHub Release step. The release version is already available as
`steps.get_version.outputs.RELEASE_VERSION`.

Two changes are required to `publish-release.yml`:

1. Add `packages: write` to the `permissions` block (required to push to GHCR):

```yaml
permissions:
  contents: write
  packages: write
```

1. Append these steps after `Create GitHub Release`:

```yaml
- name: Log in to GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}

- name: Build and push container image
  uses: docker/build-push-action@v5
  with:
    context: .
    file: crates/server/Dockerfile
    push: true
    tags: |
      ghcr.io/pvandervelde/merge_warden/server:latest
      ghcr.io/pvandervelde/merge_warden/server:${{ steps.get_version.outputs.RELEASE_VERSION }}
```

No changes are needed to `prepare-release.yml`.

---

## Startup Sequence After Migration

```
main()
  │
  ├── init_logging()              telemetry.rs (console + optional OTLP)
  │
  ├── read_env_secrets()          GITHUB_APP_ID, GITHUB_APP_PRIVATE_KEY,
  │                               GITHUB_WEBHOOK_SECRET  (fail-fast if missing)
  │
  ├── load_config()               MERGE_WARDEN_CONFIG_FILE or env vars or defaults
  │
  ├── init_github_client()        GitHubClient::builder(auth).build()  (SDK — task 0.1)
  │
  ├── match MERGE_WARDEN_RECEIVER_MODE
  │     "webhook" → start Axum only
  │     "queue"   → start Axum + spawn queue processor task  (task 0.3)
  │
  └── axum::serve(listener, router).await
```

---

## Responsibilities

### `server::config` module

**Knows:** all environment variable names; parsed `ApplicationDefaults`; TOML config path.
**Does:** reads env vars + optional TOML at startup; produces fully-populated config struct;
fails fast with clear error if required secrets are absent.

### `server::telemetry` module (extended from current `telemetry.rs`)

**Knows:** `OTEL_EXPORTER_OTLP_ENDPOINT` presence.
**Does:** initialises tracing subscriber stack; adds OTLP layer when endpoint configured.

### `server::webhook` module (from current `main.rs` handlers)

**Knows:** `AppState` (GitHub client, config, webhook secret).
**Does:** validates signature (SDK), dispatches `EventEnvelope` to event pipeline.

---

## Behavioral Assertions

1. **Missing required env var must cause immediate exit**
   - Given: `GITHUB_APP_ID` not set
   - When: binary starts
   - Then: logs clear error, exits with code 1 before binding the port

2. **Absent TOML config file must fall back to defaults, not error**
   - Given: `MERGE_WARDEN_CONFIG_FILE` not set and file absent
   - When: `load_config()` runs
   - Then: `ApplicationDefaults::default()` used; log at INFO level

3. **Health endpoint must respond 200 before processing any webhook**
   - Given: binary has started and is listening
   - When: `GET /api/merge_warden`
   - Then: HTTP 200 OK

4. **OTLP layer must be inactive when endpoint env var is absent**
   - Given: `OTEL_EXPORTER_OTLP_ENDPOINT` not set
   - When: binary starts
   - Then: no OTLP connection attempted; console logging only

5. **Docker image must pass health check within 30s of start**
   - Given: all required env vars set; no network access needed for startup
   - When: container starts
   - Then: `GET /api/merge_warden` returns 200 within health check window

---

## Testing Strategy

- **Unit tests**: `server::config` — test each required/optional env var, missing required,
  partial TOML, env override of TOML field
- **Unit tests**: `server::telemetry` — test OTLP layer activation/deactivation; no real
  OTLP endpoint required (mock or skip network)
- **Integration test**: build the Docker image in CI with `docker build`; run with
  `docker run --health-cmd` and assert health check passes
- **Existing tests**: all current integration tests must pass unchanged after rename

---

## Dependencies Removed

From `crates/azure-functions` / new `crates/server`:

- `azure-identity`
- `azure-security-keyvault-secrets`
- `azure-core` (unless transitively required by something else)
- `azure-data-appconfig` (used in `app_config_client.rs`)

Files deleted:

- `crates/server/src/app_config_client.rs`
- `crates/server/src/app_config_client_tests.rs`
- `get_secret_from_keyvault` function
- `get_azure_secrets` function
- `get_application_config` function
