// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::error::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
    nbf: i64,
    role: String,
    jti: String,
    iss: String,
    aud: String,
}

impl Claims {
    pub fn sub(&self) -> &str {
        &self.sub
    }
    pub fn role(&self) -> &str {
        &self.role
    }
    pub fn jti(&self) -> &str {
        &self.jti
    }

    pub fn anonymous(role: &str) -> Self {
        let now = Utc::now();
        Self {
            sub: "anonymous".to_string(),
            exp: (now + Duration::hours(24)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            role: role.to_string(),
            jti: uuid::Uuid::new_v4().to_string(),
            iss: "tetramem-v12".to_string(),
            aud: "tetramem-api".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct JwtConfig {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    header: Header,
    expiry_secs: u64,
}

impl std::fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field("expiry_secs", &self.expiry_secs)
            .finish_non_exhaustive()
    }
}

impl JwtConfig {
    pub fn new(secret: String, expiry_secs: u64) -> Self {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "iat", "nbf", "sub", "jti", "iss", "aud"]);
        validation.leeway = 60;
        validation.set_issuer(&["tetramem-v12"]);
        validation.set_audience(&["tetramem-api"]);
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            header: Header {
                kid: Some("tetramem-v12".to_string()),
                ..Default::default()
            },
            validation,
            expiry_secs,
        }
    }

    pub fn create_token(&self, subject: &str, role: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            sub: subject.to_string(),
            exp: (now + Duration::seconds(self.expiry_secs as i64)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            role: role.to_string(),
            jti: uuid::Uuid::new_v4().to_string(),
            iss: "tetramem-v12".to_string(),
            aud: "tetramem-api".to_string(),
        };
        encode(&self.header, &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("jwt encode: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| AppError::Unauthorized(format!("invalid token: {}", e)))?;
        Ok(data.claims)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("argon2 hash: {}", e)))?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    #[serde(default)]
    pub password_hash: String,
    #[serde(default, skip_serializing)]
    pub password: String,
    #[serde(default = "default_role")]
    pub role: String,
}

impl std::fmt::Debug for UserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash_preview = if self.password_hash.is_empty() {
            "(empty)"
        } else {
            "(set)"
        };
        let pw_status = if self.password.is_empty() {
            "(empty)"
        } else {
            "(provided)"
        };
        f.debug_struct("UserConfig")
            .field("username", &self.username)
            .field("password_hash", &hash_preview)
            .field("password", &pw_status)
            .field("role", &self.role)
            .finish()
    }
}

fn default_role() -> String {
    "user".to_string()
}

#[derive(Clone)]
struct StoredUser {
    username: String,
    password_hash: String,
    role: String,
}

impl std::fmt::Debug for StoredUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredUser")
            .field("username", &self.username)
            .field("password_hash", &"(redacted)")
            .field("role", &self.role)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct UserStore {
    users: Vec<StoredUser>,
}

impl UserStore {
    pub fn new(configs: &[UserConfig], _jwt_secret: &str) -> Self {
        let users: Vec<StoredUser> = configs
            .iter()
            .map(|c| {
                let hash = if !c.password_hash.is_empty() {
                    c.password_hash.clone()
                } else if !c.password.is_empty() {
                    hash_password(&c.password).unwrap_or_else(|e| {
                        tracing::error!("failed to hash password for user '{}': {}", c.username, e);
                        String::new()
                    })
                } else {
                    String::new()
                };
                StoredUser {
                    username: c.username.clone(),
                    password_hash: hash,
                    role: c.role.clone(),
                }
            })
            .collect();
        Self { users }
    }

    pub fn verify(&self, username: &str, password: &str) -> Option<&str> {
        let user = self.users.iter().find(|u| u.username == username);
        let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAA$IrIHnE+KcLW7CRfv02DDMj/53fjTmUqsDVOHeibmAGs";
        let hash = user.map(|u| u.password_hash.as_str()).unwrap_or(dummy_hash);
        let valid = verify_password(password, hash);
        if let Some(u) = user {
            if valid {
                return Some(u.role.as_str());
            }
        }
        None
    }

    pub fn has_users(&self) -> bool {
        !self.users.is_empty()
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

pub struct TokenBlocklist {
    revoked: std::collections::HashSet<String>,
    order: std::collections::VecDeque<String>,
    max_size: usize,
}

impl TokenBlocklist {
    pub fn new(max_size: usize) -> Self {
        Self {
            revoked: std::collections::HashSet::new(),
            order: std::collections::VecDeque::new(),
            max_size,
        }
    }

    pub fn revoke(&mut self, jti: &str) {
        if self.revoked.contains(jti) {
            return;
        }
        if self.revoked.len() >= self.max_size {
            if let Some(oldest) = self.order.pop_front() {
                self.revoked.remove(&oldest);
            }
        }
        self.revoked.insert(jti.to_string());
        self.order.push_back(jti.to_string());
    }

    pub fn is_revoked(&self, jti: &str) -> bool {
        self.revoked.contains(jti)
    }

    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }
}

impl std::fmt::Debug for TokenBlocklist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenBlocklist")
            .field("count", &self.revoked.len())
            .field("max_size", &self.max_size)
            .finish()
    }
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
        assert_eq!(claims.sub(), "user1");
        assert_eq!(claims.role(), "admin");
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

    #[test]
    fn password_hashing_and_verify() {
        let h1 = hash_password("mypassword").unwrap();
        assert!(verify_password("mypassword", &h1));
        assert!(!verify_password("wrongpassword", &h1));
    }

    #[test]
    fn password_hash_unique_per_call() {
        let h1 = hash_password("mypassword").unwrap();
        let h2 = hash_password("mypassword").unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn user_store_verify_correct() {
        let store = UserStore::new(
            &[UserConfig {
                username: "admin".to_string(),
                password_hash: String::new(),
                password: "secret123".to_string(),
                role: "admin".to_string(),
            }],
            "jwt-secret",
        );
        assert_eq!(store.verify("admin", "secret123"), Some("admin"));
    }

    #[test]
    fn user_store_verify_wrong_password() {
        let store = UserStore::new(
            &[UserConfig {
                username: "admin".to_string(),
                password_hash: String::new(),
                password: "secret123".to_string(),
                role: "admin".to_string(),
            }],
            "jwt-secret",
        );
        assert_eq!(store.verify("admin", "wrong"), None);
    }

    #[test]
    fn user_store_verify_unknown_user() {
        let store = UserStore::new(
            &[UserConfig {
                username: "admin".to_string(),
                password_hash: String::new(),
                password: "secret123".to_string(),
                role: "admin".to_string(),
            }],
            "jwt-secret",
        );
        assert_eq!(store.verify("unknown", "secret123"), None);
    }

    #[test]
    fn user_store_prehashed_password() {
        let prehashed = hash_password("mypassword").unwrap();
        let store = UserStore::new(
            &[UserConfig {
                username: "admin".to_string(),
                password_hash: prehashed,
                password: String::new(),
                role: "admin".to_string(),
            }],
            "jwt-secret",
        );
        assert_eq!(store.verify("admin", "mypassword"), Some("admin"));
    }
}
