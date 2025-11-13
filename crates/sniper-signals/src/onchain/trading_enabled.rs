//! Trading enabled signal detection for on-chain events
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// Trading enabled event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingEnabledEvent {
    /// Address of the pair
    pub pair_address: String,
    /// Address of token0
    pub token0: String,
    /// Address of token1
    pub token1: String,
    /// Whether trading is enabled
    pub enabled: bool,
    /// Block number where the event occurred
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: String,
}

/// Trading enabled signal detector
pub struct TradingEnabledDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
}

impl TradingEnabledDetector {
    /// Create a new trading enabled detector
    pub fn new(chain: ChainRef) -> Self {
        Self { chain }
    }

    /// Process a trading enabled event and generate a signal
    pub fn process_trading_enabled_event(&self, event: TradingEnabledEvent) -> Result<Signal> {
        info!(
            "Processing trading {} event on chain {} for pair {}",
            if event.enabled { "enabled" } else { "disabled" },
            self.chain.name,
            event.pair_address
        );

        // Create the signal
        let signal = Signal {
            source: "dex".to_string(),
            kind: if event.enabled {
                "trading_enabled".to_string()
            } else {
                "trading_disabled".to_string()
            },
            chain: self.chain.clone(),
            token0: Some(event.token0.clone()),
            token1: Some(event.token1.clone()),
            extra: serde_json::json!({
                "pair_address": event.pair_address,
                "enabled": event.enabled,
                "block_number": event.block_number,
                "tx_hash": event.tx_hash,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated trading enabled signal: {:?}", signal);
        Ok(signal)
    }

    /// Validate a trading enabled event
    pub fn validate_trading_enabled_event(&self, event: &TradingEnabledEvent) -> bool {
        // Basic validation
        if event.pair_address.is_empty() {
            warn!("Invalid pair address in trading enabled event");
            return false;
        }

        if event.token0.is_empty() || event.token1.is_empty() {
            warn!("Invalid token addresses in trading enabled event");
            return false;
        }

        if event.token0 == event.token1 {
            warn!("Token0 and token1 are the same in trading enabled event");
            return false;
        }

        if event.block_number == 0 {
            warn!("Invalid block number in trading enabled event");
            return false;
        }

        if event.tx_hash.is_empty() {
            warn!("Invalid transaction hash in trading enabled event");
            return false;
        }

        true
    }

    /// Filter trading enabled events based on criteria
    pub fn filter_trading_enabled_event(&self, event: &TradingEnabledEvent) -> bool {
        // In a real implementation, this would check against configured filters
        // For now, we'll accept all valid events
        self.validate_trading_enabled_event(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_trading_enabled_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = TradingEnabledDetector::new(chain.clone());
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
    }

    #[test]
    fn test_trading_enabled_event_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = TradingEnabledDetector::new(chain);

        // Valid enabled event
        let valid_event = TradingEnabledEvent {
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            enabled: true,
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        assert!(detector.validate_trading_enabled_event(&valid_event));

        // Valid disabled event
        let valid_disabled_event = TradingEnabledEvent {
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            enabled: false,
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        assert!(detector.validate_trading_enabled_event(&valid_disabled_event));

        // Invalid event - empty pair address
        let mut invalid_event = valid_event.clone();
        invalid_event.pair_address = String::new();
        assert!(!detector.validate_trading_enabled_event(&invalid_event));
    }

    #[test]
    fn test_trading_enabled_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = TradingEnabledDetector::new(chain);

        // Test enabled event
        let enabled_event = TradingEnabledEvent {
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            enabled: true,
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        let enabled_signal = detector
            .process_trading_enabled_event(enabled_event)
            .unwrap();
        assert_eq!(enabled_signal.source, "dex");
        assert_eq!(enabled_signal.kind, "trading_enabled");
        assert_eq!(enabled_signal.chain.name, "ethereum");
        assert_eq!(enabled_signal.token0, Some("0xToken0".to_string()));
        assert_eq!(enabled_signal.token1, Some("0xToken1".to_string()));
        assert!(enabled_signal.seen_at_ms > 0);

        // Test disabled event
        let disabled_event = TradingEnabledEvent {
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            enabled: false,
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        let disabled_signal = detector
            .process_trading_enabled_event(disabled_event)
            .unwrap();
        assert_eq!(disabled_signal.source, "dex");
        assert_eq!(disabled_signal.kind, "trading_disabled");
        assert_eq!(disabled_signal.chain.name, "ethereum");
        assert_eq!(disabled_signal.token0, Some("0xToken0".to_string()));
        assert_eq!(disabled_signal.token1, Some("0xToken1".to_string()));
        assert!(disabled_signal.seen_at_ms > 0);
    }
}
