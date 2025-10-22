//! Trade repository for the sniper bot.
//!
//! This module provides functionality for storing and retrieving trade information.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Trade status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeStatus {
    Pending,
    Executed,
    Failed,
    Cancelled,
}

/// Trade entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub user_id: String,
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub price: f64,
    pub amount: f64,
    pub status: TradeStatus,
    pub created_at: u64,          // Unix timestamp
    pub executed_at: Option<u64>, // Unix timestamp
    pub tx_hash: Option<String>,
}

/// In-memory trade repository for demonstration
/// In a real implementation, this would use a database
pub struct TradeRepo {
    trades: Arc<RwLock<HashMap<Uuid, Trade>>>,
}

impl TradeRepo {
    /// Create a new trade repository
    pub fn new() -> Self {
        Self {
            trades: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new trade
    pub async fn create(&self, trade: &Trade) -> Result<Uuid> {
        let mut trades = self.trades.write().await;
        trades.insert(trade.id, trade.clone());
        Ok(trade.id)
    }

    /// Get a trade by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Trade>> {
        let trades = self.trades.read().await;
        Ok(trades.get(&id).cloned())
    }

    /// Update a trade
    pub async fn update(&self, trade: &Trade) -> Result<()> {
        let mut trades = self.trades.write().await;
        trades.insert(trade.id, trade.clone());
        Ok(())
    }

    /// List trades for a user
    pub async fn list_trades(&self, user_id: &str, limit: usize) -> Result<Vec<Trade>> {
        let trades = self.trades.read().await;
        let mut result = Vec::new();

        for trade in trades.values() {
            if trade.user_id == user_id {
                result.push(trade.clone());
                if result.len() >= limit {
                    break;
                }
            }
        }

        Ok(result)
    }
}

impl Default for TradeRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trade_repo() -> Result<()> {
        let repo = TradeRepo::new();

        let trade = Trade {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: "buy".to_string(),
            price: 50000.0,
            amount: 1.0,
            status: TradeStatus::Pending,
            created_at: 1234567890,
            executed_at: None,
            tx_hash: None,
        };

        // Test create
        let id = repo.create(&trade).await?;
        assert_eq!(id, trade.id);

        // Test get by id
        let retrieved = repo.get_by_id(id).await?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.user_id, "user1");
        assert_eq!(retrieved.symbol, "BTC/USDT");
        assert_eq!(retrieved.side, "buy");
        assert_eq!(retrieved.price, 50000.0);

        // Test list trades
        let trades = repo.list_trades("user1", 10).await?;
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].id, id);

        Ok(())
    }
}
