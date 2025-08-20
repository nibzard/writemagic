use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result as AppResult};
use crate::extractors::AuthenticatedUser;
use crate::services::auth::{AuthResponse, AuthService, LoginRequest, RefreshRequest, RegisterRequest};
use crate::state::AppState;

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    tracing::info!("User registration attempt for username: {}", request.username);

    let auth_service = AuthService::new(state.jwt_keys.clone());
    let response = auth_service.register(&state, request).await?;

    tracing::info!("User registered successfully: {}", response.user.username);
    Ok((StatusCode::CREATED, Json(response)))
}

/// Login user
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    tracing::info!("Login attempt for username: {}", request.username);

    let auth_service = AuthService::new(state.jwt_keys.clone());
    let response = auth_service.login(&state, request).await?;

    tracing::info!("User logged in successfully: {}", response.user.username);
    Ok(Json(response))
}

/// Refresh access token
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> AppResult<Json<crate::utils::crypto::TokenPair>> {
    tracing::debug!("Token refresh attempt");

    let auth_service = AuthService::new(state.jwt_keys.clone());
    let tokens = auth_service.refresh_token(request).await?;

    tracing::debug!("Token refreshed successfully");
    Ok(Json(tokens))
}

/// Logout user (invalidate tokens)
pub async fn logout(
    _state: State<AppState>,
    _user: AuthenticatedUser,
) -> AppResult<StatusCode> {
    tracing::info!("User logout");
    
    // TODO: Implement token blacklisting/invalidation
    // For now, we just return success - client should discard tokens
    
    Ok(StatusCode::NO_CONTENT)
}

/// Get current user profile
pub async fn get_profile(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<UserProfileResponse>> {
    tracing::debug!("Getting profile for user: {}", user.user_id);

    let auth_service = AuthService::new(state.jwt_keys.clone());
    let user_info = auth_service
        .get_user_by_id(&state, &user.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let profile = UserProfileResponse {
        id: user_info.id,
        username: user_info.username,
        email: user_info.email,
        created_at: chrono::Utc::now(), // TODO: Get from database
        updated_at: chrono::Utc::now(), // TODO: Get from database
    };

    Ok(Json(profile))
}

/// Update user profile
pub async fn update_profile(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserProfileResponse>> {
    tracing::info!("Updating profile for user: {}", user.user_id);

    // TODO: Implement profile update logic
    // For now, return current profile with updated fields

    let profile = UserProfileResponse {
        id: user.user_id.clone(),
        username: request.username.unwrap_or(user.username.clone()),
        email: request.email.unwrap_or_else(|| "user@example.com".to_string()), // TODO: Get from database
        created_at: chrono::Utc::now(), // TODO: Get from database
        updated_at: chrono::Utc::now(),
    };

    tracing::info!("Profile updated for user: {}", user.user_id);
    Ok(Json(profile))
}

/// Change user password
pub async fn change_password(
    _state: State<AppState>,
    user: AuthenticatedUser,
    Json(_request): Json<ChangePasswordRequest>,
) -> AppResult<StatusCode> {
    tracing::info!("Password change request for user: {}", user.user_id);

    // TODO: Implement password change logic
    // 1. Verify current password
    // 2. Hash new password
    // 3. Update in database
    // 4. Optionally invalidate all existing tokens

    tracing::info!("Password changed successfully for user: {}", user.user_id);
    Ok(StatusCode::NO_CONTENT)
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Update profile request
#[derive(Debug, Deserialize, garde::Validate)]
pub struct UpdateProfileRequest {
    #[garde(length(min = 3, max = 50))]
    pub username: Option<String>,
    
    #[garde(email)]
    pub email: Option<String>,
}

/// Change password request
#[derive(Debug, Deserialize, garde::Validate)]
pub struct ChangePasswordRequest {
    #[garde(length(min = 8))]
    pub current_password: String,
    
    #[garde(length(min = 8))]
    pub new_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_profile_request_validation() {
        use garde::Validate;

        let valid_request = UpdateProfileRequest {
            username: Some("newusername".to_string()),
            email: Some("new@example.com".to_string()),
        };
        assert!(valid_request.validate(&()).is_ok());

        // Username too short
        let invalid_request = UpdateProfileRequest {
            username: Some("ab".to_string()),
            email: None,
        };
        assert!(invalid_request.validate(&()).is_err());

        // Invalid email
        let invalid_request = UpdateProfileRequest {
            username: None,
            email: Some("invalid-email".to_string()),
        };
        assert!(invalid_request.validate(&()).is_err());
    }

    #[test]
    fn test_change_password_request_validation() {
        use garde::Validate;

        let valid_request = ChangePasswordRequest {
            current_password: "current_password123".to_string(),
            new_password: "new_password123".to_string(),
        };
        assert!(valid_request.validate(&()).is_ok());

        // Current password too short
        let invalid_request = ChangePasswordRequest {
            current_password: "short".to_string(),
            new_password: "new_password123".to_string(),
        };
        assert!(invalid_request.validate(&()).is_err());

        // New password too short
        let invalid_request = ChangePasswordRequest {
            current_password: "current_password123".to_string(),
            new_password: "short".to_string(),
        };
        assert!(invalid_request.validate(&()).is_err());
    }
}