//! Validation module for the sniper bot.
//!
//! This module provides functionality for validating smart contracts and ensuring
//! they follow security best practices like the Checks-Effects-Interactions (CEI) pattern.

pub mod cei;
pub mod input;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
