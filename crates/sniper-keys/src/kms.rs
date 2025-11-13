//! Key Management Service Module
//!
//! This module provides functionality for managing cryptographic keys,
//! including generation, storage, encryption, and signing operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Sha256, Digest};

/// Key Management Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsConfig {
    /// Enable/disable KMS
    pub enabled: bool,
    /// Default key algorithm
    pub default_algorithm: KeyAlgorithm,
    /// Key rotation interval in seconds (0 = no rotation)
    pub rotation_interval_seconds: u64,
    /// Master key for encrypting other keys
    pub master_key: Option<String>,
}

impl Default for KmsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_algorithm: KeyAlgorithm::Aes256Gcm,
            rotation_interval_seconds: 86400, // 24 hours
            master_key: None,
        }
    }
}

/// Key algorithms supported by the KMS
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// Ed25519 for signing
    Ed25519,
}

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Key identifier
    pub key_id: String,
    /// Key algorithm
    pub algorithm: KeyAlgorithm,
    /// Creation timestamp
    pub created_at: u64,
    /// Last rotation timestamp
    pub last_rotated: u64,
    /// Key usage
    pub usage: KeyUsage,
    /// Key tags
    pub tags: Vec<String>,
}

/// Key usage types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyUsage {
    /// Encryption/decryption
    Encryption,
    /// Signing/verification
    Signing,
    /// Both encryption and signing
    Both,
}

/// Key Management Service
pub struct KeyManagementService {
    /// Configuration
    config: KmsConfig,
    /// Stored keys (encrypted in practice)
    keys: HashMap<String, StoredKey>,
    /// Key metadata
    metadata: HashMap<String, KeyMetadata>,
}

/// Stored key representation
#[derive(Debug, Clone)]
struct StoredKey {
    /// Encrypted key material
    encrypted_key: Vec<u8>,
    /// Nonce used for encryption
    nonce: Vec<u8>,
}

impl KeyManagementService {
    /// Create a new Key Management Service
    pub fn new(config: KmsConfig) -> Self {
        Self {
            config,
            keys: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Generate a new key
    pub fn generate_key(&mut self, usage: KeyUsage, tags: Vec<String>) -> Result<String> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        let key_id = self.generate_key_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate key material based on algorithm
        let key_material = match self.config.default_algorithm {
            KeyAlgorithm::Aes256Gcm => {
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                key.to_vec()
            }
            KeyAlgorithm::Ed25519 => {
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                key.to_vec()
            }
        };

        // Encrypt the key material
        let (encrypted_key, nonce) = self.encrypt_key_material(&key_material)?;

        let stored_key = StoredKey {
            encrypted_key,
            nonce,
        };

        let metadata = KeyMetadata {
            key_id: key_id.clone(),
            algorithm: self.config.default_algorithm.clone(),
            created_at: now,
            last_rotated: now,
            usage,
            tags,
        };

        self.keys.insert(key_id.clone(), stored_key);
        self.metadata.insert(key_id.clone(), metadata);

        info!("Generated new key: {}", key_id);
        Ok(key_id)
    }

    /// Encrypt data using a key
    pub fn encrypt(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        let key = self.get_decrypted_key(key_id)?;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
        
        let mut ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.append(&mut ciphertext);
        Ok(result)
    }

    /// Decrypt data using a key
    pub fn decrypt(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        if data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext"));
        }

        let key = self.get_decrypted_key(key_id)?;
        let nonce_bytes = &data[0..12];
        let ciphertext = &data[12..];

        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Sign data using a key
    pub fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        let metadata = self.metadata.get(key_id)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key_id))?;

        if metadata.usage == KeyUsage::Encryption {
            return Err(anyhow::anyhow!("Key cannot be used for signing"));
        }

        let key = self.get_decrypted_key(key_id)?;

        match metadata.algorithm {
            KeyAlgorithm::Ed25519 => {
                // For Ed25519, we would normally use a proper signing library
                // This is a simplified implementation for demonstration
                let mut hasher = Sha256::new();
                hasher.update(&key);
                hasher.update(data);
                let result = hasher.finalize();
                Ok(result.to_vec())
            }
            KeyAlgorithm::Aes256Gcm => {
                // AES is not typically used for signing, but we can create a MAC
                let mut nonce_bytes = [0u8; 12];
                rand::thread_rng().fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

                let mut mac_data = data.to_vec();
                mac_data.extend_from_slice(&nonce_bytes);

                let mut mac = cipher
                    .encrypt(nonce, &mac_data[..])
                    .map_err(|e| anyhow::anyhow!("MAC generation failed: {}", e))?;

                // Prepend nonce to MAC
                let mut result = nonce_bytes.to_vec();
                result.append(&mut mac);
                Ok(result)
            }
        }
    }

    /// Verify signature using a key
    pub fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        let metadata = self.metadata.get(key_id)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key_id))?;

        if metadata.usage == KeyUsage::Encryption {
            return Err(anyhow::anyhow!("Key cannot be used for verification"));
        }

        // In a real implementation, we would verify the signature properly
        // This is a simplified check for demonstration
        let computed_signature = self.sign(key_id, data)?;
        Ok(computed_signature == signature)
    }

    /// Get key metadata
    pub fn get_key_metadata(&self, key_id: &str) -> Option<&KeyMetadata> {
        self.metadata.get(key_id)
    }

    /// List all keys
    pub fn list_keys(&self) -> Vec<&KeyMetadata> {
        self.metadata.values().collect()
    }

    /// Delete a key
    pub fn delete_key(&mut self, key_id: &str) -> Result<bool> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        match (self.keys.remove(key_id), self.metadata.remove(key_id)) {
            (Some(_), Some(metadata)) => {
                info!("Deleted key: {}", key_id);
                Ok(true)
            }
            _ => {
                warn!("Attempted to delete non-existent key: {}", key_id);
                Ok(false)
            }
        }
    }

    /// Rotate a key
    pub fn rotate_key(&mut self, key_id: &str) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("KMS is disabled"));
        }

        let metadata = self.metadata.get_mut(key_id)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key_id))?;

        // Generate new key material
        let key_material = match metadata.algorithm {
            KeyAlgorithm::Aes256Gcm => {
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                key.to_vec()
            }
            KeyAlgorithm::Ed25519 => {
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                key.to_vec()
            }
        };

        // Encrypt the new key material
        let (encrypted_key, nonce) = self.encrypt_key_material(&key_material)?;

        let stored_key = StoredKey {
            encrypted_key,
            nonce,
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update stored key and metadata
        self.keys.insert(key_id.to_string(), stored_key);
        metadata.last_rotated = now;

        info!("Rotated key: {}", key_id);
        Ok(())
    }

    /// Generate a unique key identifier
    fn generate_key_id(&self) -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    /// Encrypt key material using master key
    fn encrypt_key_material(&self, key_material: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Use a fixed key for demonstration (in practice, this would be the master key)
        let master_key_bytes = if let Some(ref master_key) = self.config.master_key {
            let mut hasher = Sha256::new();
            hasher.update(master_key.as_bytes());
            hasher.finalize().to_vec()
        } else {
            // Fallback to a default key for demonstration
            vec![0u8; 32]
        };

        let cipher = Aes256Gcm::new_from_slice(&master_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create master cipher: {}", e))?;

        let encrypted_key = cipher
            .encrypt(nonce, key_material)
            .map_err(|e| anyhow::anyhow!("Key encryption failed: {}", e))?;

        Ok((encrypted_key, nonce_bytes.to_vec()))
    }

    /// Decrypt key material using master key
    fn decrypt_key_material(&self, encrypted_key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        if nonce.len() != 12 {
            return Err(anyhow::anyhow!("Invalid nonce length"));
        }

        let nonce = Nonce::from_slice(nonce);

        // Use a fixed key for demonstration (in practice, this would be the master key)
        let master_key_bytes = if let Some(ref master_key) = self.config.master_key {
            let mut hasher = Sha256::new();
            hasher.update(master_key.as_bytes());
            hasher.finalize().to_vec()
        } else {
            // Fallback to a default key for demonstration
            vec![0u8; 32]
        };

        let cipher = Aes256Gcm::new_from_slice(&master_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create master cipher: {}", e))?;

        let decrypted_key = cipher
            .decrypt(nonce, encrypted_key)
            .map_err(|e| anyhow::anyhow!("Key decryption failed: {}", e))?;

        Ok(decrypted_key)
    }

    /// Get decrypted key material
    fn get_decrypted_key(&self, key_id: &str) -> Result<Vec<u8>> {
        let stored_key = self.keys.get(key_id)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key_id))?;

        self.decrypt_key_material(&stored_key.encrypted_key, &stored_key.nonce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kms_creation() {
        let config = KmsConfig::default();
        let kms = KeyManagementService::new(config);
        
        assert!(kms.keys.is_empty());
        assert!(kms.metadata.is_empty());
    }

    #[test]
    fn test_key_generation() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        let key_id = kms.generate_key(KeyUsage::Both, vec!["test".to_string()]).unwrap();
        assert!(!key_id.is_empty());
        assert_eq!(kms.keys.len(), 1);
        assert_eq!(kms.metadata.len(), 1);
        
        let metadata = kms.get_key_metadata(&key_id).unwrap();
        assert_eq!(metadata.key_id, key_id);
        assert_eq!(metadata.algorithm, KeyAlgorithm::Aes256Gcm);
        assert_eq!(metadata.usage, KeyUsage::Both);
        assert_eq!(metadata.tags, vec!["test".to_string()]);
    }

    #[test]
    fn test_encryption_decryption() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        let key_id = kms.generate_key(KeyUsage::Encryption, vec![]).unwrap();
        let plaintext = b"Hello, World!";
        
        let ciphertext = kms.encrypt(&key_id, plaintext).unwrap();
        let decrypted = kms.decrypt(&key_id, &ciphertext).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_signing_verification() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        let key_id = kms.generate_key(KeyUsage::Signing, vec![]).unwrap();
        let data = b"Hello, World!";
        
        let signature = kms.sign(&key_id, data).unwrap();
        let verified = kms.verify(&key_id, data, &signature).unwrap();
        
        assert!(verified);
    }

    #[test]
    fn test_key_deletion() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        let key_id = kms.generate_key(KeyUsage::Both, vec![]).unwrap();
        assert_eq!(kms.keys.len(), 1);
        assert_eq!(kms.metadata.len(), 1);
        
        let deleted = kms.delete_key(&key_id).unwrap();
        assert!(deleted);
        assert_eq!(kms.keys.len(), 0);
        assert_eq!(kms.metadata.len(), 0);
        
        // Try to delete non-existent key
        let deleted = kms.delete_key(&key_id).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_key_rotation() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        let key_id = kms.generate_key(KeyUsage::Both, vec![]).unwrap();
        let metadata_before = kms.get_key_metadata(&key_id).unwrap().clone();
        
        // Wait a bit to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        kms.rotate_key(&key_id).unwrap();
        let metadata_after = kms.get_key_metadata(&key_id).unwrap();
        
        assert!(metadata_after.last_rotated > metadata_before.last_rotated);
    }

    #[test]
    fn test_list_keys() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        kms.generate_key(KeyUsage::Encryption, vec!["tag1".to_string()]).unwrap();
        kms.generate_key(KeyUsage::Signing, vec!["tag2".to_string()]).unwrap();
        
        let keys = kms.list_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_disabled_kms() {
        let mut config = KmsConfig::default();
        config.enabled = false;
        let mut kms = KeyManagementService::new(config);
        
        let result = kms.generate_key(KeyUsage::Both, vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "KMS is disabled");
    }

    #[test]
    fn test_invalid_operations() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        // Try to encrypt with non-existent key
        let result = kms.encrypt("non-existent", b"data");
        assert!(result.is_err());
        
        // Try to decrypt with non-existent key
        let result = kms.decrypt("non-existent", b"data");
        assert!(result.is_err());
        
        // Try to sign with non-existent key
        let result = kms.sign("non-existent", b"data");
        assert!(result.is_err());
        
        // Try to verify with non-existent key
        let result = kms.verify("non-existent", b"data", b"signature");
        assert!(result.is_err());
    }

    #[test]
    fn test_usage_restrictions() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        // Create encryption-only key
        let enc_key_id = kms.generate_key(KeyUsage::Encryption, vec![]).unwrap();
        
        // Try to sign with encryption-only key
        let result = kms.sign(&enc_key_id, b"data");
        assert!(result.is_err());
        
        // Try to verify with encryption-only key
        let result = kms.verify(&enc_key_id, b"data", b"signature");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_update() {
        let config = KmsConfig::default();
        let mut kms = KeyManagementService::new(config);
        
        let new_config = KmsConfig {
            enabled: false,
            default_algorithm: KeyAlgorithm::Ed25519,
            rotation_interval_seconds: 43200, // 12 hours
            master_key: Some("new_master_key".to_string()),
        };
        
        kms.config = new_config.clone();
        assert_eq!(kms.config.default_algorithm, KeyAlgorithm::Ed25519);
        assert_eq!(kms.config.rotation_interval_seconds, 43200);
        assert_eq!(kms.config.master_key, Some("new_master_key".to_string()));
    }
}