use axum::Router;
use axum_test::TestServer;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tempfile::TempDir;

use writemagic_web::{
    config::Config,
    entities::{user, User},
    routes::create_router,
    state::AppState,
    utils::crypto::PasswordHasher,
};

/// Test application builder for integration tests
pub struct TestApp {
    pub server: TestServer,
    pub app_state: AppState,
    pub temp_dir: TempDir,
}

impl TestApp {
    /// Create a new test application with in-memory database
    pub async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

        // Create test configuration
        let config = Config::test_config(&database_url);
        
        // Initialize application state
        let app_state = AppState::new(config)
            .await
            .expect("Failed to create test app state");

        // Create router
        let app = create_router(app_state.clone());
        
        // Create test server
        let server = TestServer::new(app).expect("Failed to create test server");

        Self {
            server,
            app_state,
            temp_dir,
        }
    }

    /// Create a test user and return the user model
    pub async fn create_test_user(&self, username: &str, email: &str, password: &str) -> user::Model {
        let password_hash = PasswordHasher::hash_password(password)
            .expect("Failed to hash password");

        let user_model = user::ActiveModel {
            id: sea_orm::Set(uuid::Uuid::new_v4().to_string()),
            username: sea_orm::Set(username.to_string()),
            email: sea_orm::Set(email.to_string()),
            password_hash: sea_orm::Set(password_hash),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
            is_active: sea_orm::Set(true),
            role: sea_orm::Set("user".to_string()),
        };

        user_model
            .insert(&self.app_state.db)
            .await
            .expect("Failed to create test user")
    }

    /// Create an admin test user
    pub async fn create_admin_user(&self, username: &str, email: &str, password: &str) -> user::Model {
        let password_hash = PasswordHasher::hash_password(password)
            .expect("Failed to hash password");

        let user_model = user::ActiveModel {
            id: sea_orm::Set(uuid::Uuid::new_v4().to_string()),
            username: sea_orm::Set(username.to_string()),
            email: sea_orm::Set(email.to_string()),
            password_hash: sea_orm::Set(password_hash),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
            is_active: sea_orm::Set(true),
            role: sea_orm::Set("admin".to_string()),
        };

        user_model
            .insert(&self.app_state.db)
            .await
            .expect("Failed to create admin test user")
    }

    /// Register a user via API and return auth response
    pub async fn register_user(&self, username: &str, email: &str, password: &str) -> serde_json::Value {
        let register_request = serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        });

        self.server
            .post("/api/v1/auth/register")
            .json(&register_request)
            .await
            .json::<serde_json::Value>()
    }

    /// Login a user via API and return auth response
    pub async fn login_user(&self, username: &str, password: &str) -> serde_json::Value {
        let login_request = serde_json::json!({
            "username": username,
            "password": password
        });

        self.server
            .post("/api/v1/auth/login")
            .json(&login_request)
            .await
            .json::<serde_json::Value>()
    }

    /// Get authorization header for a user
    pub async fn auth_header_for_user(&self, username: &str, password: &str) -> String {
        let response = self.login_user(username, password).await;
        let access_token = response["tokens"]["access_token"]
            .as_str()
            .expect("Failed to get access token");
        format!("Bearer {}", access_token)
    }

    /// Clear all data from the database
    pub async fn clear_database(&self) {
        use sea_orm::{EntityTrait, QuerySelect};

        // Delete all users (cascades to documents and projects)
        User::delete_many()
            .exec(&self.app_state.db)
            .await
            .expect("Failed to clear users");
    }
}

use writemagic_web::config::{AuthConfig, Config, CorsConfig, DatabaseConfig, ServerConfig};

/// Test configuration extensions
impl Config {
    /// Create a test configuration with overrides
    pub fn test_config(database_url: &str) -> Self {
        let mut config = Self::test_default();
        config.database.url = database_url.to_string();
        config
    }
}

/// Assertion helpers for testing
pub mod assertions {
    use axum_test::TestResponse;
    use serde_json::Value;

    /// Assert that response is successful (2xx status code)
    pub fn assert_success(response: &TestResponse) {
        assert!(
            response.status_code().is_success(),
            "Expected successful response, got: {} - {}",
            response.status_code(),
            response.text()
        );
    }

    /// Assert that response has specific status code
    pub fn assert_status(response: &TestResponse, expected_status: u16) {
        assert_eq!(
            response.status_code().as_u16(),
            expected_status,
            "Expected status {}, got: {} - {}",
            expected_status,
            response.status_code(),
            response.text()
        );
    }

    /// Assert that response contains specific JSON field
    pub fn assert_json_field(response: &TestResponse, field: &str) {
        let json: Value = response.json();
        assert!(
            json.get(field).is_some(),
            "Expected field '{}' in response: {}",
            field,
            json
        );
    }

    /// Assert that response JSON field has specific value
    pub fn assert_json_field_eq(response: &TestResponse, field: &str, expected: &Value) {
        let json: Value = response.json();
        let actual = json.get(field).unwrap_or(&Value::Null);
        assert_eq!(
            actual, expected,
            "Expected field '{}' to be {}, got: {}",
            field, expected, actual
        );
    }

    /// Assert that response has authentication error
    pub fn assert_auth_error(response: &TestResponse) {
        assert_eq!(response.status_code().as_u16(), 401);
        let json: Value = response.json();
        assert_eq!(json["error"], "authentication_error");
    }

    /// Assert that response has validation error
    pub fn assert_validation_error(response: &TestResponse) {
        assert_eq!(response.status_code().as_u16(), 422);
        let json: Value = response.json();
        assert_eq!(json["error"], "VALIDATION_ERROR");
    }

    /// Assert that response has rate limit error
    pub fn assert_rate_limit_error(response: &TestResponse) {
        assert_eq!(response.status_code().as_u16(), 429);
        let json: Value = response.json();
        assert_eq!(json["error"], "rate_limit_exceeded");
    }
}