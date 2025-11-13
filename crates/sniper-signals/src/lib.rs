//! Signals module for the sniper bot.
//!
//! This module provides functionality for detecting and processing various types of signals
//! from on-chain events, off-chain data sources, and social media.

pub mod offchain;
pub mod onchain;
pub mod social;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
