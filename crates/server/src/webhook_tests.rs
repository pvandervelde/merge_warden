use axum::{http::StatusCode, response::IntoResponse};
use chrono::Utc;
use github_bot_sdk::{
    client::{ClientConfig, GitHubClient, OwnerType, Repository, RepositoryOwner},
    events::{EventEnvelope, EventPayload},
    webhook::WebhookHandler,
};
use merge_warden_core::config::{ApplicationDefaults, RepositoryScope};
use merge_warden_developer_platforms::app_auth::AppAuthProvider;
use serde_json::json;

use super::health_check;
use super::MergeWardenWebhookHandler;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// RSA private key used only in tests.  Generated offline; never used in
/// production.  Must be a valid PEM-encoded PKCS#8 or traditional RSA key so
/// `AppAuthProvider` can parse it.
const TEST_PEM: &str = include_str!("../../developer_platforms/testdata/test-rsa-key.pem");

fn make_test_handler() -> MergeWardenWebhookHandler {
    let auth = AppAuthProvider::new(12345, TEST_PEM, "https://api.github.com")
        .expect("test RSA key must be valid");
    let github_client = GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .expect("GitHub client must build");
    MergeWardenWebhookHandler::new(github_client, ApplicationDefaults::default())
}

fn make_status_envelope(context: &str) -> EventEnvelope {
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
        "status".to_string(),
        repo,
        EventPayload::new(json!({
            "context": context,
            "sha": "deadbeef",
            "state": "pending",
            "installation": { "id": 99 }
        })),
    )
}

// ---------------------------------------------------------------------------
// Test helpers — FR-009: Repository Scope Filtering
//
// See docs/spec/architecture/event-processing.md#repository-scope-filtering
// ---------------------------------------------------------------------------

/// Builds a handler whose `ApplicationDefaults.repository_scope` is set to
/// `scope`. All other policy defaults are left at their compiled-in values.
fn make_test_handler_with_scope(scope: Option<RepositoryScope>) -> MergeWardenWebhookHandler {
    let auth = AppAuthProvider::new(12345, TEST_PEM, "https://api.github.com")
        .expect("test RSA key must be valid");
    let github_client = GitHubClient::builder(auth)
        .config(ClientConfig::default())
        .build()
        .expect("GitHub client must build");
    let policies = ApplicationDefaults {
        repository_scope: scope,
        ..ApplicationDefaults::default()
    };
    MergeWardenWebhookHandler::new(github_client, policies)
}

fn make_repository(name: &str) -> Repository {
    Repository {
        id: 1,
        name: name.to_string(),
        full_name: format!("owner/{name}"),
        owner: RepositoryOwner {
            login: "owner".to_string(),
            id: 1,
            avatar_url: "https://example.com/avatar.png".to_string(),
            owner_type: OwnerType::User,
        },
        private: false,
        description: None,
        default_branch: "main".to_string(),
        html_url: format!("https://github.com/owner/{name}"),
        clone_url: format!("https://github.com/owner/{name}.git"),
        ssh_url: format!("git@github.com:owner/{name}.git"),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Builds a `pull_request` envelope with action `"opened"` for `repo_name`.
///
/// `installation_id` deliberately controls whether `installation.id` is
/// present in the raw payload. Omitting it (`None`) lets a test reach
/// `handle_pull_request`'s "missing installation ID" `ProcessingError` branch
/// — a purely offline, payload-parsing-only failure that occurs *before* any
/// GitHub API call. This is used as a fast, deterministic, network-free
/// discriminator: if the repository-scope gate incorrectly lets an
/// out-of-scope repository through, the test fails immediately with this
/// diagnostic error instead of hanging on a live network call to
/// `https://api.github.com`.
fn make_pull_request_envelope(
    repo_name: &str,
    pr_number: u32,
    installation_id: Option<u64>,
) -> EventEnvelope {
    let repo = make_repository(repo_name);
    let mut payload = json!({
        "action": "opened",
        "pull_request": { "number": pr_number },
        // Real GitHub webhook bodies always carry a top-level "repository"
        // object; the repository-scope gate reads its "name" field from
        // here (the raw JSON), not from the structured `repo` argument
        // above, per docs/spec/architecture/event-processing.md.
        "repository": { "name": repo_name },
    });
    if let Some(id) = installation_id {
        payload["installation"] = json!({ "id": id });
    }
    EventEnvelope::new("pull_request".to_string(), repo, EventPayload::new(payload))
}

/// Builds a `status` envelope for `repo_name` with the given `context`.
///
/// `installation_id` controls whether `installation.id` is present in the
/// raw payload, for the same offline-discriminator reasons documented on
/// [`make_pull_request_envelope`].
fn make_status_envelope_with_scope_fixtures(
    repo_name: &str,
    context: &str,
    installation_id: Option<u64>,
) -> EventEnvelope {
    let repo = make_repository(repo_name);
    let mut payload = json!({
        "context": context,
        "sha": "deadbeef",
        "state": "pending",
        // See the equivalent comment in `make_pull_request_envelope` — the
        // repository-scope gate reads "repository.name" from the raw JSON
        // payload, which real GitHub webhook bodies always include.
        "repository": { "name": repo_name },
    });
    if let Some(id) = installation_id {
        payload["installation"] = json!({ "id": id });
    }
    EventEnvelope::new("status".to_string(), repo, EventPayload::new(payload))
}

/// Builds an envelope whose *structured* `repository.name` field is a
/// well-formed, unrelated placeholder value, while the *raw* JSON payload's
/// `repository` object is set to `raw_repository` (used to simulate a
/// missing/malformed `repository.name` in the actual webhook payload).
///
/// Per docs/spec/architecture/event-processing.md ("Repository Scope
/// Filtering"), the scope-check gate must extract `repo_name` from
/// `envelope.payload.raw()["repository"]["name"]` — NOT from the structured
/// `envelope.repository.name` field the rest of this file uses for logging.
/// The placeholder structured name below is a normal, well-formed repo name
/// that would NOT be flagged as malformed on its own; this pins down that the
/// implementation reads the raw JSON, not the structured field (an
/// implementation that reads the structured field instead would incorrectly
/// treat these payloads as well-formed and in-scope).
fn make_envelope_with_raw_repository_name(
    event_type: &str,
    raw_repository: serde_json::Value,
    mut payload_fields: serde_json::Value,
) -> EventEnvelope {
    let repo = make_repository("structured-field-placeholder-should-be-ignored");
    payload_fields["repository"] = raw_repository;
    EventEnvelope::new(
        event_type.to_string(),
        repo,
        EventPayload::new(payload_fields),
    )
}

/// Asserts that `result` is an `Err` whose message contains `needle`. Used to
/// confirm that `handle_event` proceeded past the repository-scope gate into
/// real dispatch logic (proving the gate did NOT filter the event), without
/// needing to downcast the boxed `dyn Error`.
fn assert_err_contains(
    result: &Result<(), Box<dyn std::error::Error + Send + Sync>>,
    needle: &str,
) {
    match result {
        Err(e) => assert!(
            e.to_string().contains(needle),
            "expected error containing '{needle}', got: {e}"
        ),
        Ok(()) => panic!("expected an Err containing '{needle}', got Ok(())"),
    }
}

// ---------------------------------------------------------------------------
// health_check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_check_returns_200_ok() {
    let response = health_check().await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// handle_status_event
// ---------------------------------------------------------------------------

/// A status event whose context is NOT `renovate/stability-days` must be
/// silently ignored — no API calls, no errors.
#[tokio::test]
async fn handle_status_event_ignores_non_renovate_context() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let result = handler.handle_status_event(&envelope).await;

    assert!(
        result.is_ok(),
        "non-renovate status event should be ignored: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// handle_event routing
// ---------------------------------------------------------------------------

/// A `status` event must be dispatched to `handle_status_event`.
/// Using a non-renovate context means no GitHub API calls are made, so the
/// handler returns Ok(()) without a live network connection.
#[tokio::test]
async fn handle_event_routes_status_events() {
    let handler = make_test_handler();
    let envelope = make_status_envelope("ci/build");

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "status event should be routed and return Ok(()): {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// FR-009: Repository Scope Filtering — handle_event gate
//
// See docs/spec/architecture/event-processing.md#repository-scope-filtering
// See docs/spec/requirements/functional-requirements.md#fr-009-repository-scope-filtering
//
// All fixtures below deliberately omit `installation.id` from the payload.
// This makes every test fully offline and fast-failing: if the scope gate
// incorrectly filters (or incorrectly lets through) an event, the test fails
// immediately with a `ProcessingError` diagnostic instead of hanging or
// erroring on a live network call to `https://api.github.com`.
// ---------------------------------------------------------------------------

// --- Tier 1: specification tests — basic gate behaviour ---

#[tokio::test]
async fn handle_event_filters_out_of_scope_repository_for_pull_request_event() {
    let scope = Some(RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_pull_request_envelope("blocked-repo", 1, None);

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "out-of-scope repository must be acknowledged with Ok(()), got: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_processes_in_scope_repository_for_pull_request_event() {
    let scope = Some(RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_pull_request_envelope("allowed-api", 1, None);

    let result = handler.handle_event(&envelope).await;

    // In-scope events must fall through to the existing dispatch logic. We
    // prove this without a network call: omitting installation.id makes
    // handle_pull_request fail with a specific ProcessingError before any
    // GitHub API call, which only happens if the scope gate let it through.
    assert_err_contains(&result, "Missing installation ID");
}

#[tokio::test]
async fn handle_event_filters_out_of_scope_repository_for_status_event() {
    let scope = Some(RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_status_envelope_with_scope_fixtures(
        "blocked-repo",
        merge_warden_core::config::RENOVATE_STABILITY_CHECK_CONTEXT,
        None,
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "out-of-scope repository must be acknowledged with Ok(()), got: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_processes_in_scope_repository_for_status_event() {
    let scope = Some(RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_status_envelope_with_scope_fixtures(
        "allowed-api",
        merge_warden_core::config::RENOVATE_STABILITY_CHECK_CONTEXT,
        None,
    );

    let result = handler.handle_event(&envelope).await;

    assert_err_contains(&result, "Missing installation ID");
}

#[tokio::test]
async fn handle_event_treats_missing_repository_name_as_out_of_scope() {
    // No repository_scope configured at all (None) — fail-closed on an
    // unparseable repository name must still apply.
    let handler = make_test_handler_with_scope(None);
    let raw_repository = json!({ "id": 1, "full_name": "owner/unknown" }); // no "name" key
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "missing repository.name must be treated as out of scope (fail-closed): {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_treats_empty_string_repository_name_as_malformed() {
    let handler = make_test_handler_with_scope(None);
    let raw_repository = json!({ "id": 1, "name": "", "full_name": "owner/" });
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "empty-string repository.name must be treated as malformed (fail-closed): {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_treats_non_string_repository_name_as_malformed() {
    let handler = make_test_handler_with_scope(None);
    let raw_repository = json!({ "id": 1, "name": 12345, "full_name": "owner/12345" });
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "non-string repository.name must be treated as malformed (fail-closed): {:?}",
        result
    );
}

// --- Tier 2: adversarial / boundary tests ---

#[tokio::test]
async fn handle_event_exclude_pattern_overrides_include_end_to_end() {
    let scope = Some(RepositoryScope {
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["blocked-repo".to_string()],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_pull_request_envelope("blocked-repo", 1, None);

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "exclude_patterns must override a matching include_patterns entry end-to-end: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_case_insensitive_scope_matching_end_to_end() {
    let scope = Some(RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    // Different case than the configured pattern.
    let envelope = make_pull_request_envelope("Allowed-API", 1, None);

    let result = handler.handle_event(&envelope).await;

    assert_err_contains(&result, "Missing installation ID");
}

#[tokio::test]
async fn handle_event_empty_include_patterns_blocks_all_repositories_end_to_end() {
    let scope = Some(RepositoryScope {
        include_patterns: vec![],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_pull_request_envelope("any-repo-whatsoever", 1, None);

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "an explicitly empty include_patterns list must block every repository: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_out_of_scope_repository_never_reaches_missing_pr_number_check() {
    // Even a payload missing BOTH installation.id and pull_request.number must
    // still be acknowledged silently for an out-of-scope repo — proving the
    // scope gate is evaluated first, before any further payload parsing.
    let scope = Some(RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let repo = make_repository("blocked-repo");
    let envelope = EventEnvelope::new(
        "pull_request".to_string(),
        repo,
        EventPayload::new(json!({ "action": "opened" })), // no pull_request.number either
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "out-of-scope repository must short-circuit before pr_number extraction: {:?}",
        result
    );
}

// --- Regression: repository_scope absent (None) leaves existing behaviour unaffected ---

#[tokio::test]
async fn handle_event_pull_request_unaffected_by_absent_repository_scope() {
    let handler = make_test_handler_with_scope(None);
    let envelope = make_pull_request_envelope("any-repo-name", 1, None);

    let result = handler.handle_event(&envelope).await;

    // Same offline discriminator: reaching the "missing installation ID"
    // error proves dispatch proceeded normally, i.e. repository_scope = None
    // introduces no new filtering versus pre-FR-009 behaviour.
    assert_err_contains(&result, "Missing installation ID");
}

#[tokio::test]
async fn handle_event_status_unaffected_by_absent_repository_scope() {
    let handler = make_test_handler_with_scope(None);
    let envelope = make_status_envelope_with_scope_fixtures(
        "any-repo-name",
        merge_warden_core::config::RENOVATE_STABILITY_CHECK_CONTEXT,
        None,
    );

    let result = handler.handle_event(&envelope).await;

    assert_err_contains(&result, "Missing installation ID");
}

#[tokio::test]
async fn handle_event_status_non_renovate_context_unaffected_by_configured_scope() {
    // Regression: an in-scope repo with a non-renovate status context must
    // still be silently ignored by handle_status_event's own logic, exactly
    // as before FR-009 — the scope gate must not change this outcome.
    let scope = Some(RepositoryScope {
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_status_envelope_with_scope_fixtures("any-repo-name", "ci/build", None);

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "non-renovate status context must still be ignored when the repo is in scope: {:?}",
        result
    );
}

// --- Tier 3: integration — consistency between the pure function and the gate ---

/// Cross-checks that `handle_event`'s observable outcome (filtered vs.
/// processed) is consistent with `is_repository_in_scope` for a table of
/// repository names, across both `pull_request` and `status` events. This is
/// the end-to-end analogue of the pure-function unit tests in
/// `crates/core/src/config_tests.rs`, run through the actual gate.
#[tokio::test]
async fn handle_event_outcome_matches_is_repository_in_scope_for_pull_request_events() {
    let scope = RepositoryScope {
        include_patterns: vec!["allowed-*".to_string(), "checkout".to_string()],
        exclude_patterns: vec!["allowed-legacy".to_string()],
    };
    let handler = make_test_handler_with_scope(Some(scope.clone()));

    let cases = [
        "allowed-api",
        "allowed-legacy", // excluded despite matching include
        "checkout",
        "not-covered",
        "Allowed-API", // case-insensitive match
    ];

    for repo_name in cases {
        let expected_in_scope =
            merge_warden_core::config::is_repository_in_scope(&Some(scope.clone()), repo_name);
        let envelope = make_pull_request_envelope(repo_name, 1, None);
        let result = handler.handle_event(&envelope).await;

        if expected_in_scope {
            assert_err_contains(&result, "Missing installation ID");
        } else {
            assert!(
                result.is_ok(),
                "repo '{repo_name}' expected out of scope (Ok(())), got: {:?}",
                result
            );
        }
    }
}

// ---------------------------------------------------------------------------
// QA audit (post-implementation): adversarial input probing
//
// Manual fuzz-substitute probing (cargo-fuzz is not configured in this
// repo). See docs/spec/test-coverage.md for the full audit report.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handle_event_treats_object_repository_name_as_malformed() {
    let handler = make_test_handler_with_scope(None);
    let raw_repository = json!({ "id": 1, "name": { "nested": "value" }, "full_name": "owner/x" });
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "object-valued repository.name must be treated as malformed (fail-closed), not panic: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_treats_array_repository_name_as_malformed() {
    let handler = make_test_handler_with_scope(None);
    let raw_repository = json!({ "id": 1, "name": ["a", "b"], "full_name": "owner/x" });
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "array-valued repository.name must be treated as malformed (fail-closed), not panic: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_treats_null_repository_name_as_malformed() {
    let handler = make_test_handler_with_scope(None);
    let raw_repository = json!({ "id": 1, "name": null, "full_name": "owner/x" });
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "null-valued repository.name must be treated as malformed (fail-closed), not panic: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_treats_entirely_null_repository_object_as_malformed() {
    // Simulates a payload where "repository" itself is `null` rather than an
    // object with a missing/malformed "name" field -- probes that indexing
    // `Value::Null["name"]` (serde_json returns a static Null rather than
    // panicking) is handled safely by the scope gate.
    let handler = make_test_handler_with_scope(None);
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        json!(null),
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "a null 'repository' object must not panic and must be treated as malformed: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_repository_name_containing_embedded_null_byte_does_not_panic() {
    // Embedded NUL bytes are valid inside a JSON string; the scope gate must
    // not panic on them, and the '.*' wildcard translation must treat the
    // NUL byte as an ordinary character for matching purposes.
    let scope = Some(RepositoryScope {
        include_patterns: vec!["payments-*".to_string()],
        exclude_patterns: vec![],
    });
    let handler = make_test_handler_with_scope(scope);
    let raw_repository = json!({ "id": 1, "name": "payments-\u{0000}api", "full_name": "owner/x" });
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        raw_repository,
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    // "payments-\0api" matches "payments-*" (the wildcard's '.*' matches the
    // NUL byte like any other character), so the event is in scope and
    // dispatch proceeds -- reaching the offline "Missing installation ID"
    // discriminator proves no panic occurred and scope matching succeeded.
    assert_err_contains(&result, "Missing installation ID");
}

// ---------------------------------------------------------------------------
// QA audit (post-implementation): mutation-analysis kill test
//
// See docs/spec/test-coverage.md for the full audit report. Confirmed
// empirically: changing the scope-gate's `is_repository_in_scope` call to
// read `envelope.repository.name` (the SDK-populated structured field)
// instead of the raw-JSON-derived `repo_name` leaves the entire pre-existing
// webhook-level scope-gate test suite (22 tests) green, because every other
// fixture in this file sets the structured and raw repository names to the
// same value.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handle_event_scope_decision_uses_raw_repository_name_not_structured_field() {
    // The envelope's *structured* `Repository.name` is a well-formed but
    // UNRELATED placeholder ("structured-field-placeholder-should-be-ignored",
    // set by `make_envelope_with_raw_repository_name`), while the *raw* JSON
    // payload's "repository.name" is "blocked-repo". The configured scope
    // matches everything EXCEPT "blocked-repo".
    //
    // - Correct implementation (reads raw JSON): repo_name = "blocked-repo",
    //   which matches the exclude pattern -> filtered -> Ok(()).
    // - Structured-field mutant: repo_name = the placeholder string, which
    //   matches "*" and does NOT match the "blocked-repo" exclude pattern ->
    //   treated as in-scope -> dispatch proceeds -> distinguishable error
    //   (no installation.id in the payload) instead of a silent Ok(()).
    let scope = Some(RepositoryScope {
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["blocked-repo".to_string()],
    });
    let handler = make_test_handler_with_scope(scope);
    let envelope = make_envelope_with_raw_repository_name(
        "pull_request",
        json!({ "id": 1, "name": "blocked-repo", "full_name": "owner/blocked-repo" }),
        json!({ "action": "opened", "pull_request": { "number": 1 } }),
    );

    let result = handler.handle_event(&envelope).await;

    assert!(
        result.is_ok(),
        "scope decision must be based on the raw JSON repository name, not the SDK-populated structured field: {:?}",
        result
    );
}

#[tokio::test]
async fn handle_event_outcome_matches_is_repository_in_scope_for_status_events() {
    let scope = RepositoryScope {
        include_patterns: vec!["allowed-*".to_string()],
        exclude_patterns: vec!["allowed-legacy".to_string()],
    };
    let handler = make_test_handler_with_scope(Some(scope.clone()));

    let cases = ["allowed-api", "allowed-legacy", "not-covered"];

    for repo_name in cases {
        let expected_in_scope =
            merge_warden_core::config::is_repository_in_scope(&Some(scope.clone()), repo_name);
        let envelope = make_status_envelope_with_scope_fixtures(
            repo_name,
            merge_warden_core::config::RENOVATE_STABILITY_CHECK_CONTEXT,
            None,
        );
        let result = handler.handle_event(&envelope).await;

        if expected_in_scope {
            assert_err_contains(&result, "Missing installation ID");
        } else {
            assert!(
                result.is_ok(),
                "repo '{repo_name}' expected out of scope (Ok(())), got: {:?}",
                result
            );
        }
    }
}
