// See docs/spec/interfaces/server-ingress.md — AppState

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
