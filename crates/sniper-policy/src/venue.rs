//! Venue policy implementation
use crate::{PolicyVerdict, UserContext, VenueId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Venue policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenuePolicyConfig {
    /// Venues that are explicitly allowed
    pub allowed_venues: Vec<VenueId>,
    /// Venues that are explicitly blocked
    pub blocked_venues: Vec<VenueId>,
    /// Rules for specific venues
    pub venue_rules: HashMap<VenueId, Vec<VenueRule>>,
    /// Time-based restrictions for venues
    pub time_restrictions: HashMap<VenueId, Vec<TimeRestriction>>,
}

/// A rule that applies to a venue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueRule {
    /// Name of the rule
    pub name: String,
    /// Description of the rule
    pub description: String,
    /// Whether this rule is enforced
    pub enabled: bool,
}

/// Time-based restriction for a venue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    /// Day of week (0-6, where 0 is Sunday)
    pub day_of_week: u8,
    /// Start time in minutes from midnight (0-1439)
    pub start_time_minutes: u16,
    /// End time in minutes from midnight (0-1439)
    pub end_time_minutes: u16,
}

/// Venue policy engine
#[derive(Debug, Clone)]
pub struct VenuePolicy {
    config: VenuePolicyConfig,
}

impl VenuePolicy {
    /// Create a new venue policy engine
    pub fn new(config: VenuePolicyConfig) -> Self {
        Self { config }
    }

    /// Check if a venue is explicitly allowed
    pub fn is_venue_allowed(&self, venue_id: &VenueId) -> bool {
        // If there are no allowed venues specified, all venues are allowed by default
        if self.config.allowed_venues.is_empty() {
            true
        } else {
            self.config.allowed_venues.contains(venue_id)
        }
    }

    /// Check if a venue is explicitly blocked
    pub fn is_venue_blocked(&self, venue_id: &VenueId) -> bool {
        self.config.blocked_venues.contains(venue_id)
    }

    /// Get rules for a specific venue
    pub fn get_venue_rules(&self, venue_id: &VenueId) -> Option<&Vec<VenueRule>> {
        self.config.venue_rules.get(venue_id)
    }

    /// Check if a venue has time restrictions
    pub fn has_time_restrictions(&self, venue_id: &VenueId) -> bool {
        self.config
            .time_restrictions
            .get(venue_id)
            .map(|restrictions| !restrictions.is_empty())
            .unwrap_or(false)
    }

    /// Evaluate if a user can access a venue based on venue policies
    pub fn evaluate_venue_access(&self, context: &UserContext) -> PolicyVerdict {
        let venue_id = &context.venue_id;
        let mut reasons = Vec::new();

        info!("Evaluating venue access for venue: {}", venue_id.0);

        // Check if the venue is explicitly blocked
        if self.is_venue_blocked(venue_id) {
            warn!("Venue {} is explicitly blocked", venue_id.0);
            return PolicyVerdict {
                allowed: false,
                reasons: vec![format!("Venue {} is explicitly blocked", venue_id.0)],
            };
        }

        // Check if the venue is explicitly allowed (if there's an allow list)
        if !self.config.allowed_venues.is_empty() && !self.is_venue_allowed(venue_id) {
            warn!("Venue {} is not in allowed list", venue_id.0);
            return PolicyVerdict {
                allowed: false,
                reasons: vec![format!("Venue {} is not in allowed list", venue_id.0)],
            };
        }

        // Check venue-specific rules
        if let Some(rules) = self.get_venue_rules(venue_id) {
            debug!("Checking {} rules for venue {}", rules.len(), venue_id.0);
            for rule in rules {
                if rule.enabled {
                    reasons.push(format!("Rule applied: {}", rule.name));
                    // In a real implementation, we would check the rule against the context
                    // For now, we just log that the rule exists
                }
            }
        }

        // Check time restrictions
        if self.has_time_restrictions(venue_id) {
            if let Some(restrictions) = self.config.time_restrictions.get(venue_id) {
                // In a real implementation, we would check the current time against restrictions
                // For now, we just note that time restrictions exist
                reasons.push(format!(
                    "Venue has {} time restriction(s)",
                    restrictions.len()
                ));
            }
        }

        // If we get here, the action is allowed
        info!("Venue access granted for venue: {}", venue_id.0);
        PolicyVerdict {
            allowed: true,
            reasons,
        }
    }

    /// Add a rule to a venue
    pub fn add_venue_rule(&mut self, venue_id: VenueId, rule: VenueRule) {
        self.config
            .venue_rules
            .entry(venue_id)
            .or_default()
            .push(rule);
    }

    /// Add a time restriction to a venue
    pub fn add_time_restriction(&mut self, venue_id: VenueId, restriction: TimeRestriction) {
        self.config
            .time_restrictions
            .entry(venue_id)
            .or_default()
            .push(restriction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KycStatus, UserContext, VenueId};

    #[test]
    fn test_venue_policy_creation() {
        let config = VenuePolicyConfig {
            allowed_venues: vec![VenueId("binance".to_string())],
            blocked_venues: vec![VenueId("kucoin".to_string())],
            venue_rules: HashMap::new(),
            time_restrictions: HashMap::new(),
        };

        let policy = VenuePolicy::new(config);
        assert!(policy.is_venue_allowed(&VenueId("binance".to_string())));
        assert!(!policy.is_venue_allowed(&VenueId("coinbase".to_string())));
        assert!(policy.is_venue_blocked(&VenueId("kucoin".to_string())));
        assert!(!policy.is_venue_blocked(&VenueId("binance".to_string())));
    }

    #[test]
    fn test_venue_access_allowed() {
        let config = VenuePolicyConfig {
            allowed_venues: vec![VenueId("binance".to_string())],
            blocked_venues: vec![],
            venue_rules: HashMap::new(),
            time_restrictions: HashMap::new(),
        };

        let policy = VenuePolicy::new(config);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(crate::GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate_venue_access(&context);
        assert!(verdict.allowed);
    }

    #[test]
    fn test_venue_access_blocked() {
        let config = VenuePolicyConfig {
            allowed_venues: vec![],
            blocked_venues: vec![VenueId("kucoin".to_string())],
            venue_rules: HashMap::new(),
            time_restrictions: HashMap::new(),
        };

        let policy = VenuePolicy::new(config);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(crate::GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("kucoin".to_string()),
        };

        let verdict = policy.evaluate_venue_access(&context);
        assert!(!verdict.allowed);
        assert_eq!(
            verdict.reasons,
            vec!["Venue kucoin is explicitly blocked".to_string()]
        );
    }

    #[test]
    fn test_venue_rules() {
        let mut config = VenuePolicyConfig {
            allowed_venues: vec![],
            blocked_venues: vec![],
            venue_rules: HashMap::new(),
            time_restrictions: HashMap::new(),
        };

        let rule = VenueRule {
            name: "test_rule".to_string(),
            description: "A test rule".to_string(),
            enabled: true,
        };

        config
            .venue_rules
            .insert(VenueId("binance".to_string()), vec![rule]);

        let policy = VenuePolicy::new(config);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(crate::GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate_venue_access(&context);
        assert!(verdict.allowed);
        assert!(!verdict.reasons.is_empty());
        assert!(verdict.reasons[0].contains("Rule applied"));
    }

    #[test]
    fn test_add_venue_rule() {
        let config = VenuePolicyConfig {
            allowed_venues: vec![],
            blocked_venues: vec![],
            venue_rules: HashMap::new(),
            time_restrictions: HashMap::new(),
        };

        let mut policy = VenuePolicy::new(config);

        let rule = VenueRule {
            name: "new_rule".to_string(),
            description: "A new rule".to_string(),
            enabled: true,
        };

        policy.add_venue_rule(VenueId("binance".to_string()), rule);

        assert!(policy
            .get_venue_rules(&VenueId("binance".to_string()))
            .is_some());
        assert_eq!(
            policy
                .get_venue_rules(&VenueId("binance".to_string()))
                .unwrap()
                .len(),
            1
        );
    }
}
