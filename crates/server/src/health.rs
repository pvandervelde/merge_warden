// See .llm/task.md — "Enhanced Health Check"
//
// STATUS: pre-implementation stub (TDD RED phase). `health_check_handler`
// intentionally ignores mode (basic/full), receiver mode (webhook/queue), and
// actual dependency reachability — it always reports a hardcoded "healthy"
// shell. See `health_tests.rs` for the behavioural contract the real
// implementation must satisfy.

use std::{collections::BTreeMap, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::webhook::AppState;

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;

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
        // STUB: always returns 200 regardless of variant. A correct
        // implementation must map Degraded -> 207 and Unhealthy -> 503.
        StatusCode::OK
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
    /// A short, non-sensitive diagnostic message when not healthy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Full `GET /health` response body.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct HealthReport {
    /// Aggregate status derived from `checks` (the least-healthy value wins).
    pub status: HealthState,
    /// Per-dependency health, keyed by component name
    /// (`"github_api"`, `"config"`, and — queue mode only — `"queue"`).
    pub checks: BTreeMap<String, ComponentHealth>,
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
        // STUB: always basic mode; MERGE_WARDEN_HEALTH_CHECKS is never read.
        HealthCheckConfig {
            full_checks_enabled: false,
        }
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
/// In full mode, probes GitHub API reachability, re-validates configuration,
/// and (queue mode only) checks queue connectivity/depth. The `"queue"` key
/// is present in `checks` if and only if the server is running in queue mode
/// (`state.queue_client.is_some()`).
///
/// # Responses
/// - `200 OK` — overall status `healthy`.
/// - `207 Multi-Status` — overall status `degraded`.
/// - `503 Service Unavailable` — overall status `unhealthy`.
pub async fn health_check_handler(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // STUB: hardcoded healthy report containing only "config". Ignores mode,
    // queue presence, and GitHub reachability entirely.
    let mut checks = BTreeMap::new();
    checks.insert(
        "config".to_string(),
        ComponentHealth {
            status: HealthState::Healthy,
            latency_ms: None,
            depth: None,
            message: None,
        },
    );

    let report = HealthReport {
        status: HealthState::Healthy,
        checks,
    };

    (report.status.http_status(), Json(report))
}
