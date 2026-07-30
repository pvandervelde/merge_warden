// See .llm/task.md — "Enhanced Health Check"
//
// See docs/user/reference/environment-variables.md — MERGE_WARDEN_HEALTH_CHECKS
// and the "Queue Health Check Limitations" note for why the `queue` check
// never reports depth.

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use github_bot_sdk::client::GitHubClient;

use crate::webhook::AppState;

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;

/// Upper bound on the GitHub reachability probe in full mode. Chosen to be
/// short enough that a slow/unreachable GitHub API does not make `/health`
/// itself feel unresponsive to an operator or an orchestrator's liveness
/// probe, while still allowing time for a real (if slow) round trip.
const GITHUB_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------
// HealthState
// ---------------------------------------------------------------------------

/// Overall or per-component health classification.
///
/// Serializes to lowercase strings (`"healthy"`, `"degraded"`, `"unhealthy"`)
/// per the JSON shape in `.llm/task.md`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    /// Everything checked is fully operational.
    Healthy,
    /// At least one non-critical dependency is impaired, but the service can
    /// still process requests.
    ///
    /// No dependency check in this binary currently classifies anything as
    /// merely "degraded" (github_api is always the whole point of this
    /// service, so its unreachability is `Unhealthy` rather than
    /// `Degraded`); this variant is kept as public API for a future
    /// soft-dependency check (e.g. a queue-depth threshold once
    /// `queue-runtime` gains a depth-query API — see `set_queue_depth`).
    #[allow(dead_code)]
    Degraded,
    /// A critical dependency is unavailable; the service cannot reliably
    /// process requests.
    Unhealthy,
}

impl HealthState {
    /// Maps this health state to the HTTP status code `GET /health` must return.
    ///
    /// `Healthy` -> `200 OK`, `Degraded` -> `207 Multi-Status`, `Unhealthy` -> `503 Service Unavailable`.
    pub fn http_status(&self) -> StatusCode {
        match self {
            HealthState::Healthy => StatusCode::OK,
            HealthState::Degraded => StatusCode::MULTI_STATUS,
            HealthState::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Numeric severity rank used by [`aggregate_status`] to combine several
    /// component states into one overall state (higher = worse).
    fn severity(&self) -> u8 {
        match self {
            HealthState::Healthy => 0,
            HealthState::Degraded => 1,
            HealthState::Unhealthy => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// ComponentHealth / HealthReport
// ---------------------------------------------------------------------------

/// Health of a single dependency (`github_api`, `config`, or `queue`).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ComponentHealth {
    /// This component's health classification.
    pub status: HealthState,
    /// Round-trip latency of the check, in milliseconds, if measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// Approximate queue depth, if this component is `queue`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u64>,
    /// A short, non-sensitive diagnostic message. Always present on failure;
    /// also used (while still `Healthy`) to disclose a check's known
    /// limitations, e.g. [`QUEUE_HEALTH_LIMITATION_MESSAGE`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ComponentHealth {
    /// A component that is always reported healthy, with no measured latency
    /// or depth (e.g. `config`, or `queue`/`github_api` in basic mode).
    fn healthy() -> Self {
        ComponentHealth {
            status: HealthState::Healthy,
            latency_ms: None,
            depth: None,
            message: None,
        }
    }
}

/// Full `GET /health` response body.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct HealthReport {
    /// Aggregate status derived from `checks` (the least-healthy value wins).
    pub status: HealthState,
    /// UTC time at which this report was generated. Every `GET /health` call
    /// runs its checks fresh (nothing is cached), so this is always
    /// approximately "now" — it exists so orchestrators, monitoring systems,
    /// and operators reading a saved/forwarded copy of a response body can
    /// tell how old it is, rather than because responses are ever stale at
    /// the point of generation.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Per-dependency health, keyed by component name
    /// (`"github_api"`, `"config"`, and — queue mode only — `"queue"`).
    pub checks: BTreeMap<String, ComponentHealth>,
}

/// Combines several component states into one overall state: the
/// least-healthy value wins (`Unhealthy` > `Degraded` > `Healthy`).
/// An empty iterator is vacuously `Healthy`.
fn aggregate_status<'a>(checks: impl Iterator<Item = &'a ComponentHealth>) -> HealthState {
    checks
        .map(|c| c.status.clone())
        .max_by_key(|s| s.severity())
        .unwrap_or(HealthState::Healthy)
}

// ---------------------------------------------------------------------------
// HealthCheckConfig
// ---------------------------------------------------------------------------

/// Controls whether `GET /health` performs real dependency probing.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// `true` when `MERGE_WARDEN_HEALTH_CHECKS == "full"`. `false` (basic,
    /// liveness-only) for unset or any other value, including `"basic"`.
    pub full_checks_enabled: bool,
}

impl HealthCheckConfig {
    /// Reads `MERGE_WARDEN_HEALTH_CHECKS` from the environment.
    ///
    /// Never fails — absent or unrecognised values fall back to basic mode.
    pub fn from_env() -> Self {
        HealthCheckConfig {
            full_checks_enabled: std::env::var("MERGE_WARDEN_HEALTH_CHECKS")
                .map(|v| v == "full")
                .unwrap_or(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Dependency probes
// ---------------------------------------------------------------------------

/// Probes GitHub API reachability (and, transitively, whether this app's
/// credentials can still authenticate) by requesting app-level metadata
/// (`GET /app`) with a fresh app JWT.
///
/// A deliberately generic, static diagnostic message is used on failure —
/// the underlying SDK error may otherwise embed raw HTTP response bodies
/// from GitHub, which must never be echoed back verbatim in a health
/// endpoint response (see the "no internal diagnostics leaked" security
/// tests in `health_tests.rs`).
///
/// Bounded by [`GITHUB_PROBE_TIMEOUT`] so a hanging connection cannot make
/// `/health` itself hang.
async fn probe_github_api(github_client: &GitHubClient) -> ComponentHealth {
    let start = Instant::now();
    let outcome = tokio::time::timeout(GITHUB_PROBE_TIMEOUT, github_client.get_app()).await;
    let latency_ms = Some(start.elapsed().as_millis() as u64);

    match outcome {
        Ok(Ok(_app)) => ComponentHealth {
            status: HealthState::Healthy,
            latency_ms,
            depth: None,
            message: None,
        },
        Ok(Err(_e)) => ComponentHealth {
            status: HealthState::Unhealthy,
            latency_ms,
            depth: None,
            message: Some("GitHub API request failed".to_string()),
        },
        Err(_elapsed) => ComponentHealth {
            status: HealthState::Unhealthy,
            latency_ms,
            depth: None,
            message: Some("GitHub API request timed out".to_string()),
        },
    }
}

/// A short, static, non-sensitive note attached to the `queue` check in full
/// mode, so a caller reading the `/health` response body alone (not the
/// source code) can tell "checked and healthy" apart from "not actually
/// checkable" — see [`queue_health`].
const QUEUE_HEALTH_LIMITATION_MESSAGE: &str =
    "Reports only that a queue client was constructed at startup; queue-runtime 0.2.1 exposes no depth/reachability query API, so this is not a live probe";

/// Health of the queue dependency (queue mode only).
///
/// `queue-runtime` 0.2.1's `QueueClient`/`SessionClient` traits expose no
/// depth-query method, so this check cannot report a real `depth`. It also
/// deliberately never calls `accept_session` (in either basic or full mode)
/// — doing so could steal a session lock away from a genuine worker task
/// that is concurrently processing a PR. Instead, "healthy" here means only
/// "a queue client was successfully constructed at startup", which is the
/// best signal available without risking worker contention.
///
/// In full mode, [`QUEUE_HEALTH_LIMITATION_MESSAGE`] is attached so this
/// limitation is visible in the HTTP response body itself, not only in this
/// doc comment — an operator who enables `MERGE_WARDEN_HEALTH_CHECKS=full`
/// expecting a real queue probe should not be misled by a bare `"healthy"`.
/// In basic mode no message is attached, consistent with every other check
/// in basic mode reporting a plain, message-free `healthy`.
///
/// See docs/user/reference/environment-variables.md — "Queue Health Check
/// Limitations".
fn queue_health(full_checks_enabled: bool) -> ComponentHealth {
    if full_checks_enabled {
        ComponentHealth {
            status: HealthState::Healthy,
            latency_ms: None,
            depth: None,
            message: Some(QUEUE_HEALTH_LIMITATION_MESSAGE.to_string()),
        }
    } else {
        ComponentHealth::healthy()
    }
}

// ---------------------------------------------------------------------------
// health_check_handler
// ---------------------------------------------------------------------------

/// `GET /health` — structured dependency health report.
///
/// In basic mode (`state.health_check_config.full_checks_enabled == false`),
/// returns a fast, liveness-only response without contacting GitHub or the
/// queue broker — every check is reported `healthy`.
///
/// In full mode, probes GitHub API reachability and (queue mode only) checks
/// queue connectivity. Configuration is always reported `healthy` — startup
/// already validates configuration before the HTTP server exists, so
/// reaching this handler at all proves it is valid; no runtime re-check is
/// necessary. The `"queue"` key is present in `checks` if and only if the
/// server is running in queue mode (`state.queue_client.is_some()`).
///
/// # Responses
/// - `200 OK` — overall status `healthy`.
/// - `207 Multi-Status` — overall status `degraded`.
/// - `503 Service Unavailable` — overall status `unhealthy`.
pub async fn health_check_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut checks = BTreeMap::new();

    // Startup already validates configuration before the HTTP server exists,
    // so reaching this handler proves configuration is valid.
    checks.insert("config".to_string(), ComponentHealth::healthy());

    let github_health = if state.health_check_config.full_checks_enabled {
        probe_github_api(&state.github_client).await
    } else {
        // Basic (liveness-only) mode never contacts GitHub.
        ComponentHealth::healthy()
    };
    checks.insert("github_api".to_string(), github_health);

    if state.queue_client.is_some() {
        checks.insert(
            "queue".to_string(),
            queue_health(state.health_check_config.full_checks_enabled),
        );
    }

    let status = aggregate_status(checks.values());
    let report = HealthReport {
        status: status.clone(),
        timestamp: chrono::Utc::now(),
        checks,
    };

    (status.http_status(), Json(report))
}
