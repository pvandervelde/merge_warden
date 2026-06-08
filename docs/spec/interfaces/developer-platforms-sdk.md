# Interface Spec: developer_platforms — SDK Migration

**Source**: `crates/developer_platforms/src/`
**Spec**: `docs/spec/design/github-bot-sdk-migration.md`
**Task**: 1.0

---

## Summary of Changes

The `developer_platforms` crate's GitHub implementation (`github.rs`) swaps its internal
HTTP/auth stack from `octocrab` + manual JWT to `github-bot-sdk`. The public trait
surface (`PullRequestProvider`, `ConfigFetcher`) is **unchanged**. `core` requires zero
modifications.

---

## `GitHubProvider` Constructor

### Current

```rust
pub struct GitHubProvider {
    octocrab: Octocrab,
}

impl GitHubProvider {
    pub fn new(octocrab: Octocrab) -> Self { ... }
}
```

### After Migration

```rust
// github-bot-sdk types (add to Cargo.toml as git dependency)
// use github_bot_sdk::client::GitHubClient;

pub struct GitHubProvider {
    client: github_bot_sdk::client::GitHubClient,
}

impl GitHubProvider {
    /// Creates a new `GitHubProvider` from an already-constructed SDK client.
    ///
    /// The caller is responsible for building `GitHubClient` with the correct
    /// app authentication before passing it here. Rationale: the client is also
    /// needed by the server's `AppState` for non-PR API calls; constructing it
    /// externally avoids duplication and makes the provider unit-testable with
    /// a mock/stub `GitHubClient`.
    ///
    /// # Arguments
    /// * `client` - Authenticated `GitHubClient` from `github-bot-sdk`.
    ///
    /// # Example
    /// ```rust,ignore
    /// let auth = GitHubAppAuth::new(app_id, private_key)?;
    /// let client = GitHubClient::builder(auth).build()?;
    /// let provider = GitHubProvider::new(client);
    /// ```
    pub fn new(client: github_bot_sdk::client::GitHubClient) -> Self { ... }
}
```

### Functions Removed

- `pub async fn authenticate_with_access_token(...)` — replaced by
  `client.installation(id)` in the SDK
- `pub async fn create_app_client(...)` — replaced by
  `GitHubClient::builder(auth).build()` in the SDK

### Struct Fields Removed

- `JWTClaims` — JWT signing is handled internally by the SDK

---

## Error Variants Added to `developer_platforms::errors::Error`

The SDK introduces failure modes not covered by existing variants.

### New variant: `TokenRefreshFailed`

```rust
/// Installation access token could not be refreshed before expiry.
///
/// Produced when `github-bot-sdk`'s token cache fails to obtain a fresh token
/// for the requested installation. Callers should treat this as a transient
/// authentication failure and retry with exponential backoff.
///
/// Parameters: installation ID, error message from the SDK.
#[error("Failed to refresh installation token for installation {0}: {1}")]
TokenRefreshFailed(u64, String),
```

### Existing variant reuse

| SDK error condition | Map to existing variant |
| --- | --- |
| HTTP 401 / invalid app credentials | `AuthError(String)` |
| HTTP 429 / rate limit | `RateLimitExceeded` |
| HTTP 4xx / malformed request | `InvalidResponse` |
| HTTP 5xx / server error | `ApiError()` |
| Installation not found | `FailedToFindAppInstallation(owner, repo, id)` |
| Token creation failed | `FailedToCreateAccessToken(owner, repo, id)` |

---

## `EventEnvelope` Import Decision

`EventEnvelope` (from `github_bot_sdk::events`) is the type that flows from:

```text
server::webhook handler → ingress channel → server::ingress::EventIngress → event processor
```

**Decision**: `server` crate imports `github_bot_sdk` directly for `EventEnvelope` and
`SignatureValidator`. `developer_platforms` does **not** re-export it — the SDK is an
infrastructure concern that lives at the server boundary, not in the platform abstraction.

During the stub phase (before the git dependency is added), `EventEnvelope` is defined
as a local placeholder struct in `crates/server/src/ingress.rs`. See the replacement
note in that file.

---

## `MergeWardenWebhookHandler` Struct (in `server` crate)

Implements `github_bot_sdk::webhook::WebhookHandler` to replace the inline
`match action` dispatch in `handle_post_request`.

```rust
/// Implements the SDK's WebhookHandler trait to dispatch validated GitHub events.
///
/// # Fields
/// * `state` — Shared application state (GitHub client, config, webhook secret).
/// * `event_sender` — Channel sender; sends `EventEnvelope` into the ingress pipeline.
///   `None` when `MERGE_WARDEN_RECEIVER_MODE=queue` (the handler enqueues instead).
pub struct MergeWardenWebhookHandler {
    state: Arc<AppState>,
}

// Implements github_bot_sdk::webhook::WebhookHandler:
//
// async fn handle(&self, envelope: EventEnvelope) -> Result<(), SdkError> {
//     match envelope.event_type.as_str() {
//         "pull_request" => self.handle_pull_request(envelope).await,
//         "status"       => self.handle_status_event(envelope).await,
//         _ => Ok(()), // unsupported actions are silently ignored
//     }
// }
```

---

## Cargo.toml Changes

### `crates/developer_platforms/Cargo.toml`

```toml
# Remove
octocrab = { workspace = true }
jsonwebtoken = { workspace = true }

# Add
github-bot-sdk = { workspace = true }
```

### `Cargo.toml` (workspace)

```toml
# Add (git until crates.io release; pin to commit SHA for reproducibility)
github-bot-sdk = { git = "https://github.com/pvandervelde/github-bot-sdk", branch = "master" }
```

---

## Behavioral Postconditions

1. All methods on `PullRequestProvider` and `ConfigFetcher` must produce identical
   results before and after the migration for the same GitHub repository state.
2. `GitHubProvider::new` must not perform any network I/O — authentication is lazy.
3. `TokenRefreshFailed` must be returned (not panicked) when the SDK token cache
   cannot refresh; the error must include the installation ID.
4. The removed free functions (`authenticate_with_access_token`, `create_app_client`)
   must not appear in any `pub use` or public re-export after migration.

---

## FR-007 Additions

The following changes are required by
[FR-007 (Configuration Change Validation)](../requirements/functional-requirements.md#fr-007-configuration-change-validation).
They extend existing types rather than introducing new ones.

### `models::PullRequest` — new field `head_sha`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub draft: bool,
    pub body: Option<String>,
    pub author: Option<User>,
    #[serde(default)]
    pub milestone_number: Option<u64>,

    /// SHA of the head commit for this pull request.
    ///
    /// Used by [`core`] to fetch files at the exact revision being reviewed
    /// rather than the default branch.  Populated from `pull_request.head.sha`
    /// in GitHub webhook payloads and API responses.
    ///
    /// Mapped from the GitHub API field `head.sha` inside the pull request object.
    pub head_sha: String,
}
```

All existing construction sites (`github.rs`, test fixtures, integration tests) must
supply this field.  The GitHub API response for a pull request always includes
`head.sha`; a missing or empty value should be treated as an API error.

### `ConfigFetcher` trait — new method `fetch_config_at_ref`

```rust
/// Trait to fetch configuration files from remote repositories.
#[async_trait]
pub trait ConfigFetcher: Sync + Send {
    /// Fetch the content of a configuration file at the given path from the
    /// repository's default branch.
    ///
    /// Returns `Ok(Some(content))` if found, `Ok(None)` if not found, or `Err` on
    /// error.
    async fn fetch_config(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
    ) -> Result<Option<String>, Error>;

    /// Fetch the content of a configuration file at `path` as it exists at
    /// `git_ref` (a branch name, tag, or commit SHA).
    ///
    /// This is used by `core` to read the proposed version of
    /// `.github/merge-warden.toml` from the PR head SHA rather than the
    /// default branch.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` — Repository owner.
    /// * `repo_name`  — Repository name.
    /// * `path`       — Path to the configuration file relative to the
    ///                  repository root.
    /// * `git_ref`    — Branch name, tag, or commit SHA to read from.
    ///
    /// # Returns
    ///
    /// `Ok(Some(content))` if the file exists at the given ref,
    /// `Ok(None)` if the file is absent (HTTP 404), or `Err` for any other
    /// API failure.
    async fn fetch_config_at_ref(
        &self,
        repo_owner: &str,
        repo_name: &str,
        path: &str,
        git_ref: &str,
    ) -> Result<Option<String>, Error>;
}
```

#### `GitHubProvider` implementation

`GitHubProvider` already contains the private helper
`fetch_file_content(owner, repo, path, reference) -> Result<Option<String>, Error>`.
The implementation is a one-line delegation:

```rust
async fn fetch_config_at_ref(
    &self,
    repo_owner: &str,
    repo_name: &str,
    path: &str,
    git_ref: &str,
) -> Result<Option<String>, Error> {
    self.fetch_file_content(repo_owner, repo_name, path, git_ref).await
}
```

#### Behavioral postconditions for `fetch_config_at_ref`

1. `Ok(None)` must be returned when the file does not exist at `git_ref` (HTTP 404).
   It must not be treated as an error.
2. An `Err` must be returned for all non-404 API failures (permission denied, rate
   limit, server error, etc.).
3. The method must not fall back to the default branch when `git_ref` is not found —
   that would silently return stale content.

---

## Renovate Stability Days Additions

The following changes are required to support
[FR-008 (Renovate Stability Label Management)](../requirements/functional-requirements.md#fr-008-renovate-stability-label-management).
They extend existing types rather than introducing new ones.

### New model: `CommitStatus`

Location: `crates/developer_platforms/src/models.rs`

```rust
/// A single commit status entry returned by the GitHub Commit Statuses API.
///
/// GitHub returns commit statuses newest-first. When multiple entries exist for
/// the same context, callers should use the first occurrence (i.e. the newest).
///
/// Mapped from `GET /repos/{owner}/{repo}/commits/{sha}/statuses`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStatus {
    /// The context string that identifies which check produced this status.
    ///
    /// Example: `"renovate/stability-days"`.
    pub context: String,

    /// The state of the status.
    ///
    /// GitHub-defined values: `"pending"`, `"success"`, `"failure"`, `"error"`.
    pub state: String,

    /// Optional human-readable description of the status.
    pub description: Option<String>,
}
```

### New method on `PullRequestProvider`: `get_commit_statuses`

```rust
/// Fetches commit statuses for the given commit SHA.
///
/// Returns the first page of statuses only (same precedent as
/// `get_pull_request_files`). GitHub returns statuses newest-first; callers
/// wishing to use the most recent entry per context should take the first
/// occurrence of each context value.
///
/// # Arguments
///
/// * `repo_owner` — Repository owner.
/// * `repo_name`  — Repository name.
/// * `commit_sha` — Full SHA of the commit to query.
///
/// # Returns
///
/// `Ok(Vec<CommitStatus>)` — possibly empty if no statuses exist for the commit.
/// `Err` on any API failure.
///
/// # GitHub API
///
/// `GET /repos/{owner}/{repo}/commits/{sha}/statuses`
async fn get_commit_statuses(
    &self,
    repo_owner: &str,
    repo_name: &str,
    commit_sha: &str,
) -> Result<Vec<CommitStatus>, Error>;
```

**Deduplication responsibility:** callers are responsible for deduplicating by context.
The API returns entries in newest-first order; keeping the first occurrence per context
yields the most recent status for each check.

**Pagination:** first page only. This is sufficient because Renovate updates its status
entry in place and the relevant entry is always on the first page. Document and revisit if
evidence of missed statuses emerges in production.

#### `GitHubProvider` implementation of `get_commit_statuses`

Maps the GitHub API response array to `Vec<CommitStatus>`. HTTP 404 (commit not found)
returns `Err`. An empty statuses array returns `Ok(vec![])`.

#### Behavioral postconditions for `get_commit_statuses`

1. `Ok(vec![])` is returned when the commit exists but has no associated statuses.
2. `Err` is returned for all non-200 API responses (including 404 commit-not-found).
3. The returned vector preserves the API's newest-first ordering.
4. Only the first page of results is fetched; no pagination loop is performed.

---

### New method on `PullRequestProvider`: `find_pull_requests_for_commit`

```rust
/// Finds open pull requests whose HEAD commit matches `commit_sha`.
///
/// Used by the `status` event handler to map a commit SHA back to the pull
/// requests that should be re-evaluated when a commit status changes.
///
/// # Arguments
///
/// * `repo_owner` — Repository owner.
/// * `repo_name`  — Repository name.
/// * `commit_sha` — Full SHA of the commit to look up.
///
/// # Returns
///
/// `Ok(Vec<u64>)` — pull request numbers (possibly empty if no open PRs exist
/// for this commit).
/// `Err` on any API failure.
///
/// # GitHub API
///
/// `GET /repos/{owner}/{repo}/commits/{sha}/pulls`
///
/// This endpoint is generally available and no longer requires a preview
/// `Accept` header. Use the standard `application/vnd.github+json` header.
async fn find_pull_requests_for_commit(
    &self,
    repo_owner: &str,
    repo_name: &str,
    commit_sha: &str,
) -> Result<Vec<u64>, Error>;
```

**Usage:** this method is called only from the `status` event routing path
(`handle_status_event` in `crates/server/src/webhook.rs`). It is not called during the
normal `pull_request` event processing path.

#### `GitHubProvider` implementation of `find_pull_requests_for_commit`

Calls `GET /repos/{owner}/{repo}/commits/{sha}/pulls`, extracts the `number` field from
each entry in the JSON array, and returns those as `Vec<u64>`.

#### Behavioral postconditions for `find_pull_requests_for_commit`

1. `Ok(vec![])` is returned when no open pull requests reference the given commit SHA.
2. `Err` is returned for all non-200 API responses.
3. Only PR numbers are returned; no other PR data is surfaced through this method.
