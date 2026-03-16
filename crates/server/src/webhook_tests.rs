use axum::{http::StatusCode, response::IntoResponse};

use super::health_check;

// ---------------------------------------------------------------------------
// health_check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_check_returns_200_ok() {
    let response = health_check().await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}
