// Merge Warden server binary entry point.
//
// See docs/spec/interfaces/server-config.md    — startup configuration
// See docs/spec/interfaces/server-ingress.md   — event ingress abstraction
// See docs/spec/design/containerisation.md     — deployment spec
// See docs/spec/design/queue-architecture.md   — queue-mode wiring

mod config;
mod errors;
mod ingress;
mod telemetry;
mod webhook;

use std::sync::Arc;

use config::ReceiverMode;
use errors::ServerError;
use github_bot_sdk::client::{ClientConfig, GitHubClient};
use merge_warden_developer_platforms::app_auth::AppAuthProvider;
use queue_runtime::{QueueClientFactory, QueueName};
use tracing::{debug, error, info};

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

    // 4. Fail fast: webhook mode requires GITHUB_WEBHOOK_SECRET.
    if server_config.receiver_mode == ReceiverMode::Webhook
        && secrets.github_webhook_secret.is_none()
    {
        return Err(ServerError::MissingEnvVar(
            "GITHUB_WEBHOOK_SECRET".to_string(),
        ));
    }

    // 5. Initialise GitHub App client.
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

    // 6. Optionally create the queue client (queue mode only).
    let queue_pair: Option<(Arc<dyn queue_runtime::QueueClient>, QueueName)> =
        if server_config.receiver_mode == ReceiverMode::Queue {
            let queue_cfg = server_config
                .queue
                .as_ref()
                .expect("queue config present when mode=queue")
                .to_queue_config()?;

            let queue_name_str = server_config
                .queue
                .as_ref()
                .expect("queue config present when mode=queue")
                .queue_name
                .clone();

            let queue_name = QueueName::new(queue_name_str.clone()).map_err(|e| {
                ServerError::ConfigError(format!("Invalid queue name '{}': {}", queue_name_str, e))
            })?;

            info!(queue_name = %queue_name, "Creating queue client");

            let client = QueueClientFactory::create_client(queue_cfg)
                .await
                .map_err(|e| {
                    ServerError::ConfigError(format!("Failed to create queue client: {}", e))
                })?;

            Some((Arc::from(client), queue_name))
        } else {
            None
        };

    // 7. Build the WebhookReceiver (webhook mode only) or skip (queue mode).
    //
    //    In queue mode, merge-warden is a pure queue consumer. A separate service
    //    receives GitHub webhooks, validates signatures, and enqueues messages.
    //    merge-warden does not expose a POST endpoint in queue mode.
    let (receiver_opt, webhook_rx) = if server_config.receiver_mode == ReceiverMode::Webhook {
        let secret = secrets
            .github_webhook_secret
            .as_ref()
            .expect("webhook secret checked above")
            .expose();
        let (receiver, rx) = webhook::build_webhook_receiver(secret).await;
        (Some(receiver), Some(rx))
    } else {
        (None, None)
    };

    // 8. Build AppState.
    let (queue_client_opt, queue_name_opt) = match queue_pair {
        Some((c, n)) => (Some(c), Some(n)),
        None => (None, None),
    };

    let state = Arc::new(webhook::AppState {
        receiver: receiver_opt,
        github_client: github_client.clone(),
        policies: server_config.application_defaults.clone(),
    });

    // 9. Spawn processor tasks.
    //
    //    Webhook mode: one worker reads from the in-process mpsc channel via
    //    `WebhookIngress`. All events flow through the same `run_event_processor`
    //    loop regardless of mode, giving consistent failure handling and observability.
    //
    //    Queue mode: `concurrency` workers each hold an independent session lock,
    //    so different PRs are processed concurrently while per-PR ordering is
    //    preserved within a session.
    let processor_handles: Vec<tokio::task::JoinHandle<()>> =
        if server_config.receiver_mode == ReceiverMode::Queue {
            let client = queue_client_opt
                .as_ref()
                .expect("queue client present when mode=queue");
            let name = queue_name_opt
                .as_ref()
                .expect("queue name present when mode=queue");
            let concurrency = server_config
                .queue
                .as_ref()
                .expect("queue config present when mode=queue")
                .concurrency;

            info!(workers = concurrency, "Spawning queue processor tasks");

            (0..concurrency)
                .map(|worker_id| {
                    let worker_client = Arc::clone(client);
                    let worker_name = name.clone();
                    let processor_state = Arc::clone(&state);

                    tokio::spawn(async move {
                        let ingress =
                            Box::new(ingress::QueueIngress::new(worker_client, worker_name));
                        if let Err(e) =
                            ingress::run_event_processor(ingress, processor_state).await
                        {
                            error!(worker = worker_id, error = %e, "Queue processor task terminated with error");
                        }
                    })
                })
                .collect()
        } else {
            let rx = webhook_rx.expect("mpsc receiver present in webhook mode");
            let processor_state = Arc::clone(&state);

            info!("Spawning webhook processor task");

            vec![tokio::spawn(async move {
                let ingress = Box::new(ingress::WebhookIngress::new(rx));
                if let Err(e) = ingress::run_event_processor(ingress, processor_state).await {
                    error!(error = %e, "Webhook processor task terminated with error");
                }
            })]
        };

    // 10. Build mode-specific router.
    let router = if server_config.receiver_mode == ReceiverMode::Queue {
        webhook::build_queue_router(Arc::clone(&state))
    } else {
        webhook::build_router(Arc::clone(&state))
    };

    // 11. Bind the TCP listener.
    let addr = format!("0.0.0.0:{}", server_config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ServerError::StartupError(format!("Failed to bind {}: {}", addr, e)))?;

    info!(address = addr.as_str(), "Listening for requests");

    // 12. Serve.  On shutdown abort all processor tasks.
    axum::serve(listener, router)
        .await
        .map_err(|e| ServerError::StartupError(format!("Server error: {}", e)))?;

    for handle in processor_handles {
        handle.abort();
    }

    Ok(())
}
