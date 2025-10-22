//! Outbox pattern implementation for idempotency.
//!
//! This module provides functionality for ensuring exactly-once processing
//! of events through the outbox pattern with idempotency keys.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Outbox message status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutboxStatus {
    Pending,
    Processed,
    Failed,
}

/// Outbox message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub id: Uuid,
    pub idempotency_key: String,
    pub payload: String,     // JSON payload
    pub destination: String, // Where to send the message
    pub status: OutboxStatus,
    pub created_at: u64,           // Unix timestamp
    pub processed_at: Option<u64>, // Unix timestamp
    pub retry_count: i32,
}

/// In-memory outbox repository for demonstration
/// In a real implementation, this would use a database
pub struct OutboxRepo {
    messages: Arc<RwLock<HashMap<Uuid, OutboxMessage>>>,
    idempotency_keys: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl OutboxRepo {
    /// Create a new outbox repository
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            idempotency_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new outbox message
    pub async fn create(&self, message: &OutboxMessage) -> Result<Uuid> {
        let mut messages = self.messages.write().await;
        let mut idempotency_keys = self.idempotency_keys.write().await;

        messages.insert(message.id, message.clone());
        idempotency_keys.insert(message.idempotency_key.clone(), message.id);

        Ok(message.id)
    }

    /// Check if a message with the given idempotency key already exists
    pub async fn exists_by_idempotency_key(&self, idempotency_key: &str) -> Result<bool> {
        let idempotency_keys = self.idempotency_keys.read().await;
        Ok(idempotency_keys.contains_key(idempotency_key))
    }

    /// Get pending messages
    pub async fn get_pending_messages(&self, limit: usize) -> Result<Vec<OutboxMessage>> {
        let messages = self.messages.read().await;
        let mut result = Vec::new();

        for message in messages.values() {
            if matches!(message.status, OutboxStatus::Pending) {
                result.push(message.clone());
                if result.len() >= limit {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Mark a message as processed
    pub async fn mark_as_processed(&self, id: Uuid, processed_at: u64) -> Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(message) = messages.get_mut(&id) {
            message.status = OutboxStatus::Processed;
            message.processed_at = Some(processed_at);
        }
        Ok(())
    }

    /// Mark a message as failed
    pub async fn mark_as_failed(&self, id: Uuid) -> Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(message) = messages.get_mut(&id) {
            message.status = OutboxStatus::Failed;
            message.retry_count += 1;
        }
        Ok(())
    }
}

impl Default for OutboxRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_outbox_repo() -> Result<()> {
        let repo = OutboxRepo::new();

        let message = OutboxMessage {
            id: Uuid::new_v4(),
            idempotency_key: "test_key_1".to_string(),
            payload: r#"{"event":"test","data":"value"}"#.to_string(),
            destination: "test_queue".to_string(),
            status: OutboxStatus::Pending,
            created_at: 1234567890,
            processed_at: None,
            retry_count: 0,
        };

        // Test create
        let id = repo.create(&message).await?;
        assert_eq!(id, message.id);

        // Test exists by idempotency key
        let exists = repo.exists_by_idempotency_key("test_key_1").await?;
        assert!(exists);

        let exists = repo.exists_by_idempotency_key("nonexistent_key").await?;
        assert!(!exists);

        // Test get pending messages
        let pending = repo.get_pending_messages(10).await?;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].idempotency_key, "test_key_1");

        // Test mark as processed
        repo.mark_as_processed(id, 1234567900).await?;

        let pending = repo.get_pending_messages(10).await?;
        assert_eq!(pending.len(), 0);

        Ok(())
    }
}
