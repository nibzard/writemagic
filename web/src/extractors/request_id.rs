use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Request ID extractor
/// Extracts the request ID from headers or generates a new one
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to extract request ID from headers
        let request_id = parts
            .headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(|id| RequestId(id.to_string()))
            .unwrap_or_else(RequestId::new);

        Ok(request_id)
    }
}

/// Middleware to add request ID to all requests
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add request ID to request extensions for easy access
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // Add request ID to headers if not present
    if !request.headers().contains_key("x-request-id") {
        request.headers_mut().insert(
            "x-request-id",
            request_id.parse().expect("Generated UUID should be valid"),
        );
    }

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().expect("Request ID should be valid"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{HeaderMap, HeaderValue, Request},
        middleware::from_fn,
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[test]
    fn test_request_id_creation() {
        let request_id = RequestId::new();
        assert!(!request_id.get().is_empty());
        assert!(uuid::Uuid::parse_str(request_id.get()).is_ok());
    }

    #[test]
    fn test_request_id_default() {
        let request_id = RequestId::default();
        assert!(!request_id.get().is_empty());
        assert!(uuid::Uuid::parse_str(request_id.get()).is_ok());
    }

    #[tokio::test]
    async fn test_request_id_middleware() {
        async fn handler() -> &'static str {
            "OK"
        }

        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn(request_id_middleware));

        // Test request without request ID
        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Response should have request ID header
        assert!(response.headers().contains_key("x-request-id"));
        let response_id = response.headers().get("x-request-id").unwrap();
        assert!(uuid::Uuid::parse_str(response_id.to_str().unwrap()).is_ok());

        // Test request with existing request ID
        let existing_id = "custom-request-id-123";
        let request = Request::builder()
            .uri("/")
            .header("x-request-id", existing_id)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // Response should preserve the existing request ID
        let response_id = response.headers().get("x-request-id").unwrap();
        assert_eq!(response_id.to_str().unwrap(), existing_id);
    }
}