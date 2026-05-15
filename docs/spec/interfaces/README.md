# Interface Specifications

This folder contains the concrete type-level contracts that implementors write against.
Each document specifies exact struct/trait/function signatures, error variants, and
behavioral postconditions for a bounded area of the codebase.

## Documents

### [developer-platforms-sdk.md](./developer-platforms-sdk.md)

Contracts for the `developer_platforms` crate. Covers the **github-bot-sdk migration**
(task 1.0) — updated `GitHubProvider` constructor, new error variants, and the
`EventEnvelope` type origin decision — and the **FR-007 additions**: the new
`PullRequest.head_sha` field and the `ConfigFetcher::fetch_config_at_ref` method.

### [core-config-validation.md](./core-config-validation.md)

Contracts for the `core` crate additions required by
**FR-007 (Configuration Change Validation)**. Covers `CONFIG_COMMENT_MARKER`,
`ConfigValidationOutcome`, `validate_config_content`, the updated `MergeWarden<P>`
trait bound, and the private `communicate_config_validity_status` method.

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
  └── defines PullRequest.head_sha, ConfigFetcher::fetch_config_at_ref (FR-007)
  └── defines EventEnvelope (placeholder → SDK type in task 1.0)
        └── used in server-ingress.md (ProcessableEvent.envelope)

core-config-validation.md
  └── depends on ConfigFetcher::fetch_config_at_ref  (developer-platforms-sdk.md)
  └── depends on PullRequest.head_sha               (developer-platforms-sdk.md)
  └── defines ConfigValidationOutcome, validate_config_content

server-config.md
  └── defines ServerConfig, ReceiverMode, QueueServerConfig
        └── QueueServerConfig used in server-ingress.md (QueueIngress construction)

server-ingress.md
  └── defines EventIngress, IngressError
        └── IngressError used in server-config.md (ServerError::IngressError)
```

## Source Code Stubs

The corresponding Rust stubs live in `crates/`:

| Spec document | Source file |
|---|---|
| developer-platforms-sdk.md | `crates/developer_platforms/src/lib.rs` (ConfigFetcher trait), `crates/developer_platforms/src/models.rs` (PullRequest), `crates/developer_platforms/src/github.rs` (GitHubProvider impl), `crates/developer_platforms/src/errors.rs` |
| core-config-validation.md | `crates/core/src/config.rs`, `crates/core/src/lib.rs` |
| server-config.md | `crates/server/src/config.rs`, `crates/server/src/errors.rs`, `crates/server/src/telemetry.rs` |
| server-ingress.md | `crates/server/src/ingress.rs` |
