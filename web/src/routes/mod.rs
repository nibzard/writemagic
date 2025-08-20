use axum::{
    http::Method,
    middleware,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, DefaultOnRequest, TraceLayer},
};
use tracing::Level;

use crate::{
    extractors::request_id_middleware,
    state::AppState,
    websocket,
};

pub mod api;
pub mod auth;
pub mod documents;
pub mod health;

/// Create the main application router with all middleware and routes
/// Following the middleware layering order from the best practices guide
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state.config.cors.allowed_origins
                .iter()
                .map(|origin| origin.parse().expect("Invalid CORS origin"))
                .collect::<Vec<_>>()
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .max_age(std::time::Duration::from_secs(state.config.cors.max_age_secs));

    Router::new()
        .merge(health::router())
        .nest("/api", api::router())
        .merge(websocket::handler::websocket_routes())
        // Add more route modules here as they are implemented
        // Apply middleware layers in the correct order
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(state.config.server.body_limit_bytes))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(state.config.server.request_timeout()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        )
        .layer(middleware::from_fn(request_id_middleware))
        .with_state(state)
}