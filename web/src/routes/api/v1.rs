use axum::Router;

use crate::{routes::{auth, documents}, state::AppState};

/// Create API v1 routes
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/documents", documents::router())
        // Add more API endpoints here as they are implemented
        // .nest("/projects", projects::router())
        // .nest("/ai", ai::router())
}