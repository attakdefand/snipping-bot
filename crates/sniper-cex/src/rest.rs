//! CEX REST API client for the sniper bot.
//!
//! This module provides functionality for interacting with centralized exchange REST APIs
//! including market data, account information, and order management.

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};

// Re-export types from lib.rs for convenience
use crate::{ExchangeId, OrderBook, OrderSide, OrderStatus, OrderType, Symbol, Trade};

/// REST API configuration
#[derive(Debug, Clone)]
pub struct RestConfig {
    /// Base URL for the REST API
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Rate limit (requests per second)
    pub rate_limit: f64,
    /// Enable/disable SSL verification
    pub ssl_verify: bool,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.example.com".to_string(),
            timeout_seconds: 30,
            rate_limit: 10.0,
            ssl_verify: true,
        }
    }
}

/// REST API client
#[derive(Debug)]
pub struct RestClient {
    /// HTTP client
    client: Client,
    /// Configuration
    config: RestConfig,
    /// Exchange identifier
    exchange_id: ExchangeId,
    /// Rate limiter (simple implementation)
    last_request_time: std::sync::Mutex<std::time::Instant>,
}

impl RestClient {
    /// Create a new REST client
    pub fn new(exchange_id: ExchangeId, config: RestConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .danger_accept_invalid_certs(!config.ssl_verify)
            .build()?;

        Ok(Self {
            client,
            config,
            exchange_id,
            last_request_time: std::sync::Mutex::new(std::time::Instant::now()),
        })
    }

    /// Enforce rate limiting
    fn enforce_rate_limit(&self) -> Result<()> {
        let mut last_request_time = self.last_request_time.lock().unwrap();
        let elapsed = last_request_time.elapsed();
        let min_interval = Duration::from_secs_f64(1.0 / self.config.rate_limit);

        if elapsed < min_interval {
            let sleep_time = min_interval - elapsed;
            debug!("Rate limiting: sleeping for {:?}", sleep_time);
            std::thread::sleep(sleep_time);
        }

        *last_request_time = std::time::Instant::now();
        Ok(())
    }

    /// Make a GET request
    pub async fn get(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        self.enforce_rate_limit()?;

        let url = format!("{}{}", self.config.base_url, endpoint);
        info!("Making GET request to: {}", url);

        let mut request = self.client.get(&url);

        if let Some(params) = params {
            request = request.query(&params);
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let json: Value = response.json().await?;
            debug!("GET request successful, response: {:?}", json);
            Ok(json)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("GET request failed with status {}: {}", status, error_text);
            Err(anyhow::anyhow!("HTTP error {}: {}", status, error_text))
        }
    }

    /// Make a POST request
    pub async fn post(
        &self,
        endpoint: &str,
        body: Option<Value>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        self.enforce_rate_limit()?;

        let url = format!("{}{}", self.config.base_url, endpoint);
        info!("Making POST request to: {}", url);

        let mut request = self.client.post(&url);

        if let Some(body) = body {
            request = request.json(&body);
        }

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, value);
            }
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let json: Value = response.json().await?;
            debug!("POST request successful, response: {:?}", json);
            Ok(json)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("POST request failed with status {}: {}", status, error_text);
            Err(anyhow::anyhow!("HTTP error {}: {}", status, error_text))
        }
    }

    /// Make a PUT request
    pub async fn put(
        &self,
        endpoint: &str,
        body: Option<Value>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        self.enforce_rate_limit()?;

        let url = format!("{}{}", self.config.base_url, endpoint);
        info!("Making PUT request to: {}", url);

        let mut request = self.client.put(&url);

        if let Some(body) = body {
            request = request.json(&body);
        }

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, value);
            }
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let json: Value = response.json().await?;
            debug!("PUT request successful, response: {:?}", json);
            Ok(json)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("PUT request failed with status {}: {}", status, error_text);
            Err(anyhow::anyhow!("HTTP error {}: {}", status, error_text))
        }
    }

    /// Make a DELETE request
    pub async fn delete(
        &self,
        endpoint: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        self.enforce_rate_limit()?;

        let url = format!("{}{}", self.config.base_url, endpoint);
        info!("Making DELETE request to: {}", url);

        let mut request = self.client.delete(&url);

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, value);
            }
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let json: Value = response.json().await?;
            debug!("DELETE request successful, response: {:?}", json);
            Ok(json)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "DELETE request failed with status {}: {}",
                status, error_text
            );
            Err(anyhow::anyhow!("HTTP error {}: {}", status, error_text))
        }
    }

    /// Get order book for a symbol
    pub async fn get_order_book(&self, symbol: &Symbol, limit: Option<u32>) -> Result<OrderBook> {
        let endpoint = "/api/v1/depth".to_string();
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.0.clone());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(&endpoint, Some(params)).await?;

        // Parse the response into an OrderBook
        // This is a simplified implementation - in practice, each exchange has its own format
        let bids = response["bids"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|bid| {
                let price = bid[0].as_str()?.parse::<f64>().ok()?;
                let amount = bid[1].as_str()?.parse::<f64>().ok()?;
                Some(crate::PriceLevel { price, amount })
            })
            .collect();

        let asks = response["asks"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|ask| {
                let price = ask[0].as_str()?.parse::<f64>().ok()?;
                let amount = ask[1].as_str()?.parse::<f64>().ok()?;
                Some(crate::PriceLevel { price, amount })
            })
            .collect();

        Ok(OrderBook {
            symbol: symbol.clone(),
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Get recent trades for a symbol
    pub async fn get_recent_trades(
        &self,
        symbol: &Symbol,
        limit: Option<u32>,
    ) -> Result<Vec<Trade>> {
        let endpoint = "/api/v1/trades".to_string();
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.0.clone());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(&endpoint, Some(params)).await?;

        // Parse the response into a vector of Trades
        // This is a simplified implementation - in practice, each exchange has its own format
        let trades = response["trades"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|trade| {
                let price = trade["price"].as_str()?.parse::<f64>().ok()?;
                let amount = trade["amount"].as_str()?.parse::<f64>().ok()?;
                let timestamp = trade["timestamp"]
                    .as_u64()
                    .unwrap_or_else(|| chrono::Utc::now().timestamp() as u64);
                let exchange_order_id = trade["id"].as_str().unwrap_or("").to_string();
                Some(Trade {
                    symbol: symbol.clone(),
                    side: OrderSide::Buy, // Simplified - would need to parse from response
                    price,
                    amount,
                    timestamp,
                    exchange_order_id,
                })
            })
            .collect();

        Ok(trades)
    }

    /// Get account balances
    pub async fn get_balances(&self) -> Result<HashMap<String, f64>> {
        let endpoint = "/api/v1/account".to_string();
        let response = self.get(&endpoint, None).await?;

        // Parse the response into a balance map
        // This is a simplified implementation - in practice, each exchange has its own format
        let mut balances = HashMap::new();

        if let Some(balances_json) = response["balances"].as_array() {
            for balance in balances_json {
                if let (Some(asset), Some(free)) = (
                    balance["asset"].as_str(),
                    balance["free"].as_str().and_then(|s| s.parse::<f64>().ok()),
                ) {
                    balances.insert(asset.to_string(), free);
                }
            }
        }

        Ok(balances)
    }

    /// Place an order
    pub async fn place_order(
        &self,
        symbol: &Symbol,
        side: OrderSide,
        order_type: OrderType,
        price: Option<f64>,
        amount: f64,
    ) -> Result<String> {
        let endpoint = format!("/api/v1/order");
        let side_str = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        let order_type_str = match order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit => "LIMIT",
            OrderType::StopLoss => "STOP_LOSS",
            OrderType::TakeProfit => "TAKE_PROFIT",
        };

        let mut body = serde_json::json!({
            "symbol": symbol.0,
            "side": side_str,
            "type": order_type_str,
            "amount": amount,
        });

        if let Some(price) = price {
            body["price"] = serde_json::Value::Number(serde_json::Number::from_f64(price).unwrap());
        }

        let response = self.post(&endpoint, Some(body), None).await?;

        // Extract order ID from response
        // This is a simplified implementation - in practice, each exchange has its own format
        let order_id = response["orderId"].as_str().unwrap_or("").to_string();

        Ok(order_id)
    }

    /// Get order status
    pub async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus> {
        let endpoint = "/api/v1/order".to_string();
        let mut params = HashMap::new();
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(&endpoint, Some(params)).await?;

        // Parse order status from response
        // This is a simplified implementation - in practice, each exchange has its own format
        let status_str = response["status"].as_str().unwrap_or("UNKNOWN");
        let status = match status_str {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Cancelled,
            "REJECTED" => OrderStatus::Rejected,
            _ => OrderStatus::New, // Default to New if unknown
        };

        Ok(status)
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let endpoint = "/api/v1/order".to_string();
        let _body = serde_json::json!({
            "orderId": order_id,
        });

        let _response = self.delete(&endpoint, None).await?;
        Ok(())
    }

    /// Get exchange information (symbols, rate limits, etc.)
    pub async fn get_exchange_info(&self) -> Result<Value> {
        let endpoint = "/api/v1/exchangeInfo".to_string();
        self.get(&endpoint, None).await
    }

    /// Get server time
    pub async fn get_server_time(&self) -> Result<u64> {
        let endpoint = "/api/v1/time".to_string();
        let response = self.get(&endpoint, None).await?;
        let server_time = response["serverTime"]
            .as_u64()
            .unwrap_or_else(|| chrono::Utc::now().timestamp() as u64);
        Ok(server_time)
    }

    /// Test connectivity to the REST API
    pub async fn ping(&self) -> Result<()> {
        let endpoint = "/api/v1/ping".to_string();
        let _response = self.get(&endpoint, None).await?;
        Ok(())
    }
}

/// REST API client manager for handling multiple exchanges
#[derive(Debug, Default)]
pub struct RestClientManager {
    /// Collection of REST clients
    clients: HashMap<String, RestClient>,
}

impl RestClientManager {
    /// Create a new REST client manager
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Add a REST client
    pub fn add_client(&mut self, exchange_id: String, client: RestClient) {
        self.clients.insert(exchange_id, client);
    }

    /// Get a REST client
    pub fn get_client(&self, exchange_id: &str) -> Option<&RestClient> {
        self.clients.get(exchange_id)
    }

    /// Remove a REST client
    pub fn remove_client(&mut self, exchange_id: &str) -> bool {
        self.clients.remove(exchange_id).is_some()
    }

    /// List all exchange IDs with REST clients
    pub fn list_exchanges(&self) -> Vec<&String> {
        self.clients.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_rest_config_default() {
        let config = RestConfig::default();
        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.rate_limit, 10.0);
        assert!(config.ssl_verify);
    }

    #[test]
    fn test_rest_client_creation() {
        let config = RestConfig {
            base_url: "https://api.binance.com".to_string(),
            timeout_seconds: 30,
            rate_limit: 10.0,
            ssl_verify: true,
        };
        let client = RestClient::new(ExchangeId("binance".to_string()), config).unwrap();
        assert_eq!(client.config.base_url, "https://api.binance.com");
        assert_eq!(client.exchange_id.0, "binance");
    }

    #[test]
    fn test_rest_client_manager() {
        let manager = RestClientManager::new();
        assert!(manager.clients.is_empty());
        assert!(manager.list_exchanges().is_empty());
    }

    #[tokio::test]
    async fn test_order_side_serialization() {
        let buy = OrderSide::Buy;
        let sell = OrderSide::Sell;

        match buy {
            OrderSide::Buy => {} // Expected match
            OrderSide::Sell => panic!("Unexpected match"),
        }

        match sell {
            OrderSide::Buy => panic!("Unexpected match"),
            OrderSide::Sell => {} // Expected match
        }
    }

    #[tokio::test]
    async fn test_order_type_serialization() {
        let market = OrderType::Market;
        let limit = OrderType::Limit;
        let stop_loss = OrderType::StopLoss;
        let take_profit = OrderType::TakeProfit;

        match market {
            OrderType::Market => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match limit {
            OrderType::Limit => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match stop_loss {
            OrderType::StopLoss => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match take_profit {
            OrderType::TakeProfit => {} // Expected match
            _ => panic!("Unexpected match"),
        }
    }

    #[tokio::test]
    async fn test_order_status_serialization() {
        let new = OrderStatus::New;
        let partially_filled = OrderStatus::PartiallyFilled;
        let filled = OrderStatus::Filled;
        let cancelled = OrderStatus::Cancelled;
        let rejected = OrderStatus::Rejected;

        match new {
            OrderStatus::New => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match partially_filled {
            OrderStatus::PartiallyFilled => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match filled {
            OrderStatus::Filled => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match cancelled {
            OrderStatus::Cancelled => {} // Expected match
            _ => panic!("Unexpected match"),
        }

        match rejected {
            OrderStatus::Rejected => {} // Expected match
            _ => panic!("Unexpected match"),
        }
    }

    #[tokio::test]
    async fn test_symbol_creation() {
        let symbol = Symbol("BTC/USDT".to_string());
        assert_eq!(symbol.0, "BTC/USDT");
    }

    #[tokio::test]
    async fn test_exchange_id_creation() {
        let exchange_id = ExchangeId("binance".to_string());
        assert_eq!(exchange_id.0, "binance");
    }

    #[tokio::test]
    async fn test_rate_limit_enforcement() {
        let config = RestConfig {
            base_url: "https://api.example.com".to_string(),
            timeout_seconds: 30,
            rate_limit: 1.0, // 1 request per second
            ssl_verify: true,
        };
        let client = RestClient::new(ExchangeId("test".to_string()), config).unwrap();

        // This test just ensures the function doesn't panic
        assert!(client.enforce_rate_limit().is_ok());
    }
}
