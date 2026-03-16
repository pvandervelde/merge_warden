// Merge Warden server binary entry point.
//
// See docs/spec/interfaces/server-config.md    — startup configuration
// See docs/spec/interfaces/server-ingress.md   — event ingress abstraction
// See docs/spec/design/containerisation.md     — deployment spec

mod config;
mod errors;
mod ingress;
mod telemetry;
mod webhook;

use std::sync::Arc;

use errors::ServerError;
use github_bot_sdk::client::{ClientConfig, GitHubClient};
use merge_warden_developer_platforms::app_auth::AppAuthProvider;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    // 1. Initialise telemetry first so all subsequent log messages are captured.
    let telemetry_config = telemetry::TelemetryConfig::from_env();
    telemetry::init_telemetry(&telemetry_config)?;

    info!("Starting merge-warden-server");

    // 2. Load secrets (fail fast if any required env var is absent).
    debug!("Loading secrets from environment");
    let secrets = config::load_secrets()?;

    // 3. Load application configuration.
    debug!("Loading application configuration");
    let server_config = config::load_config()?;

    info!(
        port = server_config.port,
        receiver_mode = ?server_config.receiver_mode,
        "Configuration loaded"
    );

    // 4. Initialise GitHub App client.
    debug!(
        app_id = secrets.github_app_id,
        "Initialising GitHub App client"
    );
    let auth = AppAuthProvider::new(
        secrets.github_app_id,
        secrets.github_app_private_key.expose(),
        "https://api.github.com",
    )
    .map_err(|e| {
        ServerError::AuthError(format!("Failed to create GitHub App auth provider: {}", e))
    })?;

    let github_client = GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .map_err(|e| ServerError::AuthError(format!("Failed to build GitHub client: {}", e)))?;

    debug!("GitHub App client initialised");

    // 5. Build the WebhookReceiver with the MergeWardenWebhookHandler registered.
    let receiver = webhook::build_webhook_receiver(
        secrets.github_webhook_secret.expose(),
        github_client.clone(),
        server_config.application_defaults.clone(),
    )
    .await;

    // 6. Build AppState and Axum router.
    let state = Arc::new(webhook::AppState {
        receiver,
        github_client,
        policies: server_config.application_defaults,
    });

    let router = webhook::build_router(Arc::clone(&state));

    // 7. Bind the TCP listener.
    let addr = format!("0.0.0.0:{}", server_config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ServerError::AuthError(format!("Failed to bind {}: {}", addr, e)))?;

    info!(address = addr.as_str(), "Listening for requests");

    // 8. Serve. Queue mode processor (task 3.0) would be spawned here.
    axum::serve(listener, router)
        .await
        .map_err(|e| ServerError::AuthError(format!("Server error: {}", e)))?;

    Ok(())
}
