//! Exit strategy module for the sniper bot.
//!
//! This module provides functionality for exiting positions based on various strategies.

pub mod event_based;
pub mod stop_loss;
pub mod take_profit;

/// Common trade side type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSide {
    /// Long position (buy)
    Long,
    /// Short position (sell)
    Short,
}

/// Main exit strategy manager
pub struct ExitManager {
    /// Take profit strategy
    take_profit: take_profit::TakeProfitStrategy,
    /// Stop loss strategy
    stop_loss: stop_loss::StopLossStrategy,
    /// Event-based strategy
    event_based: event_based::EventBasedStrategy,
}

impl ExitManager {
    /// Create a new exit manager
    pub fn new(
        take_profit: take_profit::TakeProfitStrategy,
        stop_loss: stop_loss::StopLossStrategy,
        event_based: event_based::EventBasedStrategy,
    ) -> Self {
        Self {
            take_profit,
            stop_loss,
            event_based,
        }
    }

    /// Check if any exit strategy should be triggered for a position
    pub fn should_exit(&mut self, position: &Position) -> ExitDecision {
        // Check take profit
        if should_take_profit(position, &self.take_profit) {
            return ExitDecision::TakeProfit;
        }

        // Check stop loss
        if should_stop_loss(position, &mut self.stop_loss) {
            return ExitDecision::StopLoss;
        }

        // Check event-based (this is typically triggered externally)
        // For demonstration, we'll check a sample event
        if let Some(action) = should_exit_on_event(position, &mut self.event_based, "sample_event")
        {
            return ExitDecision::EventBased(action);
        }

        ExitDecision::Hold
    }
}

/// Position information (common type for all exit strategies)
#[derive(Debug, Clone)]
pub struct Position {
    /// Trading pair
    pub pair: String,
    /// Entry price
    pub entry_price: f64,
    /// Current price
    pub current_price: f64,
    /// Position side
    pub side: TradeSide,
    /// Position size
    pub size: f64,
}

impl Position {
    /// Create a new position
    pub fn new(
        pair: String,
        entry_price: f64,
        current_price: f64,
        side: TradeSide,
        size: f64,
    ) -> Self {
        Self {
            pair,
            entry_price,
            current_price,
            side,
            size,
        }
    }

    /// Calculate current profit/loss in percentage
    pub fn profit_percentage(&self) -> f64 {
        match self.side {
            TradeSide::Long => ((self.current_price - self.entry_price) / self.entry_price) * 100.0,
            TradeSide::Short => {
                ((self.entry_price - self.current_price) / self.entry_price) * 100.0
            }
        }
    }

    /// Calculate current profit/loss in absolute value
    pub fn profit_value(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.current_price - self.entry_price) * self.size,
            TradeSide::Short => (self.entry_price - self.current_price) * self.size,
        }
    }
}

/// Check if take profit should be triggered for a position
fn should_take_profit(position: &Position, strategy: &take_profit::TakeProfitStrategy) -> bool {
    strategy.should_take_profit(
        &position.pair,
        position.entry_price,
        position.current_price,
        position.side,
    )
}

/// Check if stop loss should be triggered for a position
fn should_stop_loss(position: &Position, strategy: &mut stop_loss::StopLossStrategy) -> bool {
    strategy.should_stop_loss(
        &position.pair,
        position.entry_price,
        position.current_price,
        position.side,
    )
}

/// Check if event-based exit should be triggered for a position
fn should_exit_on_event(
    position: &Position,
    strategy: &mut event_based::EventBasedStrategy,
    event_name: &str,
) -> Option<event_based::ExitAction> {
    strategy.should_exit_on_event(event_name, &position.pair)
}

/// Exit decision types
#[derive(Debug, Clone, PartialEq)]
pub enum ExitDecision {
    /// Hold the position
    Hold,
    /// Take profit
    TakeProfit,
    /// Stop loss
    StopLoss,
    /// Event-based exit with specific action
    EventBased(event_based::ExitAction),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_exit_manager_creation() {
        let take_profit_config = take_profit::TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };
        let take_profit = take_profit::TakeProfitStrategy::new(take_profit_config);

        let stop_loss_config = stop_loss::StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        let stop_loss = stop_loss::StopLossStrategy::new(stop_loss_config);

        let event_config = event_based::EventBasedConfig {
            enabled: true,
            exit_events: HashMap::new(),
            default_action: event_based::ExitAction::CloseAll,
        };
        let event_based = event_based::EventBasedStrategy::new(event_config);

        let _manager = ExitManager::new(take_profit, stop_loss, event_based);
        assert!(true); // Just testing that we can create an exit manager
    }

    #[test]
    fn test_exit_decision() {
        assert_ne!(ExitDecision::Hold, ExitDecision::TakeProfit);
        assert_ne!(ExitDecision::TakeProfit, ExitDecision::StopLoss);
        assert_ne!(ExitDecision::StopLoss, ExitDecision::Hold);
    }
}
