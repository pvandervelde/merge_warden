// See docs/spec/interfaces/server-ingress.md for the full contract.
//
// Two ingress modes are provided:
//   - `WebhookIngress`: in-process `mpsc` channel, `NoOpAck`
//   - `QueueIngress`:   `queue-runtime` session consumer, `QueueMessageAck`
//
// The shared `run_event_processor` loop is mode-agnostic.
use std::sync::Arc;

use github_bot_sdk::events::{EventProcessor, ProcessorConfig};
use github_bot_sdk::webhook::WebhookHandler;
use queue_runtime::{QueueClient, QueueError, QueueName, SessionClient};
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
#[derive(Debug, thiserror::Error)]
pub enum IngressError {
    /// The queue provider returned an unrecoverable error.
    #[error("Queue error: {message}")]
    QueueError { message: String },
}

// ---------------------------------------------------------------------------
// EventAcknowledger
// ---------------------------------------------------------------------------

/// Lifecycle hook for acknowledging an event to the underlying broker.
///
/// See docs/spec/interfaces/server-ingress.md — `EventAcknowledger`
#[async_trait::async_trait]
pub trait EventAcknowledger: Send {
    /// Marks the event as successfully processed.
    ///
    /// For queue backends, deletes or completes the broker message then
    /// releases the session lock.  For webhook backend (`NoOpAck`), this is
    /// a no-op.
    ///
    /// # Errors
    /// Returns [`IngressError::QueueError`] if the broker cannot be reached.
    async fn complete(self: Box<Self>) -> Result<(), IngressError>;

    /// Marks the event as permanently failed.
    ///
    /// For queue backends, moves the message to the dead-letter queue then
    /// releases the session lock.  The `reason` string is stored as a
    /// diagnostic property on the dead-lettered message.
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookQueueMessage {
    /// Schema version byte for forward-compatible migration. Currently `1`.
    pub schema_version: u8,
    /// GitHub-Event header value (e.g. `"pull_request"`).
    pub event_type: String,
    /// X-GitHub-Delivery UUID string.
    pub delivery_id: String,
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
/// GitHub has already received the 202 before the event reaches the processing
/// loop, so there is nothing to acknowledge to a broker.
///
/// See docs/spec/interfaces/server-ingress.md — `NoOpAck`
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
// QueueMessageAck
// ---------------------------------------------------------------------------

/// [`EventAcknowledger`] for queue mode.
///
/// Delegates to the underlying [`SessionClient`] to complete or dead-letter
/// the queue message, then closes the session lock so the next event in the
/// same session can be processed.
///
/// `session_client` is wrapped in `Option` so that the `Drop` impl can
/// take it; after `complete()` or `reject()` drains it the field is `None`
/// and the `Drop` impl becomes a no-op.
///
/// See docs/spec/interfaces/server-ingress.md — `QueueMessageAck`
pub(crate) struct QueueMessageAck {
    session_client: Option<Box<dyn SessionClient>>,
    receipt: queue_runtime::ReceiptHandle,
}

impl Drop for QueueMessageAck {
    /// Closes the session lock if the ack was dropped without being consumed.
    ///
    /// Limits the stuck-session window (e.g. Azure Service Bus 5-minute lock)
    /// when a processor task is aborted mid-flight: rather than waiting for the
    /// broker timeout to release the lock, a best-effort `close_session()` is
    /// spawned immediately.
    fn drop(&mut self) {
        if let Some(session) = self.session_client.take() {
            // Use try_current so a non-runtime context (e.g. test teardown)
            // does not panic.  If no runtime is available the session lock
            // expires naturally.
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let _ = session.close_session().await;
                });
            }
        }
    }
}

#[async_trait::async_trait]
impl EventAcknowledger for QueueMessageAck {
    async fn complete(mut self: Box<Self>) -> Result<(), IngressError> {
        let session = self
            .session_client
            .take()
            .expect("session_client already consumed — Drop ran before complete()");
        session
            .complete_message(self.receipt.clone())
            .await
            .map_err(|e| IngressError::QueueError {
                message: e.to_string(),
            })?;
        // Ignore close error — the message was already successfully completed.
        let _ = session.close_session().await;
        Ok(())
    }

    async fn reject(mut self: Box<Self>, reason: &str) -> Result<(), IngressError> {
        let session = self
            .session_client
            .take()
            .expect("session_client already consumed — Drop ran before reject()");
        session
            .dead_letter_message(self.receipt.clone(), reason.to_string())
            .await
            .map_err(|e| IngressError::QueueError {
                message: e.to_string(),
            })?;
        let _ = session.close_session().await;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// WebhookIngress
// ---------------------------------------------------------------------------

/// [`EventIngress`] implementation for webhook receiver mode.
///
/// Events arrive via an in-process `tokio::sync::mpsc` channel whose sender is
/// owned by [`crate::webhook::ChannelForwardingHandler`]. EOF is signalled by
/// dropping all senders (i.e. when the Axum server shuts down).
///
/// The `mpsc` channel capacity acts as a back-pressure limit: once the channel
/// is full, new webhook POSTs will block inside the SDK handler until a worker
/// drains a slot.
///
/// See docs/spec/interfaces/server-ingress.md — `WebhookIngress`
pub struct WebhookIngress {
    receiver: tokio::sync::mpsc::Receiver<EventEnvelope>,
}

impl WebhookIngress {
    /// Creates a new `WebhookIngress` from the receiving end of an mpsc channel.
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
            // All senders dropped — signal EOF so run_event_processor exits cleanly.
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// QueueIngress
// ---------------------------------------------------------------------------

/// [`EventIngress`] implementation for queue receiver mode.
///
/// Reads [`WebhookQueueMessage`] payloads from the configured queue provider
/// using session-based ordering.  Each PR's events are processed sequentially
/// via their session lock; events for different PRs may be processed
/// concurrently by multiple `QueueIngress` instances running in parallel tasks.
///
/// Malformed messages (unknown schema version, unparseable payload, or
/// unprocessable `EventEnvelope`) are automatically dead-lettered and the
/// ingress loop continues with the next available session.
///
/// `next_event()` never returns `Ok(None)` — it polls indefinitely until an
/// unrecoverable [`QueueError`] forces early termination.  The caller (i.e.
/// the spawned processor task) is stopped by aborting its `JoinHandle`.
///
/// See docs/spec/interfaces/server-ingress.md — `QueueIngress`
pub struct QueueIngress {
    /// Provider-agnostic queue client shared with the enqueue side.
    queue_client: Arc<dyn QueueClient>,
    /// Queue name to consume.
    queue_name: QueueName,
    /// Re-usable event processor for reconstructing `EventEnvelope` from raw bytes.
    event_processor: EventProcessor,
}

impl QueueIngress {
    /// Creates a new `QueueIngress`.
    ///
    /// In production (Azure Service Bus, AWS SQS) pass any client configured
    /// for the target queue — the queue client is independent of any enqueue
    /// side. For the in-memory provider (dev/test) pass the **same** client
    /// instance used to enqueue so both sides share the same in-memory state.
    ///
    /// # Arguments
    /// - `queue_client`: Shared queue client.
    /// - `queue_name`:   Name of the queue to consume.
    pub fn new(queue_client: Arc<dyn QueueClient>, queue_name: QueueName) -> Self {
        QueueIngress {
            queue_client,
            queue_name,
            event_processor: EventProcessor::new(ProcessorConfig::default()),
        }
    }
}

#[async_trait::async_trait]
impl EventIngress for QueueIngress {
    async fn next_event(&mut self) -> Result<Option<ProcessableEvent>, IngressError> {
        loop {
            // Accept any available session.  `SessionNotFound` / `QueueNotFound`
            // mean there are no messages yet — wait and retry.
            let session = match self
                .queue_client
                .accept_session(&self.queue_name, None)
                .await
            {
                Ok(s) => s,
                Err(QueueError::SessionNotFound { .. }) | Err(QueueError::QueueNotFound { .. }) => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
                Err(e) => {
                    return Err(IngressError::QueueError {
                        message: e.to_string(),
                    });
                }
            };

            // Receive the first message in this session.
            let received = match session.receive_message(chrono::Duration::seconds(5)).await {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    // Session lock acquired but no message available — release and retry.
                    let _ = session.close_session().await;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => {
                    let _ = session.close_session().await;
                    return Err(IngressError::QueueError {
                        message: e.to_string(),
                    });
                }
            };

            let receipt = received.receipt_handle;

            // Deserialize the WebhookQueueMessage from the raw message body.
            let queue_msg: WebhookQueueMessage = match serde_json::from_slice(&received.body) {
                Ok(m) => m,
                Err(e) => {
                    let reason = format!("Deserialization error: {e}");
                    error!("{reason} — dead-lettering message");
                    let _ = session.dead_letter_message(receipt, reason).await;
                    let _ = session.close_session().await;
                    continue;
                }
            };

            // Validate schema version.
            if queue_msg.schema_version != 1 {
                let reason = format!(
                    "Unknown schema version {} — expected 1",
                    queue_msg.schema_version
                );
                error!("{reason} — dead-lettering message");
                let _ = session.dead_letter_message(receipt, reason).await;
                let _ = session.close_session().await;
                continue;
            }

            // Reconstruct EventEnvelope from the stored raw payload bytes.
            let envelope = match self
                .event_processor
                .process_webhook(
                    &queue_msg.event_type,
                    queue_msg.raw_payload.as_bytes(),
                    Some(&queue_msg.delivery_id),
                )
                .await
            {
                Ok(env) => env,
                Err(e) => {
                    let reason = format!("EventEnvelope reconstruction failed: {e}");
                    error!("{reason} — dead-lettering message");
                    let _ = session.dead_letter_message(receipt, reason).await;
                    let _ = session.close_session().await;
                    continue;
                }
            };

            return Ok(Some(ProcessableEvent {
                envelope,
                ack: Box::new(QueueMessageAck {
                    session_client: Some(session),
                    receipt,
                }),
            }));
        }
    }
}

// ---------------------------------------------------------------------------
// run_event_processor
// ---------------------------------------------------------------------------

/// Drives the core event-processing pipeline from an ingress source.
///
/// Intended to be spawned as a background `tokio::task` at startup.  Runs
/// until the ingress source signals EOF (`Ok(None)`) or an unrecoverable
/// error occurs.
///
/// # Processing Loop
/// 1. `ingress.next_event().await`
/// 2. On `Ok(Some(event))`: run handler; call `ack.complete()` on success or
///    `ack.reject(reason)` on domain error.
/// 3. On `Ok(None)`: break, return `Ok(())`.
/// 4. On `Err(e)`: return `Err(e)`.
///
/// # Cancellation
/// The function does not install its own cancellation signal.  Abort the
/// spawned `JoinHandle` (or drop all ingress senders in webhook mode) to stop.
///
/// See docs/spec/interfaces/server-ingress.md — `run_event_processor()`
///
/// # Errors
/// Returns the first [`IngressError`] that is not recoverable in-loop.
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
