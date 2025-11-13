//! CEX authentication mechanisms for the sniper bot.
//!
//! This module provides functionality for authenticating with centralized exchanges
//! using various authentication methods including API keys, HMAC signatures, and OAuth.

use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication method types
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// API key authentication
    ApiKey { api_key: String, api_secret: String },
    /// HMAC signature authentication
    Hmac {
        api_key: String,
        api_secret: String,
        algorithm: HmacAlgorithm,
    },
    /// OAuth 2.0 authentication
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<u64>,
    },
    /// JWT token authentication
    Jwt {
        token: String,
        expires_at: Option<u64>,
    },
}

/// HMAC algorithm types
#[derive(Debug, Clone)]
pub enum HmacAlgorithm {
    Sha256,
    Sha512,
}

/// Authentication credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Authentication method
    pub method: AuthMethod,
    /// Additional headers to include in requests
    pub headers: HashMap<String, String>,
    /// Query parameters to include in requests
    pub query_params: HashMap<String, String>,
}

impl Credentials {
    /// Create new API key credentials
    pub fn new_api_key(api_key: String, api_secret: String) -> Self {
        Self {
            method: AuthMethod::ApiKey {
                api_key,
                api_secret,
            },
            headers: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    /// Create new HMAC credentials
    pub fn new_hmac(api_key: String, api_secret: String, algorithm: HmacAlgorithm) -> Self {
        Self {
            method: AuthMethod::Hmac {
                api_key,
                api_secret,
                algorithm,
            },
            headers: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    /// Create new OAuth credentials
    pub fn new_oauth(
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<u64>,
    ) -> Self {
        Self {
            method: AuthMethod::OAuth {
                access_token,
                refresh_token,
                expires_at,
            },
            headers: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    /// Create new JWT credentials
    pub fn new_jwt(token: String, expires_at: Option<u64>) -> Self {
        Self {
            method: AuthMethod::Jwt { token, expires_at },
            headers: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    /// Add a header to the credentials
    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    /// Add a query parameter to the credentials
    pub fn add_query_param(&mut self, key: String, value: String) {
        self.query_params.insert(key, value);
    }

    /// Generate HMAC signature
    pub fn generate_hmac_signature(&self, message: &str) -> Result<String> {
        match &self.method {
            AuthMethod::Hmac {
                api_secret,
                algorithm,
                ..
            } => {
                let signature = match algorithm {
                    HmacAlgorithm::Sha256 => {
                        let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
                            .map_err(|e| anyhow::anyhow!("Invalid HMAC key: {}", e))?;
                        mac.update(message.as_bytes());
                        hex::encode(mac.finalize().into_bytes())
                    }
                    HmacAlgorithm::Sha512 => {
                        let mut mac = Hmac::<sha2::Sha512>::new_from_slice(api_secret.as_bytes())
                            .map_err(|e| anyhow::anyhow!("Invalid HMAC key: {}", e))?;
                        mac.update(message.as_bytes());
                        hex::encode(mac.finalize().into_bytes())
                    }
                };
                Ok(signature)
            }
            _ => Err(anyhow::anyhow!(
                "HMAC signature not applicable for this auth method"
            )),
        }
    }

    /// Get current timestamp in milliseconds
    pub fn get_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Get current timestamp in seconds
    pub fn get_timestamp_sec() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Check if credentials are expired
    pub fn is_expired(&self) -> bool {
        match &self.method {
            AuthMethod::OAuth { expires_at, .. } => {
                if let Some(expiry) = expires_at {
                    Self::get_timestamp_sec() >= *expiry
                } else {
                    false
                }
            }
            AuthMethod::Jwt { expires_at, .. } => {
                if let Some(expiry) = expires_at {
                    Self::get_timestamp_sec() >= *expiry
                } else {
                    false
                }
            }
            _ => false, // API key and HMAC don't expire
        }
    }

    /// Get authorization header value
    pub fn get_auth_header(&self) -> Result<Option<String>> {
        match &self.method {
            AuthMethod::ApiKey { api_key, .. } => Ok(Some(format!("Bearer {}", api_key))),
            AuthMethod::Hmac { api_key, .. } => Ok(Some(format!("Bearer {}", api_key))),
            AuthMethod::OAuth { access_token, .. } => Ok(Some(format!("Bearer {}", access_token))),
            AuthMethod::Jwt { token, .. } => Ok(Some(format!("Bearer {}", token))),
        }
    }

    /// Refresh OAuth token (placeholder implementation)
    pub async fn refresh_oauth_token(&mut self) -> Result<()> {
        match &mut self.method {
            AuthMethod::OAuth {
                refresh_token: Some(_refresh_token),
                access_token,
                expires_at,
            } => {
                // In a real implementation, this would make a request to the exchange's token endpoint
                // For now, we'll simulate a refresh
                *access_token = format!("refreshed_{}", access_token);
                *expires_at = Some(Self::get_timestamp_sec() + 3600); // 1 hour from now
                Ok(())
            }
            AuthMethod::OAuth {
                refresh_token: None,
                ..
            } => Err(anyhow::anyhow!("No refresh token available")),
            _ => Err(anyhow::anyhow!("Not an OAuth authentication method")),
        }
    }
}

/// Authentication manager for handling multiple exchange credentials
#[derive(Debug, Default)]
pub struct AuthManager {
    /// Credentials for different exchanges
    credentials: HashMap<String, Credentials>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }

    /// Add credentials for an exchange
    pub fn add_credentials(&mut self, exchange_id: String, credentials: Credentials) {
        self.credentials.insert(exchange_id, credentials);
    }

    /// Get credentials for an exchange
    pub fn get_credentials(&self, exchange_id: &str) -> Option<&Credentials> {
        self.credentials.get(exchange_id)
    }

    /// Remove credentials for an exchange
    pub fn remove_credentials(&mut self, exchange_id: &str) -> bool {
        self.credentials.remove(exchange_id).is_some()
    }

    /// List all exchange IDs with credentials
    pub fn list_exchanges(&self) -> Vec<&String> {
        self.credentials.keys().collect()
    }

    /// Check if credentials for an exchange are expired
    pub fn is_expired(&self, exchange_id: &str) -> bool {
        if let Some(credentials) = self.get_credentials(exchange_id) {
            credentials.is_expired()
        } else {
            true // If we don't have credentials, treat as expired
        }
    }

    /// Refresh credentials for an exchange (if applicable)
    pub async fn refresh_credentials(&mut self, exchange_id: &str) -> Result<()> {
        if let Some(credentials) = self.credentials.get_mut(exchange_id) {
            match &credentials.method {
                AuthMethod::OAuth { .. } => credentials.refresh_oauth_token().await,
                _ => Ok(()), // Other auth methods don't need refreshing
            }
        } else {
            Err(anyhow::anyhow!(
                "No credentials found for exchange: {}",
                exchange_id
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_credentials() {
        let credentials =
            Credentials::new_api_key("test_key".to_string(), "test_secret".to_string());

        match &credentials.method {
            AuthMethod::ApiKey {
                api_key,
                api_secret,
            } => {
                assert_eq!(api_key, "test_key");
                assert_eq!(api_secret, "test_secret");
            }
            _ => panic!("Expected ApiKey auth method"),
        }

        assert!(credentials.headers.is_empty());
        assert!(credentials.query_params.is_empty());
    }

    #[test]
    fn test_hmac_credentials() {
        let credentials = Credentials::new_hmac(
            "test_key".to_string(),
            "test_secret".to_string(),
            HmacAlgorithm::Sha256,
        );

        match &credentials.method {
            AuthMethod::Hmac {
                api_key,
                api_secret,
                algorithm,
            } => {
                assert_eq!(api_key, "test_key");
                assert_eq!(api_secret, "test_secret");
                match algorithm {
                    HmacAlgorithm::Sha256 => {} // Expected match
                    HmacAlgorithm::Sha512 => panic!("Expected Sha256"),
                }
            }
            _ => panic!("Expected Hmac auth method"),
        }
    }

    #[test]
    fn test_oauth_credentials() {
        let credentials = Credentials::new_oauth(
            "access_token".to_string(),
            Some("refresh_token".to_string()),
            Some(1234567890),
        );

        match &credentials.method {
            AuthMethod::OAuth {
                access_token,
                refresh_token,
                expires_at,
            } => {
                assert_eq!(access_token, "access_token");
                assert_eq!(refresh_token, &Some("refresh_token".to_string()));
                assert_eq!(expires_at, &Some(1234567890));
            }
            _ => panic!("Expected OAuth auth method"),
        }
    }

    #[test]
    fn test_jwt_credentials() {
        let credentials = Credentials::new_jwt("jwt_token".to_string(), Some(1234567890));

        match &credentials.method {
            AuthMethod::Jwt { token, expires_at } => {
                assert_eq!(token, "jwt_token");
                assert_eq!(expires_at, &Some(1234567890));
            }
            _ => panic!("Expected Jwt auth method"),
        }
    }

    #[test]
    fn test_add_header_and_query_param() {
        let mut credentials =
            Credentials::new_api_key("test_key".to_string(), "test_secret".to_string());
        credentials.add_header("X-Custom-Header".to_string(), "custom_value".to_string());
        credentials.add_query_param("param1".to_string(), "value1".to_string());

        assert_eq!(
            credentials.headers.get("X-Custom-Header"),
            Some(&"custom_value".to_string())
        );
        assert_eq!(
            credentials.query_params.get("param1"),
            Some(&"value1".to_string())
        );
    }

    #[test]
    fn test_timestamp_functions() {
        let timestamp_ms = Credentials::get_timestamp_ms();
        let timestamp_sec = Credentials::get_timestamp_sec();

        // Check that timestamps are reasonable (not zero)
        assert!(timestamp_ms > 0);
        assert!(timestamp_sec > 0);

        // Check that millisecond timestamp is larger than second timestamp
        assert!(timestamp_ms >= timestamp_sec * 1000);
    }

    #[test]
    fn test_expiration_check() {
        // Test non-expiring credentials
        let api_key_credentials =
            Credentials::new_api_key("test_key".to_string(), "test_secret".to_string());
        assert!(!api_key_credentials.is_expired());

        let hmac_credentials = Credentials::new_hmac(
            "test_key".to_string(),
            "test_secret".to_string(),
            HmacAlgorithm::Sha256,
        );
        assert!(!hmac_credentials.is_expired());

        // Test expired JWT
        let expired_jwt_credentials =
            Credentials::new_jwt("jwt_token".to_string(), Some(1234567890));
        assert!(expired_jwt_credentials.is_expired());

        // Test expired OAuth
        let expired_oauth_credentials = Credentials::new_oauth(
            "access_token".to_string(),
            Some("refresh_token".to_string()),
            Some(1234567890),
        );
        assert!(expired_oauth_credentials.is_expired());
    }

    #[test]
    fn test_auth_header() {
        let api_key_credentials =
            Credentials::new_api_key("test_key".to_string(), "test_secret".to_string());
        let header = api_key_credentials.get_auth_header().unwrap();
        assert_eq!(header, Some("Bearer test_key".to_string()));

        let oauth_credentials = Credentials::new_oauth("access_token".to_string(), None, None);
        let header = oauth_credentials.get_auth_header().unwrap();
        assert_eq!(header, Some("Bearer access_token".to_string()));

        let jwt_credentials = Credentials::new_jwt("jwt_token".to_string(), None);
        let header = jwt_credentials.get_auth_header().unwrap();
        assert_eq!(header, Some("Bearer jwt_token".to_string()));
    }

    #[test]
    fn test_auth_manager() {
        let mut auth_manager = AuthManager::new();

        // Test adding credentials
        let credentials =
            Credentials::new_api_key("test_key".to_string(), "test_secret".to_string());
        auth_manager.add_credentials("binance".to_string(), credentials);

        // Test getting credentials
        assert!(auth_manager.get_credentials("binance").is_some());
        assert!(auth_manager.get_credentials("kucoin").is_none());

        // Test listing exchanges
        let exchanges = auth_manager.list_exchanges();
        assert_eq!(exchanges.len(), 1);
        assert_eq!(exchanges[0], "binance");

        // Test removing credentials
        assert!(auth_manager.remove_credentials("binance"));
        assert!(!auth_manager.remove_credentials("binance")); // Already removed
        assert!(auth_manager.get_credentials("binance").is_none());
    }

    #[tokio::test]
    async fn test_hmac_signature() {
        let credentials = Credentials::new_hmac(
            "test_key".to_string(),
            "test_secret".to_string(),
            HmacAlgorithm::Sha256,
        );

        let signature = credentials.generate_hmac_signature("test_message").unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }

    #[tokio::test]
    async fn test_hmac_signature_sha512() {
        let credentials = Credentials::new_hmac(
            "test_key".to_string(),
            "test_secret".to_string(),
            HmacAlgorithm::Sha512,
        );

        let signature = credentials.generate_hmac_signature("test_message").unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 128); // SHA512 produces 64 bytes = 128 hex chars
    }

    #[tokio::test]
    async fn test_invalid_hmac_signature() {
        let credentials =
            Credentials::new_api_key("test_key".to_string(), "test_secret".to_string());
        let result = credentials.generate_hmac_signature("test_message");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "HMAC signature not applicable for this auth method"
        );
    }
}
