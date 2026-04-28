---
title: "How to run the server locally"
description: "Build the Docker image and run Merge Warden on your local machine to receive live GitHub webhooks."
---

# How to run the server locally

This guide uses `samples/run-local.ps1` to build the Merge Warden container image, start it
locally, and relay live GitHub webhooks to it using [smee.io](https://smee.io).

**Prerequisites:**

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed and running
- [Node.js](https://nodejs.org/) installed (provides `npx` for smee-client)
- [PowerShell](https://github.com/PowerShell/PowerShell) (any platform)
- A GitHub App installed on your test repository — see
  [Tutorial: Your first deployment](../tutorials/01-getting-started.md)

---

## 1 — Create a smee channel

1. Open [smee.io](https://smee.io) and click **Start a new channel**.
2. Copy the URL shown (e.g. `https://smee.io/AbCdEfGhIj123456`).
3. In your GitHub App settings, set the **Webhook URL** to this smee URL.

---

## 2 — Load credentials into your shell

```powershell
$env:MERGE_WARDEN_GITHUB_APP_ID          = "123456"
$env:MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY = Get-Content "path\to\app.private-key.pem" -Raw
$env:GITHUB_WEBHOOK_SECRET               = "your-webhook-secret"
```

---

## 3 — Run the script

From the repository root:

```powershell
# Build image, start server, begin relaying webhooks
.\samples\run-local.ps1 -SmeeUrl "https://smee.io/AbCdEfGhIj123456"
```

To apply an application-level policy file (see
[Set application-level defaults](set-app-level-defaults.md)):

```powershell
.\samples\run-local.ps1 `
  -SmeeUrl "https://smee.io/AbCdEfGhIj123456" `
  -AppConfigFile ".\samples\app-config.sample.toml"
```

On subsequent runs, skip the Docker build to save time:

```powershell
.\samples\run-local.ps1 -SmeeUrl "https://smee.io/AbCdEfGhIj123456" -SkipBuild
```

Wait until the script reports **"Server is ready"** and **"Relaying webhooks"**.

---

## 4 — Verify the server

```powershell
Invoke-RestMethod http://localhost:3000/api/merge_warden
# Returns: HTTP 200 OK
```

---

## 5 — Trigger a webhook event

Open or update a pull request in your test repository. The smee-client output will show each
incoming event, and the server logs (visible in the Docker container output) will show the
processing details.

---

## Script parameters

| Parameter | Required | Default | Description |
| :--- | :---: | :--- | :--- |
| `-SmeeUrl` | Yes | — | smee.io channel URL; must match your GitHub App webhook URL |
| `-Port` | No | `3000` | Host port to bind the server on |
| `-ImageTag` | No | `merge-warden-server:local` | Docker image tag to build and run |
| `-AppConfigFile` | No | — | Path to TOML app-level defaults file; mounted into the container as `MERGE_WARDEN_CONFIG_FILE`. Must use `ApplicationDefaults` field names — not the per-repo format. |
| `-SkipBuild` | No | off | Skip `docker build` and use the existing local image |

---

## How the smee relay works

smee-client subscribes to your smee channel using Server-Sent Events. When GitHub delivers
a webhook to the smee URL, smee stores it and the client forwards it — headers and body
verbatim — to your local server. The original HMAC-SHA256 signature is forwarded unchanged,
so the server's signature validation passes correctly.

For more background, see [Test webhooks with smee relay](test-with-smee.md).

---

## Related

- [Test webhooks with smee relay](test-with-smee.md)
- [Environment variables reference](../reference/environment-variables.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
