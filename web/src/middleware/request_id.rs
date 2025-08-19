use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware to add a unique request ID to each request
/// This enables request tracing across the entire request lifecycle
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Check if request already has an ID (from client or proxy)
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add request ID to tracing context
    let span = tracing::Span::current();
    span.record("request_id", &request_id);

    // Ensure the request has the ID in headers
    request.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    // Process the request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}