---
title: "How to test webhooks with smee relay"
description: "Use smee.io to forward live GitHub webhooks to a server running on your local machine."
---

# How to test webhooks with smee relay

GitHub cannot reach `localhost` directly, so when testing Merge Warden locally you need a
relay service to forward webhook events. [smee.io](https://smee.io) provides this for free.

---

## How smee relay works

1. You create a public channel at smee.io (a unique URL such as `https://smee.io/AbCdEfGh12`).
2. You configure your GitHub App's webhook URL to point at this smee channel.
3. You run `smee-client` locally. It subscribes to the channel using Server-Sent Events (SSE)
   and forwards any received payload to a local port.
4. GitHub sends a webhook to the smee URL. smee stores the payload and the client immediately
   forwards it — headers and body verbatim — to your local server.

Because the entire HTTP request (including the `X-Hub-Signature-256` header) is forwarded
unchanged, the HMAC-SHA256 signature computed by GitHub arrives at your local server intact
and passes verification. No re-signing occurs.

---

## Create a smee channel

1. Open [smee.io](https://smee.io).
2. Click **Start a new channel**.
3. Copy the channel URL (e.g. `https://smee.io/AbCdEfGhIj123456`).
4. In your GitHub App settings, set **Webhook URL** to this URL.

---

## Run smee-client

### Using npx (no installation required)

```bash
npx smee-client \
  --url https://smee.io/AbCdEfGhIj123456 \
  --target http://localhost:3000/api/merge_warden
```

### Global installation

```bash
npm install --global smee-client
smee --url https://smee.io/AbCdEfGhIj123456 --target http://localhost:3000/api/merge_warden
```

Leave the client running in its own terminal while you test.

---

## Verify

Trigger a webhook by opening a pull request in your test repository. The smee-client
terminal will display something like:

```
Forwarding https://smee.io/AbCdEfGhIj123456 to http://localhost:3000/api/merge_warden
Connected https://smee.io/AbCdEfGhIj123456
POST http://localhost:3000/api/merge_warden - 202
```

The **202** response comes from the local Merge Warden server confirming the event was
received.

---

## Using smee with run-local.ps1

The `samples/run-local.ps1` script handles smee automatically — it starts
`npx smee-client` for you. You only need to supply the channel URL:

```powershell
.\samples\run-local.ps1 -SmeeUrl "https://smee.io/AbCdEfGhIj123456"
```

See [Run the server locally](run-locally.md) for the full walkthrough.

---

## Related

- [Run the server locally](run-locally.md)
- [Tutorial: Your first Merge Warden deployment](../tutorials/01-getting-started.md)
