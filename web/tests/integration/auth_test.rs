use serde_json::json;

use crate::common::{assertions::*, TestApp};

#[tokio::test]
async fn test_user_registration_success() {
    let app = TestApp::new().await;

    let register_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let response = app.server
        .post("/api/v1/auth/register")
        .json(&register_request)
        .await;

    assert_success(&response);
    assert_json_field(&response, "user");
    assert_json_field(&response, "tokens");
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["user"]["username"], "testuser");
    assert_eq!(json_response["user"]["email"], "test@example.com");
    assert!(json_response["tokens"]["access_token"].is_string());
    assert!(json_response["tokens"]["refresh_token"].is_string());
}

#[tokio::test]
async fn test_user_registration_duplicate_username() {
    let app = TestApp::new().await;

    // Create first user
    app.create_test_user("testuser", "test1@example.com", "password123").await;

    // Try to register with same username
    let register_request = json!({
        "username": "testuser",
        "email": "test2@example.com",
        "password": "password123"
    });

    let response = app.server
        .post("/api/v1/auth/register")
        .json(&register_request)
        .await;

    assert_status(&response, 409);
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["message"], "Username already exists");
}

#[tokio::test]
async fn test_user_registration_duplicate_email() {
    let app = TestApp::new().await;

    // Create first user
    app.create_test_user("testuser1", "test@example.com", "password123").await;

    // Try to register with same email
    let register_request = json!({
        "username": "testuser2",
        "email": "test@example.com",
        "password": "password123"
    });

    let response = app.server
        .post("/api/v1/auth/register")
        .json(&register_request)
        .await;

    assert_status(&response, 409);
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["message"], "Email already exists");
}

#[tokio::test]
async fn test_user_registration_validation_errors() {
    let app = TestApp::new().await;

    // Test short username
    let register_request = json!({
        "username": "ab",
        "email": "test@example.com",
        "password": "password123"
    });

    let response = app.server
        .post("/api/v1/auth/register")
        .json(&register_request)
        .await;

    assert_validation_error(&response);

    // Test invalid email
    let register_request = json!({
        "username": "testuser",
        "email": "invalid-email",
        "password": "password123"
    });

    let response = app.server
        .post("/api/v1/auth/register")
        .json(&register_request)
        .await;

    assert_validation_error(&response);

    // Test short password
    let register_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "short"
    });

    let response = app.server
        .post("/api/v1/auth/register")
        .json(&register_request)
        .await;

    assert_validation_error(&response);
}

#[tokio::test]
async fn test_user_login_success() {
    let app = TestApp::new().await;
    let username = "testuser";
    let password = "password123";

    // Create user directly in database
    app.create_test_user(username, "test@example.com", password).await;

    let login_request = json!({
        "username": username,
        "password": password
    });

    let response = app.server
        .post("/api/v1/auth/login")
        .json(&login_request)
        .await;

    assert_success(&response);
    assert_json_field(&response, "user");
    assert_json_field(&response, "tokens");
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["user"]["username"], username);
    assert!(json_response["tokens"]["access_token"].is_string());
    assert!(json_response["tokens"]["refresh_token"].is_string());
}

#[tokio::test]
async fn test_user_login_invalid_credentials() {
    let app = TestApp::new().await;

    // Create user
    app.create_test_user("testuser", "test@example.com", "correct_password").await;

    // Try login with wrong password
    let login_request = json!({
        "username": "testuser",
        "password": "wrong_password"
    });

    let response = app.server
        .post("/api/v1/auth/login")
        .json(&login_request)
        .await;

    assert_auth_error(&response);

    // Try login with non-existent user
    let login_request = json!({
        "username": "nonexistent",
        "password": "password123"
    });

    let response = app.server
        .post("/api/v1/auth/login")
        .json(&login_request)
        .await;

    assert_auth_error(&response);
}

#[tokio::test]
async fn test_token_refresh() {
    let app = TestApp::new().await;

    // Register user and get tokens
    let auth_response = app.register_user("testuser", "test@example.com", "password123").await;
    let refresh_token = auth_response["tokens"]["refresh_token"]
        .as_str()
        .expect("Failed to get refresh token");

    let refresh_request = json!({
        "refresh_token": refresh_token
    });

    let response = app.server
        .post("/api/v1/auth/refresh")
        .json(&refresh_request)
        .await;

    assert_success(&response);
    assert_json_field(&response, "access_token");
    assert_json_field(&response, "refresh_token");
    
    let json_response: serde_json::Value = response.json();
    assert!(json_response["access_token"].is_string());
    assert!(json_response["refresh_token"].is_string());
}

#[tokio::test]
async fn test_get_profile() {
    let app = TestApp::new().await;

    // Create and login user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    let response = app.server
        .get("/api/v1/auth/profile")
        .add_header("Authorization", &auth_header)
        .await;

    assert_success(&response);
    assert_json_field(&response, "id");
    assert_json_field(&response, "username");
    assert_json_field(&response, "email");
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["username"], "testuser");
    assert_eq!(json_response["email"], "test@example.com");
}

#[tokio::test]
async fn test_get_profile_unauthorized() {
    let app = TestApp::new().await;

    // Try to get profile without authentication
    let response = app.server
        .get("/api/v1/auth/profile")
        .await;

    assert_auth_error(&response);

    // Try with invalid token
    let response = app.server
        .get("/api/v1/auth/profile")
        .add_header("Authorization", "Bearer invalid_token")
        .await;

    assert_auth_error(&response);
}

#[tokio::test]
async fn test_logout() {
    let app = TestApp::new().await;

    // Create and login user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    let response = app.server
        .post("/api/v1/auth/logout")
        .add_header("Authorization", &auth_header)
        .await;

    assert_status(&response, 204);
}

#[tokio::test]
async fn test_admin_user_creation() {
    let app = TestApp::new().await;

    // Create admin user
    let admin_user = app.create_admin_user("admin", "admin@example.com", "admin_password").await;
    
    assert_eq!(admin_user.username, "admin");
    assert_eq!(admin_user.role, "admin");
    assert!(admin_user.is_admin());
}

#[tokio::test]
async fn test_concurrent_user_registration() {
    let app = TestApp::new().await;

    // Create multiple registration requests concurrently
    let tasks = (0..10).map(|i| {
        let app = app.server.clone();
        tokio::spawn(async move {
            let register_request = json!({
                "username": format!("user{}", i),
                "email": format!("user{}@example.com", i),
                "password": "password123"
            });

            app.post("/api/v1/auth/register")
                .json(&register_request)
                .await
        })
    });

    let results = futures::future::join_all(tasks).await;

    // All registrations should succeed
    for result in results {
        let response = result.expect("Task should complete");
        assert_success(&response);
    }
}

#[tokio::test]
async fn test_password_hashing_security() {
    let app = TestApp::new().await;

    // Create user with password
    let user = app.create_test_user("testuser", "test@example.com", "password123").await;
    
    // Password hash should not be the plain password
    assert_ne!(user.password_hash, "password123");
    
    // Password hash should be long enough (Argon2 produces long hashes)
    assert!(user.password_hash.len() > 50);
    
    // Hash should start with Argon2 identifier
    assert!(user.password_hash.starts_with("$argon2"));
}