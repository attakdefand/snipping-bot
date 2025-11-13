//! Nonce Management Module
//!
//! This module provides functionality for managing transaction nonces,
//! including tracking, synchronization, and recovery mechanisms.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Nonce management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceConfig {
    /// Enable/disable nonce management
    pub enabled: bool,
    /// Maximum number of pending transactions per account
    pub max_pending_transactions: usize,
    /// Nonce synchronization interval in seconds
    pub sync_interval_seconds: u64,
    /// Enable automatic nonce recovery
    pub auto_recovery: bool,
    /// Number of confirmations required for nonce update
    pub confirmations_required: u64,
}

impl Default for NonceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_pending_transactions: 100,
            sync_interval_seconds: 30, // 30 seconds
            auto_recovery: true,
            confirmations_required: 12, // 12 confirmations
        }
    }
}

/// Nonce manager
pub struct NonceManager {
    /// Configuration
    config: NonceConfig,
    /// Nonce state for each account
    accounts: HashMap<String, AccountNonceState>,
}

/// Account nonce state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountNonceState {
    /// Current on-chain nonce
    pub on_chain_nonce: u64,
    /// Next local nonce to use
    pub local_nonce: u64,
    /// Pending transactions
    pub pending_transactions: HashMap<u64, PendingTransaction>,
    /// Last sync timestamp
    pub last_sync: u64,
    /// Account address
    pub address: String,
}

/// Pending transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    /// Transaction hash
    pub tx_hash: String,
    /// Nonce
    pub nonce: u64,
    /// Submission timestamp
    pub submitted_at: u64,
    /// Gas price
    pub gas_price: u64,
    /// Gas limit
    pub gas_limit: u64,
}

/// Nonce update parameters
#[derive(Debug, Clone)]
pub struct NonceUpdateParams {
    /// On-chain nonce
    pub on_chain_nonce: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl NonceManager {
    /// Create a new nonce manager
    pub fn new(config: NonceConfig) -> Self {
        Self {
            config,
            accounts: HashMap::new(),
        }
    }

    /// Get the next nonce for an account
    pub fn get_next_nonce(&mut self, account_address: &str) -> Result<u64> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Nonce management is disabled"));
        }

        let account = self.accounts
            .entry(account_address.to_string())
            .or_insert_with(|| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                AccountNonceState {
                    on_chain_nonce: 0,
                    local_nonce: 0,
                    pending_transactions: HashMap::new(),
                    last_sync: now,
                    address: account_address.to_string(),
                }
            });

        // Check if we have too many pending transactions
        if account.pending_transactions.len() >= self.config.max_pending_transactions {
            return Err(anyhow::anyhow!(
                "Too many pending transactions for account {}: {} >= {}",
                account_address,
                account.pending_transactions.len(),
                self.config.max_pending_transactions
            ));
        }

        let nonce = account.local_nonce;
        account.local_nonce += 1;

        info!("Assigned nonce {} to account {}", nonce, account_address);
        Ok(nonce)
    }

    /// Register a pending transaction
    pub fn register_pending_transaction(
        &mut self,
        account_address: &str,
        tx_hash: String,
        nonce: u64,
        gas_price: u64,
        gas_limit: u64,
    ) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Nonce management is disabled"));
        }

        let account = self.accounts
            .get_mut(account_address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_address))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let pending_tx = PendingTransaction {
            tx_hash,
            nonce,
            submitted_at: now,
            gas_price,
            gas_limit,
        };

        account.pending_transactions.insert(nonce, pending_tx);
        info!("Registered pending transaction for account {} with nonce {}", account_address, nonce);
        Ok(())
    }

    /// Update account nonce state
    pub fn update_account_nonce(&mut self, account_address: &str, params: NonceUpdateParams) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Nonce management is disabled"));
        }

        let account = self.accounts
            .get_mut(account_address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_address))?;

        // Update on-chain nonce
        account.on_chain_nonce = params.on_chain_nonce;
        account.last_sync = params.timestamp;

        // Clean up confirmed transactions
        account.pending_transactions.retain(|nonce, _| *nonce >= params.on_chain_nonce);

        // Update local nonce if necessary
        if account.local_nonce < params.on_chain_nonce {
            account.local_nonce = params.on_chain_nonce;
        }

        info!("Updated nonce for account {}: on-chain={}, local={}", account_address, params.on_chain_nonce, account.local_nonce);
        Ok(())
    }

    /// Recover nonce for an account
    pub fn recover_nonce(&mut self, account_address: &str) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Nonce management is disabled"));
        }

        if !self.config.auto_recovery {
            return Err(anyhow::anyhow!("Automatic nonce recovery is disabled"));
        }

        let account = self.accounts
            .get_mut(account_address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_address))?;

        // In a real implementation, we would query the blockchain for the current nonce
        // For this implementation, we'll simulate a recovery by resetting to on-chain nonce
        let recovered_nonce = account.on_chain_nonce;
        account.local_nonce = recovered_nonce;
        
        // Clear pending transactions as they may be invalid
        account.pending_transactions.clear();

        info!("Recovered nonce for account {}: {}", account_address, recovered_nonce);
        Ok(())
    }

    /// Get account nonce state
    pub fn get_account_state(&self, account_address: &str) -> Option<&AccountNonceState> {
        self.accounts.get(account_address)
    }

    /// List all accounts
    pub fn list_accounts(&self) -> Vec<&AccountNonceState> {
        self.accounts.values().collect()
    }

    /// Remove an account
    pub fn remove_account(&mut self, account_address: &str) -> Result<bool> {
        match self.accounts.remove(account_address) {
            Some(_) => {
                info!("Removed account: {}", account_address);
                Ok(true)
            }
            None => {
                warn!("Attempted to remove non-existent account: {}", account_address);
                Ok(false)
            }
        }
    }

    /// Get pending transactions for an account
    pub fn get_pending_transactions(&self, account_address: &str) -> Result<Vec<&PendingTransaction>> {
        let account = self.accounts
            .get(account_address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_address))?;

        Ok(account.pending_transactions.values().collect())
    }

    /// Cancel a pending transaction
    pub fn cancel_pending_transaction(&mut self, account_address: &str, nonce: u64) -> Result<bool> {
        let account = self.accounts
            .get_mut(account_address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_address))?;

        match account.pending_transactions.remove(&nonce) {
            Some(_) => {
                info!("Cancelled pending transaction for account {} with nonce {}", account_address, nonce);
                Ok(true)
            }
            None => {
                warn!("Attempted to cancel non-existent pending transaction for account {} with nonce {}", account_address, nonce);
                Ok(false)
            }
        }
    }

    /// Get nonce statistics
    pub fn get_statistics(&self) -> NonceStatistics {
        let total_accounts = self.accounts.len();
        let total_pending_transactions: usize = self.accounts
            .values()
            .map(|account| account.pending_transactions.len())
            .sum();

        let max_pending_per_account = self.accounts
            .values()
            .map(|account| account.pending_transactions.len())
            .max()
            .unwrap_or(0);

        NonceStatistics {
            total_accounts,
            total_pending_transactions,
            max_pending_per_account,
        }
    }

    /// Check if an account needs nonce synchronization
    pub fn needs_sync(&self, account_address: &str) -> Result<bool> {
        let account = self.accounts
            .get(account_address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", account_address))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(now - account.last_sync > self.config.sync_interval_seconds)
    }
}

/// Nonce statistics
#[derive(Debug, Clone)]
pub struct NonceStatistics {
    /// Total number of accounts
    pub total_accounts: usize,
    /// Total number of pending transactions
    pub total_pending_transactions: usize,
    /// Maximum pending transactions per account
    pub max_pending_per_account: usize,
}

impl Default for NonceManager {
    fn default() -> Self {
        Self::new(NonceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_manager_creation() {
        let config = NonceConfig::default();
        let manager = NonceManager::new(config);
        
        assert!(manager.accounts.is_empty());
    }

    #[test]
    fn test_get_next_nonce() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        let nonce1 = manager.get_next_nonce("0x123").unwrap();
        let nonce2 = manager.get_next_nonce("0x123").unwrap();
        let nonce3 = manager.get_next_nonce("0x456").unwrap();
        
        assert_eq!(nonce1, 0);
        assert_eq!(nonce2, 1);
        assert_eq!(nonce3, 0);
        
        // Check account states
        let account1 = manager.get_account_state("0x123").unwrap();
        assert_eq!(account1.local_nonce, 2);
        assert_eq!(account1.on_chain_nonce, 0);
        
        let account2 = manager.get_account_state("0x456").unwrap();
        assert_eq!(account2.local_nonce, 1);
        assert_eq!(account2.on_chain_nonce, 0);
    }

    #[test]
    fn test_register_pending_transaction() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Get a nonce first
        let nonce = manager.get_next_nonce("0x123").unwrap();
        
        // Register the pending transaction
        let result = manager.register_pending_transaction(
            "0x123",
            "0xabc".to_string(),
            nonce,
            1000000000,
            21000,
        );
        
        assert!(result.is_ok());
        
        // Check pending transactions
        let pending = manager.get_pending_transactions("0x123").unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].tx_hash, "0xabc");
        assert_eq!(pending[0].nonce, nonce);
    }

    #[test]
    fn test_update_account_nonce() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Get a few nonces
        let nonce1 = manager.get_next_nonce("0x123").unwrap();
        let nonce2 = manager.get_next_nonce("0x123").unwrap();
        
        // Register pending transactions
        manager.register_pending_transaction("0x123", "0xabc".to_string(), nonce1, 1000000000, 21000).unwrap();
        manager.register_pending_transaction("0x123", "0xdef".to_string(), nonce2, 1000000000, 21000).unwrap();
        
        // Update account nonce
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let params = NonceUpdateParams {
            on_chain_nonce: 1,
            timestamp: now,
        };
        
        let result = manager.update_account_nonce("0x123", params);
        assert!(result.is_ok());
        
        // Check that first transaction is cleaned up
        let pending = manager.get_pending_transactions("0x123").unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].nonce, 1);
    }

    #[test]
    fn test_nonce_recovery() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Get a few nonces
        let _nonce1 = manager.get_next_nonce("0x123").unwrap();
        let _nonce2 = manager.get_next_nonce("0x123").unwrap();
        
        // Register pending transactions
        manager.register_pending_transaction("0x123", "0xabc".to_string(), 0, 1000000000, 21000).unwrap();
        manager.register_pending_transaction("0x123", "0xdef".to_string(), 1, 1000000000, 21000).unwrap();
        
        // Update on-chain nonce
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let params = NonceUpdateParams {
            on_chain_nonce: 1,
            timestamp: now,
        };
        
        manager.update_account_nonce("0x123", params).unwrap();
        
        // Recover nonce
        let result = manager.recover_nonce("0x123");
        assert!(result.is_ok());
        
        // Check that pending transactions are cleared and nonce is reset
        let account = manager.get_account_state("0x123").unwrap();
        assert_eq!(account.local_nonce, 1);
        assert_eq!(account.pending_transactions.len(), 0);
    }

    #[test]
    fn test_cancel_pending_transaction() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Get a nonce
        let nonce = manager.get_next_nonce("0x123").unwrap();
        
        // Register pending transaction
        manager.register_pending_transaction("0x123", "0xabc".to_string(), nonce, 1000000000, 21000).unwrap();
        
        // Check pending transactions
        let pending = manager.get_pending_transactions("0x123").unwrap();
        assert_eq!(pending.len(), 1);
        
        // Cancel pending transaction
        let cancelled = manager.cancel_pending_transaction("0x123", nonce).unwrap();
        assert!(cancelled);
        
        // Check pending transactions again
        let pending = manager.get_pending_transactions("0x123").unwrap();
        assert_eq!(pending.len(), 0);
        
        // Try to cancel non-existent transaction
        let cancelled = manager.cancel_pending_transaction("0x123", nonce).unwrap();
        assert!(!cancelled);
    }

    #[test]
    fn test_remove_account() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Add an account
        let _nonce = manager.get_next_nonce("0x123").unwrap();
        assert_eq!(manager.accounts.len(), 1);
        
        // Remove account
        let removed = manager.remove_account("0x123").unwrap();
        assert!(removed);
        assert_eq!(manager.accounts.len(), 0);
        
        // Try to remove non-existent account
        let removed = manager.remove_account("0x123").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_statistics() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Add accounts and pending transactions
        let _nonce1 = manager.get_next_nonce("0x123").unwrap();
        let _nonce2 = manager.get_next_nonce("0x123").unwrap();
        let _nonce3 = manager.get_next_nonce("0x456").unwrap();
        
        manager.register_pending_transaction("0x123", "0xabc".to_string(), 0, 1000000000, 21000).unwrap();
        manager.register_pending_transaction("0x123", "0xdef".to_string(), 1, 1000000000, 21000).unwrap();
        manager.register_pending_transaction("0x456", "0xghi".to_string(), 0, 1000000000, 21000).unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_accounts, 2);
        assert_eq!(stats.total_pending_transactions, 3);
        assert_eq!(stats.max_pending_per_account, 2);
    }

    #[test]
    fn test_needs_sync() {
        let config = NonceConfig::default();
        let mut manager = NonceManager::new(config);
        
        // Add an account
        let _nonce = manager.get_next_nonce("0x123").unwrap();
        
        // Check if needs sync (should be false initially)
        let needs_sync = manager.needs_sync("0x123").unwrap();
        assert!(!needs_sync);
    }

    #[test]
    fn test_max_pending_transactions() {
        let mut config = NonceConfig::default();
        config.max_pending_transactions = 2;
        let mut manager = NonceManager::new(config);
        
        // Add account and get max pending transactions
        let _nonce1 = manager.get_next_nonce("0x123").unwrap();
        manager.register_pending_transaction("0x123", "0xabc".to_string(), 0, 1000000000, 21000).unwrap();
        
        let _nonce2 = manager.get_next_nonce("0x123").unwrap();
        manager.register_pending_transaction("0x123", "0xdef".to_string(), 1, 1000000000, 21000).unwrap();
        
        // Try to get another nonce (should fail)
        let result = manager.get_next_nonce("0x123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Too many pending transactions"));
    }

    #[test]
    fn test_disabled_manager() {
        let mut config = NonceConfig::default();
        config.enabled = false;
        let mut manager = NonceManager::new(config);
        
        let result = manager.get_next_nonce("0x123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Nonce management is disabled");
    }

    #[test]
    fn test_disabled_recovery() {
        let mut config = NonceConfig::default();
        config.auto_recovery = false;
        let mut manager = NonceManager::new(config);
        
        let result = manager.recover_nonce("0x123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Automatic nonce recovery is disabled");
    }

    #[test]
    fn test_nonexistent_account() {
        let config = NonceConfig::default();
        let manager = NonceManager::new(config);
        
        let result = manager.get_pending_transactions("0x123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Account not found"));
    }
}