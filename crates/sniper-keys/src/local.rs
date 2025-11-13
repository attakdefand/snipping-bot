//! Local key storage implementation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Local key storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalKeyConfig {
    /// Path to the key storage directory
    pub storage_path: String,
    /// Enable encryption of stored keys
    pub encrypt_keys: bool,
    /// Master password for key encryption (in practice, this should come from a secure source)
    pub master_password: Option<String>,
}

impl Default for LocalKeyConfig {
    fn default() -> Self {
        Self {
            storage_path: "./keys".to_string(),
            encrypt_keys: true,
            master_password: None,
        }
    }
}

/// Key types supported by local storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocalKeyType {
    /// Ethereum private key
    Ethereum,
    /// BIP39 mnemonic phrase
    Mnemonic,
    /// Raw private key
    PrivateKey,
    /// JSON Web Key
    Jwk,
}

/// Stored key representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKey {
    /// Key identifier
    pub key_id: String,
    /// Key type
    pub key_type: LocalKeyType,
    /// Encrypted key material
    pub encrypted_key: Vec<u8>,
    /// Creation timestamp
    pub created_at: u64,
    /// Tags for key organization
    pub tags: Vec<String>,
    /// Nonce used for encryption (if applicable)
    pub nonce: Option<Vec<u8>>,
}

/// Local key storage manager
pub struct LocalKeyStorage {
    /// Configuration
    config: LocalKeyConfig,
    /// In-memory cache of keys
    key_cache: HashMap<String, StoredKey>,
}

impl LocalKeyStorage {
    /// Create a new local key storage manager
    pub fn new(config: LocalKeyConfig) -> Result<Self> {
        // Ensure storage directory exists
        fs::create_dir_all(&config.storage_path)?;
        
        Ok(Self {
            config,
            key_cache: HashMap::new(),
        })
    }

    /// Store a key locally
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// * `key_type` - Type of key being stored
    /// * `key_material` - Raw key material to store
    /// * `tags` - Tags for organizing the key
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn store_key(&mut self, key_id: &str, key_type: LocalKeyType, key_material: &[u8], tags: Vec<String>) -> Result<()> {
        info!("Storing key: {}", key_id);
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Encrypt the key material if encryption is enabled
        let (encrypted_key, nonce) = if self.config.encrypt_keys {
            self.encrypt_key_material(key_material)?
        } else {
            (key_material.to_vec(), None)
        };
        
        let stored_key = StoredKey {
            key_id: key_id.to_string(),
            key_type,
            encrypted_key,
            created_at: now,
            tags,
            nonce,
        };
        
        // Save to file
        let key_file_path = format!("{}/{}.key", self.config.storage_path, key_id);
        let key_data = serde_json::to_vec(&stored_key)?;
        fs::write(&key_file_path, key_data)?;
        
        // Cache in memory
        self.key_cache.insert(key_id.to_string(), stored_key);
        
        info!("Key stored successfully: {}", key_id);
        Ok(())
    }

    /// Retrieve a key from local storage
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - The decrypted key material or error
    pub fn retrieve_key(&mut self, key_id: &str) -> Result<Vec<u8>> {
        info!("Retrieving key: {}", key_id);
        
        // Check cache first
        if let Some(stored_key) = self.key_cache.get(key_id) {
            debug!("Key found in cache: {}", key_id);
            return self.decrypt_key_material(stored_key);
        }
        
        // Load from file
        let key_file_path = format!("{}/{}.key", self.config.storage_path, key_id);
        if !Path::new(&key_file_path).exists() {
            return Err(anyhow::anyhow!("Key not found: {}", key_id));
        }
        
        let key_data = fs::read(&key_file_path)?;
        let stored_key: StoredKey = serde_json::from_slice(&key_data)?;
        
        // Cache the key
        self.key_cache.insert(key_id.to_string(), stored_key.clone());
        
        // Decrypt and return
        self.decrypt_key_material(&stored_key)
    }

    /// List all stored keys
    /// 
    /// # Returns
    /// * `Result<Vec<String>>` - List of key identifiers
    pub fn list_keys(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        
        // List files in storage directory
        let entries = fs::read_dir(&self.config.storage_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "key") {
                if let Some(file_name) = path.file_stem() {
                    if let Some(key_id) = file_name.to_str() {
                        keys.push(key_id.to_string());
                    }
                }
            }
        }
        
        Ok(keys)
    }

    /// Delete a key from local storage
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `Result<bool>` - True if key was deleted, false if not found
    pub fn delete_key(&mut self, key_id: &str) -> Result<bool> {
        info!("Deleting key: {}", key_id);
        
        // Remove from cache
        self.key_cache.remove(key_id);
        
        // Remove file
        let key_file_path = format!("{}/{}.key", self.config.storage_path, key_id);
        if Path::new(&key_file_path).exists() {
            fs::remove_file(&key_file_path)?;
            info!("Key deleted successfully: {}", key_id);
            Ok(true)
        } else {
            warn!("Key not found for deletion: {}", key_id);
            Ok(false)
        }
    }

    /// Check if a key exists in local storage
    /// 
    /// # Arguments
    /// * `key_id` - Unique identifier for the key
    /// 
    /// # Returns
    /// * `bool` - True if key exists, false otherwise
    pub fn key_exists(&self, key_id: &str) -> bool {
        // Check cache first
        if self.key_cache.contains_key(key_id) {
            return true;
        }
        
        // Check file system
        let key_file_path = format!("{}/{}.key", self.config.storage_path, key_id);
        Path::new(&key_file_path).exists()
    }

    /// Encrypt key material using master password
    fn encrypt_key_material(&self, key_material: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        if !self.config.encrypt_keys {
            return Ok((key_material.to_vec(), None));
        }
        
        // In a real implementation, we would use a proper encryption library
        // For this implementation, we'll use a simple XOR cipher for demonstration
        // (This is NOT secure for production use!)
        
        let master_password = self.config.master_password
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Master password not configured"))?;
        
        let mut encrypted = Vec::new();
        let password_bytes = master_password.as_bytes();
        
        for (i, byte) in key_material.iter().enumerate() {
            let password_byte = password_bytes[i % password_bytes.len()];
            encrypted.push(byte ^ password_byte);
        }
        
        Ok((encrypted, None))
    }

    /// Decrypt key material using master password
    fn decrypt_key_material(&self, stored_key: &StoredKey) -> Result<Vec<u8>> {
        if !self.config.encrypt_keys {
            return Ok(stored_key.encrypted_key.clone());
        }
        
        // In a real implementation, we would use a proper decryption library
        // For this implementation, we'll use the same XOR cipher for demonstration
        // (This is NOT secure for production use!)
        
        let master_password = self.config.master_password
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Master password not configured"))?;
        
        let mut decrypted = Vec::new();
        let password_bytes = master_password.as_bytes();
        let encrypted_bytes = &stored_key.encrypted_key;
        
        for (i, byte) in encrypted_bytes.iter().enumerate() {
            let password_byte = password_bytes[i % password_bytes.len()];
            decrypted.push(byte ^ password_byte);
        }
        
        Ok(decrypted)
    }
}

impl Default for LocalKeyStorage {
    fn default() -> Self {
        Self::new(LocalKeyConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_local_key_storage_creation() {
        let config = LocalKeyConfig {
            storage_path: "./test_keys".to_string(),
            encrypt_keys: false,
            master_password: None,
        };
        
        let storage = LocalKeyStorage::new(config);
        assert!(storage.is_ok());
        
        // Clean up
        let _ = fs::remove_dir_all("./test_keys");
    }

    #[test]
    fn test_key_storage_and_retrieval() {
        let config = LocalKeyConfig {
            storage_path: "./test_keys".to_string(),
            encrypt_keys: false,
            master_password: None,
        };
        
        let mut storage = LocalKeyStorage::new(config).unwrap();
        let key_material = b"test private key material";
        
        // Store a key
        let result = storage.store_key(
            "test-key-1",
            LocalKeyType::PrivateKey,
            key_material,
            vec!["test".to_string()]
        );
        assert!(result.is_ok());
        
        // Check if key exists
        assert!(storage.key_exists("test-key-1"));
        
        // Retrieve the key
        let retrieved = storage.retrieve_key("test-key-1").unwrap();
        assert_eq!(retrieved, key_material);
        
        // Clean up
        let _ = fs::remove_dir_all("./test_keys");
    }

    #[test]
    fn test_key_listing() {
        let config = LocalKeyConfig {
            storage_path: "./test_keys".to_string(),
            encrypt_keys: false,
            master_password: None,
        };
        
        let mut storage = LocalKeyStorage::new(config).unwrap();
        let key_material = b"test key material";
        
        // Store multiple keys
        storage.store_key("key-1", LocalKeyType::PrivateKey, key_material, vec![]).unwrap();
        storage.store_key("key-2", LocalKeyType::Mnemonic, key_material, vec![]).unwrap();
        
        // List keys
        let keys = storage.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key-1".to_string()));
        assert!(keys.contains(&"key-2".to_string()));
        
        // Clean up
        let _ = fs::remove_dir_all("./test_keys");
    }

    #[test]
    fn test_key_deletion() {
        let config = LocalKeyConfig {
            storage_path: "./test_keys".to_string(),
            encrypt_keys: false,
            master_password: None,
        };
        
        let mut storage = LocalKeyStorage::new(config).unwrap();
        let key_material = b"test key material";
        
        // Store a key
        storage.store_key("test-key", LocalKeyType::PrivateKey, key_material, vec![]).unwrap();
        assert!(storage.key_exists("test-key"));
        
        // Delete the key
        let deleted = storage.delete_key("test-key").unwrap();
        assert!(deleted);
        assert!(!storage.key_exists("test-key"));
        
        // Try to delete non-existent key
        let deleted = storage.delete_key("non-existent").unwrap();
        assert!(!deleted);
        
        // Clean up
        let _ = fs::remove_dir_all("./test_keys");
    }

    #[test]
    fn test_encrypted_key_storage() {
        let config = LocalKeyConfig {
            storage_path: "./test_keys".to_string(),
            encrypt_keys: true,
            master_password: Some("test-password".to_string()),
        };
        
        let mut storage = LocalKeyStorage::new(config).unwrap();
        let key_material = b"secret key material";
        
        // Store an encrypted key
        let result = storage.store_key(
            "encrypted-key",
            LocalKeyType::PrivateKey,
            key_material,
            vec!["encrypted".to_string()]
        );
        assert!(result.is_ok());
        
        // Retrieve and decrypt the key
        let retrieved = storage.retrieve_key("encrypted-key").unwrap();
        assert_eq!(retrieved, key_material);
        
        // Clean up
        let _ = fs::remove_dir_all("./test_keys");
    }
}