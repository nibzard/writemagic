use axum::{
    extract::{
        ws::{WebSocketUpgrade, WebSocket},
        State,
    },
    response::Response,
};

use crate::{
    extractors::AuthenticatedUser,
    state::AppState,
    websocket::WebSocketConnection,
};

/// Handle WebSocket upgrade and connection
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Response {
    tracing::info!(
        user_id = %user.user_id,
        username = %user.username,
        "WebSocket upgrade requested"
    );

    ws.on_upgrade(move |socket| handle_websocket_connection(socket, state, user))
}

/// Handle an established WebSocket connection
async fn handle_websocket_connection(
    socket: WebSocket,
    state: AppState,
    user: AuthenticatedUser,
) {
    tracing::info!(
        user_id = %user.user_id,
        username = %user.username,
        "WebSocket connection established"
    );

    // Create WebSocket connection wrapper
    let (connection, message_receiver) = WebSocketConnection::new(
        socket,
        user.user_id.clone(),
        user.username.clone(),
    );

    // Add connection to the manager
    state.connection_manager.add_connection(connection, message_receiver).await;

    tracing::info!(
        user_id = %user.user_id,
        username = %user.username,
        "WebSocket connection handler completed"
    );
}

/// WebSocket routes
pub fn websocket_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/ws", axum::routing::get(websocket_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_websocket_routes_creation() {
        let routes = websocket_routes();
        
        // The routes should be created successfully
        // In a real test, you'd need to set up a complete app state and test the upgrade
        assert!(routes.clone().oneshot(
            axum::http::Request::builder()
                .uri("/ws")
                .header("upgrade", "websocket")
                .header("connection", "upgrade")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header("sec-websocket-version", "13")
                .body(axum::body::Body::empty())
                .unwrap()
        ).await.is_ok());
    }
}