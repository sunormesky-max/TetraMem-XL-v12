use crate::universe::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    secret: String,
    expiry_secs: u64,
}

impl JwtConfig {
    pub fn new(secret: String, expiry_secs: u64) -> Self {
        Self { secret, expiry_secs }
    }

    pub fn create_token(&self, subject: &str, role: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            sub: subject.to_string(),
            exp: (now + Duration::seconds(self.expiry_secs as i64)).timestamp(),
            iat: now.timestamp(),
            role: role.to_string(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("jwt encode: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("invalid token: {}", e)))?;
        Ok(data.claims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig::new("test-secret".to_string(), 3600)
    }

    #[test]
    fn create_and_validate_token() {
        let config = test_config();
        let token = config.create_token("user1", "admin").unwrap();
        let claims = config.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user1");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn invalid_token_rejected() {
        let config = test_config();
        let result = config.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn wrong_secret_rejected() {
        let config1 = JwtConfig::new("secret1".to_string(), 3600);
        let config2 = JwtConfig::new("secret2".to_string(), 3600);
        let token = config1.create_token("user1", "admin").unwrap();
        let result = config2.validate_token(&token);
        assert!(result.is_err());
    }
}
