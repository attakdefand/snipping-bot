//! Price feeds signal detection for off-chain events
use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// Price feed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    /// Token symbol
    pub token: String,
    /// Token address
    pub token_address: Option<String>,
    /// Price in USD
    pub price_usd: f64,
    /// Price change percentage in last 24 hours
    pub price_change_24h: f64,
    /// Trading volume in USD
    pub volume_24h: f64,
    /// Market cap in USD
    pub market_cap: Option<f64>,
    /// Timestamp of the price feed
    pub timestamp: u64,
}

/// Price feed signal detector
pub struct PriceFeedsDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
    /// Price threshold for generating signals
    price_threshold: Option<f64>,
    /// Price change threshold for generating signals
    price_change_threshold: Option<f64>,
}

impl PriceFeedsDetector {
    /// Create a new price feeds detector
    pub fn new(
        chain: ChainRef,
        price_threshold: Option<f64>,
        price_change_threshold: Option<f64>,
    ) -> Self {
        Self {
            chain,
            price_threshold,
            price_change_threshold,
        }
    }

    /// Process a price feed and generate a signal if thresholds are met
    pub fn process_price_feed(&self, feed: PriceFeed) -> Option<Signal> {
        info!(
            "Processing price feed for {} on chain {} - Price: ${}, Change: {}%",
            feed.token, self.chain.name, feed.price_usd, feed.price_change_24h
        );

        // Check if thresholds are met
        let mut should_generate_signal = false;
        let mut reasons = Vec::new();

        if let Some(threshold) = self.price_threshold {
            if feed.price_usd >= threshold {
                should_generate_signal = true;
                reasons.push(format!(
                    "Price ${} >= threshold ${}",
                    feed.price_usd, threshold
                ));
            }
        }

        if let Some(threshold) = self.price_change_threshold {
            if feed.price_change_24h >= threshold {
                should_generate_signal = true;
                reasons.push(format!(
                    "Price change {}% >= threshold {}%",
                    feed.price_change_24h, threshold
                ));
            }
        }

        // If no thresholds are set, always generate a signal
        if self.price_threshold.is_none() && self.price_change_threshold.is_none() {
            should_generate_signal = true;
            reasons.push("No thresholds set".to_string());
        }

        if should_generate_signal {
            // Create the signal
            let signal = Signal {
                source: "dex".to_string(),
                kind: "price_feed".to_string(),
                chain: self.chain.clone(),
                token0: Some(feed.token.clone()),
                token1: Some("USD".to_string()),
                extra: serde_json::json!({
                    "token_address": feed.token_address,
                    "price_usd": feed.price_usd,
                    "price_change_24h": feed.price_change_24h,
                    "volume_24h": feed.volume_24h,
                    "market_cap": feed.market_cap,
                    "timestamp": feed.timestamp,
                    "reasons": reasons,
                }),
                seen_at_ms: chrono::Utc::now().timestamp_millis(),
            };

            debug!("Generated price feed signal: {:?}", signal);
            Some(signal)
        } else {
            debug!("Price feed does not meet thresholds, no signal generated");
            None
        }
    }

    /// Validate a price feed
    pub fn validate_price_feed(&self, feed: &PriceFeed) -> bool {
        // Basic validation
        if feed.token.is_empty() {
            warn!("Invalid token in price feed");
            return false;
        }

        if feed.price_usd <= 0.0 {
            warn!("Invalid price in price feed for token {}", feed.token);
            return false;
        }

        if feed.timestamp == 0 {
            warn!("Invalid timestamp in price feed for token {}", feed.token);
            return false;
        }

        true
    }

    /// Filter price feeds based on criteria
    pub fn filter_price_feed(&self, feed: &PriceFeed) -> bool {
        // In a real implementation, this would check against configured filters
        // For now, we'll accept all valid feeds
        self.validate_price_feed(feed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_price_feeds_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = PriceFeedsDetector::new(chain.clone(), Some(100.0), Some(5.0));
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
        assert_eq!(detector.price_threshold, Some(100.0));
        assert_eq!(detector.price_change_threshold, Some(5.0));
    }

    #[test]
    fn test_price_feed_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = PriceFeedsDetector::new(chain, None, None);

        // Valid feed
        let valid_feed = PriceFeed {
            token: "ETH".to_string(),
            token_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            price_usd: 2000.0,
            price_change_24h: 5.0,
            volume_24h: 1000000000.0,
            market_cap: Some(200000000000.0),
            timestamp: 1234567890,
        };

        assert!(detector.validate_price_feed(&valid_feed));

        // Invalid feed - zero price
        let mut invalid_feed = valid_feed.clone();
        invalid_feed.price_usd = 0.0;
        assert!(!detector.validate_price_feed(&invalid_feed));

        // Invalid feed - empty token
        let mut invalid_feed = valid_feed.clone();
        invalid_feed.token = String::new();
        assert!(!detector.validate_price_feed(&invalid_feed));
    }

    #[test]
    fn test_price_feed_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        // Test with thresholds
        let detector = PriceFeedsDetector::new(chain.clone(), Some(100.0), Some(5.0));

        // Feed that meets price threshold
        let high_price_feed = PriceFeed {
            token: "ETH".to_string(),
            token_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            price_usd: 2000.0,
            price_change_24h: 2.0,
            volume_24h: 1000000000.0,
            market_cap: Some(200000000000.0),
            timestamp: 1234567890,
        };

        let signal = detector.process_price_feed(high_price_feed);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.source, "dex");
        assert_eq!(signal.kind, "price_feed");
        assert_eq!(signal.chain.name, "ethereum");
        assert_eq!(signal.token0, Some("ETH".to_string()));
        assert_eq!(signal.token1, Some("USD".to_string()));
        assert!(signal.seen_at_ms > 0);

        // Feed that doesn't meet thresholds
        let low_price_feed = PriceFeed {
            token: "SHIB".to_string(),
            token_address: Some("0x1234567890123456789012345678901234567891".to_string()),
            price_usd: 0.00001,
            price_change_24h: 1.0,
            volume_24h: 1000000.0,
            market_cap: Some(10000000.0),
            timestamp: 1234567890,
        };

        let signal = detector.process_price_feed(low_price_feed.clone());
        assert!(signal.is_none());

        // Test without thresholds (should always generate signal)
        let detector_no_thresholds = PriceFeedsDetector::new(chain, None, None);
        let signal = detector_no_thresholds.process_price_feed(low_price_feed);
        assert!(signal.is_some());
    }
}
