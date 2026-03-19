# Event Processing Architecture

**Last Updated:** March 2026

This document describes how GitHub webhook events flow through Merge Warden from
reception to action — covering both receiver modes, the processing pipeline, and the
actions taken on a pull request.

## Overview

Merge Warden operates as a GitHub App. GitHub sends webhook POST requests when
pull request events occur. Depending on the receiver mode, this may be sent directly
to merge_warden (webhook mode) or to a separate receiver service (queue mode).
Either way, merge_warden processes the event and responds by updating labels, adding
comments, and setting commit statuses on the PR.

The processing path is split into two stages:

1. **Reception** — validate the webhook signature, acknowledge receipt to GitHub (202),
   and route the event to the processing pipeline
2. **Processing** — load repository configuration, run all validation checks, and apply
   the results to the PR via the GitHub API

These two stages are always decoupled: GitHub receives its 202 before any processing
begins, eliminating timeout risk regardless of how long processing takes.

> **Signature validation is the SDK's responsibility.** In webhook mode the
> `github-bot-sdk` `WebhookReceiver::receive_webhook()` performs HMAC-SHA256
> verification before any merge_warden code runs. In queue mode the separate
> receiver service is responsible for that verification; merge_warden is a pure
> queue consumer and never sees raw webhook payloads.

---

## Receiver Modes

A single environment variable — `MERGE_WARDEN_RECEIVER_MODE` — controls how the
reception and processing stages are connected. Both modes use the same Axum HTTP
server and the same processing pipeline; only the channel between them differs.

### Webhook Mode (default)

```
GitHub
  │
  │  POST /api/merge_warden
  ▼
Axum handler
  │  Calls WebhookReceiver::receive_webhook() [github-bot-sdk]
  │    1. SDK validates HMAC-SHA256 signature against GITHUB_WEBHOOK_SECRET
  │         └─ Invalid signature → 401 returned to GitHub; event dropped
  │    2. SDK parses raw body into EventEnvelope
  │    3. SDK calls ChannelForwardingHandler::handle_event()
  │         └─ Sends envelope to mpsc channel (cap: 64; blocks if full)
  │    4. Returns 202 Accepted  ◄── GitHub receives this immediately
  │
  ▼
WebhookIngress worker task
  │  Reads from channel, wraps in ProcessableEvent { envelope, NoOpAck }
  │
  ▼
run_event_processor loop
  (see Processing Pipeline below)
```

**Key properties:**

- No external infrastructure required
- `GITHUB_WEBHOOK_SECRET` must be set; startup fails without it
- The SDK (`WebhookReceiver`) owns all signature validation logic; merge_warden
  supplies the secret via the `SecretProvider` trait but does not perform any
  cryptographic operations itself
- The mpsc channel provides back-pressure (blocks at capacity 64)
- One worker task; events processed one at a time
- Clean shutdown: when the Axum server stops, senders are dropped, the channel
  drains, and the worker exits via `Ok(None)`

### Queue Mode

In queue mode merge_warden is a **pure queue consumer**. It does not expose a
webhook POST endpoint and never receives raw GitHub payloads. A **separate
receiver service** (outside merge_warden) is responsible for:

1. Receiving the GitHub webhook POST
2. Validating the HMAC-SHA256 signature
3. Serialising the payload into a `WebhookQueueMessage`
4. Enqueuing it with `session_id = "{org}/{repo}/{pr_number}"`
5. Returning 202 to GitHub

Merge Warden then consumes from that queue:

```
[Separate receiver service — outside merge_warden]
  │  1. Receives GitHub POST
  │  2. Validates HMAC-SHA256 signature
  │  3. Serialises payload to WebhookQueueMessage (JSON)
  │  4. Enqueues with session_id = "{org}/{repo}/{pr_number}"
  │  5. Returns 202 Accepted to GitHub
  │
  ▼
External queue (Azure Service Bus / AWS SQS FIFO / in-memory)
  │
  │  (N worker tasks, configured by MERGE_WARDEN_QUEUE_CONCURRENCY)
  ▼
QueueIngress worker task(s) [inside merge_warden]
  │  1. accept_session() — acquires session lock for one PR
  │  2. receive_message() — reads WebhookQueueMessage
  │  3. Deserialise + reconstruct EventEnvelope
  │  4. Wrap in ProcessableEvent { envelope, QueueMessageAck }
  │
  ▼
run_event_processor loop
  (see Processing Pipeline below)
```

In queue mode the Axum server only exposes a health-check endpoint (`GET /health`).
`GITHUB_WEBHOOK_SECRET` is not required and is ignored if set.

**Key properties:**

- Merge_warden in queue mode has no inbound webhook surface — it cannot receive
  GitHub payloads directly
- Session ID `"{org}/{repo}/{pr_number}"` guarantees per-PR sequential ordering
- Different PRs (different sessions) may be processed in parallel across N workers
- Failed messages are dead-lettered (not requeued) to prevent poison-pill loops
- Supported providers: Azure Service Bus (sessions), AWS SQS FIFO, in-memory (dev/test)

### Queue Message Schema

`WebhookQueueMessage` is the serialised form stored in the queue:

```
schema_version: u8         // currently 1; increment on breaking changes
event_type:     String     // e.g. "pull_request"
delivery_id:    String     // X-GitHub-Delivery UUID
installation_id: u64       // GitHub App installation ID
received_at:    DateTime   // UTC timestamp of original webhook receipt
raw_payload:    String     // original GitHub JSON body, verbatim
```

The session ID is stored in the broker envelope, not in this struct.

---

## Processing Pipeline

Both modes converge on the same `run_event_processor` loop. This loop is
mode-agnostic: it receives a `ProcessableEvent` and does not know whether the
event came from a channel or a queue.

```
ProcessableEvent { envelope, ack }
  │
  ▼
MergeWardenWebhookHandler::handle_event(envelope)
  │
  ├─ event_type != "pull_request" ?
  │    └─ return Ok(())  (ignored; non-PR events currently unsupported)
  │
  ├─ action not in { opened, edited, ready_for_review, reopened, unlocked, synchronize } ?
  │    └─ return Ok(())  (no-op for irrelevant actions)
  │
  ├─ Extract pr_number and installation_id from payload
  │
  ├─ github_client.installation_by_id(installation_id)
  │    └─ Err → return ProcessingError
  │
  ├─ Load .github/merge-warden.toml from the repository
  │    ├─ Ok  → use repo config merged with application defaults
  │    └─ Err → fall back to application defaults (logged as warning)
  │
  ├─ MergeWarden::process_pull_request(owner, repo, pr_number)
  │    (see Validation Actions below)
  │    └─ Err → return ProcessingError
  │
  └─ return Ok(())
  │
  ▼
Acknowledgement
  ├─ Ok(())  → ack.complete()
  │              webhook mode:  no-op
  │              queue mode:    session.complete_message() + close_session()
  │
  └─ Err(e)  → ack.reject(reason)
                 webhook mode:  no-op  (event is simply not retried)
                 queue mode:    session.dead_letter_message(reason) + close_session()
```

---

## Validation Actions

`MergeWarden::process_pull_request` runs all configured checks and applies their
results to the PR. The checks run in order; all checks run regardless of earlier
failures (no short-circuit).

```
process_pull_request(owner, repo, pr_number)
  │
  ├─ Fetch PR details via GitHub API
  │
  ├─ Check: PR is not a draft
  │    └─ Draft PRs are skipped entirely (no labels, no comments, no status)
  │
  ├─ Check: title matches configured pattern (enforce_title_convention)
  │    ├─ Pass → remove invalid-title label (if present)
  │    └─ Fail → add invalid-title label; add/update failure comment
  │
  ├─ Check: body contains work item reference (enforce_work_item_references)
  │    ├─ Pass → remove missing-work-item label (if present)
  │    └─ Fail → add missing-work-item label; add/update failure comment
  │
  ├─ Check: PR size within configured limits (pr_size_check)
  │    ├─ Compute: additions + deletions across changed files
  │    ├─ Determine size bucket (xs/s/m/l/xl/xxl)
  │    └─ Add/update size label; remove stale size labels
  │
  ├─ Apply change-type labels based on changed file paths (change_type_labels)
  │    └─ Add labels for matching path patterns; remove stale change-type labels
  │
  ├─ Evaluate bypass rules
  │    └─ If bypass active: skip check-run failure status for bypassed checks
  │
  └─ Set commit status check on the PR head SHA
       ├─ All checks pass (or bypassed): success status
       └─ Any check fails:               failure status with summary message
```

### Configuration Precedence

Each repository may provide a `.github/merge-warden.toml` file that overrides the
application defaults. If the file is absent or unreadable, application defaults
(set via `MERGE_WARDEN_CONFIG_FILE` or built-in defaults) are used.

See [configuration-management.md](../operations/configuration-management.md) and
[server-config.md](../interfaces/server-config.md) for the full configuration schema.

---

## Error Handling Summary

| Stage | Error | Behaviour |
|-------|-------|-----------|
| Signature validation (webhook mode) | Invalid HMAC | SDK returns 401 to GitHub; event dropped |
| Channel send (webhook mode) | Channel full (back-pressure) | Sender blocks until slot available |
| Queue receive (queue mode) | Provider connect error | `IngressError` logged; worker task exits |
| Message deserialisation (queue mode) | Malformed JSON | Message dead-lettered; loop continues |
| Unknown schema version | `schema_version != 1` | Message dead-lettered; loop continues |
| EventEnvelope reconstruction | SDK parse error | Message dead-lettered; loop continues |
| PR processing | GitHub API error | `ack.reject()` → dead-letter (queue) or logged (webhook) |
| Worker task crash | Unrecoverable `IngressError` | Error logged; task exits; process continues (other workers unaffected) |

---

## Concurrency Model

```
                 Axum HTTP server
                 (1 Tokio runtime, N threads)
                        │
           ┌────────────┴────────────┐
           │ webhook mode            │ queue mode
           │                         │
     mpsc channel              external queue
     (capacity 64)             (N sessions)
           │                         │
     1 worker task            N worker tasks
           │                   (configurable)
           └────────────┬────────────┘
                        │
               run_event_processor
               (sequential per worker)
```

In webhook mode, events are processed one at a time (single worker). In queue mode,
up to `MERGE_WARDEN_QUEUE_CONCURRENCY` (default: 4) events may be processed in parallel,
but events for the same PR are always serialised because they share a session lock.

---

## Related Documents

- [queue-architecture.md](../design/queue-architecture.md) — detailed design and migration strategy
- [containerisation.md](../design/containerisation.md) — HTTP server, Dockerfile, and deployment
- [server-ingress.md](../interfaces/server-ingress.md) — `EventIngress`, `EventAcknowledger`, and `WebhookQueueMessage` interface contracts
- [server-config.md](../interfaces/server-config.md) — environment variable reference
- [deployment.md](../operations/deployment.md) — deployment procedures and infrastructure requirements
- [monitoring.md](../operations/monitoring.md) — metrics, alerts, and observability
