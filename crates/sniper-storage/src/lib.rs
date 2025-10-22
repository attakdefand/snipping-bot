//! Storage module for the sniper bot.
//!
//! This module provides functionality for database storage, position tracking,
//! distributed locks, and idempotency mechanisms.

pub mod outbox;
pub mod redis_locks;
pub mod repo_positions;
pub mod repo_runs;
pub mod repo_trades;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

/// Database connection enum
pub enum Database {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
    pub redis_url: String,
}

/// Main storage system
pub struct Storage {
    // In a real implementation, this would contain database and Redis connections
}

impl Storage {
    /// Create a new storage system
    pub async fn new(_config: StorageConfig) -> Result<Self> {
        // In a real implementation, this would initialize database and Redis connections
        Ok(Self {})
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        // In a real implementation, this would run database migrations
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config() {
        let config = StorageConfig {
            database_url: "sqlite://test.db".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
        };

        assert_eq!(config.database_url, "sqlite://test.db");
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
    }
}
