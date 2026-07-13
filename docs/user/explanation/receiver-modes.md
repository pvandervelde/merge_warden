---
title: "Webhook vs queue receiver modes"
description: "An explanation of the two event receiver modes and when to choose each one."
---

# Webhook vs queue receiver modes

Merge Warden supports two modes for receiving GitHub webhook events, controlled by the
`MERGE_WARDEN_RECEIVER_MODE` environment variable.

---

## `webhook` mode (default)

GitHub sends a webhook POST directly to the server. The event is verified, processed, and
the HTTP response is returned — all within the same request.

```
GitHub
  POST /api/github/webhook
      ↓
  HMAC verification
      ↓
  202 Accepted returned to GitHub  ← response sent here
      ↓
  Policy evaluation
      ↓
  GitHub API calls (labels, check runs, comments)
```

**Characteristics:**

- Simple to operate — no queue infrastructure required.
- The `202 Accepted` response is returned immediately after HMAC verification, before any
  policy evaluation. This keeps the server well within GitHub's 10-second webhook delivery
  timeout.
- If the server crashes or is overloaded after accepting a request but before finishing
  processing, the event may be lost. GitHub considers the webhook delivered as soon as it
  receives the 202 response.
- Suitable for most deployments where event volume is moderate and steady.

---

## `queue` mode

> **This is a two-service architecture, not a single-container mode switch.**
> In `queue` mode, the Merge Warden server itself becomes a **pure queue
> consumer**. It does **not** expose a webhook POST endpoint — only
> `GET /health` is registered. A separate, independent service must receive
> the GitHub webhook, verify the `X-Hub-Signature-256` HMAC signature, and
> enqueue the payload. Do not point GitHub's webhook URL at the Merge Warden
> container when running in `queue` mode — there is nothing there to receive
> it.

```
Separate receiver service (not Merge Warden)
  GitHub
    POST <receiver's webhook URL>
        ↓
    HMAC verification
        ↓
    Payload enqueued
        ↓
    202 Accepted returned to GitHub

                    ↓ (queue)

Merge Warden container (MERGE_WARDEN_RECEIVER_MODE=queue)
  Only route registered: GET /health
      ↓
  Consumer reads from queue
      ↓
  Policy evaluation
      ↓
  GitHub API calls
```

**Characteristics:**

- Merge Warden's own HTTP server exposes only the health-check endpoint in
  this mode — no `/api/github/webhook` route is registered. It never
  validates a webhook signature and never receives a webhook payload
  directly.
- `GITHUB_WEBHOOK_SECRET` is not used by Merge Warden in this mode —
  signature validation is the separate receiver service's responsibility. See
  [Environment variables reference](../reference/environment-variables.md).
- You must operate (or otherwise provision) the separate receiver service
  yourself. It is responsible for accepting the GitHub webhook POST, verifying
  the HMAC signature, and publishing a message to the queue in the format the
  configured queue provider expects.
- Useful when processing a large volume of PR events simultaneously. In webhook
  mode each request ties up a server thread until processing completes;
  concentrated bursts can exhaust available threads and cause the server to
  drop incoming requests. Queue mode decouples receipt from processing so
  bursts are absorbed by the queue.
- Useful for bursty traffic patterns (e.g. many PRs opened at the same time
  during a release freeze lift) where webhook mode could be momentarily
  overwhelmed.
- Requires additional infrastructure: the queue itself (Azure Service Bus, AWS
  SQS, or an in-memory queue for local testing only) and the separate receiver
  service.
- Processing results (labels, check runs) appear on the pull request with a
  slight delay relative to webhook mode.

See [How to run Merge Warden in queue mode](../how-to/run-in-queue-mode.md) for
setup details, including what the receiver service must do and the queue
provider environment variables.

---

## Which mode to choose

| Situation | Recommended mode |
| :--- | :--- |
| Standard deployment with normal PR event rates | `webhook` |
| Processing consistently takes more than a few seconds | `queue` |
| High PR volume or bursty traffic (e.g. many PRs opened simultaneously) | `queue` |
| You cannot afford to lose events if the server is briefly overloaded | `queue` |
| You want the simplest possible deployment (single container, no extra service) | `webhook` |
| You need resilience against transient downstream failures | `queue` |
| You are not able to stand up and operate a separate receiver service | `webhook` |

Most deployments should use `webhook` mode. Switch to `queue` only if you observe GitHub
reporting webhook delivery timeouts in the App settings, and only if you can operate the
additional receiver service that queue mode requires.

---

## Configuration

Set the mode via environment variable:

```bash
# Default — no setting needed
MERGE_WARDEN_RECEIVER_MODE=webhook

# Queue mode — also requires MERGE_WARDEN_QUEUE_PROVIDER and, depending on
# provider, additional variables. See the environment variables reference and
# the queue-mode how-to guide linked above.
MERGE_WARDEN_RECEIVER_MODE=queue
```

---

## Related

- [How to run Merge Warden in queue mode](../how-to/run-in-queue-mode.md)
- [Environment variables reference](../reference/environment-variables.md)
- [HTTP endpoints reference](../reference/http-endpoints.md)
