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
  POST /api/merge_warden
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

The server receives the incoming webhook, verifies the HMAC signature, and enqueues the
payload. A separate consumer task processes events from the queue asynchronously.

```
GitHub
  POST /api/merge_warden
      ↓
  HMAC verification
      ↓
  Payload enqueued
      ↓
  202 Accepted returned to GitHub immediately

  (separately)
  Consumer reads from queue
      ↓
  Policy evaluation
      ↓
  GitHub API calls
```

**Characteristics:**

- The HTTP response to GitHub is returned as soon as the payload is enqueued — before any
  policy evaluation occurs.
- Useful when processing a large volume of PR events simultaneously. In webhook mode each
  request ties up a server thread until processing completes; concentrated bursts can
  exhaust available threads and cause the server to drop incoming requests. Queue mode
  decouples receipt from processing so bursts are absorbed by the queue.
- Useful for bursty traffic patterns (e.g. many PRs opened at the same time during a
  release freeze lift) where webhook mode could be momentarily overwhelmed.
- Requires additional infrastructure for the queue.
- Processing results (labels, check runs) appear on the pull request with a slight delay.

---

## Which mode to choose

| Situation | Recommended mode |
| :--- | :--- |
| Standard deployment with normal PR event rates | `webhook` |
| Processing consistently takes more than a few seconds | `queue` |
| High PR volume or bursty traffic (e.g. many PRs opened simultaneously) | `queue` |
| You cannot afford to lose events if the server is briefly overloaded | `queue` |
| You want the simplest possible deployment | `webhook` |
| You need resilience against transient downstream failures | `queue` |

Most deployments should use `webhook` mode. Switch to `queue` only if you observe GitHub
reporting webhook delivery timeouts in the App settings.

---

## Configuration

Set the mode via environment variable:

```bash
# Default — no setting needed
MERGE_WARDEN_RECEIVER_MODE=webhook

# Queue mode
MERGE_WARDEN_RECEIVER_MODE=queue
```

---

## Related

- [Environment variables reference](../reference/environment-variables.md)
- [HTTP endpoints reference](../reference/http-endpoints.md)
