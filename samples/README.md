# Samples

This directory contains sample files and scripts for working with merge-warden-server.

| File | Purpose |
|---|---|
| `app-config.sample.toml` | Annotated **app-level** policy defaults (loaded via `MERGE_WARDEN_CONFIG_FILE`) |
| `merge-warden.sample.toml` | Annotated **per-repository** policy configuration (placed at `.github/merge-warden.toml` in each repo) |
| `run-local.ps1` | PowerShell script to run the server locally and test it with live GitHub webhooks |

---

## app-config.sample.toml — app-level defaults

This file is loaded by the server via the `MERGE_WARDEN_CONFIG_FILE` environment
variable. It defines the **application-level** policy defaults that apply to every
repository handled by this server instance.

> **Important:** This file uses different field names to `merge-warden.sample.toml`.
> `MERGE_WARDEN_CONFIG_FILE` expects `ApplicationDefaults` fields (e.g.
> `enforceTitleValidation`, `titlePattern`). The per-repo config uses a different
> structure (`[policies.pullRequests.prTitle]`). Pointing `MERGE_WARDEN_CONFIG_FILE`
> at the per-repo sample will silently produce no enforcement.

Pass it to `run-local.ps1` with:

```powershell
.\samples\run-local.ps1 -Repo "owner/repo" -AppConfigFile ".\samples\app-config.sample.toml"
```

---

## merge-warden.sample.toml — per-repository config

This is the per-repository configuration file. When merge-warden processes a pull
request event it fetches `.github/merge-warden.toml` from the target repository
via the GitHub API and uses it to determine which policies to enforce.

To use it in your test repository:

```powershell
# Copy to your test repo
Copy-Item samples\merge-warden.sample.toml path\to\your-test-repo\.github\merge-warden.toml
```

Then commit it on the default branch. Subsequent PR events will pick up the
configuration automatically — no server restart required.

> **Note:** If a repository has no `.github/merge-warden.toml`, the server falls
> back to compiled-in defaults. Both title and work-item validation are **disabled**
> by default, so the bot will run without enforcing any policies.

---

## run-local.ps1 — local development script

Builds the Docker image, starts the server, and forwards live GitHub webhook events
to it using the GitHub CLI. Press Ctrl+C to stop; the container is cleaned up
automatically.

### Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (running)
- [Node.js](https://nodejs.org/) — provides `npx` to run `smee-client` on demand. Or install globally once:

  ```powershell
  npm install --global smee-client
  ```

### GitHub App setup

Create a GitHub App (Settings → Developer settings → GitHub Apps → New GitHub App)
with the following permissions:

| Permission | Level |
|---|---|
| Pull requests | Read & Write |
| Issues | Read |
| Contents | Read |
| Checks | Write |
| Metadata | Read (required) |

The following **organisation** permission is also required (listed under *Organization permissions*, not *Repository permissions*):

| Organisation Permission | Level |
|---|---|
| Projects | Read & Write |

> **Note:** Projects v2 propagation only works for repositories belonging to a **GitHub organisation**. Personal repositories cannot use the Projects v2 GraphQL API with GitHub App tokens, so `sync_project_from_issue` will have no effect when the repository is owned by an individual user.

Subscribe to events: **Pull request**, **Pull request review**.

1. Create a free channel at [smee.io](https://smee.io) and copy the URL (e.g. `https://smee.io/abc123`).
2. Set your GitHub App's **Webhook URL** to that smee URL.
3. Set a **Webhook secret** and note the value.
4. Download the **private key** PEM and install the App on your test repository.

### Usage

```powershell
# 1. Load credentials into the current session
$env:GITHUB_APP_ID          = "123456"           # numeric App ID from GitHub App settings
$env:GITHUB_APP_PRIVATE_KEY = Get-Content "path\to\app.private-key.pem" -Raw
$env:GITHUB_WEBHOOK_SECRET  = "your-webhook-secret"

# 2. Run (builds image, starts container, begins relaying)
.\samples\run-local.ps1 -SmeeUrl "https://smee.io/abc123"

# Supply an app-level config file (mounted into the container as MERGE_WARDEN_CONFIG_FILE)
.\samples\run-local.ps1 -SmeeUrl "https://smee.io/abc123" -AppConfigFile ".\samples\app-config.sample.toml"

# On subsequent runs you can skip the Docker build
.\samples\run-local.ps1 -SmeeUrl "https://smee.io/abc123" -SkipBuild
```

### Parameters

| Parameter | Required | Default | Description |
|---|---|---|---|
| `-SmeeUrl` | Yes | — | smee.io channel URL (must match your GitHub App's Webhook URL) |
| `-Port` | No | `3000` | Host port to bind the server on |
| `-ImageTag` | No | `merge-warden-server:local` | Docker image tag to build and run |
| `-AppConfigFile` | No | — | Path to an app-level TOML defaults file (e.g. `samples/app-config.sample.toml`). Mounted into the container and set as `MERGE_WARDEN_CONFIG_FILE`. Applies app-level policy defaults to all repositories that have no per-repo `.github/merge-warden.toml`. Must use `ApplicationDefaults` field names — **not** the per-repo format. |
| `-SkipBuild` | No | off | Skip `docker build` and use the existing local image |

### Testing the running server

Once the script reports *"Server is ready"* and *"Relaying webhooks"*:

```powershell
# Health check
Invoke-RestMethod http://localhost:3000/api/merge_warden
# Returns: HTTP 200 OK

# Trigger a real event — open or update a PR in your test repo
# The smee-client output will show each incoming event
# The server logs (via docker logs <container-id>) show processing details
```

### How smee relay works

`smee-client` subscribes to your smee channel using Server-Sent Events (SSE).
When GitHub delivers a webhook to the smee URL, smee stores it and the client
forwards it — headers and body verbatim — to the local server. The original
HMAC-SHA256 signature computed by GitHub is forwarded unchanged, so the server's
signature validation passes correctly. Unlike `gh webhook forward`, no re-signing
occurs and the full App webhook payload — including the `installation` object —
is preserved.
3. Open an issue in the [merge_warden repository](https://github.com/pvandervelde/merge_warden/issues)
