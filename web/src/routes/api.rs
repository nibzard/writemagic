use axum::Router;

use crate::state::AppState;

pub mod v1;

/// Create the main API router with versioning
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/v1", v1::router())
        // Add future API versions here
        // .nest("/v2", v2::router())
}