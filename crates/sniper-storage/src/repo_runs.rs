//! Repository for Trade Runs
//!
//! This module provides functionality for storing and retrieving trade run data,
//! including run metadata, performance metrics, and execution statistics.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

/// Trade run entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRun {
    /// Unique identifier for the run
    pub id: String,
    /// Run name/description
    pub name: String,
    /// Strategy used for this run
    pub strategy: String,
    /// Start time of the run
    pub start_time: u64,
    /// End time of the run (None if still running)
    pub end_time: Option<u64>,
    /// Status of the run
    pub status: RunStatus,
    /// Total profit/loss in USD
    pub total_pnl: f64,
    /// Number of successful trades
    pub successful_trades: usize,
    /// Number of failed trades
    pub failed_trades: usize,
    /// Maximum drawdown percentage
    pub max_drawdown: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Tags associated with this run
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Status of a trade run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    /// Run is currently executing
    Running,
    /// Run completed successfully
    Completed,
    /// Run was cancelled
    Cancelled,
    /// Run failed
    Failed,
}

/// Trade run creation parameters
#[derive(Debug, Clone)]
pub struct CreateTradeRunParams {
    /// Run name/description
    pub name: String,
    /// Strategy used for this run
    pub strategy: String,
    /// Tags associated with this run
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Trade run update parameters
#[derive(Debug, Clone)]
pub struct UpdateTradeRunParams {
    /// End time of the run
    pub end_time: Option<u64>,
    /// Status of the run
    pub status: Option<RunStatus>,
    /// Total profit/loss in USD
    pub total_pnl: Option<f64>,
    /// Number of successful trades
    pub successful_trades: Option<usize>,
    /// Number of failed trades
    pub failed_trades: Option<usize>,
    /// Maximum drawdown percentage
    pub max_drawdown: Option<f64>,
    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,
}

/// Repository for trade runs
pub struct TradeRunRepository {
    /// In-memory storage for trade runs
    runs: HashMap<String, TradeRun>,
}

impl TradeRunRepository {
    /// Create a new trade run repository
    pub fn new() -> Self {
        Self {
            runs: HashMap::new(),
        }
    }

    /// Create a new trade run
    pub fn create_run(&mut self, params: CreateTradeRunParams) -> Result<TradeRun> {
        let id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let run = TradeRun {
            id: id.clone(),
            name: params.name,
            strategy: params.strategy,
            start_time: now,
            end_time: None,
            status: RunStatus::Running,
            total_pnl: 0.0,
            successful_trades: 0,
            failed_trades: 0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            tags: params.tags,
            metadata: params.metadata,
        };

        self.runs.insert(id, run.clone());
        info!("Created new trade run: {} ({})", run.name, run.id);
        Ok(run)
    }

    /// Get a trade run by ID
    pub fn get_run(&self, id: &str) -> Option<&TradeRun> {
        self.runs.get(id)
    }

    /// Update a trade run
    pub fn update_run(
        &mut self,
        id: &str,
        params: UpdateTradeRunParams,
    ) -> Result<Option<TradeRun>> {
        match self.runs.get_mut(id) {
            Some(run) => {
                if let Some(end_time) = params.end_time {
                    run.end_time = Some(end_time);
                }
                if let Some(status) = params.status {
                    run.status = status;
                }
                if let Some(total_pnl) = params.total_pnl {
                    run.total_pnl = total_pnl;
                }
                if let Some(successful_trades) = params.successful_trades {
                    run.successful_trades = successful_trades;
                }
                if let Some(failed_trades) = params.failed_trades {
                    run.failed_trades = failed_trades;
                }
                if let Some(max_drawdown) = params.max_drawdown {
                    run.max_drawdown = max_drawdown;
                }
                if let Some(sharpe_ratio) = params.sharpe_ratio {
                    run.sharpe_ratio = sharpe_ratio;
                }

                info!("Updated trade run: {} ({})", run.name, run.id);
                Ok(Some(run.clone()))
            }
            None => {
                warn!("Attempted to update non-existent trade run: {}", id);
                Ok(None)
            }
        }
    }

    /// List all trade runs
    pub fn list_runs(&self) -> Vec<&TradeRun> {
        self.runs.values().collect()
    }

    /// List trade runs by strategy
    pub fn list_runs_by_strategy(&self, strategy: &str) -> Vec<&TradeRun> {
        self.runs
            .values()
            .filter(|run| run.strategy == strategy)
            .collect()
    }

    /// List trade runs by status
    pub fn list_runs_by_status(&self, status: RunStatus) -> Vec<&TradeRun> {
        self.runs
            .values()
            .filter(|run| run.status == status)
            .collect()
    }

    /// List trade runs by tag
    pub fn list_runs_by_tag(&self, tag: &str) -> Vec<&TradeRun> {
        self.runs
            .values()
            .filter(|run| run.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Delete a trade run
    pub fn delete_run(&mut self, id: &str) -> Result<bool> {
        match self.runs.remove(id) {
            Some(run) => {
                info!("Deleted trade run: {} ({})", run.name, run.id);
                Ok(true)
            }
            None => {
                warn!("Attempted to delete non-existent trade run: {}", id);
                Ok(false)
            }
        }
    }

    /// Get the total count of trade runs
    pub fn count_runs(&self) -> usize {
        self.runs.len()
    }

    /// Get statistics about trade runs
    pub fn get_statistics(&self) -> RunStatistics {
        let total_runs = self.runs.len();
        let running_runs = self.list_runs_by_status(RunStatus::Running).len();
        let completed_runs = self.list_runs_by_status(RunStatus::Completed).len();
        let failed_runs = self.list_runs_by_status(RunStatus::Failed).len();
        let cancelled_runs = self.list_runs_by_status(RunStatus::Cancelled).len();

        // Calculate average PnL for completed runs
        let completed_runs_list = self.list_runs_by_status(RunStatus::Completed);
        let total_pnl: f64 = completed_runs_list.iter().map(|run| run.total_pnl).sum();
        let avg_pnl = if completed_runs > 0 {
            total_pnl / (completed_runs as f64)
        } else {
            0.0
        };

        RunStatistics {
            total_runs,
            running_runs,
            completed_runs,
            failed_runs,
            cancelled_runs,
            average_pnl: avg_pnl,
        }
    }
}

/// Statistics about trade runs
#[derive(Debug, Clone)]
pub struct RunStatistics {
    /// Total number of runs
    pub total_runs: usize,
    /// Number of running runs
    pub running_runs: usize,
    /// Number of completed runs
    pub completed_runs: usize,
    /// Number of failed runs
    pub failed_runs: usize,
    /// Number of cancelled runs
    pub cancelled_runs: usize,
    /// Average profit/loss for completed runs
    pub average_pnl: f64,
}

impl Default for TradeRunRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_repository_creation() {
        let repo = TradeRunRepository::new();
        assert_eq!(repo.count_runs(), 0);
    }

    #[test]
    fn test_create_run() {
        let mut repo = TradeRunRepository::new();
        let params = CreateTradeRunParams {
            name: "Test Run".to_string(),
            strategy: "Test Strategy".to_string(),
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
        };

        let run = repo.create_run(params).unwrap();
        assert_eq!(run.name, "Test Run");
        assert_eq!(run.strategy, "Test Strategy");
        assert_eq!(run.status, RunStatus::Running);
        assert_eq!(run.tags, vec!["test".to_string()]);
        assert!(run.start_time > 0);
        assert_eq!(run.end_time, None);
        assert_eq!(repo.count_runs(), 1);
    }

    #[test]
    fn test_get_run() {
        let mut repo = TradeRunRepository::new();
        let params = CreateTradeRunParams {
            name: "Test Run".to_string(),
            strategy: "Test Strategy".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };

        let run = repo.create_run(params).unwrap();
        let retrieved = repo.get_run(&run.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, run.id);
    }

    #[test]
    fn test_update_run() {
        let mut repo = TradeRunRepository::new();
        let params = CreateTradeRunParams {
            name: "Test Run".to_string(),
            strategy: "Test Strategy".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };

        let run = repo.create_run(params).unwrap();

        let update_params = UpdateTradeRunParams {
            end_time: Some(1234567890),
            status: Some(RunStatus::Completed),
            total_pnl: Some(100.0),
            successful_trades: Some(5),
            failed_trades: Some(1),
            max_drawdown: Some(2.5),
            sharpe_ratio: Some(1.5),
        };

        let updated = repo.update_run(&run.id, update_params).unwrap();
        assert!(updated.is_some());

        let retrieved = repo.get_run(&run.id).unwrap();
        assert_eq!(retrieved.end_time, Some(1234567890));
        assert_eq!(retrieved.status, RunStatus::Completed);
        assert_eq!(retrieved.total_pnl, 100.0);
        assert_eq!(retrieved.successful_trades, 5);
        assert_eq!(retrieved.failed_trades, 1);
        assert_eq!(retrieved.max_drawdown, 2.5);
        assert_eq!(retrieved.sharpe_ratio, 1.5);
    }

    #[test]
    fn test_list_runs() {
        let mut repo = TradeRunRepository::new();

        // Create a few runs
        let params1 = CreateTradeRunParams {
            name: "Run 1".to_string(),
            strategy: "Strategy A".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        repo.create_run(params1).unwrap();

        let params2 = CreateTradeRunParams {
            name: "Run 2".to_string(),
            strategy: "Strategy B".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        repo.create_run(params2).unwrap();

        let runs = repo.list_runs();
        assert_eq!(runs.len(), 2);
    }

    #[test]
    fn test_list_runs_by_strategy() {
        let mut repo = TradeRunRepository::new();

        let params1 = CreateTradeRunParams {
            name: "Run 1".to_string(),
            strategy: "Strategy A".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        repo.create_run(params1).unwrap();

        let params2 = CreateTradeRunParams {
            name: "Run 2".to_string(),
            strategy: "Strategy A".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        repo.create_run(params2).unwrap();

        let params3 = CreateTradeRunParams {
            name: "Run 3".to_string(),
            strategy: "Strategy B".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        repo.create_run(params3).unwrap();

        let runs = repo.list_runs_by_strategy("Strategy A");
        assert_eq!(runs.len(), 2);

        let runs = repo.list_runs_by_strategy("Strategy B");
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn test_list_runs_by_status() {
        let mut repo = TradeRunRepository::new();

        let params1 = CreateTradeRunParams {
            name: "Run 1".to_string(),
            strategy: "Strategy A".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        let run1 = repo.create_run(params1).unwrap();

        let params2 = CreateTradeRunParams {
            name: "Run 2".to_string(),
            strategy: "Strategy B".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        let run2 = repo.create_run(params2).unwrap();

        // Update one run to completed
        let update_params = UpdateTradeRunParams {
            status: Some(RunStatus::Completed),
            ..Default::default()
        };
        repo.update_run(&run1.id, update_params).unwrap();

        let running_runs = repo.list_runs_by_status(RunStatus::Running);
        assert_eq!(running_runs.len(), 1);

        let completed_runs = repo.list_runs_by_status(RunStatus::Completed);
        assert_eq!(completed_runs.len(), 1);
    }

    #[test]
    fn test_list_runs_by_tag() {
        let mut repo = TradeRunRepository::new();

        let mut tags1 = HashMap::new();
        tags1.insert("environment".to_string(), "testnet".to_string());

        let params1 = CreateTradeRunParams {
            name: "Run 1".to_string(),
            strategy: "Strategy A".to_string(),
            tags: vec!["testnet".to_string(), "v1".to_string()],
            metadata: HashMap::new(),
        };
        repo.create_run(params1).unwrap();

        let params2 = CreateTradeRunParams {
            name: "Run 2".to_string(),
            strategy: "Strategy B".to_string(),
            tags: vec!["mainnet".to_string()],
            metadata: HashMap::new(),
        };
        repo.create_run(params2).unwrap();

        let testnet_runs = repo.list_runs_by_tag("testnet");
        assert_eq!(testnet_runs.len(), 1);

        let v1_runs = repo.list_runs_by_tag("v1");
        assert_eq!(v1_runs.len(), 1);

        let mainnet_runs = repo.list_runs_by_tag("mainnet");
        assert_eq!(mainnet_runs.len(), 1);
    }

    #[test]
    fn test_delete_run() {
        let mut repo = TradeRunRepository::new();
        let params = CreateTradeRunParams {
            name: "Test Run".to_string(),
            strategy: "Test Strategy".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };

        let run = repo.create_run(params).unwrap();
        assert_eq!(repo.count_runs(), 1);

        let deleted = repo.delete_run(&run.id).unwrap();
        assert!(deleted);
        assert_eq!(repo.count_runs(), 0);

        // Try to delete non-existent run
        let deleted = repo.delete_run(&run.id).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_get_statistics() {
        let mut repo = TradeRunRepository::new();

        // Create runs with different statuses
        let params1 = CreateTradeRunParams {
            name: "Run 1".to_string(),
            strategy: "Strategy A".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        let run1 = repo.create_run(params1).unwrap();

        let params2 = CreateTradeRunParams {
            name: "Run 2".to_string(),
            strategy: "Strategy B".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        let run2 = repo.create_run(params2).unwrap();

        let params3 = CreateTradeRunParams {
            name: "Run 3".to_string(),
            strategy: "Strategy C".to_string(),
            tags: vec![],
            metadata: HashMap::new(),
        };
        let run3 = repo.create_run(params3).unwrap();

        // Update runs to different statuses and PnLs
        let update_params1 = UpdateTradeRunParams {
            status: Some(RunStatus::Completed),
            total_pnl: Some(100.0),
            ..Default::default()
        };
        repo.update_run(&run1.id, update_params1).unwrap();

        let update_params2 = UpdateTradeRunParams {
            status: Some(RunStatus::Completed),
            total_pnl: Some(200.0),
            ..Default::default()
        };
        repo.update_run(&run2.id, update_params2).unwrap();

        let update_params3 = UpdateTradeRunParams {
            status: Some(RunStatus::Failed),
            ..Default::default()
        };
        repo.update_run(&run3.id, update_params3).unwrap();

        let stats = repo.get_statistics();
        assert_eq!(stats.total_runs, 3);
        assert_eq!(stats.running_runs, 0);
        assert_eq!(stats.completed_runs, 2);
        assert_eq!(stats.failed_runs, 1);
        assert_eq!(stats.cancelled_runs, 0);
        assert_eq!(stats.average_pnl, 150.0); // (100 + 200) / 2
    }

    #[test]
    fn test_default_implementation() {
        let repo = TradeRunRepository::default();
        assert_eq!(repo.count_runs(), 0);
    }
}

impl Default for UpdateTradeRunParams {
    fn default() -> Self {
        Self {
            end_time: None,
            status: None,
            total_pnl: None,
            successful_trades: None,
            failed_trades: None,
            max_drawdown: None,
            sharpe_ratio: None,
        }
    }
}
