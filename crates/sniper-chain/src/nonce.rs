//! Transaction nonce management for blockchain transactions
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Nonce manager for tracking transaction nonces
#[derive(Debug, Clone)]
pub struct NonceManager {
    /// Map of address to current nonce
    nonces: Arc<RwLock<HashMap<String, u64>>>,
}

impl NonceManager {
    /// Create a new nonce manager
    pub fn new() -> Self {
        Self {
            nonces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the next nonce for an address
    /// 
    /// # Arguments
    /// * `address` - The address to get the next nonce for
    /// 
    /// # Returns
    /// * `Result<u64>` - The next nonce for the address
    pub async fn get_next_nonce(&self, address: &str) -> Result<u64> {
        let mut nonces = self.nonces.write().await;
        let nonce = nonces.entry(address.to_string()).or_insert(0);
        let next_nonce = *nonce;
        *nonce += 1;
        Ok(next_nonce)
    }

    /// Set the nonce for an address
    /// 
    /// # Arguments
    /// * `address` - The address to set the nonce for
    /// * `nonce` - The nonce to set
    pub async fn set_nonce(&self, address: &str, nonce: u64) {
        let mut nonces = self.nonces.write().await;
        nonces.insert(address.to_string(), nonce);
    }

    /// Get the current nonce for an address without incrementing
    /// 
    /// # Arguments
    /// * `address` - The address to get the current nonce for
    /// 
    /// # Returns
    /// * `Option<u64>` - The current nonce for the address, if it exists
    pub async fn get_current_nonce(&self, address: &str) -> Option<u64> {
        let nonces = self.nonces.read().await;
        nonces.get(address).copied()
    }

    /// Reset the nonce for an address
    /// 
    /// # Arguments
    /// * `address` - The address to reset the nonce for
    pub async fn reset_nonce(&self, address: &str) {
        let mut nonces = self.nonces.write().await;
        nonces.remove(address);
    }

    /// Reset all nonces
    pub async fn reset_all_nonces(&self) {
        let mut nonces = self.nonces.write().await;
        nonces.clear();
    }
}

impl Default for NonceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Nonce strategy for handling nonce management
#[derive(Debug, Clone)]
pub enum NonceStrategy {
    /// Sequential nonce management (0, 1, 2, ...)
    Sequential,
    /// Random nonce management (for testing)
    Random,
    /// Timestamp-based nonce management
    Timestamp,
}

/// Advanced nonce manager with different strategies
#[derive(Debug, Clone)]
pub struct AdvancedNonceManager {
    /// Basic nonce manager
    base_manager: NonceManager,
    /// Nonce strategy to use
    strategy: NonceStrategy,
}

impl AdvancedNonceManager {
    /// Create a new advanced nonce manager
    /// 
    /// # Arguments
    /// * `strategy` - The nonce strategy to use
    pub fn new(strategy: NonceStrategy) -> Self {
        Self {
            base_manager: NonceManager::new(),
            strategy,
        }
    }

    /// Get the next nonce for an address based on the strategy
    /// 
    /// # Arguments
    /// * `address` - The address to get the next nonce for
    /// 
    /// # Returns
    /// * `Result<u64>` - The next nonce for the address
    pub async fn get_next_nonce(&self, address: &str) -> Result<u64> {
        match self.strategy {
            NonceStrategy::Sequential => {
                self.base_manager.get_next_nonce(address).await
            }
            NonceStrategy::Random => {
                // For testing purposes, return a random nonce
                Ok(rand::random::<u64>() % 1000)
            }
            NonceStrategy::Timestamp => {
                // Use timestamp as nonce (not recommended for production)
                Ok(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0))
            }
        }
    }

    /// Set the nonce for an address
    /// 
    /// # Arguments
    /// * `address` - The address to set the nonce for
    /// * `nonce` - The nonce to set
    pub async fn set_nonce(&self, address: &str, nonce: u64) {
        self.base_manager.set_nonce(address, nonce).await;
    }
}

impl Default for AdvancedNonceManager {
    fn default() -> Self {
        Self::new(NonceStrategy::Sequential)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nonce_manager() {
        let manager = NonceManager::new();
        
        // Test getting next nonce for new address
        let nonce1 = manager.get_next_nonce("0xAddress1").await.unwrap();
        assert_eq!(nonce1, 0);
        
        // Test getting next nonce again
        let nonce2 = manager.get_next_nonce("0xAddress1").await.unwrap();
        assert_eq!(nonce2, 1);
        
        // Test getting nonce for different address
        let nonce3 = manager.get_next_nonce("0xAddress2").await.unwrap();
        assert_eq!(nonce3, 0);
        
        // Test setting nonce
        manager.set_nonce("0xAddress1", 10).await;
        let nonce4 = manager.get_current_nonce("0xAddress1").await.unwrap();
        assert_eq!(nonce4, 10);
        
        // Test getting next nonce after setting
        let nonce5 = manager.get_next_nonce("0xAddress1").await.unwrap();
        assert_eq!(nonce5, 11);
    }

    #[tokio::test]
    async fn test_nonce_manager_reset() {
        let manager = NonceManager::new();
        
        // Set some nonces
        manager.set_nonce("0xAddress1", 5).await;
        manager.set_nonce("0xAddress2", 10).await;
        
        // Verify nonces are set
        assert_eq!(manager.get_current_nonce("0xAddress1").await, Some(5));
        assert_eq!(manager.get_current_nonce("0xAddress2").await, Some(10));
        
        // Reset one nonce
        manager.reset_nonce("0xAddress1").await;
        assert_eq!(manager.get_current_nonce("0xAddress1").await, None);
        assert_eq!(manager.get_current_nonce("0xAddress2").await, Some(10));
        
        // Reset all nonces
        manager.reset_all_nonces().await;
        assert_eq!(manager.get_current_nonce("0xAddress2").await, None);
    }

    #[tokio::test]
    async fn test_advanced_nonce_manager() {
        let sequential_manager = AdvancedNonceManager::new(NonceStrategy::Sequential);
        
        // Test sequential strategy
        let nonce1 = sequential_manager.get_next_nonce("0xAddress1").await.unwrap();
        assert_eq!(nonce1, 0);
        
        let nonce2 = sequential_manager.get_next_nonce("0xAddress1").await.unwrap();
        assert_eq!(nonce2, 1);
        
        // Test random strategy
        let random_manager = AdvancedNonceManager::new(NonceStrategy::Random);
        let _nonce3 = random_manager.get_next_nonce("0xAddress1").await.unwrap();
        // We can't assert a specific value for random, just that it works
        
        // Test timestamp strategy
        let timestamp_manager = AdvancedNonceManager::new(NonceStrategy::Timestamp);
        let nonce4 = timestamp_manager.get_next_nonce("0xAddress1").await.unwrap();
        // Timestamp should be a recent value
        assert!(nonce4 > 1000000000); // Unix timestamp should be larger than this
    }
}