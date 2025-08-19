use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod entities;
mod error;
mod state;
mod routes;
mod handlers;
mod services;
mod middleware;
mod utils;
mod extractors;
mod telemetry;
mod websocket;

use crate::{config::Config, state::AppState, routes::create_router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured tracing
    telemetry::init_tracing()?;

    // Load configuration
    let config = Config::from_env()?;
    
    // Create application state
    let state = AppState::new(config.clone()).await?;
    
    // Start background tasks
    let metrics_task = tokio::spawn(telemetry::metrics_collection_task(state.clone()));
    let rate_limit_cleanup_task = tokio::spawn(
        crate::middleware::rate_limit::rate_limit_cleanup_task(state.rate_limiter.clone())
    );
    
    // Create router
    let app = create_router(state.clone());
    
    // Create server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(&addr).await?;
    
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        addr = %addr,
        "WriteMagic Web Server starting"
    );
    
    // Start server with graceful shutdown
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());
    
    // Wait for either the server to finish or background tasks to complete
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = metrics_task => {
            tracing::warn!("Metrics task completed unexpectedly");
        }
        _ = rate_limit_cleanup_task => {
            tracing::warn!("Rate limit cleanup task completed unexpectedly");
        }
    }
    
    // Graceful shutdown
    state.shutdown().await;
    tracing::info!("WriteMagic Web Server shutdown complete");
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}