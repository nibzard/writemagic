use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, PaginatorTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{user, User};
use crate::error::{AppError, Result as AppResult};
use crate::state::AppState;
use crate::utils::crypto::{Claims, JwtKeys, PasswordHasher, TokenManager, TokenPair};

/// User registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// User login request  
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// User authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserInfo,
    pub tokens: TokenPair,
}

/// User information
#[derive(Debug, Serialize, Clone)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// Use the SeaORM user model directly
impl From<user::Model> for UserInfo {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

/// Authentication service
pub struct AuthService {
    jwt_keys: Arc<JwtKeys>,
}

impl AuthService {
    pub fn new(jwt_keys: Arc<JwtKeys>) -> Self {
        Self { jwt_keys }
    }

    /// Register a new user
    pub async fn register(&self, state: &AppState, request: RegisterRequest) -> AppResult<AuthResponse> {
        // Validate input
        self.validate_registration(&request)?;
        
        // Check if user already exists
        if self.user_exists_by_username(state, &request.username).await? {
            return Err(AppError::Conflict("Username already exists".to_string()));
        }
        
        if self.user_exists_by_email(state, &request.email).await? {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        // Hash password
        let password_hash = PasswordHasher::hash_password(&request.password)?;

        // Create user using SeaORM
        let user_model = user::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            username: Set(request.username),
            email: Set(request.email),
            password_hash: Set(password_hash),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            is_active: Set(true),
            role: Set("user".to_string()),
        };

        let user = user_model.insert(&state.db).await
            .map_err(|e| AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to create user: {}", e))))?;

        // Generate tokens
        let tokens = TokenManager::generate_token_pair(&self.jwt_keys, &user.id, &user.username)?;

        Ok(AuthResponse {
            user: UserInfo::from(user),
            tokens,
        })
    }

    /// Authenticate user login
    pub async fn login(&self, state: &AppState, request: LoginRequest) -> AppResult<AuthResponse> {
        // Find user by username
        let user = self.find_user_by_username(state, &request.username).await?
            .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = PasswordHasher::verify_password(&request.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Authentication("Invalid credentials".to_string()));
        }

        // Generate tokens
        let tokens = TokenManager::generate_token_pair(&self.jwt_keys, &user.id, &user.username)?;

        Ok(AuthResponse {
            user: UserInfo::from(user),
            tokens,
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, request: RefreshRequest) -> AppResult<TokenPair> {
        TokenManager::refresh_token(&self.jwt_keys, &request.refresh_token)
    }

    /// Validate JWT token and return claims
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        TokenManager::validate_token(&self.jwt_keys, token)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, state: &AppState, user_id: &str) -> AppResult<Option<UserInfo>> {
        let user = self.find_user_by_id(state, user_id).await?;
        Ok(user.map(UserInfo::from))
    }

    // Validation helpers
    fn validate_registration(&self, request: &RegisterRequest) -> AppResult<()> {
        if request.username.len() < 3 {
            return Err(AppError::Validation("Username must be at least 3 characters".to_string()));
        }

        if request.username.len() > 50 {
            return Err(AppError::Validation("Username must be less than 50 characters".to_string()));
        }

        if !request.email.contains('@') {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }

        if request.password.len() < 8 {
            return Err(AppError::Validation("Password must be at least 8 characters".to_string()));
        }

        Ok(())
    }

    // Database operations using SeaORM
    async fn user_exists_by_username(&self, state: &AppState, username: &str) -> AppResult<bool> {
        let count = User::find()
            .filter(user::Column::Username.eq(username))
            .count(&state.db)
            .await
            .map_err(|e| AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to check username existence: {}", e))))?;
        
        Ok(count > 0)
    }

    async fn user_exists_by_email(&self, state: &AppState, email: &str) -> AppResult<bool> {
        let count = User::find()
            .filter(user::Column::Email.eq(email))
            .count(&state.db)
            .await
            .map_err(|e| AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to check email existence: {}", e))))?;
        
        Ok(count > 0)
    }

    async fn find_user_by_username(&self, state: &AppState, username: &str) -> AppResult<Option<user::Model>> {
        let user = User::find()
            .filter(user::Column::Username.eq(username))
            .filter(user::Column::IsActive.eq(true))
            .one(&state.db)
            .await
            .map_err(|e| AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to find user by username: {}", e))))?;
        
        Ok(user)
    }

    async fn find_user_by_id(&self, state: &AppState, user_id: &str) -> AppResult<Option<user::Model>> {
        let user = User::find_by_id(user_id)
            .filter(user::Column::IsActive.eq(true))
            .one(&state.db)
            .await
            .map_err(|e| AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to find user by ID: {}", e))))?;
        
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::crypto::JwtKeys;

    fn create_test_auth_service() -> AuthService {
        let (jwt_keys, _) = JwtKeys::generate();
        AuthService::new(Arc::new(jwt_keys))
    }

    #[test]
    fn test_validate_registration() {
        let auth_service = create_test_auth_service();

        // Valid registration
        let valid_request = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(auth_service.validate_registration(&valid_request).is_ok());

        // Username too short
        let invalid_request = RegisterRequest {
            username: "ab".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(auth_service.validate_registration(&invalid_request).is_err());

        // Invalid email
        let invalid_request = RegisterRequest {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(auth_service.validate_registration(&invalid_request).is_err());

        // Password too short
        let invalid_request = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(auth_service.validate_registration(&invalid_request).is_err());
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert!(!user.id.is_empty());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");

        let user_info = user.to_user_info();
        assert_eq!(user_info.id, user.id);
        assert_eq!(user_info.username, user.username);
        assert_eq!(user_info.email, user.email);
    }
}