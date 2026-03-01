# Design: github-bot-sdk Migration

## Status

Draft — awaiting implementation (Phase 1.0)

## Context

`crates/developer_platforms/src/github.rs` currently depends on:

- `octocrab` as the GitHub HTTP client
- `jsonwebtoken` for manual JWT signing
- Manual HMAC-SHA256 webhook verification in `crates/azure-functions/src/main.rs`
- Manual installation token exchange via `authenticate_with_access_token`

The `github-bot-sdk` crate (owned by the same author) provides all of the above as
production-ready, security-hardened implementations. This document describes the
migration boundary, new internal structure, and what remains unchanged.

**`github-bot-sdk` is currently a git dependency; it will be published to crates.io
in a future release.**

---

## Scope and Non-Scope

### In scope

- Replacing `octocrab` + `jsonwebtoken` internals inside `developer_platforms`
- Replacing manual HMAC verification in `server` (formerly `azure-functions`)
- Adopting `github-bot-sdk` webhook parsing (`EventEnvelope`) as the common event type
- Adopting `WebhookHandler` trait as the dispatch abstraction in `server`

### Not in scope

- Changes to `PullRequestProvider` or `ConfigFetcher` public traits
- Any changes to `core` crate
- Any changes to `cli` crate
- Changing the current PR check behaviour

---

## What the SDK Provides

| Current mechanism | SDK replacement |
|---|---|
| `jsonwebtoken` + manual JWT claims | `GitHubAppAuth` — automatic RS256 JWT, max 10-min expiry |
| `authenticate_with_access_token` | `GitHubClient::installation(id)` — scoped installation client with token caching |
| Manual HMAC-SHA256 in `verify_github_signature` | `SignatureValidator` — constant-time comparison, validated by default |
| `serde_json::from_str` on raw body | `parse_webhook(headers, body)` → `EventEnvelope` |
| `Octocrab` methods for PR/label/comment | Typed wrappers on `InstallationClient` |
| `EventProcessor` pattern | `WebhookHandler` trait + `EventEnvelope` dispatch |

---

## Migration Boundary

```
┌─────────────────────────────────────────────────────────────────────┐
│ server crate                                                        │
│                                                                     │
│  Axum handler                                                       │
│    │ raw bytes + headers                                            │
│    ▼                                                                │
│  SignatureValidator (SDK)  ──── replaces verify_github_signature    │
│    │ validated body                                                 │
│    ▼                                                                │
│  parse_webhook (SDK)  ──────── replaces manual serde_json::from_str │
│    │ EventEnvelope                                                  │
│    ▼                                                                │
│  WebhookHandler dispatch  ─── replaces inline match action          │
└─────────────────────────────────────────────────────────────────────┘
         │ EventEnvelope (github-bot-sdk type)
         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ developer_platforms crate                                            │
│                                                                      │
│  GitHubProvider   ────────── wraps GitHubClient (SDK) internally     │
│                              replaces Octocrab field                 │
│  authenticate_with_access_token REMOVED                              │
│  create_app_client REMOVED (SDK handles internally)                  │
│                                                                      │
│  PullRequestProvider trait  ──── UNCHANGED (core depends on this)    │
│  ConfigFetcher trait         ──── UNCHANGED                          │
└──────────────────────────────────────────────────────────────────────┘
         │ PullRequestProvider
         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ core crate  ──── ZERO CHANGES                                        │
└──────────────────────────────────────────────────────────────────────┘
```

### Rule: no new crate required

`developer_platforms` already is the adapter layer between GitHub and `core`.
Inserting another crate between them would add indirection without benefit.
The SDK replaces the internals of `developer_platforms`, not the layer itself.

---

## Component Responsibilities After Migration

### `developer_platforms::github::GitHubProvider`

**Knows:**

- `GitHubClient` (SDK) — the authenticated app-level client
- How to construct an installation-scoped client from an `InstallationId`

**Does:**

- Implements `PullRequestProvider` and `ConfigFetcher` for GitHub
- Delegates all API calls to `InstallationClient` methods from the SDK
- Maps SDK error types to `developer_platforms::errors::Error`

**Removes:**

- `authenticate_with_access_token` (free function — callers in `server` migrate to SDK init)
- `create_app_client` (replaced by `GitHubClient::builder(auth).build()`)
- `JWTClaims` struct and associated JWT logic
- Direct `Octocrab` field

### `server::webhook::SignatureValidator`

**Owns:** webhook signature validation (single responsibility).
Delegates to `github_bot_sdk::webhook::SignatureValidator`.

### `server::webhook::WebhookHandler` (new impl)

**Implements** `github_bot_sdk::webhook::WebhookHandler`.
**Knows:** `AppState` (shared server state).
**Does:** dispatches `EventEnvelope` to the event processing pipeline.

Replaces the inline `match action` block in `handle_post_request`.

---

## `EventEnvelope` as the Common Event Type

After migration, every part of the server that deals with an incoming GitHub event
uses `github_bot_sdk::events::EventEnvelope`:

```
EventEnvelope {
    event_type: String,      // "pull_request", "pull_request_review", etc.
    delivery_id: String,     // X-GitHub-Delivery header
    installation_id: Option<u64>,
    payload: serde_json::Value,
}
```

This is the type that flows into the queue architecture (task 0.3). Storing
`EventEnvelope` in the queue means the processor can reconstruct the full event
without re-fetching anything from GitHub's API.

---

## Cargo.toml Changes

### `developer_platforms/Cargo.toml`

```toml
# Remove
octocrab = { ... }
jsonwebtoken = { ... }

# Add (git until crates.io release)
github-bot-sdk = { git = "https://github.com/pvandervelde/github-bot-sdk", branch = "master" }
```

### `server/Cargo.toml` (formerly `azure-functions`)

```toml
# Remove
hmac = { ... }
sha2 = { ... }
hex = { ... }

# Add (same git dep — workspace-level preferred)
github-bot-sdk = { workspace = true }
```

---

## Migration Sequence

1. Add `github-bot-sdk` as a workspace dependency (git, pinned to a commit SHA for
   reproducibility).
2. Migrate `developer_platforms::github::GitHubProvider` internals:
   - Replace `Octocrab` field with `GitHubClient`
   - Rewrite each `PullRequestProvider` method against `InstallationClient` API
   - Delete `authenticate_with_access_token`, `create_app_client`, `JWTClaims`
3. Migrate `server` webhook handling:
   - Replace `verify_github_signature` with SDK `SignatureValidator`
   - Replace manual `WebhookPayload` deserialization with `parse_webhook` → `EventEnvelope`
   - Replace inline `match action` with a `WebhookHandler` impl
4. Delete now-unreachable dead code (JWT structs, HMAC imports).
5. Keep all existing tests green throughout; add new unit tests for the `WebhookHandler` impl.

---

## Behavioral Assertions

1. **Valid signature must proceed to event handling**
   - Given: POST with correct HMAC-SHA256 header
   - When: `SignatureValidator::validate` is called
   - Then: handler proceeds; no 401

2. **Invalid signature must be rejected without processing**
   - Given: POST with tampered body or wrong signature
   - When: `SignatureValidator::validate` is called
   - Then: returns 401; no business logic runs; payload not deserialized

3. **Unsupported action must return 200 without processing**
   - Given: valid webhook for action `"labeled"`
   - When: `WebhookHandler::handle` dispatches
   - Then: returns OK; `MergeWarden::process_pull_request` not called

4. **Supported action must invoke core processing**
   - Given: valid webhook for action `"opened"`, `"synchronize"`, etc.
   - When: `WebhookHandler::handle` dispatches
   - Then: `MergeWarden::process_pull_request` called with correct owner/repo/pr_number

5. **Token caching must not re-fetch installation tokens within TTL**
   - Given: two sequential requests for same installation within 60 seconds
   - When: `GitHubClient::installation(id)` is called twice
   - Then: SDK cache returns same token; GitHub `/app/installations/{id}/access_tokens`
     called only once

6. **`PullRequestProvider` contract must not change**
   - Given: existing `core` tests using mock `PullRequestProvider`
   - When: migration is complete
   - Then: all existing tests compile and pass without modification

---

## Testing Strategy

- **Unit tests**: mock `GitHubClient` at the SDK boundary; test each `PullRequestProvider`
  method in isolation against `wiremock` mock server (matching SDK's own test pattern)
- **Contract tests**: run existing `integration-tests` crate against real or mocked GitHub
  API using the new provider; assert all current behaviours are preserved
- **Signature validation tests**: verify constant-time rejection; fuzz the signature header

---

## Open Decisions

| Decision | Status |
|---|---|
| Pin `github-bot-sdk` to commit SHA or branch? | Recommend SHA for reproducibility |
| Expose `EventEnvelope` re-exported from `developer_platforms`? | Prefer re-export from `developer_platforms` to avoid SDK leakage into `server` directly — revisit during interface design |
