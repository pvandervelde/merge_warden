# Interface Spec: server — Event Ingress Abstraction

**Source**: `crates/server/src/ingress.rs`, `webhook.rs`
**Spec**: `docs/spec/design/queue-architecture.md`
**Task**: 3.0 (interfaces defined now; implementations added in task 3.0)

---

## Overview

The ingress layer decouples event delivery from event processing. A `run_event_processor`
task pulls `ProcessableEvent` values one at a time from a `Box<dyn EventIngress>` and
passes each to the core processing pipeline. The rest of the server is unaware of whether
events arrive via a live Axum handler or a queue consumer.

```
Axum POST handler ──► mpsc::Sender<EventEnvelope>  ──┐
                                                       ├► EventIngress ──► run_event_processor ──► core
queue-runtime Receiver ────────────────────────────────┘
```

---

## `EventEnvelope` (local placeholder)

> **NOTE TO IMPLEMENTOR (task 1.0)**: Remove this local type and replace every
> reference with `github_bot_sdk::events::EventEnvelope` once the SDK is wired
> into the workspace `Cargo.toml`.

```rust
/// Placeholder for the event envelope provided by `github-bot-sdk`.
/// Carries the raw, deserialised data read from a webhook POST or queue message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventEnvelope {
    /// GitHub-Event header value, e.g. `"pull_request"`.
    pub event_type: String,
    /// X-GitHub-Delivery header value (UUID string).
    pub delivery_id: String,
    /// App installation id from the JWT payload, when available.
    pub installation_id: Option<u64>,
    /// Full JSON body deserialized as an opaque value.
    pub payload: serde_json::Value,
}
```

---

## `EventAcknowledger`

```rust
/// Lifecycle hook that allows the ingress layer to signal message-broker
/// infrastructure after an event has been processed (or failed).
///
/// Implementations are specific to each ingress backend:
/// - **Webhook mode**: `NoOpAck` — no acknowledgement is required.
/// - **Queue mode**: marks the message as complete or dead-lettered on the broker.
///
/// # Object-Safety
/// Designed for use as `Box<dyn EventAcknowledger + Send>`.
#[async_trait::async_trait]
pub trait EventAcknowledger: Send {
    /// Marks the event as successfully processed.
    ///
    /// For queue backends this deletes or completes the message so it is not
    /// redelivered. For webhook backends this is a no-op.
    ///
    /// # Errors
    /// Returns `IngressError::QueueError` if the broker cannot be reached.
    async fn complete(self: Box<Self>) -> Result<(), IngressError>;

    /// Marks the event as permanently failed.
    ///
    /// For queue backends this moves the message to the dead-letter queue.
    /// `reason` is stored as a diagnostic property on the dead-lettered message.
    ///
    /// # Errors
    /// Returns `IngressError::QueueError` if the broker cannot be reached.
    async fn reject(self: Box<Self>, reason: &str) -> Result<(), IngressError>;
}
```

---

## `ProcessableEvent`

```rust
/// A single GitHub event ready for core processing, together with its
/// acknowledgement handle.
///
/// Owners of this struct must call either `ack.complete()` or `ack.reject()`
/// after processing to avoid message redelivery in queue mode.
pub struct ProcessableEvent {
    pub envelope: EventEnvelope,
    pub ack: Box<dyn EventAcknowledger + Send>,
}
```

---

## `EventIngress`

```rust
/// Async event source that produces `ProcessableEvent` values one at a time.
///
/// Consumers call `next_event()` in a loop until they receive `Ok(None)` (EOF)
/// or a terminal error.
///
/// # Cancel Safety
/// `next_event()` MUST be cancel-safe: if the future is dropped after being
/// polled but before it yields a value, no event must be silently lost by the
/// implementation. Webhook mode achieves this because `tokio::sync::mpsc::Receiver::recv`
/// is cancel-safe. Queue implementations must honour this guarantee.
///
/// # Object-Safety
/// Designed for use as `Box<dyn EventIngress + Send>`.
#[async_trait::async_trait]
pub trait EventIngress: Send {
    /// Returns the next available event, or `None` if the source has closed.
    ///
    /// Blocks asynchronously until an event is available or the source closes.
    ///
    /// # Errors
    /// - `IngressError::ChannelClosed` — only returned when the channel unexpectedly
    ///   drops; normal EOF is signalled by `Ok(None)`.
    /// - `IngressError::QueueError` — connection to the broker was lost and
    ///   cannot be recovered in-process.
    /// - `IngressError::DeserializationError` — a message arrived but could not
    ///   be deserialized (implementation should log and skip); this variant is
    ///   reserved for non-recoverable decode failures.
    async fn next_event(&mut self) -> Result<Option<ProcessableEvent>, IngressError>;
}
```

---

## `IngressError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum IngressError {
    /// The in-process event channel was closed before the receiver could drain it.
    #[error("Event channel closed unexpectedly")]
    ChannelClosed,

    /// Queue provider returned an unrecoverable error.
    #[error("Queue error: {message}")]
    QueueError { message: String },

    /// A queue message payload could not be deserialized.
    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    /// `WebhookQueueMessage.schema_version` is not supported by this binary.
    #[error("Unsupported schema version: {0}")]
    UnknownSchemaVersion(u8),

    /// Catch-all for unexpected internal errors.
    #[error("Internal ingress error: {0}")]
    Internal(String),
}
```

---

## `WebhookQueueMessage`

Defines the JSON schema written to (and read from) the queue by the
webhook-to-queue bridge. Session ID is NOT stored in this struct — it is
the session metadata field on the queue-runtime `Message` envelope and is
set to `"{org}/{repo}/{pr_number}"` by the enqueuing side.

```rust
/// Serialized representation of one GitHub webhook event stored in the queue.
///
/// Schema version **1** is the initial format. Increment `schema_version` and
/// add a migration arm in `QueueIngress::next_event()` for every breaking change.
///
/// # Session ID (out-of-band)
/// The session identifier is NOT a field of this struct. It is set on the
/// broker-level message envelope as `"{org}/{repo}/{pr_number}"` so that the
/// queue runtime can guarantee ordered processing per pull request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookQueueMessage {
    /// Magic byte for forward-compatible schema migration. Currently `1`.
    pub schema_version: u8,
    /// GitHub-Event header value (e.g. `"pull_request"`).
    pub event_type: String,
    /// X-GitHub-Delivery UUID string.
    pub delivery_id: String,
    /// GitHub App installation id.
    pub installation_id: u64,
    /// UTC timestamp at which the webhook POST was received by the server.
    pub received_at: chrono::DateTime<chrono::Utc>,
    /// Raw JSON body (string-encoded so binary brokers can store it verbatim).
    pub raw_payload: String,
}
```

---

## `NoOpAck`

```rust
/// `EventAcknowledger` for the webhook receiver mode.
///
/// Webhook deliveries require no explicit acknowledgement; both `complete()`
/// and `reject()` are no-ops that always return `Ok(())`.
pub struct NoOpAck;
```

---

## `WebhookIngress`

```rust
/// `EventIngress` implementation for webhook mode.
///
/// Events arrive via an in-process `tokio::sync::mpsc` channel whose sender
/// is owned by the Axum POST handler. EOF is signalled by dropping all senders.
///
/// All events are acknowledged with `NoOpAck`.
pub struct WebhookIngress {
    receiver: tokio::sync::mpsc::Receiver<EventEnvelope>,
}

impl WebhookIngress {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<EventEnvelope>) -> Self;
}
```

---

## `QueueIngress`

```rust
/// `EventIngress` implementation for queue mode.
///
/// Reads `WebhookQueueMessage` payloads from the configured queue provider.
///
/// > **NOTE TO IMPLEMENTOR (task 3.0)**: Add a
/// > `queue_client: std::sync::Arc<dyn queue_runtime::QueueClient>` field and
/// > implement the actual queue polling logic once `queue-runtime` is wired
/// > into the workspace `Cargo.toml`.
///
/// The `concurrency` field dictates how many messages may be in-flight simultaneously
/// (i.e. fetched but not yet acknowledged). Managed by the concrete implementation.
pub struct QueueIngress {
    pub queue_name: String,
    pub concurrency: usize,
}

impl QueueIngress {
    pub fn new(queue_name: String, concurrency: usize) -> Self;
}
```

---

## `run_event_processor()`

```rust
/// Drives the core event-processing pipeline from an ingress source.
///
/// Spawned as a background `tokio::task` at startup. Runs until the ingress
/// source signals EOF (`Ok(None)`) or an unrecoverable error.
///
/// # Processing Loop
/// 1. Call `ingress.next_event().await`.
/// 2. On `Ok(Some(event))`:
///    a. Build a `GitHubProvider` from `state` (task 1.0 replaces this with SDK).
///    b. Run the core `check_pull_request` (or equivalent) logic.
///    c. On success: call `event.ack.complete().await`.
///    d. On domain error: call `event.ack.reject(&err.to_string()).await`.
/// 3. On `Ok(None)`: break, return `Ok(())`.
/// 4. On `Err(e)`: return `Err(e)` (caller decides whether to restart).
///
/// # Cancellation
/// The function does NOT install its own cancellation signal. The caller should
/// abort the spawned `JoinHandle` (or drop the ingress sender) to stop it.
///
/// # Arguments
/// - `ingress`: The event source. Consumed by this function.
/// - `state`: Shared Axum application state carrying secrets and policy config.
///
/// # Errors
/// Returns the first `IngressError` that is not recoverable in-loop.
pub async fn run_event_processor(
    ingress: Box<dyn EventIngress + Send>,
    state: std::sync::Arc<crate::webhook::AppState>,
) -> Result<(), IngressError>;
```

---

## `AppState` (in `webhook.rs`)

```rust
/// Shared state threaded through the Axum router and the processor task.
///
/// Constructed once in `main()` from `ServerSecrets` and `ServerConfig`.
///
/// > **NOTE TO IMPLEMENTOR (task 1.0)**: Replace `github_app_id`,
/// > `github_app_private_key`, and `webhook_secret` fields with an
/// > `Arc<github_bot_sdk::client::GitHubClient>` once the SDK is integrated.
#[derive(Clone)]
pub struct AppState {
    pub github_app_id: u64,
    /// PEM-encoded private key. Stored as `String` here so it can be cloned
    /// into request handlers; callers must not log this value.
    pub github_app_private_key: String,
    pub webhook_secret: String,
    pub policies: merge_warden_core::config::ApplicationDefaults,
    /// Present only in `ReceiverMode::Queue`; `None` in `ReceiverMode::Webhook`.
    pub event_sender: Option<tokio::sync::mpsc::Sender<crate::ingress::EventEnvelope>>,
}
```
