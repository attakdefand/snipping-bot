//! Pair created signal detection for on-chain events
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// Pair created event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairCreatedEvent {
    /// Address of the newly created pair
    pub pair_address: String,
    /// Address of token0
    pub token0: String,
    /// Address of token1
    pub token1: String,
    /// Fee tier (for Uniswap V3 style pairs)
    pub fee_tier: Option<u32>,
    /// Block number where the pair was created
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: String,
}

/// Pair created signal detector
pub struct PairCreatedDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
}

impl PairCreatedDetector {
    /// Create a new pair created detector
    pub fn new(chain: ChainRef) -> Self {
        Self { chain }
    }

    /// Process a pair created event and generate a signal
    pub fn process_pair_created_event(&self, event: PairCreatedEvent) -> Result<Signal> {
        info!(
            "Processing pair created event on chain {} for pair {}",
            self.chain.name, event.pair_address
        );

        // Create the signal
        let signal = Signal {
            source: "dex".to_string(),
            kind: "pair_created".to_string(),
            chain: self.chain.clone(),
            token0: Some(event.token0.clone()),
            token1: Some(event.token1.clone()),
            extra: serde_json::json!({
                "pair_address": event.pair_address,
                "fee_tier": event.fee_tier,
                "block_number": event.block_number,
                "tx_hash": event.tx_hash,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated pair created signal: {:?}", signal);
        Ok(signal)
    }

    /// Validate a pair created event
    pub fn validate_pair_created_event(&self, event: &PairCreatedEvent) -> bool {
        // Basic validation
        if event.pair_address.is_empty() {
            warn!("Invalid pair address in pair created event");
            return false;
        }

        if event.token0.is_empty() || event.token1.is_empty() {
            warn!("Invalid token addresses in pair created event");
            return false;
        }

        if event.token0 == event.token1 {
            warn!("Token0 and token1 are the same in pair created event");
            return false;
        }

        if event.block_number == 0 {
            warn!("Invalid block number in pair created event");
            return false;
        }

        if event.tx_hash.is_empty() {
            warn!("Invalid transaction hash in pair created event");
            return false;
        }

        true
    }

    /// Filter pair created events based on criteria
    pub fn filter_pair_created_event(&self, event: &PairCreatedEvent) -> bool {
        // In a real implementation, this would check against configured filters
        // For now, we'll accept all valid events
        self.validate_pair_created_event(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_pair_created_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = PairCreatedDetector::new(chain.clone());
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
    }

    #[test]
    fn test_pair_created_event_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = PairCreatedDetector::new(chain);

        // Valid event
        let valid_event = PairCreatedEvent {
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            fee_tier: Some(3000),
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        assert!(detector.validate_pair_created_event(&valid_event));

        // Invalid event - empty pair address
        let mut invalid_event = valid_event.clone();
        invalid_event.pair_address = String::new();
        assert!(!detector.validate_pair_created_event(&invalid_event));

        // Invalid event - same tokens
        let mut invalid_event = valid_event.clone();
        invalid_event.token1 = "0xToken0".to_string();
        assert!(!detector.validate_pair_created_event(&invalid_event));
    }

    #[test]
    fn test_pair_created_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = PairCreatedDetector::new(chain);

        let event = PairCreatedEvent {
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            fee_tier: Some(3000),
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        let signal = detector.process_pair_created_event(event).unwrap();
        assert_eq!(signal.source, "dex");
        assert_eq!(signal.kind, "pair_created");
        assert_eq!(signal.chain.name, "ethereum");
        assert_eq!(signal.token0, Some("0xToken0".to_string()));
        assert_eq!(signal.token1, Some("0xToken1".to_string()));
        assert!(signal.seen_at_ms > 0);
    }
}
