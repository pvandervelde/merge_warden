---
title: "HTTP endpoints reference"
description: "HTTP endpoints exposed by the Merge Warden server."
---

# HTTP endpoints reference

The Merge Warden server exposes two HTTP endpoints on the configured port (default `3000`).

---

## `GET /api/merge_warden` — Health check

Returns `200 OK` when the server is running and ready to accept requests. Use this endpoint
for load balancer health probes, container readiness checks, and manual verification.

**Request:** No headers or body required.

**Response:**

| Status | Meaning |
| :--- | :--- |
| `200 OK` | Server is healthy |

**Example:**

```bash
curl -i http://localhost:3000/api/merge_warden
# HTTP/1.1 200 OK
```

---

## `POST /api/merge_warden` — GitHub webhook receiver

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

---

## Related

- [Environment variables reference](environment-variables.md)
- [GitHub App permissions](github-app-permissions.md)
- [Webhook vs queue receiver modes](../explanation/receiver-modes.md)
