// See docs/spec/interfaces/server-ingress.md    — AppState
// See docs/spec/interfaces/developer-platforms-sdk.md — MergeWardenWebhookHandler
// See docs/spec/design/containerisation.md       — HTTP routes

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Router};
use merge_warden_core::config::ApplicationDefaults;

/// Shared state threaded through the Axum router and the event-processor task.
///
/// Constructed once in `main()` from [`crate::config::ServerSecrets`] and
/// [`crate::config::ServerConfig`].
///
/// **NOTE TO IMPLEMENTOR (task 1.0)**: Replace the `github_app_id`,
/// `github_app_private_key`, and `webhook_secret` fields with an
/// `Arc<github_bot_sdk::client::GitHubClient>` once the SDK is integrated. The
/// raw private key must not be stored as a plain `String` in production.
///
/// See docs/spec/interfaces/server-ingress.md — `AppState`
#[derive(Clone)]
pub struct AppState {
    /// Numeric GitHub App ID.
    pub github_app_id: u64,
    /// PEM-encoded private key. Stored as `String` so it can be cheaply cloned
    /// into request handlers. **Do not log this value.**
    pub github_app_private_key: String,
    /// Webhook HMAC signing secret. **Do not log this value.**
    pub webhook_secret: String,
    /// Application policy defaults loaded from configuration.
    pub policies: ApplicationDefaults,
    /// Channel sender for queue mode. `None` in webhook mode.
    pub event_sender: Option<tokio::sync::mpsc::Sender<crate::ingress::EventEnvelope>>,
}

// ---------------------------------------------------------------------------
// MergeWardenWebhookHandler
// ---------------------------------------------------------------------------

/// Dispatches validated GitHub webhook events into the ingress pipeline.
///
/// **NOTE TO IMPLEMENTOR (task 1.0)**: This struct should implement
/// `github_bot_sdk::webhook::WebhookHandler` once the SDK is wired in. The
/// `handle()` method should match on `envelope.event_type` and dispatch to
/// the appropriate domain handler (currently only `"pull_request"` is handled;
/// all other event types are silently ignored).
///
/// # Dispatch logic (pseudocode)
/// ```text
/// match envelope.event_type.as_str() {
///     "pull_request" => handle_pull_request(envelope, state).await,
///     _ => Ok(()),
/// }
/// ```
///
/// See docs/spec/interfaces/developer-platforms-sdk.md — `MergeWardenWebhookHandler`
pub struct MergeWardenWebhookHandler {
    state: Arc<AppState>,
}

impl MergeWardenWebhookHandler {
    /// Creates a new handler sharing the given application state.
    pub fn new(state: Arc<AppState>) -> Self {
        MergeWardenWebhookHandler { state }
    }

    /// Processes a `pull_request` event.
    ///
    /// Extracts the PR details from `envelope.payload`, builds a
    /// `GitHubProvider` (task 1.0: from `state.client`), and delegates to
    /// `merge_warden_core::check_pull_request()`.
    async fn handle_pull_request(
        &self,
        envelope: crate::ingress::EventEnvelope,
    ) -> Result<(), crate::errors::ServerError> {
        todo!("See docs/spec/interfaces/developer-platforms-sdk.md — MergeWardenWebhookHandler")
    }
}

// ---------------------------------------------------------------------------
// HTTP route handlers
// ---------------------------------------------------------------------------

/// `POST /webhook` — receives a raw GitHub webhook POST, validates the HMAC
/// signature, and forwards the event to the ingress pipeline.
///
/// # Request requirements
/// - `X-Hub-Signature-256` header must be present and valid.
/// - `X-GitHub-Event` header identifies the event type.
/// - `X-GitHub-Delivery` header provides the idempotency UUID.
///
/// # Responses
/// - `202 Accepted` — event accepted for processing (or enqueued in queue mode).
/// - `400 Bad Request` — missing required headers or malformed body.
/// - `401 Unauthorized` — HMAC signature validation failed.
/// - `500 Internal Server Error` — unexpected error forwarding the event.
///
/// See docs/spec/design/containerisation.md — HTTP routes
pub async fn handle_webhook(
    State(_state): State<Arc<AppState>>,
    _request: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    todo!("See docs/spec/design/containerisation.md — POST /webhook handler")
        as (StatusCode, &'static str)
}

/// `GET /health` — liveness probe endpoint for container orchestrators.
///
/// Returns `200 OK` with an empty body when the server process is healthy.
/// Does not check external dependencies (GitHub API, queue).
///
/// See docs/spec/design/containerisation.md — health check
pub async fn health_check() -> impl IntoResponse {
    todo!("See docs/spec/design/containerisation.md — GET /health handler")
        as (StatusCode, &'static str)
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Builds the Axum [`Router`] with all routes and shared state attached.
///
/// Routes:
/// - `POST /webhook` → [`handle_webhook`]
/// - `GET  /health`  → [`health_check`]
///
/// See docs/spec/design/containerisation.md — HTTP routes
pub fn build_router(state: Arc<AppState>) -> Router {
    todo!("See docs/spec/design/containerisation.md — build_router()")
}
