# Design: Queue-Based Webhook Processing

## Status

Draft — awaiting implementation (Phase 3.0)
Depends on: [github-bot-sdk-migration.md](github-bot-sdk-migration.md) (0.1),
[containerisation.md](containerisation.md) (0.2)

Related issue: [#179](https://github.com/pvandervelde/merge_warden/issues/179)

---

## Problem

The current architecture processes webhook events synchronously within the HTTP handler.
GitHub requires a response within 10 seconds, which is violated by slow GitHub API calls
during processing, causing timeouts and lost events.

```
Current: GitHub → POST /webhook → validate → process (PR checks, labels, comments) → 200 OK
                                              ↑ can take > 10s
```

---

## Solution: `EventIngress` Abstraction with Queue Decoupling

Two objectives are combined:

1. **Infrastructure**: decouple webhook reception from event processing using a queue,
   eliminating timeouts.
2. **Architectural**: introduce an `EventIngress` abstraction so that identical business
   logic processes events regardless of how they arrived (HTTP webhook or queue message).

---

## Core Abstraction: `EventIngress`

Regardless of how a GitHub event arrives — directly via HTTP or via a queue — the
processing pipeline receives the same type: a **`ProcessableEvent`** carrying an
`EventEnvelope` (from `github-bot-sdk`) plus an acknowledgement handle.

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Receiver mode: "webhook"                                                 │
│                                                                          │
│  Axum POST handler                                                       │
│    validates signature (SDK)                                             │
│    parses EventEnvelope (SDK: parse_webhook)                             │
│    sends to internal channel (mpsc or bounded)                           │
│    responds 200 OK immediately                                           │
│                                                                          │
│  WebhookIngress (EventIngress impl)                                      │
│    reads from channel                                                    │
│    yields ProcessableEvent { envelope, ack: NoOpAck }                    │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│ Receiver mode: "queue"                                                   │
│                                                                          │
│  Axum POST handler                                                       │
│    validates signature (SDK)                                             │
│    extracts session_id = "{org}/{repo}/{pr_number}"                      │
│    enqueues WebhookQueueMessage (queue-runtime: send_message)            │
│    responds 200 OK immediately                                           │
│                                                                          │
│  QueueIngress (EventIngress impl)                                        │
│    accepts session (queue-runtime: accept_session)                       │
│    deserializes WebhookQueueMessage → EventEnvelope                      │
│    yields ProcessableEvent { envelope, ack: QueueMessageAck }            │
└──────────────────────────────────────────────────────────────────────────┘

                    ↓ ProcessableEvent (same type in both paths)

┌──────────────────────────────────────────────────────────────────────────┐
│ Event processing pipeline (identical for both modes)                     │
│                                                                          │
│  filter unsupported actions                                              │
│  authenticate GitHub installation (SDK)                                  │
│  load repo config                                                        │
│  MergeWarden::process_pull_request(...)           ← core (unchanged)     │
│  ack.complete() or ack.reject(reason)                                    │
└──────────────────────────────────────────────────────────────────────────┘
```

### Trait definition (conceptual — exact signatures are interface designer's responsibility)

```
EventIngress:
  - next_event() → Result<Option<ProcessableEvent>, IngressError>

EventAcknowledger (sealed to server crate):
  - complete() → Result<(), IngressError>
  - reject(reason: &str) → Result<(), IngressError>

ProcessableEvent:
  - envelope: EventEnvelope          (github-bot-sdk type)
  - ack: Box<dyn EventAcknowledger>
```

**`WebhookIngress`**: acknowledger is a no-op (in-process channel; no external
message broker to confirm to).

**`QueueIngress`**: acknowledger calls `session.complete_message(handle)` or
`session.dead_letter_message(handle, reason)` on the `queue-runtime` session.

---

## Queue Message Schema

When the webhook receiver enqueues an event, it serializes a `WebhookQueueMessage`:

```rust
WebhookQueueMessage {
    schema_version: u8,        // 1 — for forward compatibility
    event_type: String,        // "pull_request", "pull_request_review", etc.
    delivery_id: String,       // X-GitHub-Delivery header value
    installation_id: u64,
    received_at: DateTime<Utc>,
    raw_payload: String,       // raw JSON body from GitHub (not re-parsed here)
}
```

The `raw_payload` field stores the original GitHub JSON body verbatim. The queue
processor reconstructs the `EventEnvelope` from `event_type` + `raw_payload`, matching
exactly what `parse_webhook` would produce from the original HTTP request.

The **session ID** is an envelope field on the `queue-runtime::Message`, not part of
`WebhookQueueMessage`:

```
session_id = "{org_name}/{repo_name}/{pr_number}"
```

Examples:

- `"acme-corp/web-app/456"` — PR #456 in acme-corp/web-app
- `"acme-corp/api-service/456"` — PR #456 in a different repo (different session)

This convention ensures:

- All events for one PR are processed sequentially (Azure Service Bus session guarantee)
- Events for different PRs are processed in parallel (independent sessions)

---

## Session ID Extraction

The session ID is extracted by the HTTP webhook handler before enqueuing. The
handler must parse the repository full name and PR number from the GitHub payload:

```
repository.full_name  →  "acme-corp/web-app"   gives org and repo
pull_request.number   →  456

session_id = "acme-corp/web-app/456"
```

If the payload does not contain a PR number (e.g., a repository event), use
`"{org}/{repo}/0"` or drop the event (the existing filter for unsupported actions
already rejects non-PR events before enqueuing).

---

## Receiver Mode: one at a time

A single binary runs in exactly one receiver mode, chosen at startup via
`MERGE_WARDEN_RECEIVER_MODE`:

| Value | Behaviour |
|---|---|
| `webhook` (default) | Axum handler → in-process channel → `WebhookIngress` processing loop |
| `queue` | Axum handler → queue; separate Tokio task runs `QueueIngress` polling loop |

Both modes use the same Axum server (for health checks and, in `queue` mode, to receive
and forward incoming webhooks). The Axum server is always started regardless of mode.

A `--disable-queue-processor` flag is reserved for future split deployments (separate
receiver and processor containers) but is not implemented initially.

---

## Queue Provider

Uses `queue-runtime` crate for provider-agnostic queue access.

| Environment | Provider | Config |
|---|---|---|
| Production (Azure) | `AzureServiceBus` | `AZURE_SERVICEBUS_NAMESPACE` + Managed Identity |
| Production (AWS) | `AwsSqs` (planned in queue-runtime) | standard AWS env vars |
| Testing | `InMemory` | no external infrastructure |
| Local dev | `InMemory` or connection string | `AZURE_SERVICEBUS_CONNECTION_STRING` |

`use_sessions: true` is required for both Azure provider (native) and the future AWS
provider (emulated by queue-runtime).

Queue configuration uses environment-based selection (same pattern as
`queue-runtime`'s own `create_queue_config()` example):

```
MERGE_WARDEN_QUEUE_PROVIDER=azure|aws|memory
AZURE_SERVICEBUS_NAMESPACE=...
MERGE_WARDEN_QUEUE_NAME=merge-warden-events
```

---

## Component Responsibilities

### `server::ingress::WebhookIngress`

**Knows:** the bounded `mpsc` channel shared with the Axum POST handler.
**Does:**

- Implements `EventIngress`; `next_event()` reads from receiver end of channel
- `EventAcknowledger` is a no-op (in-process delivery has no external broker)

### `server::ingress::QueueIngress`

**Knows:** `queue-runtime::QueueClient`; queue name; session timeout.
**Does:**

- Implements `EventIngress`; `next_event()` calls `accept_session` then
  `session.receive_message`
- On `complete()`: calls `session.complete_message(handle)`
- On `reject(reason)`: calls `session.dead_letter_message(handle, reason)`
- Deserializes `WebhookQueueMessage` from raw bytes; reconstructs `EventEnvelope`

### `server::ingress::EventProcessor` (internal loop)

**Knows:** `Box<dyn EventIngress>`; `AppState`.
**Does:**

- Runs as a Tokio task: `loop { event = ingress.next_event(); process(event); ack }`
- Calls the same processing pipeline regardless of ingress source
- On processing error: calls `ack.reject(error_message)` → DLQ routing

### Axum POST handler (webhook mode)

**Does:** validate signature → parse `EventEnvelope` → extract session_id fields →
**in webhook mode**: send `EventEnvelope` to in-process channel → return 200 OK
**in queue mode**: serialize to `WebhookQueueMessage` → `queue_client.send_message(...)` → return 200 OK

---

## Migration Strategy

Issue #179's feature-flag approach is adopted, simplified to a single env var.

### Phase 1: infrastructure in parallel (no traffic change)

- Deploy `queue-runtime` Azure Service Bus resources (separate Terraform repo)
- Run binary with `MERGE_WARDEN_RECEIVER_MODE=webhook` (default — no behaviour change)
- Verify queue connectivity from the binary (startup health check)

### Phase 2: canary (queue mode for selected repos)

At this phase, run a second instance of the binary with
`MERGE_WARDEN_RECEIVER_MODE=queue` and redirect traffic to it for a subset of
repositories using GitHub App installation scoping or a routing rule.

A simpler alternative: dual-process within the same deployment — route only
explicitly configured repos to queue mode using a config list
(`MERGE_WARDEN_QUEUE_REPOS=owner/repo,...`). The webhook handler checks this list
before deciding whether to enqueue or process inline.

### Phase 3: full rollout

`MERGE_WARDEN_RECEIVER_MODE=queue` for all traffic.

### Phase 4: clean up

Remove `WebhookIngress` in-process channel path; `webhook` mode becomes
`queue` mode with local in-memory queue (for dev/test only).

---

## Infrastructure (via separate Terraform repo)

The following Azure Service Bus resources are required. Terraform module changes go
in the separate infrastructure repository:

| Resource | Configuration |
|---|---|
| Service Bus namespace | Standard or Premium tier |
| Queue | `RequiresSession=true`, `MaxDeliveryCount=3` |
| Dead letter queue | Enabled by default on Service Bus |
| Session lock duration | 5 minutes |
| Message retention | 14 days |

Managed Identity must be granted `Azure Service Bus Data Owner` role on the namespace.

---

## Behavioral Assertions

1. **Webhook receiver must respond within 500ms regardless of processing time**
   - Given: `MERGE_WARDEN_RECEIVER_MODE=queue`; processing takes 15 seconds
   - When: GitHub POST arrives
   - Then: HTTP 200 OK returned before processing begins; queue message enqueued

2. **Events for the same PR must be processed sequentially**
   - Given: three events for PR #42 in `org/repo` arrive within 1 second
   - When: queue processor runs
   - Then: events processed in arrival order; no concurrent processing of same session

3. **Events for different PRs must be processable in parallel**
   - Given: events for PR #1 and PR #2 in `org/repo` are in the queue simultaneously
   - When: queue processor runs with multiple session workers
   - Then: both PRs may be processed concurrently

4. **Failed processing must route to dead letter queue, not re-block the session**
   - Given: processing throws an unrecoverable error for a message
   - When: `ack.reject(reason)` is called
   - Then: message moves to DLQ; session releases; next message in session is processed

5. **Both ingress modes must produce identical processing outcomes**
   - Given: same GitHub event payload delivered via webhook mode and queue mode
   - When: `EventProcessor` runs in each mode
   - Then: same `MergeWarden::process_pull_request` call made with same arguments

6. **In-memory provider must be used in all automated tests**
   - Given: integration tests run
   - When: `MERGE_WARDEN_QUEUE_PROVIDER` is absent or `memory`
   - Then: no external Azure or AWS dependency; tests pass without cloud access

7. **Startup must fail fast if queue mode selected but provider unconfigurable**
   - Given: `MERGE_WARDEN_RECEIVER_MODE=queue`; `MERGE_WARDEN_QUEUE_PROVIDER=azure`;
     no Service Bus credentials in environment
   - When: binary starts
   - Then: logs clear error; exits with code 1 before accepting any HTTP requests

---

## Monitoring and Alerting

Key metrics to emit (via `tracing` structured events, exportable via OTLP):

| Metric | Alert threshold |
|---|---|
| `ingress.webhook.enqueue_duration_ms` | p95 > 100ms |
| `ingress.queue.processing_duration_ms` | p95 > 30 000ms |
| `ingress.queue.depth` | > 100 messages |
| `ingress.queue.age_oldest_message_secs` | > 300s (5 min) |
| `ingress.queue.dlq_count` | > 0 |
| `processing.success_rate` | < 99.9% |

---

## Testing Strategy

- **Unit tests**: `WebhookQueueMessage` serialization/deserialization round-trip;
  session ID extraction from various payload shapes including missing PR number
- **Unit tests**: `EventProcessor` loop with `InMemoryConfig` queue; verify
  `complete` called on success, `reject` called on error
- **Contract tests**: `WebhookIngress` and `QueueIngress` both satisfy `EventIngress`
  contract test (same event in → same processing call out)
- **Integration tests**: end-to-end `queue` mode with in-memory provider:
  POST to Axum → enqueue → dequeue → `process_pull_request` called
- **Load scenario** (manual / CI gate): 100 concurrent PRs × 5 events each;
  verify per-PR ordering and p95 processing latency < 30s

---

## Open Decisions

| Decision | Status |
|---|---|
| `queue-runtime` version: git or crates.io? | Same as `github-bot-sdk` — git pin until crates.io release |
| Session worker concurrency: single session at a time or multiple parallel sessions? | Start with configurable concurrency (default: 4 parallel sessions); `MERGE_WARDEN_QUEUE_CONCURRENCY` |
| Phase 2 canary: dual-process routing or explicit repo list? | Recommend explicit repo list (`MERGE_WARDEN_QUEUE_REPOS`) — simpler and no DNS routing change needed |
