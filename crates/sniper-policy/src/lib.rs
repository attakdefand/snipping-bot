//! Policy engine module for the sniper bot.
//!
//! This module provides functionality for enforcing various policies
//! including geographic restrictions, venue rules, and KYC requirements.

pub mod geo;
pub mod kyc;
pub mod venue;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Geographic policy engine
pub struct GeoPolicy {
    allowed_regions: Vec<GeoRegion>,
    blocked_regions: Vec<GeoRegion>,
}

impl GeoPolicy {
    /// Create a new geographic policy engine
    pub fn new(allowed_regions: Vec<GeoRegion>, blocked_regions: Vec<GeoRegion>) -> Self {
        Self {
            allowed_regions,
            blocked_regions,
        }
    }
}

impl PolicyEngine for GeoPolicy {
    fn evaluate(&self, context: &UserContext) -> PolicyVerdict {
        // If no geo region is provided, we can't make a decision
        let geo_region = match &context.geo_region {
            Some(region) => region,
            None => {
                return PolicyVerdict {
                    allowed: false,
                    reasons: vec!["No geographic region provided".to_string()],
                }
            }
        };

        // Check if the region is explicitly blocked
        if self.blocked_regions.contains(geo_region) {
            return PolicyVerdict {
                allowed: false,
                reasons: vec![format!("Region {} is blocked", geo_region.0)],
            };
        }

        // If there are allowed regions specified, check if the region is in the list
        if !self.allowed_regions.is_empty() && !self.allowed_regions.contains(geo_region) {
            return PolicyVerdict {
                allowed: false,
                reasons: vec![format!("Region {} is not in allowed list", geo_region.0)],
            };
        }

        // If we get here, the action is allowed
        PolicyVerdict {
            allowed: true,
            reasons: vec![],
        }
    }
}

/// Venue policy engine
pub struct VenuePolicy {
    allowed_venues: Vec<VenueId>,
    blocked_venues: Vec<VenueId>,
    venue_rules: HashMap<VenueId, Vec<String>>,
}

impl VenuePolicy {
    /// Create a new venue policy engine
    pub fn new(allowed_venues: Vec<VenueId>, blocked_venues: Vec<VenueId>) -> Self {
        Self {
            allowed_venues,
            blocked_venues,
            venue_rules: HashMap::new(),
        }
    }

    /// Add rules for a specific venue
    pub fn add_venue_rules(&mut self, venue_id: VenueId, rules: Vec<String>) {
        self.venue_rules.insert(venue_id, rules);
    }
}

impl PolicyEngine for VenuePolicy {
    fn evaluate(&self, context: &UserContext) -> PolicyVerdict {
        // Check if the venue is explicitly blocked
        if self.blocked_venues.contains(&context.venue_id) {
            return PolicyVerdict {
                allowed: false,
                reasons: vec![format!("Venue {} is blocked", context.venue_id.0)],
            };
        }

        // If there are allowed venues specified, check if the venue is in the list
        if !self.allowed_venues.is_empty() && !self.allowed_venues.contains(&context.venue_id) {
            return PolicyVerdict {
                allowed: false,
                reasons: vec![format!(
                    "Venue {} is not in allowed list",
                    context.venue_id.0
                )],
            };
        }

        // Check venue-specific rules
        let mut reasons = Vec::new();
        if let Some(rules) = self.venue_rules.get(&context.venue_id) {
            // In a real implementation, we would check these rules against the context
            // For now, we'll just add them as reasons
            for rule in rules {
                reasons.push(format!("Venue rule: {}", rule));
            }
        }

        // If we get here, the action is allowed
        PolicyVerdict {
            allowed: true,
            reasons,
        }
    }
}

/// KYC policy engine
pub struct KycPolicy {
    required_for_venues: Vec<VenueId>,
}

impl KycPolicy {
    /// Create a new KYC policy engine
    pub fn new(required_for_venues: Vec<VenueId>) -> Self {
        Self {
            required_for_venues,
        }
    }
}

impl PolicyEngine for KycPolicy {
    fn evaluate(&self, context: &UserContext) -> PolicyVerdict {
        // Check if KYC is required for this venue
        if !self.required_for_venues.contains(&context.venue_id) {
            return PolicyVerdict {
                allowed: true,
                reasons: vec![],
            };
        }

        // If KYC is required, check the user's KYC status
        match context.kyc_status {
            KycStatus::Verified => PolicyVerdict {
                allowed: true,
                reasons: vec!["KYC verified".to_string()],
            },
            KycStatus::Pending => PolicyVerdict {
                allowed: false,
                reasons: vec!["KYC verification pending".to_string()],
            },
            KycStatus::Rejected => PolicyVerdict {
                allowed: false,
                reasons: vec!["KYC verification rejected".to_string()],
            },
        }
    }
}

/// Composite policy engine that combines multiple policy engines
pub struct CompositePolicy {
    engines: Vec<Box<dyn PolicyEngine>>,
}

impl Default for CompositePolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositePolicy {
    /// Create a new composite policy engine
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
        }
    }

    /// Add a policy engine to the composite
    pub fn add_engine(&mut self, engine: Box<dyn PolicyEngine>) {
        self.engines.push(engine);
    }
}

impl PolicyEngine for CompositePolicy {
    fn evaluate(&self, context: &UserContext) -> PolicyVerdict {
        let mut all_reasons = Vec::new();

        // Evaluate all policy engines
        for engine in &self.engines {
            let verdict = engine.evaluate(context);

            // If any policy engine denies the action, deny it overall
            if !verdict.allowed {
                all_reasons.extend(verdict.reasons);
                return PolicyVerdict {
                    allowed: false,
                    reasons: all_reasons,
                };
            }

            // Collect reasons from allowed policies
            all_reasons.extend(verdict.reasons);
        }

        // If all policy engines allow the action, allow it overall
        PolicyVerdict {
            allowed: true,
            reasons: all_reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_policy_allowed_region() {
        let policy = GeoPolicy::new(
            vec![GeoRegion("US".to_string()), GeoRegion("CA".to_string())],
            vec![],
        );

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate(&context);
        assert!(verdict.allowed);
        assert!(verdict.reasons.is_empty());
    }

    #[test]
    fn test_geo_policy_blocked_region() {
        let policy = GeoPolicy::new(
            vec![],
            vec![GeoRegion("CN".to_string()), GeoRegion("RU".to_string())],
        );

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(GeoRegion("CN".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate(&context);
        assert!(!verdict.allowed);
        assert_eq!(verdict.reasons, vec!["Region CN is blocked"]);
    }

    #[test]
    fn test_venue_policy_allowed_venue() {
        let policy = VenuePolicy::new(
            vec![
                VenueId("binance".to_string()),
                VenueId("coinbase".to_string()),
            ],
            vec![],
        );

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate(&context);
        assert!(verdict.allowed);
        assert!(verdict.reasons.is_empty());
    }

    #[test]
    fn test_kyc_policy_verified() {
        let policy = KycPolicy::new(vec![VenueId("binance".to_string())]);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate(&context);
        assert!(verdict.allowed);
        assert_eq!(verdict.reasons, vec!["KYC verified"]);
    }

    #[test]
    fn test_composite_policy() {
        let mut composite = CompositePolicy::new();
        composite.add_engine(Box::new(GeoPolicy::new(vec![], vec![])));
        composite.add_engine(Box::new(VenuePolicy::new(vec![], vec![])));
        composite.add_engine(Box::new(KycPolicy::new(vec![])));

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = composite.evaluate(&context);
        assert!(verdict.allowed);
    }
}
