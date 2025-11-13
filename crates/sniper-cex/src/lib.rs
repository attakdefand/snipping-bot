//! CEX (Centralized Exchange) integration module for the sniper bot.
//!
//! This module provides functionality for interacting with centralized exchanges
//! including REST APIs and WebSocket feeds for price data and order management.

pub mod auth;
pub mod rest;
pub mod ws;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// CEX exchange identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeId(pub String);

/// Market symbol (e.g., "BTC/USDT")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol(pub String);

/// Order side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub amount: f64,
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: u64,
}

/// Trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub price: f64,
    pub amount: f64,
    pub timestamp: u64,
    pub exchange_order_id: String,
}

/// CEX client trait that all exchange implementations should implement
#[async_trait]
pub trait CexClient {
    /// Get order book for a symbol
    async fn get_order_book(&self, symbol: &Symbol) -> Result<OrderBook>;

    /// Place an order
    async fn place_order(
        &self,
        symbol: &Symbol,
        side: OrderSide,
        order_type: OrderType,
        price: Option<f64>,
        amount: f64,
    ) -> Result<String>; // Returns order ID

    /// Get order status
    async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus>;

    /// Cancel an order
    async fn cancel_order(&self, order_id: &str) -> Result<()>;

    /// Get account balances
    async fn get_balances(&self) -> Result<std::collections::HashMap<String, f64>>;
}

/// Main CEX client that can connect to different exchanges
pub struct Client {
    exchange_id: ExchangeId,
    api_key: String,
    api_secret: String,
    rest_endpoint: String,
    ws_endpoint: String,
}

impl Client {
    /// Create a new CEX client
    pub fn new(
        exchange_id: ExchangeId,
        api_key: String,
        api_secret: String,
        rest_endpoint: String,
        ws_endpoint: String,
    ) -> Self {
        Self {
            exchange_id,
            api_key,
            api_secret,
            rest_endpoint,
            ws_endpoint,
        }
    }

    /// Get exchange identifier
    pub fn exchange_id(&self) -> &ExchangeId {
        &self.exchange_id
    }

    /// Get REST endpoint
    pub fn rest_endpoint(&self) -> &str {
        &self.rest_endpoint
    }

    /// Get WebSocket endpoint
    pub fn ws_endpoint(&self) -> &str {
        &self.ws_endpoint
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get API secret
    pub fn api_secret(&self) -> &str {
        &self.api_secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new(
            ExchangeId("binance".to_string()),
            "api_key".to_string(),
            "api_secret".to_string(),
            "https://api.binance.com".to_string(),
            "wss://stream.binance.com:9443".to_string(),
        );

        assert_eq!(client.exchange_id().0, "binance");
        assert_eq!(client.rest_endpoint(), "https://api.binance.com");
        assert_eq!(client.ws_endpoint(), "wss://stream.binance.com:9443");
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol("BTC/USDT".to_string());
        assert_eq!(symbol.0, "BTC/USDT");
    }

    #[test]
    fn test_order_side() {
        let buy = OrderSide::Buy;
        let sell = OrderSide::Sell;

        match buy {
            OrderSide::Buy => {} // Expected match
            OrderSide::Sell => panic!("Expected buy order"),
        }

        match sell {
            OrderSide::Buy => panic!("Expected sell order"),
            OrderSide::Sell => {} // Expected match
        }
    }
}
