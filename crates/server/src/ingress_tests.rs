use chrono::Utc;
use github_bot_sdk::{
    client::{OwnerType, Repository, RepositoryOwner},
    events::EventPayload,
};
use serde_json::json;
use tokio::sync::mpsc;

use super::*;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn make_envelope(event_type: &str) -> EventEnvelope {
    let repo = Repository {
        id: 1,
        name: "test-repo".to_string(),
        full_name: "owner/test-repo".to_string(),
        owner: RepositoryOwner {
            login: "owner".to_string(),
            id: 1,
            avatar_url: "https://example.com/avatar.png".to_string(),
            owner_type: OwnerType::User,
        },
        private: false,
        description: None,
        default_branch: "main".to_string(),
        html_url: "https://github.com/owner/test-repo".to_string(),
        clone_url: "https://github.com/owner/test-repo.git".to_string(),
        ssh_url: "git@github.com:owner/test-repo.git".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    EventEnvelope::new(
        event_type.to_string(),
        repo,
        EventPayload::new(json!({"action": "opened"})),
    )
}

// ---------------------------------------------------------------------------
// NoOpAck
// ---------------------------------------------------------------------------

#[tokio::test]
async fn noop_ack_complete_always_succeeds() {
    let ack = Box::new(NoOpAck);
    assert!(ack.complete().await.is_ok());
}

#[tokio::test]
async fn noop_ack_reject_always_succeeds() {
    let ack = Box::new(NoOpAck);
    assert!(ack.reject("test reason").await.is_ok());
}

// ---------------------------------------------------------------------------
// WebhookIngress
// ---------------------------------------------------------------------------

#[tokio::test]
async fn webhook_ingress_next_event_yields_sent_envelope() {
    let (tx, rx) = mpsc::channel(8);
    let mut ingress = WebhookIngress::new(rx);

    let envelope = make_envelope("pull_request");
    tx.send(envelope).await.unwrap();

    let result = ingress.next_event().await;
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(event.is_some());
    assert_eq!(event.unwrap().envelope.event_type, "pull_request");
}

#[tokio::test]
async fn webhook_ingress_next_event_returns_none_when_channel_is_closed() {
    let (tx, rx) = mpsc::channel::<EventEnvelope>(1);
    drop(tx); // close the channel immediately
    let mut ingress = WebhookIngress::new(rx);

    let result = ingress.next_event().await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn webhook_ingress_ack_is_noop() {
    let (tx, rx) = mpsc::channel(1);
    let mut ingress = WebhookIngress::new(rx);

    tx.send(make_envelope("push")).await.unwrap();
    drop(tx);

    let event = ingress.next_event().await.unwrap().unwrap();
    // Both complete and reject must succeed for webhook ingress events.
    assert!(event.ack.complete().await.is_ok());
}
