use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::{AppError, Result as AppResult};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // Subject (user ID)
    pub username: String,  // Username for convenience
    pub exp: usize,        // Expiration time
    pub iat: usize,        // Issued at
    pub jti: String,       // JWT ID for revocation
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT token pair
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize, // Access token expiration in seconds
}

/// JWT Keys for encoding/decoding tokens
#[derive(Clone)]
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    /// Create new JWT keys from a secret
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    /// Generate a secure random secret (for development/testing)
    pub fn generate() -> (Self, Vec<u8>) {
        use rand::Rng;
        let secret: Vec<u8> = (0..64).map(|_| rand::thread_rng().gen::<u8>()).collect();
        let keys = Self::new(&secret);
        (keys, secret)
    }
}

/// Password hashing utilities using Argon2
pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Authentication(format!("Failed to hash password: {}", e)))?;
        
        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Authentication(format!("Invalid password hash: {}", e)))?;
        
        let argon2 = Argon2::default();
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// JWT token management
pub struct TokenManager;

impl TokenManager {
    /// Generate a token pair (access + refresh tokens)
    pub fn generate_token_pair(
        keys: &JwtKeys,
        user_id: &str,
        username: &str,
    ) -> AppResult<TokenPair> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        // Access token (15 minutes)
        let access_exp = now + (15 * 60);
        let access_claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: access_exp,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        // Refresh token (7 days)
        let refresh_exp = now + (7 * 24 * 60 * 60);
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: refresh_exp,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        let access_token = encode(&Header::default(), &access_claims, &keys.encoding)
            .map_err(|e| AppError::Authentication(format!("Failed to create access token: {}", e)))?;

        let refresh_token = encode(&Header::default(), &refresh_claims, &keys.encoding)
            .map_err(|e| AppError::Authentication(format!("Failed to create refresh token: {}", e)))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: 15 * 60, // 15 minutes in seconds
        })
    }

    /// Validate and decode a JWT token
    pub fn validate_token(keys: &JwtKeys, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = false;

        let token_data = decode::<Claims>(token, &keys.decoding, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Authentication("Token has expired".to_string())
                }
                _ => AppError::Authentication(format!("Invalid token: {}", e)),
            })?;

        Ok(token_data.claims)
    }

    /// Refresh an access token using a valid refresh token
    pub fn refresh_token(
        keys: &JwtKeys,
        refresh_token: &str,
    ) -> AppResult<TokenPair> {
        let claims = Self::validate_token(keys, refresh_token)?;
        
        // Ensure this is actually a refresh token
        if claims.token_type != TokenType::Refresh {
            return Err(AppError::Authentication("Invalid token type for refresh".to_string()));
        }

        // Generate new token pair
        Self::generate_token_pair(keys, &claims.sub, &claims.username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing() {
        let password = "test_password_123";
        
        // Hash password
        let hash = PasswordHasher::hash_password(password).unwrap();
        assert!(!hash.is_empty());
        
        // Verify correct password
        let is_valid = PasswordHasher::verify_password(password, &hash).unwrap();
        assert!(is_valid);
        
        // Verify incorrect password
        let is_invalid = PasswordHasher::verify_password("wrong_password", &hash).unwrap();
        assert!(!is_invalid);
    }

    #[tokio::test]
    async fn test_jwt_token_generation() {
        let (keys, _secret) = JwtKeys::generate();
        let user_id = "test_user_123";
        let username = "testuser";
        
        // Generate token pair
        let token_pair = TokenManager::generate_token_pair(&keys, user_id, username).unwrap();
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.expires_in, 15 * 60);
        
        // Validate access token
        let access_claims = TokenManager::validate_token(&keys, &token_pair.access_token).unwrap();
        assert_eq!(access_claims.sub, user_id);
        assert_eq!(access_claims.username, username);
        assert_eq!(access_claims.token_type, TokenType::Access);
        
        // Validate refresh token
        let refresh_claims = TokenManager::validate_token(&keys, &token_pair.refresh_token).unwrap();
        assert_eq!(refresh_claims.sub, user_id);
        assert_eq!(refresh_claims.username, username);
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let (keys, _secret) = JwtKeys::generate();
        let user_id = "test_user_123";
        let username = "testuser";
        
        // Generate initial token pair
        let initial_tokens = TokenManager::generate_token_pair(&keys, user_id, username).unwrap();
        
        // Refresh using refresh token
        let new_tokens = TokenManager::refresh_token(&keys, &initial_tokens.refresh_token).unwrap();
        assert!(!new_tokens.access_token.is_empty());
        assert!(!new_tokens.refresh_token.is_empty());
        
        // Tokens should be different
        assert_ne!(initial_tokens.access_token, new_tokens.access_token);
        assert_ne!(initial_tokens.refresh_token, new_tokens.refresh_token);
        
        // Should fail with access token
        let result = TokenManager::refresh_token(&keys, &initial_tokens.access_token);
        assert!(result.is_err());
    }
}