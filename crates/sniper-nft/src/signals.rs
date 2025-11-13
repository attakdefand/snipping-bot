//! NFT signal detection and processing

use serde::{Deserialize, Serialize};
use sniper_core::types::{ChainRef, Signal};
use tracing::{debug, info, warn};

/// NFT listing event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftListingEvent {
    /// NFT collection address
    pub collection_address: String,
    /// Token ID
    pub token_id: String,
    /// Listing price
    pub price: f64,
    /// Currency (e.g., "ETH", "USDT")
    pub currency: String,
    /// Marketplace (e.g., "opensea", "blur", "looksRare")
    pub marketplace: String,
    /// Seller address
    pub seller: String,
    /// Listing timestamp
    pub timestamp: u64,
}

/// NFT mint event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftMintEvent {
    /// NFT collection address
    pub collection_address: String,
    /// Token ID
    pub token_id: String,
    /// Mint price
    pub price: f64,
    /// Currency (e.g., "ETH", "USDT")
    pub currency: String,
    /// Minter address
    pub minter: String,
    /// Mint timestamp
    pub timestamp: u64,
}

/// NFT sale event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSaleEvent {
    /// NFT collection address
    pub collection_address: String,
    /// Token ID
    pub token_id: String,
    /// Sale price
    pub price: f64,
    /// Currency (e.g., "ETH", "USDT")
    pub currency: String,
    /// Marketplace (e.g., "opensea", "blur", "looksRare")
    pub marketplace: String,
    /// Buyer address
    pub buyer: String,
    /// Seller address
    pub seller: String,
    /// Sale timestamp
    pub timestamp: u64,
    /// Transaction hash
    pub tx_hash: String,
}

/// NFT signal detector
pub struct NftSignalDetector {
    /// Chain this detector is monitoring
    chain: ChainRef,
    /// Watched collections
    watched_collections: Vec<String>,
}

impl NftSignalDetector {
    /// Create a new NFT signal detector
    pub fn new(chain: ChainRef, watched_collections: Vec<String>) -> Self {
        Self {
            chain,
            watched_collections,
        }
    }

    /// Process an NFT listing event and generate a signal if it's from a watched collection
    pub fn process_nft_listing(&self, event: NftListingEvent) -> Option<Signal> {
        info!(
            "Processing NFT listing for token {} in collection {} on chain {}",
            event.token_id, event.collection_address, self.chain.name
        );

        // Check if this is from a watched collection
        let is_watched = self.watched_collections.is_empty()
            || self.watched_collections.contains(&event.collection_address);

        if !is_watched {
            debug!("Listing is not from a watched collection, ignoring");
            return None;
        }

        // Create the signal
        let signal = Signal {
            source: "nft".to_string(),
            kind: "nft_listing".to_string(),
            chain: self.chain.clone(),
            token0: Some(event.collection_address.clone()),
            token1: Some(event.token_id.clone()),
            extra: serde_json::json!({
                "collection_address": event.collection_address,
                "token_id": event.token_id,
                "price": event.price,
                "currency": event.currency,
                "marketplace": event.marketplace,
                "seller": event.seller,
                "timestamp": event.timestamp,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated NFT listing signal: {:?}", signal);
        Some(signal)
    }

    /// Process an NFT mint event and generate a signal if it's from a watched collection
    pub fn process_nft_mint(&self, event: NftMintEvent) -> Option<Signal> {
        info!(
            "Processing NFT mint for token {} in collection {} on chain {}",
            event.token_id, event.collection_address, self.chain.name
        );

        // Check if this is from a watched collection
        let is_watched = self.watched_collections.is_empty()
            || self.watched_collections.contains(&event.collection_address);

        if !is_watched {
            debug!("Mint is not from a watched collection, ignoring");
            return None;
        }

        // Create the signal
        let signal = Signal {
            source: "nft".to_string(),
            kind: "nft_mint".to_string(),
            chain: self.chain.clone(),
            token0: Some(event.collection_address.clone()),
            token1: Some(event.token_id.clone()),
            extra: serde_json::json!({
                "collection_address": event.collection_address,
                "token_id": event.token_id,
                "price": event.price,
                "currency": event.currency,
                "minter": event.minter,
                "timestamp": event.timestamp,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated NFT mint signal: {:?}", signal);
        Some(signal)
    }

    /// Process an NFT sale event and generate a signal if it's from a watched collection
    pub fn process_nft_sale(&self, event: NftSaleEvent) -> Option<Signal> {
        info!(
            "Processing NFT sale for token {} in collection {} on chain {}",
            event.token_id, event.collection_address, self.chain.name
        );

        // Check if this is from a watched collection
        let is_watched = self.watched_collections.is_empty()
            || self.watched_collections.contains(&event.collection_address);

        if !is_watched {
            debug!("Sale is not from a watched collection, ignoring");
            return None;
        }

        // Create the signal
        let signal = Signal {
            source: "nft".to_string(),
            kind: "nft_sale".to_string(),
            chain: self.chain.clone(),
            token0: Some(event.collection_address.clone()),
            token1: Some(event.token_id.clone()),
            extra: serde_json::json!({
                "collection_address": event.collection_address,
                "token_id": event.token_id,
                "price": event.price,
                "currency": event.currency,
                "marketplace": event.marketplace,
                "buyer": event.buyer,
                "seller": event.seller,
                "timestamp": event.timestamp,
                "tx_hash": event.tx_hash,
            }),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };

        debug!("Generated NFT sale signal: {:?}", signal);
        Some(signal)
    }

    /// Validate an NFT listing event
    pub fn validate_nft_listing(&self, event: &NftListingEvent) -> bool {
        // Basic validation
        if event.collection_address.is_empty() {
            warn!("Invalid collection address in NFT listing event");
            return false;
        }

        if event.token_id.is_empty() {
            warn!("Invalid token ID in NFT listing event");
            return false;
        }

        if event.price <= 0.0 {
            warn!("Invalid price in NFT listing event");
            return false;
        }

        if event.currency.is_empty() {
            warn!("Invalid currency in NFT listing event");
            return false;
        }

        if event.marketplace.is_empty() {
            warn!("Invalid marketplace in NFT listing event");
            return false;
        }

        if event.seller.is_empty() {
            warn!("Invalid seller in NFT listing event");
            return false;
        }

        if event.timestamp == 0 {
            warn!("Invalid timestamp in NFT listing event");
            return false;
        }

        true
    }

    /// Validate an NFT mint event
    pub fn validate_nft_mint(&self, event: &NftMintEvent) -> bool {
        // Basic validation
        if event.collection_address.is_empty() {
            warn!("Invalid collection address in NFT mint event");
            return false;
        }

        if event.token_id.is_empty() {
            warn!("Invalid token ID in NFT mint event");
            return false;
        }

        if event.price < 0.0 {
            warn!("Invalid price in NFT mint event");
            return false;
        }

        if event.currency.is_empty() {
            warn!("Invalid currency in NFT mint event");
            return false;
        }

        if event.minter.is_empty() {
            warn!("Invalid minter in NFT mint event");
            return false;
        }

        if event.timestamp == 0 {
            warn!("Invalid timestamp in NFT mint event");
            return false;
        }

        true
    }

    /// Validate an NFT sale event
    pub fn validate_nft_sale(&self, event: &NftSaleEvent) -> bool {
        // Basic validation
        if event.collection_address.is_empty() {
            warn!("Invalid collection address in NFT sale event");
            return false;
        }

        if event.token_id.is_empty() {
            warn!("Invalid token ID in NFT sale event");
            return false;
        }

        if event.price <= 0.0 {
            warn!("Invalid price in NFT sale event");
            return false;
        }

        if event.currency.is_empty() {
            warn!("Invalid currency in NFT sale event");
            return false;
        }

        if event.marketplace.is_empty() {
            warn!("Invalid marketplace in NFT sale event");
            return false;
        }

        if event.buyer.is_empty() {
            warn!("Invalid buyer in NFT sale event");
            return false;
        }

        if event.seller.is_empty() {
            warn!("Invalid seller in NFT sale event");
            return false;
        }

        if event.tx_hash.is_empty() {
            warn!("Invalid transaction hash in NFT sale event");
            return false;
        }

        if event.timestamp == 0 {
            warn!("Invalid timestamp in NFT sale event");
            return false;
        }

        true
    }

    /// Add a collection to the watch list
    pub fn add_watched_collection(&mut self, collection_address: String) {
        self.watched_collections.push(collection_address);
    }

    /// Remove a collection from the watch list
    pub fn remove_watched_collection(&mut self, collection_address: &str) {
        self.watched_collections.retain(|c| c != collection_address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_nft_signal_detector_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let watched_collections = vec!["0x1234567890123456789012345678901234567890".to_string()];
        let detector = NftSignalDetector::new(chain.clone(), watched_collections.clone());
        assert_eq!(detector.chain.name, "ethereum");
        assert_eq!(detector.chain.id, 1);
        assert_eq!(detector.watched_collections, watched_collections);
    }

    #[test]
    fn test_nft_listing_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = NftSignalDetector::new(chain, vec![]);

        // Valid listing
        let valid_listing = NftListingEvent {
            collection_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "123".to_string(),
            price: 1.5,
            currency: "ETH".to_string(),
            marketplace: "opensea".to_string(),
            seller: "0x1234567890123456789012345678901234567891".to_string(),
            timestamp: 1234567890,
        };

        assert!(detector.validate_nft_listing(&valid_listing));

        // Invalid listing - empty collection address
        let mut invalid_listing = valid_listing.clone();
        invalid_listing.collection_address = String::new();
        assert!(!detector.validate_nft_listing(&invalid_listing));
    }

    #[test]
    fn test_nft_mint_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = NftSignalDetector::new(chain, vec![]);

        // Valid mint
        let valid_mint = NftMintEvent {
            collection_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "123".to_string(),
            price: 0.0, // Mint can be free
            currency: "ETH".to_string(),
            minter: "0x1234567890123456789012345678901234567891".to_string(),
            timestamp: 1234567890,
        };

        assert!(detector.validate_nft_mint(&valid_mint));

        // Invalid mint - negative price
        let mut invalid_mint = valid_mint.clone();
        invalid_mint.price = -0.1;
        assert!(!detector.validate_nft_mint(&invalid_mint));
    }

    #[test]
    fn test_nft_sale_validation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = NftSignalDetector::new(chain, vec![]);

        // Valid sale
        let valid_sale = NftSaleEvent {
            collection_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "123".to_string(),
            price: 1.5,
            currency: "ETH".to_string(),
            marketplace: "opensea".to_string(),
            buyer: "0x1234567890123456789012345678901234567892".to_string(),
            seller: "0x1234567890123456789012345678901234567891".to_string(),
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            timestamp: 1234567890,
        };

        assert!(detector.validate_nft_sale(&valid_sale));

        // Invalid sale - zero price
        let mut invalid_sale = valid_sale.clone();
        invalid_sale.price = 0.0;
        assert!(!detector.validate_nft_sale(&invalid_sale));
    }

    #[test]
    fn test_nft_signal_generation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let detector = NftSignalDetector::new(
            chain,
            vec!["0x1234567890123456789012345678901234567890".to_string()],
        );

        // Listing event
        let listing_event = NftListingEvent {
            collection_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "123".to_string(),
            price: 1.5,
            currency: "ETH".to_string(),
            marketplace: "opensea".to_string(),
            seller: "0x1234567890123456789012345678901234567891".to_string(),
            timestamp: 1234567890,
        };

        let signal = detector.process_nft_listing(listing_event);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.source, "nft");
        assert_eq!(signal.kind, "nft_listing");
        assert_eq!(signal.chain.name, "ethereum");
        assert_eq!(
            signal.token0,
            Some("0x1234567890123456789012345678901234567890".to_string())
        );
        assert_eq!(signal.token1, Some("123".to_string()));
        assert!(signal.seen_at_ms > 0);

        // Mint event
        let mint_event = NftMintEvent {
            collection_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "124".to_string(),
            price: 0.0,
            currency: "ETH".to_string(),
            minter: "0x1234567890123456789012345678901234567891".to_string(),
            timestamp: 1234567891,
        };

        let signal = detector.process_nft_mint(mint_event);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.source, "nft");
        assert_eq!(signal.kind, "nft_mint");
        assert_eq!(signal.chain.name, "ethereum");
        assert_eq!(
            signal.token0,
            Some("0x1234567890123456789012345678901234567890".to_string())
        );
        assert_eq!(signal.token1, Some("124".to_string()));
        assert!(signal.seen_at_ms > 0);

        // Sale event
        let sale_event = NftSaleEvent {
            collection_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "125".to_string(),
            price: 2.0,
            currency: "ETH".to_string(),
            marketplace: "opensea".to_string(),
            buyer: "0x1234567890123456789012345678901234567892".to_string(),
            seller: "0x1234567890123456789012345678901234567891".to_string(),
            tx_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            timestamp: 1234567892,
        };

        let signal = detector.process_nft_sale(sale_event);
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.source, "nft");
        assert_eq!(signal.kind, "nft_sale");
        assert_eq!(signal.chain.name, "ethereum");
        assert_eq!(
            signal.token0,
            Some("0x1234567890123456789012345678901234567890".to_string())
        );
        assert_eq!(signal.token1, Some("125".to_string()));
        assert!(signal.seen_at_ms > 0);

        // Event from unwatched collection
        let unwatched_listing = NftListingEvent {
            collection_address: "0x1234567890123456789012345678901234567899".to_string(),
            token_id: "999".to_string(),
            price: 1.0,
            currency: "ETH".to_string(),
            marketplace: "opensea".to_string(),
            seller: "0x1234567890123456789012345678901234567891".to_string(),
            timestamp: 1234567893,
        };

        let signal = detector.process_nft_listing(unwatched_listing);
        assert!(signal.is_none());
    }

    #[test]
    fn test_watched_collection_management() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };

        let mut detector = NftSignalDetector::new(chain, vec![]);

        // Add a collection
        detector.add_watched_collection("0x1234567890123456789012345678901234567890".to_string());
        assert_eq!(detector.watched_collections.len(), 1);
        assert_eq!(
            detector.watched_collections[0],
            "0x1234567890123456789012345678901234567890"
        );

        // Remove a collection
        detector.remove_watched_collection("0x1234567890123456789012345678901234567890");
        assert_eq!(detector.watched_collections.len(), 0);
    }
}
