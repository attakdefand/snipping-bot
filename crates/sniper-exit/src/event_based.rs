//! Event-based exit strategy module for the sniper bot.
//!
//! This module provides functionality for exiting positions based on specific events or conditions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Event-based exit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBasedConfig {
    /// Enable/disable event-based exits
    pub enabled: bool,
    /// Events that trigger exits
    pub exit_events: HashMap<String, ExitEventConfig>,
    /// Default action when event occurs
    pub default_action: ExitAction,
}

/// Exit event configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitEventConfig {
    /// Action to take when event occurs
    pub action: ExitAction,
    /// Whether this event is active
    pub active: bool,
    /// Pairs this event applies to (empty means all pairs)
    pub pairs: Vec<String>,
    /// Cooldown period in seconds (0 = no cooldown)
    pub cooldown_seconds: u64,
}

/// Exit action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExitAction {
    /// Close all positions
    CloseAll,
    /// Close specific pair positions
    ClosePair(String),
    /// Reduce position by percentage
    ReducePosition(f64),
    /// Pause trading
    PauseTrading,
    /// Custom action
    Custom(String),
}

/// Event-based exit strategy
pub struct EventBasedStrategy {
    /// Configuration
    config: EventBasedConfig,
    /// Event cooldown tracking
    cooldowns: HashMap<String, HashMap<String, u64>>, // event_name -> pair -> timestamp
}

impl EventBasedStrategy {
    /// Create a new event-based strategy
    pub fn new(config: EventBasedConfig) -> Self {
        Self {
            config,
            cooldowns: HashMap::new(),
        }
    }

    /// Check if an event should trigger an exit action
    pub fn should_exit_on_event(&mut self, event_name: &str, pair: &str) -> Option<ExitAction> {
        if !self.config.enabled {
            debug!("Event-based exits are disabled");
            return None;
        }

        let event_config = match self.config.exit_events.get(event_name) {
            Some(config) => config,
            None => {
                debug!("No configuration found for event: {}", event_name);
                return None;
            }
        };

        if !event_config.active {
            debug!("Event {} is not active", event_name);
            return None;
        }

        // Check if event applies to this pair
        if !event_config.pairs.is_empty() && !event_config.pairs.contains(&pair.to_string()) {
            debug!("Event {} does not apply to pair {}", event_name, pair);
            return None;
        }

        // Check cooldown
        if event_config.cooldown_seconds > 0 {
            if let Some(pair_cooldowns) = self.cooldowns.get(event_name) {
                if let Some(last_trigger) = pair_cooldowns.get(pair) {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if now - last_trigger < event_config.cooldown_seconds {
                        debug!("Event {} is in cooldown for pair {}", event_name, pair);
                        return None;
                    }
                }
            }
        }

        info!(
            "Event {} triggered exit action for pair {}",
            event_name, pair
        );

        // Update cooldown
        if event_config.cooldown_seconds > 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            self.cooldowns
                .entry(event_name.to_string())
                .or_default()
                .insert(pair.to_string(), now);
        }

        Some(event_config.action.clone())
    }

    /// Process a custom event
    pub fn process_custom_event(
        &mut self,
        event_name: &str,
        pair: &str,
        data: serde_json::Value,
    ) -> Option<ExitAction> {
        debug!(
            "Processing custom event: {} for pair: {} with data: {:?}",
            event_name, pair, data
        );
        self.should_exit_on_event(event_name, pair)
    }

    /// Add or update an exit event configuration
    pub fn set_exit_event(&mut self, event_name: String, config: ExitEventConfig) {
        self.config.exit_events.insert(event_name, config);
    }

    /// Remove an exit event configuration
    pub fn remove_exit_event(&mut self, event_name: &str) {
        self.config.exit_events.remove(event_name);
    }

    /// Activate an event
    pub fn activate_event(&mut self, event_name: &str) {
        if let Some(config) = self.config.exit_events.get_mut(event_name) {
            config.active = true;
        }
    }

    /// Deactivate an event
    pub fn deactivate_event(&mut self, event_name: &str) {
        if let Some(config) = self.config.exit_events.get_mut(event_name) {
            config.active = false;
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: EventBasedConfig) {
        self.config = config;
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        for (event_name, event_config) in &self.config.exit_events {
            if event_config.pairs.is_empty() {
                debug!("Event {} applies to all pairs", event_name);
            } else {
                debug!(
                    "Event {} applies to pairs: {:?}",
                    event_name, event_config.pairs
                );
            }

            match &event_config.action {
                ExitAction::ReducePosition(percentage) => {
                    if *percentage <= 0.0 || *percentage > 100.0 {
                        return Err(anyhow::anyhow!(
                            "Reduction percentage for event {} must be between 0 and 100",
                            event_name
                        ));
                    }
                }
                ExitAction::ClosePair(pair) => {
                    if pair.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Pair name cannot be empty for ClosePair action in event {}",
                            event_name
                        ));
                    }
                }
                ExitAction::Custom(action) => {
                    if action.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Custom action cannot be empty in event {}",
                            event_name
                        ));
                    }
                }
                _ => {} // Other actions don't need validation
            }
        }

        Ok(())
    }

    /// Get active events
    pub fn get_active_events(&self) -> Vec<&String> {
        self.config
            .exit_events
            .iter()
            .filter(|(_, config)| config.active)
            .map(|(name, _)| name)
            .collect()
    }

    /// Clear all cooldowns
    pub fn clear_cooldowns(&mut self) {
        self.cooldowns.clear();
    }

    /// Clear cooldown for specific event and pair
    pub fn clear_cooldown(&mut self, event_name: &str, pair: &str) {
        if let Some(pair_cooldowns) = self.cooldowns.get_mut(event_name) {
            pair_cooldowns.remove(pair);
        }
    }
}

/// Predefined events
pub mod events {
    /// Market volatility event
    pub const MARKET_VOLATILITY: &str = "market_volatility";
    /// Major news event
    pub const MAJOR_NEWS: &str = "major_news";
    /// Technical indicator signal
    pub const TECHNICAL_SIGNAL: &str = "technical_signal";
    /// Risk threshold exceeded
    pub const RISK_THRESHOLD: &str = "risk_threshold";
    /// External signal received
    pub const EXTERNAL_SIGNAL: &str = "external_signal";
    /// Time-based exit
    pub const TIME_BASED: &str = "time_based";
    /// Manual trigger
    pub const MANUAL_TRIGGER: &str = "manual_trigger";
}

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: u64,
    /// Associated pair (if any)
    pub pair: Option<String>,
    /// Event data
    pub data: serde_json::Value,
}

impl EventData {
    /// Create a new event
    pub fn new(name: String, pair: Option<String>, data: serde_json::Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            name,
            timestamp,
            pair,
            data,
        }
    }

    /// Check if event is recent (within specified seconds)
    pub fn is_recent(&self, seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.timestamp <= seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_event_based_strategy_creation() {
        let config = EventBasedConfig {
            enabled: true,
            exit_events: HashMap::new(),
            default_action: ExitAction::CloseAll,
        };

        let strategy = EventBasedStrategy::new(config.clone());
        assert!(strategy.config.enabled);
        assert!(strategy.cooldowns.is_empty());
    }

    #[test]
    fn test_exit_event_configuration() {
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "market_volatility".to_string(),
            ExitEventConfig {
                action: ExitAction::CloseAll,
                active: true,
                pairs: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
                cooldown_seconds: 60,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let mut strategy = EventBasedStrategy::new(config);

        // Test event that should trigger
        let action = strategy.should_exit_on_event("market_volatility", "BTC/USDT");
        assert_eq!(action, Some(ExitAction::CloseAll));

        // Test event that doesn't apply to pair
        let action = strategy.should_exit_on_event("market_volatility", "SOL/USDT");
        assert_eq!(action, None);

        // Test non-existent event
        let action = strategy.should_exit_on_event("non_existent", "BTC/USDT");
        assert_eq!(action, None);
    }

    #[test]
    fn test_event_activation() {
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "test_event".to_string(),
            ExitEventConfig {
                action: ExitAction::CloseAll,
                active: false,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let mut strategy = EventBasedStrategy::new(config);

        // Test inactive event
        let action = strategy.should_exit_on_event("test_event", "BTC/USDT");
        assert_eq!(action, None);

        // Activate event
        strategy.activate_event("test_event");

        // Test active event
        let action = strategy.should_exit_on_event("test_event", "BTC/USDT");
        assert_eq!(action, Some(ExitAction::CloseAll));

        // Deactivate event
        strategy.deactivate_event("test_event");

        // Test deactivated event
        let action = strategy.should_exit_on_event("test_event", "BTC/USDT");
        assert_eq!(action, None);
    }

    #[test]
    fn test_cooldown_functionality() {
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "cooldown_test".to_string(),
            ExitEventConfig {
                action: ExitAction::CloseAll,
                active: true,
                pairs: vec![],
                cooldown_seconds: 1, // 1 second cooldown
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let mut strategy = EventBasedStrategy::new(config);

        // First trigger should work
        let action = strategy.should_exit_on_event("cooldown_test", "BTC/USDT");
        assert_eq!(action, Some(ExitAction::CloseAll));

        // Second trigger should be in cooldown
        let action = strategy.should_exit_on_event("cooldown_test", "BTC/USDT");
        assert_eq!(action, None);

        // Wait for cooldown to expire
        sleep(Duration::from_secs(2));

        // Third trigger should work again
        let action = strategy.should_exit_on_event("cooldown_test", "BTC/USDT");
        assert_eq!(action, Some(ExitAction::CloseAll));
    }

    #[test]
    fn test_config_management() {
        let config = EventBasedConfig {
            enabled: true,
            exit_events: HashMap::new(),
            default_action: ExitAction::CloseAll,
        };

        let mut strategy = EventBasedStrategy::new(config);

        // Add new event
        let new_event = ExitEventConfig {
            action: ExitAction::ClosePair("BTC/USDT".to_string()),
            active: true,
            pairs: vec![],
            cooldown_seconds: 0,
        };
        strategy.set_exit_event("new_event".to_string(), new_event);

        // Test new event
        let action = strategy.should_exit_on_event("new_event", "BTC/USDT");
        assert_eq!(action, Some(ExitAction::ClosePair("BTC/USDT".to_string())));

        // Remove event
        strategy.remove_exit_event("new_event");

        // Test removed event
        let action = strategy.should_exit_on_event("new_event", "BTC/USDT");
        assert_eq!(action, None);

        // Update config
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "updated_event".to_string(),
            ExitEventConfig {
                action: ExitAction::ReducePosition(50.0),
                active: true,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let new_config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        strategy.update_config(new_config);

        // Test updated config
        let action = strategy.should_exit_on_event("updated_event", "BTC/USDT");
        assert_eq!(action, Some(ExitAction::ReducePosition(50.0)));
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "valid_event".to_string(),
            ExitEventConfig {
                action: ExitAction::ReducePosition(50.0),
                active: true,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let strategy = EventBasedStrategy::new(config);
        assert!(strategy.validate_config().is_ok());

        // Invalid config - reduction percentage > 100
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "invalid_event".to_string(),
            ExitEventConfig {
                action: ExitAction::ReducePosition(150.0),
                active: true,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let strategy = EventBasedStrategy::new(config);
        assert!(strategy.validate_config().is_err());

        // Invalid config - empty pair name
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "invalid_event2".to_string(),
            ExitEventConfig {
                action: ExitAction::ClosePair("".to_string()),
                active: true,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let strategy = EventBasedStrategy::new(config);
        assert!(strategy.validate_config().is_err());

        // Invalid config - empty custom action
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "invalid_event3".to_string(),
            ExitEventConfig {
                action: ExitAction::Custom("".to_string()),
                active: true,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let strategy = EventBasedStrategy::new(config);
        assert!(strategy.validate_config().is_err());
    }

    #[test]
    fn test_predefined_events() {
        assert_eq!(events::MARKET_VOLATILITY, "market_volatility");
        assert_eq!(events::MAJOR_NEWS, "major_news");
        assert_eq!(events::TECHNICAL_SIGNAL, "technical_signal");
        assert_eq!(events::RISK_THRESHOLD, "risk_threshold");
        assert_eq!(events::EXTERNAL_SIGNAL, "external_signal");
        assert_eq!(events::TIME_BASED, "time_based");
        assert_eq!(events::MANUAL_TRIGGER, "manual_trigger");
    }

    #[test]
    fn test_event_data() {
        let data = serde_json::json!({"volatility": 0.05});
        let event = EventData::new("test_event".to_string(), Some("BTC/USDT".to_string()), data);

        assert_eq!(event.name, "test_event");
        assert_eq!(event.pair, Some("BTC/USDT".to_string()));
        assert!(event.timestamp > 0);

        // Test recent event
        assert!(event.is_recent(10)); // Should be recent within 10 seconds
    }

    #[test]
    fn test_cooldown_management() {
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "test_event".to_string(),
            ExitEventConfig {
                action: ExitAction::CloseAll,
                active: true,
                pairs: vec![],
                cooldown_seconds: 60,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let mut strategy = EventBasedStrategy::new(config);

        // Trigger event to set cooldown
        strategy.should_exit_on_event("test_event", "BTC/USDT");

        // Check cooldown exists
        assert!(strategy.cooldowns.contains_key("test_event"));
        assert!(strategy
            .cooldowns
            .get("test_event")
            .unwrap()
            .contains_key("BTC/USDT"));

        // Clear specific cooldown
        strategy.clear_cooldown("test_event", "BTC/USDT");
        assert!(!strategy
            .cooldowns
            .get("test_event")
            .unwrap()
            .contains_key("BTC/USDT"));

        // Trigger again and clear all cooldowns
        strategy.should_exit_on_event("test_event", "BTC/USDT");
        strategy.should_exit_on_event("test_event", "ETH/USDT");
        assert!(!strategy.cooldowns.get("test_event").unwrap().is_empty());

        strategy.clear_cooldowns();
        assert!(strategy.cooldowns.is_empty());
    }

    #[test]
    fn test_active_events() {
        let mut exit_events = HashMap::new();
        exit_events.insert(
            "active_event".to_string(),
            ExitEventConfig {
                action: ExitAction::CloseAll,
                active: true,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );
        exit_events.insert(
            "inactive_event".to_string(),
            ExitEventConfig {
                action: ExitAction::CloseAll,
                active: false,
                pairs: vec![],
                cooldown_seconds: 0,
            },
        );

        let config = EventBasedConfig {
            enabled: true,
            exit_events,
            default_action: ExitAction::CloseAll,
        };

        let strategy = EventBasedStrategy::new(config);
        let active_events = strategy.get_active_events();
        assert_eq!(active_events.len(), 1);
        assert_eq!(active_events[0], "active_event");
    }
}
