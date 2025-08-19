use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers::auth, state::AppState};

/// Create authentication routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
        .route("/logout", post(auth::logout))
        .route("/profile", get(auth::get_profile))
        .route("/profile", post(auth::update_profile))
        .route("/change-password", post(auth::change_password))
}