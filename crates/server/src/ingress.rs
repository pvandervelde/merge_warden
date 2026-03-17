// See docs/spec/interfaces/server-ingress.md for the full contract.
//
// NOTE: Several items in this module are not yet wired into main() — they are
// the Task 3.0 queue-mode foundation and will be used once that task is
// implemented. Item-level #[allow(dead_code)] is used rather than a module-wide
// suppression so that genuinely unreachable code cannot hide behind it.
use std::sync::Arc;

use github_bot_sdk::webhook::WebhookHandler;
use tracing::error;

#[cfg(test)]
#[path = "ingress_tests.rs"]
mod tests;

// ---------------------------------------------------------------------------
// EventEnvelope
// ---------------------------------------------------------------------------

/// The canonical event envelope type for this crate — re-exported from the
/// `github-bot-sdk` so all modules share a single definition.
///
/// See docs/spec/interfaces/server-ingress.md — `EventEnvelope`
pub use github_bot_sdk::events::EventEnvelope;

// ---------------------------------------------------------------------------
// IngressError
// ---------------------------------------------------------------------------

/// All errors that can arise inside the ingress layer.
///
/// See docs/spec/interfaces/server-ingress.md — `IngressError`
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum IngressError {
    /// The in-process event channel was closed before the receiver could drain it.
    #[error("Event channel closed unexpectedly")]
    ChannelClosed,

    /// The queue provider returned an unrecoverable error.
    #[error("Queue error: {message}")]
    QueueError { message: String },

    /// A queue message payload could not be deserialized.
    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    /// `WebhookQueueMessage.schema_version` is not supported.
    #[error("Unsupported schema version: {0}")]
    UnknownSchemaVersion(u8),

    /// Catch-all for unexpected internal ingress errors.
    #[error("Internal ingress error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// EventAcknowledger
// ---------------------------------------------------------------------------

/// Lifecycle hook for acknowledging an event to the underlying broker.
///
/// See docs/spec/interfaces/server-ingress.md — `EventAcknowledger`
#[allow(dead_code)]
#[async_trait::async_trait]
pub trait EventAcknowledger: Send {
    /// Marks the event as successfully processed.
    ///
    /// For queue backends, deletes or completes the broker message.
    /// For webhook backend (`NoOpAck`), this is a no-op.
    ///
    /// # Errors
    /// Returns [`IngressError::QueueError`] if the broker cannot be reached.
    async fn complete(self: Box<Self>) -> Result<(), IngressError>;

    /// Marks the event as permanently failed.
    ///
    /// For queue backends, moves the message to the dead-letter queue.
    /// The `reason` string is stored as a diagnostic property on the dead-lettered
    /// message.
    ///
    /// # Errors
    /// Returns [`IngressError::QueueError`] if the broker cannot be reached.
    async fn reject(self: Box<Self>, reason: &str) -> Result<(), IngressError>;
}

// ---------------------------------------------------------------------------
// ProcessableEvent
// ---------------------------------------------------------------------------

/// A single GitHub event ready for core processing, with its acknowledgement handle.
///
/// Callers **must** invoke either `ack.complete()` or `ack.reject()` after
/// processing to avoid message redelivery in queue mode.
///
/// See docs/spec/interfaces/server-ingress.md — `ProcessableEvent`
#[allow(dead_code)]
pub struct ProcessableEvent {
    /// The event payload.
    pub envelope: EventEnvelope,
    /// Acknowledgement handle. Must be consumed after processing.
    pub ack: Box<dyn EventAcknowledger + Send>,
}

// ---------------------------------------------------------------------------
// EventIngress
// ---------------------------------------------------------------------------

/// Async event source that produces [`ProcessableEvent`] values one at a time.
///
/// Implementations must be cancel-safe: if the future returned by `next_event`
/// is dropped after being polled but before it yields, no event must be silently
/// lost.
///
/// See docs/spec/interfaces/server-ingress.md — `EventIngress`
#[allow(dead_code)]
#[async_trait::async_trait]
pub trait EventIngress: Send {
    /// Returns the next available event, or `None` when the source has closed.
    ///
    /// Blocks asynchronously until an event is available or the source closes.
    ///
    /// # Errors
    /// See docs/spec/interfaces/server-ingress.md — `EventIngress::next_event()`
    async fn next_event(&mut self) -> Result<Option<ProcessableEvent>, IngressError>;
}

// ---------------------------------------------------------------------------
// WebhookQueueMessage
// ---------------------------------------------------------------------------

/// Serialized format of one GitHub webhook event stored in the queue.
///
/// Schema version **1** is the initial format. Increment `schema_version` and
/// add a migration arm in `QueueIngress::next_event()` for every breaking change.
///
/// The session ID (`"{org}/{repo}/{pr_number}"`) is stored in the broker-level
/// message envelope, NOT in this struct.
///
/// See docs/spec/interfaces/server-ingress.md — `WebhookQueueMessage`
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookQueueMessage {
    /// Schema version byte for forward-compatible migration. Currently `1`.
    pub schema_version: u8,
    /// GitHub-Event header value (e.g. `"pull_request"`).
    pub event_type: String,
    /// X-GitHub-Delivery UUID string.
    pub delivery_id: String,
    /// GitHub App installation id.
    pub installation_id: u64,
    /// UTC timestamp at which the webhook POST was received.
    pub received_at: chrono::DateTime<chrono::Utc>,
    /// Raw JSON body (string-encoded for binary broker compatibility).
    pub raw_payload: String,
}

// ---------------------------------------------------------------------------
// NoOpAck
// ---------------------------------------------------------------------------

/// [`EventAcknowledger`] for webhook mode — both operations are no-ops.
///
/// See docs/spec/interfaces/server-ingress.md — `NoOpAck`
#[allow(dead_code)]
pub struct NoOpAck;

#[async_trait::async_trait]
impl EventAcknowledger for NoOpAck {
    async fn complete(self: Box<Self>) -> Result<(), IngressError> {
        Ok(())
    }

    async fn reject(self: Box<Self>, _reason: &str) -> Result<(), IngressError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// WebhookIngress
// ---------------------------------------------------------------------------

/// [`EventIngress`] implementation for webhook receiver mode.
///
/// Events arrive via an in-process `tokio::sync::mpsc` channel whose sender is
/// owned by the Axum POST handler. EOF is signalled by dropping all senders.
///
/// See docs/spec/interfaces/server-ingress.md — `WebhookIngress`
#[allow(dead_code)]
pub struct WebhookIngress {
    receiver: tokio::sync::mpsc::Receiver<EventEnvelope>,
}

impl WebhookIngress {
    /// Creates a new `WebhookIngress` from the receiving end of the channel.
    #[allow(dead_code)]
    pub fn new(receiver: tokio::sync::mpsc::Receiver<EventEnvelope>) -> Self {
        WebhookIngress { receiver }
    }
}

#[async_trait::async_trait]
impl EventIngress for WebhookIngress {
    async fn next_event(&mut self) -> Result<Option<ProcessableEvent>, IngressError> {
        match self.receiver.recv().await {
            Some(envelope) => Ok(Some(ProcessableEvent {
                envelope,
                ack: Box::new(NoOpAck),
            })),
            // All senders have been dropped — signal EOF.
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// QueueIngress
// ---------------------------------------------------------------------------

/// [`EventIngress`] implementation for queue receiver mode.
///
/// Reads [`WebhookQueueMessage`] payloads from the configured queue provider.
///
/// **NOTE TO IMPLEMENTOR (task 3.0)**: Add a
/// `queue_client: Arc<dyn queue_runtime::QueueClient>` field and implement the
/// actual queue polling logic once `queue-runtime` is wired into the workspace
/// `Cargo.toml`.
///
/// See docs/spec/interfaces/server-ingress.md — `QueueIngress`
#[allow(dead_code)]
pub struct QueueIngress {
    /// Name of the queue to consume.
    pub queue_name: String,
    /// Maximum number of messages that may be in-flight simultaneously.
    pub concurrency: usize,
}

impl QueueIngress {
    /// Creates a new `QueueIngress`.
    #[allow(dead_code)]
    pub fn new(queue_name: String, concurrency: usize) -> Self {
        QueueIngress {
            queue_name,
            concurrency,
        }
    }
}

#[async_trait::async_trait]
impl EventIngress for QueueIngress {
    async fn next_event(&mut self) -> Result<Option<ProcessableEvent>, IngressError> {
        todo!("See docs/spec/interfaces/server-ingress.md — QueueIngress::next_event() — requires task 3.0")
    }
}

// ---------------------------------------------------------------------------
// run_event_processor
// ---------------------------------------------------------------------------

/// Drives the core event-processing pipeline from an ingress source.
///
/// Intended to be spawned as a background `tokio::task` at startup. Runs until
/// the ingress source signals EOF (`Ok(None)`) or an unrecoverable error occurs.
///
/// See docs/spec/interfaces/server-ingress.md — `run_event_processor()`
///
/// # Errors
/// Returns the first [`IngressError`] that is not recoverable in the loop.
#[allow(dead_code)]
pub async fn run_event_processor(
    mut ingress: Box<dyn EventIngress + Send>,
    state: Arc<crate::webhook::AppState>,
) -> Result<(), IngressError> {
    let handler = crate::webhook::MergeWardenWebhookHandler::new(
        state.github_client.clone(),
        state.policies.clone(),
    );

    while let Some(event) = ingress.next_event().await? {
        match handler.handle_event(&event.envelope).await {
            Ok(()) => {
                if let Err(e) = event.ack.complete().await {
                    error!(error = %e, "Failed to acknowledge processed event");
                }
            }
            Err(e) => {
                let reason = e.to_string();
                error!(error = %e, "Event processing failed; dead-lettering");
                if let Err(ack_err) = event.ack.reject(&reason).await {
                    error!(error = %ack_err, "Failed to dead-letter failed event");
                }
            }
        }
    }

    Ok(())
}
