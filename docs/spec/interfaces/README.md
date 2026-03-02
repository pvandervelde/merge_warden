# Interface Specifications

This folder contains the concrete type-level contracts that implementors write against.
Each document specifies exact struct/trait/function signatures, error variants, and
behavioral postconditions for a bounded area of the codebase.

## Documents

### [developer-platforms-sdk.md](./developer-platforms-sdk.md)

Contracts for the `developer_platforms` crate changes required by the **github-bot-sdk
migration** (task 1.0). Covers the updated `GitHubProvider` constructor, new error
variants, and the `EventEnvelope` type origin decision.

### [server-config.md](./server-config.md)

Contracts for the `server` crate configuration and startup layer required by
**containerisation** (task 2.0). Covers `ServerSecrets`, `ServerConfig`,
`ReceiverMode`, `TelemetryConfig`, `ServerError`, and all environment variable names.

### [server-ingress.md](./server-ingress.md)

Contracts for the `EventIngress` abstraction and queue message schema required by
the **queue-based webhook processing** (task 3.0). Covers `EventIngress`,
`EventAcknowledger`, `ProcessableEvent`, `WebhookQueueMessage`, `IngressError`, and
`run_event_processor`.

## Dependency Map

```
developer-platforms-sdk.md
  └── defines EventEnvelope (placeholder → SDK type in task 1.0)
        └── used in server-ingress.md (ProcessableEvent.envelope)

server-config.md
  └── defines ServerConfig, ReceiverMode, QueueServerConfig
        └── QueueServerConfig used in server-ingress.md (QueueIngress construction)

server-ingress.md
  └── defines EventIngress, IngressError
        └── IngressError used in server-config.md (ServerError::IngressError)
```

## Source Code Stubs

The corresponding Rust stubs live in `crates/server/src/`:

| Spec document | Source file |
|---|---|
| server-config.md | `crates/server/src/config.rs`, `crates/server/src/errors.rs`, `crates/server/src/telemetry.rs` |
| server-ingress.md | `crates/server/src/ingress.rs` |
| developer-platforms-sdk.md | `crates/server/src/webhook.rs`, `crates/developer_platforms/src/errors.rs` |

All stubs compile under `cargo check`. Method and function bodies use `todo!()` with
a reference to the relevant spec document.
