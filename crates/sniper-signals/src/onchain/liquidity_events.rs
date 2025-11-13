//! Liquidity events signal detection for on-chain events
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// Liquidity event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LiquidityEventType {
    /// Liquidity added to a pool
    Add,
    /// Liquidity removed from a pool
    Remove,
    /// Pool was initialized
    Initialize,
}

/// Liquidity event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityEvent {
    /// Type of liquidity event
    pub event_type: LiquidityEventType,
    /// Address of the pair
    pub pair_address: String,
    /// Address of token0
    pub token0: String,
    /// Address of token1
    pub token1: String,
    /// Amount of token0
    pub amount0: u128,
    /// Amount of token1
    pub amount1: u128,
    /// Liquidity amount (for Uniswap V2 style pools)
    pub liquidity: Option<u128>,
    /// Tick information (for Uniswap V3 style pools)
    pub tick_lower: Option<i32>,
    /// Tick information (for Uniswap V3 style pools)
    pub tick_upper: Option<i32>,
    /// Block number where the event occurred
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: String,
}

/// Liquidity events signal detector
pub struct LiquidityEventsDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
}

impl LiquidityEventsDetector {
    /// Create a new liquidity events detector
    pub fn new(chain: ChainRef) -> Self {
        Self { chain }
    }

    /// Process a liquidity event and generate a signal
    pub fn process_liquidity_event(&self, event: LiquidityEvent) -> Result<Signal> {
        info!(
            "Processing liquidity event {:?} on chain {} for pair {}",
            event.event_type, self.chain.name, event.pair_address
        );

        // Create the signal
        let signal = Signal {
            source: "dex".to_string(),
            kind: match event.event_type {
                LiquidityEventType::Add => "liquidity_added".to_string(),
                LiquidityEventType::Remove => "liquidity_removed".to_string(),
                LiquidityEventType::Initialize => "pool_initialized".to_string(),
            },
            chain: self.chain.clone(),
            token0: Some(event.token0.clone()),
            token1: Some(event.token1.clone()),
            extra: serde_json::json!({
                "event_type": event.event_type,
                "pair_address": event.pair_address,
                "amount0": event.amount0,
                "amount1": event.amount1,
                "liquidity": event.liquidity,
                "tick_lower": event.tick_lower,
                "tick_upper": event.tick_upper,
                "block_number": event.block_number,
                "tx_hash": event.tx_hash,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated liquidity event signal: {:?}", signal);
        Ok(signal)
    }

    /// Validate a liquidity event
    pub fn validate_liquidity_event(&self, event: &LiquidityEvent) -> bool {
        // Basic validation
        if event.pair_address.is_empty() {
            warn!("Invalid pair address in liquidity event");
            return false;
        }

        if event.token0.is_empty() || event.token1.is_empty() {
            warn!("Invalid token addresses in liquidity event");
            return false;
        }

        if event.token0 == event.token1 {
            warn!("Token0 and token1 are the same in liquidity event");
            return false;
        }

        if event.amount0 == 0 && event.amount1 == 0 {
            warn!("Both token amounts are zero in liquidity event");
            return false;
        }

        if event.block_number == 0 {
            warn!("Invalid block number in liquidity event");
            return false;
        }

        if event.tx_hash.is_empty() {
            warn!("Invalid transaction hash in liquidity event");
            return false;
        }

        // Validate tick information for V3 events
        if event.tick_lower.is_some() && event.tick_upper.is_some() {
            let lower = event.tick_lower.unwrap();
            let upper = event.tick_upper.unwrap();
            if lower >= upper {
                warn!("Invalid tick range in liquidity event");
                return false;
            }
        }

        true
    }

    /// Filter liquidity events based on criteria
    pub fn filter_liquidity_event(&self, event: &LiquidityEvent) -> bool {
        // In a real implementation, this would check against configured filters
        // For now, we'll accept all valid events
        self.validate_liquidity_event(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_liquidity_events_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = LiquidityEventsDetector::new(chain.clone());
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
    }

    #[test]
    fn test_liquidity_event_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = LiquidityEventsDetector::new(chain);

        // Valid add event
        let valid_event = LiquidityEvent {
            event_type: LiquidityEventType::Add,
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            amount0: 1000000000000000000, // 1 ETH
            amount1: 2000000000,          // 2000 USDT
            liquidity: Some(1000000),
            tick_lower: None,
            tick_upper: None,
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        assert!(detector.validate_liquidity_event(&valid_event));

        // Valid V3 event with ticks
        let valid_v3_event = LiquidityEvent {
            event_type: LiquidityEventType::Add,
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            amount0: 1000000000000000000, // 1 ETH
            amount1: 2000000000,          // 2000 USDT
            liquidity: Some(1000000),
            tick_lower: Some(-1000),
            tick_upper: Some(1000),
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        assert!(detector.validate_liquidity_event(&valid_v3_event));

        // Invalid event - zero amounts
        let mut invalid_event = valid_event.clone();
        invalid_event.amount0 = 0;
        invalid_event.amount1 = 0;
        assert!(!detector.validate_liquidity_event(&invalid_event));

        // Invalid event - invalid ticks
        let mut invalid_event = valid_v3_event.clone();
        invalid_event.tick_lower = Some(1000);
        invalid_event.tick_upper = Some(-1000);
        assert!(!detector.validate_liquidity_event(&invalid_event));
    }

    #[test]
    fn test_liquidity_event_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = LiquidityEventsDetector::new(chain);

        let event = LiquidityEvent {
            event_type: LiquidityEventType::Add,
            pair_address: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0xToken0".to_string(),
            token1: "0xToken1".to_string(),
            amount0: 1000000000000000000, // 1 ETH
            amount1: 2000000000,          // 2000 USDT
            liquidity: Some(1000000),
            tick_lower: None,
            tick_upper: None,
            block_number: 12345678,
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        };

        let signal = detector.process_liquidity_event(event).unwrap();
        assert_eq!(signal.source, "dex");
        assert_eq!(signal.kind, "liquidity_added");
        assert_eq!(signal.chain.name, "ethereum");
        assert_eq!(signal.token0, Some("0xToken0".to_string()));
        assert_eq!(signal.token1, Some("0xToken1".to_string()));
        assert!(signal.seen_at_ms > 0);
    }
}
