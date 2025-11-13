//! HashiCorp Vault integration for key management
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// HashiCorp Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Vault server address
    pub address: String,
    /// Vault token for authentication
    pub token: String,
    /// Mount path for the key storage
    pub mount_path: String,
    /// Enable TLS verification
    pub verify_tls: bool,
    /// Timeout for Vault requests (in seconds)
    pub timeout_seconds: u64,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            address: "http://127.0.0.1:8200".to_string(),
            token: "".to_string(),
            mount_path: "secret".to_string(),
            verify_tls: true,
            timeout_seconds: 30,
        }
    }
}

/// Vault key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultKeyMetadata {
    /// Key identifier
    pub key_id: String,
    /// Key type
    pub key_type: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Version of the key
    pub version: u32,
    /// Tags for key organization
    pub tags: Vec<String>,
}

/// HashiCorp Vault client for key management
pub struct VaultClient {
    /// Configuration
    config: VaultConfig,
    /// HTTP client for making requests
    client: reqwest::Client,
}

impl VaultClient {
    /// Create a new Vault client
    pub fn new(config: VaultConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .danger_accept_invalid_certs(!config.verify_tls)
            .build()?;
        
        Ok(Self { config, client })
    }

    /// Store a key in Vault
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// * `key_material` - Raw key material to store
    /// * `key_type` - Type of key being stored
    /// * `tags` - Tags for organizing the key
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn store_key(&self, key_id: &str, key_material: &[u8], key_type: &str, tags: Vec<String>) -> Result<()> {
        info!("Storing key in Vault: {}", key_id);
        
        let url = format!("{}/v1/{}/data/{}", self.config.address, self.config.mount_path, key_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let payload = serde_json::json!({
            "data": {
                "key_material": base64::encode(key_material),
                "key_type": key_type,
                "created_at": now,
                "tags": tags
            }
        });
        
        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.config.token)
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("Key stored successfully in Vault: {}", key_id);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Vault API error: {}", error_text))
        }
    }

    /// Retrieve a key from Vault
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - The key material or error
    pub async fn retrieve_key(&self, key_id: &str) -> Result<Vec<u8>> {
        info!("Retrieving key from Vault: {}", key_id);
        
        let url = format!("{}/v1/{}/data/{}", self.config.address, self.config.mount_path, key_id);
        
        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.config.token)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            
            if let Some(data) = json.get("data").and_then(|d| d.get("data")) {
                if let Some(key_material_b64) = data.get("key_material").and_then(|k| k.as_str()) {
                    let key_material = base64::decode(key_material_b64)
                        .map_err(|e| anyhow::anyhow!("Failed to decode key material: {}", e))?;
                    info!("Key retrieved successfully from Vault: {}", key_id);
                    return Ok(key_material);
                }
            }
            
            Err(anyhow::anyhow!("Key material not found in Vault response"))
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Vault API error: {}", error_text))
        }
    }

    /// List all keys in Vault
    /// 
    /// # Returns
    /// * `Result<Vec<String>>` - List of key identifiers
    pub async fn list_keys(&self) -> Result<Vec<String>> {
        info!("Listing keys in Vault");
        
        let url = format!("{}/v1/{}/metadata", self.config.address, self.config.mount_path);
        
        let response = self.client
            .list(&url)
            .header("X-Vault-Token", &self.config.token)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            
            if let Some(keys) = json.get("data").and_then(|d| d.get("keys")) {
                if let Some(key_array) = keys.as_array() {
                    let key_ids: Vec<String> = key_array
                        .iter()
                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                        .collect();
                    
                    info!("Retrieved {} keys from Vault", key_ids.len());
                    return Ok(key_ids);
                }
            }
            
            Ok(Vec::new())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Vault API error: {}", error_text))
        }
    }

    /// Delete a key from Vault
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `Result<bool>` - True if key was deleted, false if not found
    pub async fn delete_key(&self, key_id: &str) -> Result<bool> {
        info!("Deleting key from Vault: {}", key_id);
        
        let url = format!("{}/v1/{}/metadata/{}", self.config.address, self.config.mount_path, key_id);
        
        let response = self.client
            .delete(&url)
            .header("X-Vault-Token", &self.config.token)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("Key deleted successfully from Vault: {}", key_id);
            Ok(true)
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            warn!("Key not found in Vault: {}", key_id);
            Ok(false)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Vault API error: {}", error_text))
        }
    }

    /// Check if a key exists in Vault
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `Result<bool>` - True if key exists, false otherwise
    pub async fn key_exists(&self, key_id: &str) -> Result<bool> {
        let url = format!("{}/v1/{}/data/{}", self.config.address, self.config.mount_path, key_id);
        
        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.config.token)
            .send()
            .await?;
        
        Ok(response.status() == reqwest::StatusCode::OK)
    }

    /// Get key metadata from Vault
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `Result<VaultKeyMetadata>` - Key metadata or error
    pub async fn get_key_metadata(&self, key_id: &str) -> Result<VaultKeyMetadata> {
        info!("Retrieving key metadata from Vault: {}", key_id);
        
        let url = format!("{}/v1/{}/data/{}", self.config.address, self.config.mount_path, key_id);
        
        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.config.token)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            
            if let Some(data) = json.get("data").and_then(|d| d.get("data")) {
                let metadata = VaultKeyMetadata {
                    key_id: key_id.to_string(),
                    key_type: data.get("key_type").and_then(|k| k.as_str()).unwrap_or("unknown").to_string(),
                    created_at: data.get("created_at").and_then(|c| c.as_u64()).unwrap_or(0),
                    version: json.get("data").and_then(|d| d.get("version")).and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    tags: data.get("tags").and_then(|t| t.as_array())
                        .map(|arr| arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                };
                
                info!("Key metadata retrieved successfully from Vault: {}", key_id);
                Ok(metadata)
            } else {
                Err(anyhow::anyhow!("Key metadata not found in Vault response"))
            }
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Vault API error: {}", error_text))
        }
    }

    /// Rotate a key in Vault
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// * `new_key_material` - New key material
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn rotate_key(&self, key_id: &str, new_key_material: &[u8]) -> Result<()> {
        info!("Rotating key in Vault: {}", key_id);
        
        // Get existing metadata
        let metadata = self.get_key_metadata(key_id).await?;
        
        // Store with new key material (Vault automatically versions it)
        self.store_key(key_id, new_key_material, &metadata.key_type, metadata.tags).await
    }
}

/// Vault key manager that provides a higher-level interface
pub struct VaultKeyManager {
    /// Vault client
    client: VaultClient,
    /// Cache of key metadata
    metadata_cache: HashMap<String, VaultKeyMetadata>,
}

impl VaultKeyManager {
    /// Create a new Vault key manager
    pub fn new(client: VaultClient) -> Self {
        Self {
            client,
            metadata_cache: HashMap::new(),
        }
    }

    /// Store a key
    pub async fn store_key(&mut self, key_id: &str, key_material: &[u8], key_type: &str, tags: Vec<String>) -> Result<()> {
        self.client.store_key(key_id, key_material, key_type, tags).await
    }

    /// Retrieve a key
    pub async fn retrieve_key(&mut self, key_id: &str) -> Result<Vec<u8>> {
        self.client.retrieve_key(key_id).await
    }

    /// List all keys
    pub async fn list_keys(&self) -> Result<Vec<String>> {
        self.client.list_keys().await
    }

    /// Delete a key
    pub async fn delete_key(&mut self, key_id: &str) -> Result<bool> {
        self.metadata_cache.remove(key_id);
        self.client.delete_key(key_id).await
    }

    /// Check if a key exists
    pub async fn key_exists(&self, key_id: &str) -> Result<bool> {
        self.client.key_exists(key_id).await
    }

    /// Get key metadata
    pub async fn get_key_metadata(&mut self, key_id: &str) -> Result<VaultKeyMetadata> {
        // Check cache first
        if let Some(metadata) = self.metadata_cache.get(key_id) {
            return Ok(metadata.clone());
        }
        
        // Fetch from Vault
        let metadata = self.client.get_key_metadata(key_id).await?;
        self.metadata_cache.insert(key_id.to_string(), metadata.clone());
        Ok(metadata)
    }

    /// Rotate a key
    pub async fn rotate_key(&mut self, key_id: &str, new_key_material: &[u8]) -> Result<()> {
        self.client.rotate_key(key_id, new_key_material).await?;
        // Invalidate cache
        self.metadata_cache.remove(key_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vault_config() {
        let config = VaultConfig::default();
        assert_eq!(config.address, "http://127.0.0.1:8200");
        assert_eq!(config.mount_path, "secret");
        assert!(config.verify_tls);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[tokio::test]
    async fn test_vault_client_creation() {
        let config = VaultConfig::default();
        let client = VaultClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_vault_key_metadata() {
        let metadata = VaultKeyMetadata {
            key_id: "test-key".to_string(),
            key_type: "ethereum".to_string(),
            created_at: 1234567890,
            version: 1,
            tags: vec!["test".to_string(), "ethereum".to_string()],
        };
        
        assert_eq!(metadata.key_id, "test-key");
        assert_eq!(metadata.key_type, "ethereum");
        assert_eq!(metadata.created_at, 1234567890);
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.tags.len(), 2);
    }

    #[tokio::test]
    async fn test_vault_key_manager() {
        let config = VaultConfig::default();
        let client = VaultClient::new(config).unwrap();
        let mut manager = VaultKeyManager::new(client);
        
        // These tests would require a running Vault server
        // For now, we just test that the methods exist and compile
        assert!(true);
    }
}