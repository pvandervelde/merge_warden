use bytes::Bytes;
use chrono::Utc;
use github_bot_sdk::{
    client::{OwnerType, Repository, RepositoryOwner},
    events::EventPayload,
};
use queue_runtime::{Message, QueueClientFactory, QueueName, SessionId};
use serde_json::json;
use std::sync::Arc;
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

/// Minimal valid GitHub `pull_request` JSON payload accepted by `EventProcessor`.
fn minimal_pr_payload(pr_number: u32) -> String {
    json!({
        "action": "opened",
        "number": pr_number,
        "pull_request": {
            "number": pr_number,
            "title": "Test PR",
            "state": "open",
            "html_url": "https://github.com/owner/test-repo/pull/1",
            "body": null,
            "head": { "sha": "abc123", "ref": "feature-branch" },
            "base": { "sha": "def456", "ref": "main" },
            "user": {
                "login": "test-user",
                "id": 1,
                "avatar_url": "https://example.com/avatar.png",
                "type": "User"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "draft": false,
            "merged": false,
            "labels": [],
            "additions": 10,
            "deletions": 5,
            "changed_files": 2
        },
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {
                "login": "owner",
                "id": 1,
                "avatar_url": "https://example.com/avatar.png",
                "type": "User"
            },
            "private": false,
            "description": null,
            "default_branch": "main",
            "html_url": "https://github.com/owner/test-repo",
            "clone_url": "https://github.com/owner/test-repo.git",
            "ssh_url": "git@github.com:owner/test-repo.git",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    })
    .to_string()
}

/// Creates an in-memory `QueueClient` backed by a dedicated `InMemoryProvider`.
fn make_test_queue_client() -> Arc<dyn queue_runtime::QueueClient> {
    Arc::from(QueueClientFactory::create_test_client())
}

/// Enqueues a single `WebhookQueueMessage` into the given in-memory client.
async fn enqueue_message(
    client: &Arc<dyn queue_runtime::QueueClient>,
    queue_name: &QueueName,
    msg: &WebhookQueueMessage,
    session: &str,
) {
    let body = Bytes::from(serde_json::to_vec(msg).unwrap());
    let session_id = SessionId::new(session.to_string()).unwrap();
    let message = Message::new(body).with_session_id(session_id);
    client.send_message(queue_name, message).await.unwrap();
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
async fn webhook_ingress_yields_sent_envelope() {
    let (tx, rx) = mpsc::channel(8);
    let mut ingress = WebhookIngress::new(rx);

    let envelope = make_envelope("pull_request");
    tx.send(envelope).await.unwrap();

    let event = ingress.next_event().await.unwrap().unwrap();
    assert_eq!(event.envelope.event_type, "pull_request");
}

#[tokio::test]
async fn webhook_ingress_returns_none_when_channel_is_closed() {
    let (tx, rx) = mpsc::channel::<EventEnvelope>(1);
    drop(tx);
    let mut ingress = WebhookIngress::new(rx);

    // Clean channel close → Ok(None), not an error.
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
    assert!(event.ack.complete().await.is_ok());
}

// ---------------------------------------------------------------------------
// WebhookQueueMessage — serialisation round-trip
// ---------------------------------------------------------------------------

#[test]
fn webhook_queue_message_round_trip_serde() {
    let original = WebhookQueueMessage {
        schema_version: 1,
        event_type: "pull_request".to_string(),
        delivery_id: "abc-123".to_string(),
        received_at: Utc::now(),
        raw_payload: r#"{"action":"opened"}"#.to_string(),
    };

    let json_str = serde_json::to_string(&original).expect("serialise");
    let decoded: WebhookQueueMessage = serde_json::from_str(&json_str).expect("deserialise");

    assert_eq!(decoded.schema_version, original.schema_version);
    assert_eq!(decoded.event_type, original.event_type);
    assert_eq!(decoded.delivery_id, original.delivery_id);
    assert_eq!(decoded.raw_payload, original.raw_payload);

    // installation_id must not appear in the serialised output — regression guard.
    let json_value: serde_json::Value =
        serde_json::from_str(&json_str).expect("parse json for field check");
    assert!(
        json_value.get("installation_id").is_none(),
        "installation_id must not be serialised: found {:?}",
        json_value.get("installation_id")
    );
}

#[test]
fn webhook_queue_message_json_contains_required_fields() {
    let msg = WebhookQueueMessage {
        schema_version: 1,
        event_type: "push".to_string(),
        delivery_id: "delivery-xyz".to_string(),
        received_at: Utc::now(),
        raw_payload: "{}".to_string(),
    };

    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&msg).unwrap()).unwrap();

    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["event_type"], "push");
    assert_eq!(json["delivery_id"], "delivery-xyz");
    assert!(json.get("received_at").is_some());
    assert_eq!(json["raw_payload"], "{}");
    // installation_id must not be serialised — regression guard.
    assert!(
        json.get("installation_id").is_none(),
        "installation_id must not appear in serialised output: found {:?}",
        json.get("installation_id")
    );
}

/// Regression guard: messages produced before the `installation_id` field was
/// removed must still deserialise correctly. serde ignores unknown fields by
/// default, so this test passes both BEFORE and AFTER the field is removed.
/// It prevents any future addition of `#[serde(deny_unknown_fields)]` from
/// silently breaking backwards-compatibility with old queue messages.
#[test]
fn webhook_queue_message_deserialises_legacy_json_with_installation_id() {
    let legacy_json = r#"{
        "schema_version": 1,
        "event_type": "pull_request",
        "delivery_id": "legacy-del-001",
        "installation_id": 42,
        "received_at": "2024-01-15T10:30:00Z",
        "raw_payload": "{\"action\":\"opened\"}"
    }"#;

    let result: Result<WebhookQueueMessage, _> = serde_json::from_str(legacy_json);

    assert!(
        result.is_ok(),
        "legacy JSON containing installation_id must deserialise successfully \
         (backwards-compatibility): {:?}",
        result.err()
    );
    let msg = result.unwrap();
    assert_eq!(msg.schema_version, 1);
    assert_eq!(msg.event_type, "pull_request");
    assert_eq!(msg.delivery_id, "legacy-del-001");
    assert_eq!(msg.raw_payload, r#"{"action":"opened"}"#);
}

// ---------------------------------------------------------------------------
// QueueIngress
// ---------------------------------------------------------------------------

#[tokio::test]
async fn queue_ingress_yields_enqueued_event() {
    let client = make_test_queue_client();
    let queue_name = QueueName::new("test-queue".to_string()).unwrap();

    let msg = WebhookQueueMessage {
        schema_version: 1,
        event_type: "pull_request".to_string(),
        delivery_id: "del-001".to_string(),
        received_at: Utc::now(),
        raw_payload: minimal_pr_payload(7),
    };
    enqueue_message(&client, &queue_name, &msg, "owner/test-repo/7").await;

    let mut ingress = QueueIngress::new(Arc::clone(&client), queue_name);
    let event = ingress.next_event().await.unwrap().unwrap();

    assert_eq!(event.envelope.event_type, "pull_request");
}

#[tokio::test]
async fn queue_ingress_dead_letters_unknown_schema_version() {
    let client = make_test_queue_client();
    let queue_name = QueueName::new("schema-queue".to_string()).unwrap();

    // Message with unsupported schema version 99.
    let bad_msg = WebhookQueueMessage {
        schema_version: 99,
        event_type: "pull_request".to_string(),
        delivery_id: "del-bad".to_string(),
        received_at: Utc::now(),
        raw_payload: minimal_pr_payload(1),
    };
    enqueue_message(&client, &queue_name, &bad_msg, "owner/test-repo/1").await;

    // Enqueue a valid message on a different session so the loop can return it.
    let good_msg = WebhookQueueMessage {
        schema_version: 1,
        event_type: "pull_request".to_string(),
        delivery_id: "del-good".to_string(),
        received_at: Utc::now(),
        raw_payload: minimal_pr_payload(2),
    };
    enqueue_message(&client, &queue_name, &good_msg, "owner/test-repo/2").await;

    let mut ingress = QueueIngress::new(Arc::clone(&client), queue_name);

    // The bad message should be skipped; the good one returned.
    let event = ingress.next_event().await.unwrap().unwrap();
    assert_eq!(event.envelope.event_type, "pull_request");
    // Delivery ID comes from the good message.
    assert_eq!(
        event.envelope.metadata.delivery_id.as_deref(),
        Some("del-good"),
    );
}

#[tokio::test]
async fn queue_ingress_dead_letters_malformed_body() {
    let client = make_test_queue_client();
    let queue_name = QueueName::new("malformed-queue".to_string()).unwrap();

    // Push raw bytes that are not valid JSON for WebhookQueueMessage.
    let bad_bytes = Bytes::from(b"not-valid-json".as_ref());
    let session_id = SessionId::new("owner/test-repo/1".to_string()).unwrap();
    let message = Message::new(bad_bytes).with_session_id(session_id);
    client.send_message(&queue_name, message).await.unwrap();

    // Valid message on a different session.
    let good_msg = WebhookQueueMessage {
        schema_version: 1,
        event_type: "pull_request".to_string(),
        delivery_id: "del-good".to_string(),
        received_at: Utc::now(),
        raw_payload: minimal_pr_payload(2),
    };
    enqueue_message(&client, &queue_name, &good_msg, "owner/test-repo/2").await;

    let mut ingress = QueueIngress::new(Arc::clone(&client), queue_name);
    let event = ingress.next_event().await.unwrap().unwrap();
    assert_eq!(event.envelope.event_type, "pull_request");
}

#[tokio::test]
async fn queue_ingress_ack_complete_succeeds() {
    let client = make_test_queue_client();
    let queue_name = QueueName::new("ack-complete-queue".to_string()).unwrap();

    let msg = WebhookQueueMessage {
        schema_version: 1,
        event_type: "pull_request".to_string(),
        delivery_id: "del-ack".to_string(),
        received_at: Utc::now(),
        raw_payload: minimal_pr_payload(5),
    };
    enqueue_message(&client, &queue_name, &msg, "owner/test-repo/5").await;

    let mut ingress = QueueIngress::new(Arc::clone(&client), queue_name);
    let event = ingress.next_event().await.unwrap().unwrap();

    // Completing the ack should succeed without error.
    assert!(event.ack.complete().await.is_ok());
}

#[tokio::test]
async fn queue_ingress_ack_reject_sends_to_dlq() {
    let client = make_test_queue_client();
    let queue_name = QueueName::new("ack-reject-queue".to_string()).unwrap();

    let msg = WebhookQueueMessage {
        schema_version: 1,
        event_type: "pull_request".to_string(),
        delivery_id: "del-reject".to_string(),
        received_at: Utc::now(),
        raw_payload: minimal_pr_payload(6),
    };
    enqueue_message(&client, &queue_name, &msg, "owner/test-repo/6").await;

    let mut ingress = QueueIngress::new(Arc::clone(&client), queue_name);
    let event = ingress.next_event().await.unwrap().unwrap();

    // Rejecting (dead-lettering) the ack should succeed without error.
    assert!(event.ack.reject("processing failed").await.is_ok());
}
