use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers::documents, state::AppState};

/// Create document management routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(documents::list_documents))
        .route("/", post(documents::create_document))
        .route("/:id", get(documents::get_document))
        .route("/:id", put(documents::update_document))
        .route("/:id", delete(documents::delete_document))
}