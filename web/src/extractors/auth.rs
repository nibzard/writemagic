use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use crate::services::auth::UserInfo;
use crate::state::AppState;
use crate::utils::crypto::Claims;

/// Authenticated user extractor
/// This extractor validates JWT tokens and provides user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
    pub claims: Claims,
}

impl AuthenticatedUser {
    pub fn new(claims: Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            username: claims.username.clone(),
            claims,
        }
    }

    /// Convert to UserInfo for responses
    pub fn to_user_info(&self) -> UserInfo {
        UserInfo {
            id: self.user_id.clone(),
            username: self.username.clone(),
            email: String::new(), // Will be populated from database if needed
        }
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AuthError::MissingToken)?
            .to_str()
            .map_err(|_| AuthError::InvalidToken)?;

        // Extract Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Validate token using JWT keys from app state
        let auth_service = crate::services::auth::AuthService::new(app_state.jwt_keys.clone());
        let claims = auth_service
            .validate_token(token)
            .map_err(|_| AuthError::InvalidToken)?;

        // Ensure this is an access token
        if claims.token_type != crate::utils::crypto::TokenType::Access {
            return Err(AuthError::InvalidToken);
        }

        Ok(AuthenticatedUser::new(claims))
    }
}

/// Optional authenticated user extractor
/// This extractor doesn't fail if no authentication is provided
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AuthenticatedUser>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalUser(Some(user))),
            Err(_) => Ok(OptionalUser(None)),
        }
    }
}

/// Authentication error responses
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authentication token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token"),
            AuthError::Unauthorized => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };

        let body = serde_json::json!({
            "error": "authentication_error",
            "message": message,
            "status": status.as_u16()
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Admin user extractor
/// This extractor ensures the user has admin privileges
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user: AuthenticatedUser,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        
        // For now, we'll implement a simple check
        // In a real application, you'd check user roles from the database
        if user.username == "admin" {
            Ok(AdminUser { user })
        } else {
            Err(AuthError::Unauthorized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::crypto::{JwtKeys, TokenManager, TokenType};
    use axum::http::{HeaderMap, HeaderValue};
    use std::sync::Arc;

    fn create_test_claims() -> Claims {
        use crate::utils::crypto::Claims;
        Claims {
            sub: "test_user_123".to_string(),
            username: "testuser".to_string(),
            exp: (chrono::Utc::now().timestamp() + 3600) as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            jti: "test_jti".to_string(),
            token_type: TokenType::Access,
        }
    }

    #[test]
    fn test_authenticated_user_creation() {
        let claims = create_test_claims();
        let user = AuthenticatedUser::new(claims.clone());
        
        assert_eq!(user.user_id, claims.sub);
        assert_eq!(user.username, claims.username);
        
        let user_info = user.to_user_info();
        assert_eq!(user_info.id, claims.sub);
        assert_eq!(user_info.username, claims.username);
    }

    #[test]
    fn test_auth_error_responses() {
        let missing_token_response = AuthError::MissingToken.into_response();
        assert_eq!(missing_token_response.status(), StatusCode::UNAUTHORIZED);
        
        let invalid_token_response = AuthError::InvalidToken.into_response();
        assert_eq!(invalid_token_response.status(), StatusCode::UNAUTHORIZED);
        
        let unauthorized_response = AuthError::Unauthorized.into_response();
        assert_eq!(unauthorized_response.status(), StatusCode::FORBIDDEN);
    }
}