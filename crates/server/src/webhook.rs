// See docs/spec/interfaces/server-ingress.md    — AppState
// See docs/spec/interfaces/developer-platforms-sdk.md — MergeWardenWebhookHandler
// See docs/spec/design/containerisation.md       — HTTP routes

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use github_bot_sdk::{
    auth::{GitHubAppId, InstallationId, PrivateKey, SecretProvider},
    client::GitHubClient,
    error::SecretError,
    events::{EventEnvelope, EventProcessor, ProcessorConfig},
    webhook::{WebhookHandler, WebhookReceiver, WebhookRequest},
};
use merge_warden_core::{
    config::{
        load_merge_warden_config, ApplicationDefaults, CurrentPullRequestValidationConfiguration,
    },
    MergeWarden,
};
use merge_warden_developer_platforms::github::GitHubProvider;
use queue_runtime::{Message, QueueClient, QueueName, SessionId};
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

use crate::errors::ServerError;

#[cfg(test)]
#[path = "webhook_tests.rs"]
mod tests;

// ---------------------------------------------------------------------------
// WebhookSecretProvider
// ---------------------------------------------------------------------------

/// [`SecretProvider`] backed by a webhook secret already loaded at startup.
///
/// The [`WebhookReceiver`] uses this only for HMAC-SHA256 signature validation.
/// The `get_private_key` and `get_app_id` methods are not called in this path.
struct WebhookSecretProvider {
    /// The webhook signing secret used for HMAC-SHA256 validation.
    ///
    /// Stored as a plain `String` (not `SecretString`) because:
    /// 1. This struct is private and never derived `Debug`, so the value
    ///    cannot leak into log output inadvertently.
    /// 2. The SDK's `SecretProvider::get_webhook_secret` returns `String`,
    ///    so a `SecretString` would need to be exposed immediately anyway.
    /// 3. The struct is short-lived: created once at startup and then moved
    ///    into `Arc<dyn SecretProvider>` inside `WebhookReceiver`.
    webhook_secret: String,
}

#[async_trait]
impl SecretProvider for WebhookSecretProvider {
    async fn get_private_key(&self) -> Result<PrivateKey, SecretError> {
        // Not invoked by SignatureValidator.
        Err(SecretError::NotFound {
            key: "private_key".to_string(),
        })
    }

    async fn get_app_id(&self) -> Result<GitHubAppId, SecretError> {
        // Not invoked by SignatureValidator.
        Err(SecretError::NotFound {
            key: "app_id".to_string(),
        })
    }

    async fn get_webhook_secret(&self) -> Result<String, SecretError> {
        Ok(self.webhook_secret.clone())
    }

    fn cache_duration(&self) -> chrono::Duration {
        chrono::Duration::hours(1)
    }
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Shared state threaded through the Axum router and the event-processor task.
///
/// Constructed once in `main()` from [`crate::config::ServerSecrets`] and
/// [`crate::config::ServerConfig`] and then wrapped in `Arc` before passing
/// to [`build_router`].
///
/// See docs/spec/interfaces/server-ingress.md — `AppState`
pub struct AppState {
    /// SDK webhook receiver: validates HMAC signatures and dispatches validated
    /// events to registered [`WebhookHandler`]s asynchronously (fire-and-forget).
    pub receiver: WebhookReceiver,
    /// GitHub App client for creating installation-scoped API clients.
    ///
    /// `Clone` is cheap — it shares `Arc<dyn AuthenticationProvider>` internally.
    pub github_client: GitHubClient,
    /// Application policy defaults loaded from configuration.
    pub policies: ApplicationDefaults,
}

// ---------------------------------------------------------------------------
// MergeWardenWebhookHandler
// ---------------------------------------------------------------------------

/// Dispatches validated GitHub webhook events into the Merge Warden processing
/// pipeline.
///
/// Registered on [`WebhookReceiver`] during startup. Implements the SDK's
/// [`WebhookHandler`] trait, which fires the handler asynchronously after the
/// HTTP response is sent to GitHub, satisfying GitHub's 10-second timeout.
///
/// See docs/spec/interfaces/developer-platforms-sdk.md — `MergeWardenWebhookHandler`
pub struct MergeWardenWebhookHandler {
    /// GitHub App client. `Clone` is cheap (Arc-backed internally).
    github_client: GitHubClient,
    /// Policy defaults used when no per-repo config file is found.
    policies: ApplicationDefaults,
}

impl MergeWardenWebhookHandler {
    /// Creates a new handler from the given client and policy defaults.
    pub fn new(github_client: GitHubClient, policies: ApplicationDefaults) -> Self {
        MergeWardenWebhookHandler {
            github_client,
            policies,
        }
    }

    /// Creates a new handler sharing the state already constructed for [`AppState`].
    #[allow(dead_code)]
    pub fn new_from_state(state: &AppState) -> Self {
        Self::new(state.github_client.clone(), state.policies.clone())
    }

    /// Processes a `pull_request` webhook event.
    ///
    /// Validates the action, extracts PR metadata, builds a per-installation
    /// GitHub client, loads the repo-level config, and delegates to
    /// [`MergeWarden::process_pull_request`].
    pub async fn handle_pull_request(&self, envelope: &EventEnvelope) -> Result<(), ServerError> {
        let action = envelope.payload.raw()["action"].as_str().unwrap_or("");
        match action {
            "opened" | "edited" | "ready_for_review" | "reopened" | "unlocked" | "synchronize" => {}
            _ => {
                info!(action, "Pull request action does not require processing");
                return Ok(());
            }
        }

        let pr_number = envelope
            .entity_id
            .as_deref()
            .and_then(|id| id.parse::<u32>().ok())
            .or_else(|| {
                envelope.payload.raw()["pull_request"]["number"]
                    .as_u64()
                    .map(|n| n as u32)
            });

        let pr_number = match pr_number {
            Some(n) => n,
            None => {
                error!(
                    repository = envelope.repository.full_name.as_str(),
                    "Webhook payload missing pull request number"
                );
                return Err(ServerError::ProcessingError(
                    "Missing pull request number in webhook payload".to_string(),
                ));
            }
        };

        let installation_id = match envelope.payload.raw()["installation"]["id"].as_u64() {
            Some(id) => id,
            None => {
                error!(
                    repository = envelope.repository.full_name.as_str(),
                    pull_request = pr_number,
                    "Webhook payload missing installation ID"
                );
                return Err(ServerError::ProcessingError(
                    "Missing installation ID in webhook payload".to_string(),
                ));
            }
        };

        let repo_owner = &envelope.repository.owner.login;
        let repo_name = &envelope.repository.name;

        info!(
            repository_owner = repo_owner.as_str(),
            repository = repo_name.as_str(),
            pull_request = pr_number,
            action,
            "Processing pull request"
        );

        let installation_client = self
            .github_client
            .installation_by_id(InstallationId::new(installation_id))
            .await
            .map_err(|e| {
                error!(
                    repository_owner = repo_owner.as_str(),
                    repository = repo_name.as_str(),
                    pull_request = pr_number,
                    error = %e,
                    "Failed to create installation client"
                );
                ServerError::AuthError(format!("Failed to create installation client: {}", e))
            })?;

        let provider = GitHubProvider::new(installation_client);

        let merge_warden_config_path = ".github/merge-warden.toml";

        let validation_config = match load_merge_warden_config(
            repo_owner,
            repo_name,
            merge_warden_config_path,
            &provider,
            &self.policies,
        )
        .await
        {
            Ok(config) => {
                info!(
                    "Loaded merge-warden config from {}",
                    merge_warden_config_path
                );
                config.to_validation_config(&self.policies.bypass_rules)
            }
            Err(e) => {
                warn!(
                    "Failed to load merge-warden config from {}: {}. Using defaults.",
                    merge_warden_config_path, e
                );
                CurrentPullRequestValidationConfiguration {
                    enforce_title_convention: self.policies.enable_title_validation,
                    title_pattern: self.policies.default_title_pattern.clone(),
                    invalid_title_label: self.policies.default_invalid_title_label.clone(),
                    enforce_work_item_references: self.policies.enable_work_item_validation,
                    work_item_reference_pattern: self.policies.default_work_item_pattern.clone(),
                    missing_work_item_label: self.policies.default_missing_work_item_label.clone(),
                    pr_size_check: self.policies.pr_size_check.clone(),
                    change_type_labels: Some(self.policies.change_type_labels.clone()),
                    bypass_rules: self.policies.bypass_rules.clone(),
                }
            }
        };

        let warden = MergeWarden::with_config(provider, validation_config);

        warden
            .process_pull_request(repo_owner, repo_name, pr_number.into())
            .await
            .map_err(|e| {
                error!(
                    repository_owner = repo_owner.as_str(),
                    repository = repo_name.as_str(),
                    pull_request = pr_number,
                    error = %e,
                    "Failed to process pull request"
                );
                ServerError::ProcessingError(format!("Failed to process pull request: {}", e))
            })?;

        info!(
            repository_owner = repo_owner.as_str(),
            repository = repo_name.as_str(),
            pull_request = pr_number,
            "Pull request processing completed"
        );

        Ok(())
    }
}

#[async_trait]
impl WebhookHandler for MergeWardenWebhookHandler {
    async fn handle_event(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if envelope.event_type != "pull_request" {
            debug!(event_type = %envelope.event_type, "Ignoring non-pull-request event");
            return Ok(());
        }

        self.handle_pull_request(envelope)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

// ---------------------------------------------------------------------------
// ChannelForwardingHandler
// ---------------------------------------------------------------------------

/// [`WebhookHandler`] for webhook receiver mode.
///
/// Registered on [`WebhookReceiver`] when `MERGE_WARDEN_RECEIVER_MODE=webhook`
/// (the default). Forwards validated [`EventEnvelope`]s into an in-process
/// `mpsc` channel so the shared [`crate::ingress::run_event_processor`] loop
/// can process them through [`crate::ingress::WebhookIngress`].
///
/// The HTTP handler returns 202 immediately after the SDK validates the
/// signature and before this handler is called, so the channel send does not
/// block the HTTP response path.
///
/// The `mpsc` channel capacity (set in [`build_webhook_receiver`]) provides
/// back-pressure: if the processing loop falls behind, sends will yield until
/// a slot is available.
pub(crate) struct ChannelForwardingHandler {
    sender: mpsc::Sender<crate::ingress::EventEnvelope>,
}

#[async_trait]
impl WebhookHandler for ChannelForwardingHandler {
    async fn handle_event(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sender
            .send(envelope.clone())
            .await
            .map_err(|e| format!("Failed to forward event to processing channel: {e}").into())
    }
}

// ---------------------------------------------------------------------------
// QueueEnqueueHandler
// ---------------------------------------------------------------------------

/// [`WebhookHandler`] for queue receiver mode.
///
/// Registered on [`WebhookReceiver`] when `MERGE_WARDEN_RECEIVER_MODE=queue`.
/// Instead of executing the processing pipeline inline, it serialises the
/// validated `EventEnvelope` into a [`WebhookQueueMessage`] and enqueues it
/// for asynchronous processing by the [`crate::ingress::QueueIngress`] task.
///
/// The session ID is set to `"{repo_full_name}/{pr_number}"` so that all
/// events for the same PR are processed in arrival order.
///
/// See docs/spec/design/queue-architecture.md — Component Responsibilities
pub struct QueueEnqueueHandler {
    queue_client: Arc<dyn QueueClient>,
    queue_name: QueueName,
}

impl QueueEnqueueHandler {
    /// Creates a new `QueueEnqueueHandler`.
    pub fn new(queue_client: Arc<dyn QueueClient>, queue_name: QueueName) -> Self {
        QueueEnqueueHandler {
            queue_client,
            queue_name,
        }
    }
}

#[async_trait]
impl WebhookHandler for QueueEnqueueHandler {
    async fn handle_event(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::ingress::WebhookQueueMessage;
        use bytes::Bytes;

        // Only enqueue PR-related events; drop the rest.
        if envelope.event_type != "pull_request" && envelope.event_type != "pull_request_review" {
            debug!(event_type = %envelope.event_type, "Skipping non-PR event in queue mode");
            return Ok(());
        }

        let raw_payload = serde_json::to_string(envelope.payload.raw())
            .map_err(|e| format!("Failed to serialise payload: {e}"))?;

        let installation_id = envelope.payload.raw()["installation"]["id"]
            .as_u64()
            .unwrap_or(0);

        let delivery_id = envelope
            .metadata
            .delivery_id
            .clone()
            .unwrap_or_else(|| envelope.event_id.to_string());

        let queue_msg = WebhookQueueMessage {
            schema_version: 1,
            event_type: envelope.event_type.clone(),
            delivery_id,
            installation_id,
            received_at: chrono::Utc::now(),
            raw_payload,
        };

        let body_bytes = serde_json::to_vec(&queue_msg)
            .map_err(|e| format!("Failed to serialise queue message: {e}"))?;

        // Build session ID: "{repo_full_name}/{pr_number}" for ordered per-PR processing.
        let repo_full_name = &envelope.repository.full_name;
        let pr_number = envelope
            .entity_id
            .as_deref()
            .unwrap_or("0");
        let session_str = format!("{}/{}", repo_full_name, pr_number);

        let session_id = SessionId::new(session_str.clone()).map_err(|e| {
            format!("Invalid session ID '{session_str}': {e}")
        })?;

        let message = Message::new(Bytes::from(body_bytes)).with_session_id(session_id);

        self.queue_client
            .send_message(&self.queue_name, message)
            .await
            .map_err(|e| format!("Failed to enqueue message: {e}"))?;

        debug!(
            event_type = %envelope.event_type,
            session_id = %session_str,
            "Enqueued event for queue processing"
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// HTTP route handlers
// ---------------------------------------------------------------------------

/// `POST /api/merge_warden` — receives a raw GitHub webhook POST, validates the
/// HMAC-SHA256 signature, and dispatches the validated event asynchronously.
///
/// # Responses
/// - `202 Accepted` — event accepted for processing.
/// - `400 Bad Request` — missing required headers or malformed body.
/// - `401 Unauthorized` — HMAC signature validation failed.
/// - `500 Internal Server Error` — unexpected processing error.
///
/// See docs/spec/design/containerisation.md — HTTP routes
#[instrument(skip(state, headers, body))]
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    debug!("Received webhook POST");

    let header_map: HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|v| (k.as_str().to_lowercase(), v.to_string()))
        })
        .collect();

    let request = WebhookRequest::new(header_map, body);
    let response = state.receiver.receive_webhook(request).await;

    match response.status_code() {
        200 => (StatusCode::ACCEPTED, response.message().to_string()),
        401 => (StatusCode::UNAUTHORIZED, response.message().to_string()),
        500 => (
            StatusCode::INTERNAL_SERVER_ERROR,
            response.message().to_string(),
        ),
        _ => (StatusCode::BAD_REQUEST, response.message().to_string()),
    }
}

/// `GET /api/merge_warden` — liveness probe for container orchestrators.
///
/// Returns `200 OK` without checking external dependencies (GitHub API, queue).
///
/// See docs/spec/design/containerisation.md — health check
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Builds the Axum [`Router`] with all routes and shared state attached.
///
/// Routes:
/// - `GET  /api/merge_warden` → [`health_check`]
/// - `POST /api/merge_warden` → [`handle_webhook`]
///
/// See docs/spec/design/containerisation.md — HTTP routes
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/merge_warden", get(health_check))
        .route("/api/merge_warden", post(handle_webhook))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Startup helpers
// ---------------------------------------------------------------------------

/// Builds a [`WebhookReceiver`] configured for the active receiver mode.
///
/// **Webhook mode** (`queue_client == None`): registers a
/// [`ChannelForwardingHandler`] that pushes validated events into an in-process
/// `mpsc` channel. Returns `Some(receiver)` so the caller can hand it off to
/// [`crate::ingress::WebhookIngress`] running in the shared processor loop.
///
/// **Queue mode** (`queue_client == Some(...)`): registers a
/// [`QueueEnqueueHandler`] that serialises events into the external queue.
/// Returns `None` for the channel receiver — the processor loop reads from the
/// queue instead.
///
/// The channel capacity is set to 64, providing back-pressure for webhook
/// bursts without unbounded memory growth.
pub async fn build_webhook_receiver(
    webhook_secret: &str,
    queue_client: Option<(Arc<dyn QueueClient>, QueueName)>,
) -> (WebhookReceiver, Option<mpsc::Receiver<crate::ingress::EventEnvelope>>) {
    let secret_provider = Arc::new(WebhookSecretProvider {
        webhook_secret: webhook_secret.to_string(),
    });
    let processor = EventProcessor::new(ProcessorConfig::default());
    let mut receiver = WebhookReceiver::new(secret_provider, processor);

    if let Some((client, queue_name)) = queue_client {
        let handler = Arc::new(QueueEnqueueHandler::new(client, queue_name));
        receiver.add_handler(handler).await;
        (receiver, None)
    } else {
        let (tx, rx) = mpsc::channel(64);
        let handler = Arc::new(ChannelForwardingHandler { sender: tx });
        receiver.add_handler(handler).await;
        (receiver, Some(rx))
    }
}
