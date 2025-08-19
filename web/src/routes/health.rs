use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::{error::Result, state::AppState, telemetry::MetricsCollector};

/// Health check routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/metrics", get(metrics_endpoint))
}

/// Basic health check endpoint
/// Returns 200 OK if the service is running
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "writemagic-web",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
}

/// Readiness check endpoint  
/// Returns 200 OK if the service is ready to accept traffic
/// This includes checking database connectivity and other dependencies
async fn readiness_check(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let collector = MetricsCollector::new(state);
    let health = collector.health_check().await;
    
    let status_code = if health.healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    Ok((
        status_code,
        Json(json!({
            "status": if health.healthy { "ready" } else { "not_ready" },
            "checks": {
                "database": health.database,
                "rate_limiter": health.rate_limiter,
                "cache": health.cache,
            },
            "service": "writemagic-web",
            "version": health.version,
            "timestamp": health.timestamp.to_rfc3339()
        })),
    ))
}

/// Metrics endpoint for application monitoring
async fn metrics_endpoint(State(state): State<AppState>) -> impl IntoResponse {
    let collector = MetricsCollector::new(state);
    let metrics = collector.get_metrics();
    
    (StatusCode::OK, Json(metrics))
}