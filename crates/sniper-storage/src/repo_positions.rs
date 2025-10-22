//! Position repository for the sniper bot.
//!
//! This module provides functionality for storing and retrieving position information.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Position status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    Liquidated,
}

/// Position entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub user_id: String,
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub amount: f64,
    pub leverage: f64,
    pub status: PositionStatus,
    pub created_at: u64,        // Unix timestamp
    pub updated_at: u64,        // Unix timestamp
    pub closed_at: Option<u64>, // Unix timestamp
}

/// In-memory position repository for demonstration
/// In a real implementation, this would use a database
pub struct PositionRepo {
    positions: Arc<RwLock<HashMap<Uuid, Position>>>,
}

impl PositionRepo {
    /// Create a new position repository
    pub fn new() -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new position
    pub async fn create(&self, position: &Position) -> Result<Uuid> {
        let mut positions = self.positions.write().await;
        positions.insert(position.id, position.clone());
        Ok(position.id)
    }

    /// Get a position by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Position>> {
        let positions = self.positions.read().await;
        Ok(positions.get(&id).cloned())
    }

    /// Update a position
    pub async fn update(&self, position: &Position) -> Result<()> {
        let mut positions = self.positions.write().await;
        positions.insert(position.id, position.clone());
        Ok(())
    }

    /// List open positions for a user
    pub async fn list_open_positions(&self, user_id: &str) -> Result<Vec<Position>> {
        let positions = self.positions.read().await;
        let mut result = Vec::new();

        for position in positions.values() {
            if position.user_id == user_id && matches!(position.status, PositionStatus::Open) {
                result.push(position.clone());
            }
        }

        Ok(result)
    }
}

impl Default for PositionRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_position_repo() -> Result<()> {
        let repo = PositionRepo::new();

        let position = Position {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: "buy".to_string(),
            entry_price: 50000.0,
            exit_price: None,
            amount: 1.0,
            leverage: 10.0,
            status: PositionStatus::Open,
            created_at: 1234567890,
            updated_at: 1234567890,
            closed_at: None,
        };

        // Test create
        let id = repo.create(&position).await?;
        assert_eq!(id, position.id);

        // Test get by id
        let retrieved = repo.get_by_id(id).await?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.user_id, "user1");
        assert_eq!(retrieved.symbol, "BTC/USDT");
        assert_eq!(retrieved.side, "buy");
        assert_eq!(retrieved.entry_price, 50000.0);

        // Test list open positions
        let open_positions = repo.list_open_positions("user1").await?;
        assert_eq!(open_positions.len(), 1);
        assert_eq!(open_positions[0].id, id);

        Ok(())
    }
}
