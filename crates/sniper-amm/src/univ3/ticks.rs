//! Tick management for Uniswap V3
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a tick in Uniswap V3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    /// The index of the tick
    pub index: i32,
    /// The liquidity amount at this tick
    pub liquidity_gross: u128,
    /// The net liquidity change at this tick
    pub liquidity_net: i128,
    /// Fee growth per unit of liquidity at this tick
    pub fee_growth_outside_0_x128: u128,
    /// Fee growth per unit of liquidity at this tick
    pub fee_growth_outside_1_x128: u128,
    /// Timestamp of when the tick was initialized
    pub initialized: bool,
}

/// Tick manager for Uniswap V3 pools
#[derive(Debug)]
pub struct TickManager {
    /// Mapping of tick index to tick data
    ticks: HashMap<i32, Tick>,
    /// Current tick index
    current_tick: i32,
}

impl TickManager {
    /// Create a new tick manager
    pub fn new(current_tick: i32) -> Self {
        Self {
            ticks: HashMap::new(),
            current_tick,
        }
    }

    /// Get a tick by index
    pub fn get_tick(&self, index: i32) -> Option<&Tick> {
        self.ticks.get(&index)
    }

    /// Add or update a tick
    pub fn update_tick(&mut self, index: i32, tick: Tick) {
        self.ticks.insert(index, tick);
    }

    /// Remove a tick
    pub fn remove_tick(&mut self, index: i32) -> Option<Tick> {
        self.ticks.remove(&index)
    }

    /// Get the current tick
    pub fn current_tick(&self) -> i32 {
        self.current_tick
    }

    /// Set the current tick
    pub fn set_current_tick(&mut self, tick: i32) {
        self.current_tick = tick;
    }

    /// Get all initialized ticks
    pub fn get_initialized_ticks(&self) -> Vec<&Tick> {
        self.ticks
            .values()
            .filter(|tick| tick.initialized)
            .collect()
    }

    /// Calculate the fee growth inside a tick range
    pub fn get_fee_growth_inside(
        &self,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<(u128, u128)> {
        // Get lower tick
        let tick_lower = self
            .ticks
            .get(&tick_lower_index)
            .ok_or_else(|| anyhow::anyhow!("Lower tick {} not found", tick_lower_index))?;

        // Get upper tick
        let tick_upper = self
            .ticks
            .get(&tick_upper_index)
            .ok_or_else(|| anyhow::anyhow!("Upper tick {} not found", tick_upper_index))?;

        // Calculate fee growth inside
        // This is a simplified implementation
        let fee_growth_inside_0 = tick_lower
            .fee_growth_outside_0_x128
            .saturating_sub(tick_upper.fee_growth_outside_0_x128);
        let fee_growth_inside_1 = tick_lower
            .fee_growth_outside_1_x128
            .saturating_sub(tick_upper.fee_growth_outside_1_x128);

        Ok((fee_growth_inside_0, fee_growth_inside_1))
    }

    /// Check if a tick is initialized
    pub fn is_tick_initialized(&self, index: i32) -> bool {
        self.ticks
            .get(&index)
            .map(|tick| tick.initialized)
            .unwrap_or(false)
    }
}

impl Default for TickManager {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Tick spacing for different fee tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickSpacing {
    /// Fee tier in basis points (e.g., 3000 = 0.3%)
    pub fee_tier: u32,
    /// Tick spacing
    pub spacing: i32,
}

impl TickSpacing {
    /// Get tick spacing for a fee tier
    pub fn get_tick_spacing(fee_tier: u32) -> i32 {
        match fee_tier {
            500 => 10,    // 0.05%
            3000 => 60,   // 0.3%
            10000 => 200, // 1%
            _ => 60,      // Default to 0.3%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_creation() {
        let tick = Tick {
            index: 1000,
            liquidity_gross: 1000000,
            liquidity_net: 500000,
            fee_growth_outside_0_x128: 1000,
            fee_growth_outside_1_x128: 2000,
            initialized: true,
        };

        assert_eq!(tick.index, 1000);
        assert_eq!(tick.liquidity_gross, 1000000);
        assert_eq!(tick.liquidity_net, 500000);
        assert_eq!(tick.fee_growth_outside_0_x128, 1000);
        assert_eq!(tick.fee_growth_outside_1_x128, 2000);
        assert!(tick.initialized);
    }

    #[test]
    fn test_tick_manager() {
        let mut manager = TickManager::new(0);
        assert_eq!(manager.current_tick(), 0);

        let tick = Tick {
            index: 1000,
            liquidity_gross: 1000000,
            liquidity_net: 500000,
            fee_growth_outside_0_x128: 1000,
            fee_growth_outside_1_x128: 2000,
            initialized: true,
        };

        manager.update_tick(1000, tick.clone());
        assert!(manager.get_tick(1000).is_some());
        assert_eq!(manager.get_tick(1000).unwrap().index, 1000);

        let removed = manager.remove_tick(1000);
        assert!(removed.is_some());
        assert!(manager.get_tick(1000).is_none());
    }

    #[test]
    fn test_tick_spacing() {
        assert_eq!(TickSpacing::get_tick_spacing(500), 10); // 0.05%
        assert_eq!(TickSpacing::get_tick_spacing(3000), 60); // 0.3%
        assert_eq!(TickSpacing::get_tick_spacing(10000), 200); // 1%
        assert_eq!(TickSpacing::get_tick_spacing(9999), 60); // Default
    }

    #[test]
    fn test_fee_growth_inside() {
        let mut manager = TickManager::new(0);

        let lower_tick = Tick {
            index: -600,
            liquidity_gross: 1000000,
            liquidity_net: 500000,
            fee_growth_outside_0_x128: 10000,
            fee_growth_outside_1_x128: 20000,
            initialized: true,
        };

        let upper_tick = Tick {
            index: 600,
            liquidity_gross: 1000000,
            liquidity_net: -500000,
            fee_growth_outside_0_x128: 5000,
            fee_growth_outside_1_x128: 10000,
            initialized: true,
        };

        manager.update_tick(-600, lower_tick);
        manager.update_tick(600, upper_tick);

        let (fee_growth_0, fee_growth_1) = manager.get_fee_growth_inside(-600, 600).unwrap();
        assert_eq!(fee_growth_0, 5000);
        assert_eq!(fee_growth_1, 10000);
    }
}
