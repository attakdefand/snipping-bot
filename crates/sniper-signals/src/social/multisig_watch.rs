//! Multisig wallet watch signal detection for social events
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// Multisig transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigTransaction {
    /// Address of the multisig wallet
    pub wallet_address: String,
    /// Transaction hash
    pub tx_hash: String,
    /// Transaction value in wei
    pub value: u128,
    /// Transaction data (calldata)
    pub data: String,
    /// Destination address
    pub to: String,
    /// Signers who have approved the transaction
    pub signers: Vec<String>,
    /// Required number of signatures
    pub required_signatures: u32,
    /// Block number where the transaction was submitted
    pub block_number: u64,
    /// Timestamp of the transaction
    pub timestamp: u64,
    /// Description of the transaction
    pub description: Option<String>,
}

/// Multisig wallet watch signal detector
pub struct MultisigWatchDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
    /// List of watched multisig wallets
    watched_wallets: Vec<String>,
}

impl MultisigWatchDetector {
    /// Create a new multisig watch detector
    pub fn new(chain: ChainRef, watched_wallets: Vec<String>) -> Self {
        Self {
            chain,
            watched_wallets,
        }
    }

    /// Process a multisig transaction and generate a signal if it's from a watched wallet
    pub fn process_multisig_transaction(&self, tx: MultisigTransaction) -> Option<Signal> {
        info!(
            "Processing multisig transaction {} for wallet {} on chain {}",
            tx.tx_hash, tx.wallet_address, self.chain.name
        );

        // Check if this is from a watched wallet
        let is_watched = self.watched_wallets.contains(&tx.wallet_address);

        if !is_watched {
            debug!("Transaction is not from a watched wallet, ignoring");
            return None;
        }

        // Check if the transaction has enough signatures
        let has_enough_signatures = tx.signers.len() as u32 >= tx.required_signatures;

        // Create the signal
        let signal = Signal {
            source: "social".to_string(),
            kind: if has_enough_signatures {
                "multisig_executed".to_string()
            } else {
                "multisig_proposed".to_string()
            },
            chain: self.chain.clone(),
            token0: None,
            token1: None,
            extra: serde_json::json!({
                "wallet_address": tx.wallet_address,
                "tx_hash": tx.tx_hash,
                "value": tx.value,
                "data": tx.data,
                "to": tx.to,
                "signers": tx.signers,
                "required_signatures": tx.required_signatures,
                "signatures_obtained": tx.signers.len(),
                "block_number": tx.block_number,
                "timestamp": tx.timestamp,
                "description": tx.description,
                "has_enough_signatures": has_enough_signatures,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated multisig watch signal: {:?}", signal);
        Some(signal)
    }

    /// Validate a multisig transaction
    pub fn validate_multisig_transaction(&self, tx: &MultisigTransaction) -> bool {
        // Basic validation
        if tx.wallet_address.is_empty() {
            warn!("Invalid wallet address in multisig transaction");
            return false;
        }

        if tx.tx_hash.is_empty() {
            warn!("Invalid transaction hash in multisig transaction");
            return false;
        }

        if tx.to.is_empty() {
            warn!("Invalid destination address in multisig transaction");
            return false;
        }

        if tx.block_number == 0 {
            warn!("Invalid block number in multisig transaction");
            return false;
        }

        if tx.timestamp == 0 {
            warn!("Invalid timestamp in multisig transaction");
            return false;
        }

        if tx.required_signatures == 0 {
            warn!("Invalid required signatures in multisig transaction");
            return false;
        }

        // Validate signers
        if tx.signers.len() > tx.required_signatures as usize {
            warn!("More signers than required in multisig transaction");
            return false;
        }

        true
    }

    /// Filter multisig transactions based on criteria
    pub fn filter_multisig_transaction(&self, tx: &MultisigTransaction) -> bool {
        // In a real implementation, this would check against configured filters
        // For now, we'll accept all valid transactions from watched wallets
        self.validate_multisig_transaction(tx)
    }

    /// Add a wallet to the watch list
    pub fn add_watched_wallet(&mut self, wallet_address: String) {
        self.watched_wallets.push(wallet_address);
    }

    /// Remove a wallet from the watch list
    pub fn remove_watched_wallet(&mut self, wallet_address: &str) {
        self.watched_wallets.retain(|w| w != wallet_address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_multisig_watch_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let watched_wallets = vec!["0x1234567890123456789012345678901234567890".to_string()];
        let detector = MultisigWatchDetector::new(chain.clone(), watched_wallets.clone());
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
        assert_eq!(detector.watched_wallets, watched_wallets);
    }

    #[test]
    fn test_multisig_transaction_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = MultisigWatchDetector::new(
            chain,
            vec!["0x1234567890123456789012345678901234567890".to_string()],
        );

        // Valid transaction
        let valid_tx = MultisigTransaction {
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            value: 1000000000000000000, // 1 ETH
            data: "0x1234".to_string(),
            to: "0x1234567890123456789012345678901234567891".to_string(),
            signers: vec![
                "0x1234567890123456789012345678901234567892".to_string(),
                "0x1234567890123456789012345678901234567893".to_string(),
            ],
            required_signatures: 2,
            block_number: 12345678,
            timestamp: 1234567890,
            description: Some("Transfer funds".to_string()),
        };

        assert!(detector.validate_multisig_transaction(&valid_tx));

        // Invalid transaction - empty wallet address
        let mut invalid_tx = valid_tx.clone();
        invalid_tx.wallet_address = String::new();
        assert!(!detector.validate_multisig_transaction(&invalid_tx));

        // Invalid transaction - too many signers
        let mut invalid_tx = valid_tx.clone();
        invalid_tx.signers = vec!["0x1".to_string(), "0x2".to_string(), "0x3".to_string()];
        assert!(!detector.validate_multisig_transaction(&invalid_tx));
    }

    #[test]
    fn test_multisig_transaction_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = MultisigWatchDetector::new(
            chain,
            vec!["0x1234567890123456789012345678901234567890".to_string()],
        );

        // Transaction from watched wallet with enough signatures
        let executed_tx = MultisigTransaction {
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            value: 1000000000000000000, // 1 ETH
            data: "0x1234".to_string(),
            to: "0x1234567890123456789012345678901234567891".to_string(),
            signers: vec![
                "0x1234567890123456789012345678901234567892".to_string(),
                "0x1234567890123456789012345678901234567893".to_string(),
            ],
            required_signatures: 2,
            block_number: 12345678,
            timestamp: 1234567890,
            description: Some("Transfer funds".to_string()),
        };

        let signal = detector.process_multisig_transaction(executed_tx);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.source, "social");
        assert_eq!(signal.kind, "multisig_executed");
        assert_eq!(signal.chain.name, "ethereum");
        assert!(signal.seen_at_ms > 0);

        // Transaction from watched wallet with insufficient signatures
        let proposed_tx = MultisigTransaction {
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891"
                .to_string(),
            value: 1000000000000000000, // 1 ETH
            data: "0x5678".to_string(),
            to: "0x1234567890123456789012345678901234567891".to_string(),
            signers: vec!["0x1234567890123456789012345678901234567892".to_string()],
            required_signatures: 2,
            block_number: 12345679,
            timestamp: 1234567891,
            description: Some("Large transfer".to_string()),
        };

        let signal = detector.process_multisig_transaction(proposed_tx);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.kind, "multisig_proposed");

        // Transaction from unwatched wallet
        let unwatched_tx = MultisigTransaction {
            wallet_address: "0x1234567890123456789012345678901234567899".to_string(),
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567892"
                .to_string(),
            value: 1000000000000000000, // 1 ETH
            data: "0x9abc".to_string(),
            to: "0x1234567890123456789012345678901234567891".to_string(),
            signers: vec!["0x1234567890123456789012345678901234567892".to_string()],
            required_signatures: 1,
            block_number: 12345680,
            timestamp: 1234567892,
            description: Some("Small transfer".to_string()),
        };

        let signal = detector.process_multisig_transaction(unwatched_tx);
        assert!(signal.is_none());
    }

    #[test]
    fn test_watched_wallet_management() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let mut detector = MultisigWatchDetector::new(chain, vec![]);

        // Add a wallet
        detector.add_watched_wallet("0x1234567890123456789012345678901234567890".to_string());
        assert_eq!(detector.watched_wallets.len(), 1);
        assert_eq!(
            detector.watched_wallets[0],
            "0x1234567890123456789012345678901234567890"
        );

        // Remove a wallet
        detector.remove_watched_wallet("0x1234567890123456789012345678901234567890");
        assert_eq!(detector.watched_wallets.len(), 0);
    }
}
