//! Policy engine module for the sniper bot.
//!
//! This module provides functionality for enforcing various policies
//! including geographic restrictions, venue rules, and KYC requirements.

pub mod compliance;
pub mod compliance_loader;
pub mod compliance_monitor;
pub mod compliance_service;
pub mod geo;
pub mod kyc;
pub mod venue;

use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Policy decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVerdict {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

/// Geographic region identifier (ISO 3166-1 alpha-2 country code)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GeoRegion(pub String);

/// Venue identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VenueId(pub String);

/// KYC status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KycStatus {
    Verified,
    Pending,
    Rejected,
}

/// User context for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub ip_address: Option<String>,
    pub geo_region: Option<GeoRegion>,
    pub kyc_status: KycStatus,
    pub venue_id: VenueId,
}

/// Policy engine trait that all policy implementations should implement
pub trait PolicyEngine {
    /// Evaluate if an action is allowed based on user context
    fn evaluate(&self, context: &UserContext) -> PolicyVerdict;
}
