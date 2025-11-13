//! KYC (Know Your Customer) policy implementation
use crate::{KycStatus, PolicyVerdict, UserContext, VenueId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// KYC policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycPolicyConfig {
    /// Venues that require KYC verification
    pub required_for_venues: Vec<VenueId>,
    /// Minimum KYC level required for each venue
    pub venue_kyc_requirements: HashMap<VenueId, KycLevel>,
}

/// KYC verification levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum KycLevel {
    /// Basic identity verification
    Basic,
    /// Enhanced due diligence
    Enhanced,
    /// Full verification with documentation
    Full,
}

/// KYC policy engine
#[derive(Debug, Clone)]
pub struct KycPolicy {
    config: KycPolicyConfig,
}

impl KycPolicy {
    /// Create a new KYC policy engine
    pub fn new(config: KycPolicyConfig) -> Self {
        Self { config }
    }

    /// Check if KYC is required for a venue
    pub fn is_kyc_required_for_venue(&self, venue_id: &VenueId) -> bool {
        self.config.required_for_venues.contains(venue_id)
    }

    /// Get the required KYC level for a venue
    pub fn get_required_kyc_level(&self, venue_id: &VenueId) -> Option<&KycLevel> {
        self.config.venue_kyc_requirements.get(venue_id)
    }

    /// Evaluate if a user meets KYC requirements for a venue
    pub fn evaluate_user_kyc(&self, context: &UserContext) -> PolicyVerdict {
        // Check if KYC is required for this venue
        if !self.is_kyc_required_for_venue(&context.venue_id) {
            return PolicyVerdict {
                allowed: true,
                reasons: vec!["KYC not required for this venue".to_string()],
            };
        }

        // Get the required KYC level for this venue
        let required_level = self.get_required_kyc_level(&context.venue_id);

        // If no specific level is required, just check that the user is verified
        if required_level.is_none() {
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
        } else {
            // Check if the user meets the required KYC level
            let user_level = match context.kyc_status {
                KycStatus::Verified => KycLevel::Full, // Assume verified users have full KYC
                KycStatus::Pending => {
                    return PolicyVerdict {
                        allowed: false,
                        reasons: vec!["KYC verification pending".to_string()],
                    }
                }
                KycStatus::Rejected => {
                    return PolicyVerdict {
                        allowed: false,
                        reasons: vec!["KYC verification rejected".to_string()],
                    }
                }
            };

            if let Some(required_level) = required_level {
                if user_level >= *required_level {
                    PolicyVerdict {
                        allowed: true,
                        reasons: vec![format!(
                            "KYC level {:?} meets required level {:?}",
                            user_level, required_level
                        )],
                    }
                } else {
                    PolicyVerdict {
                        allowed: false,
                        reasons: vec![format!(
                            "KYC level {:?} does not meet required level {:?}",
                            user_level, required_level
                        )],
                    }
                }
            } else {
                // This case should never happen due to the outer check, but we need to handle it
                PolicyVerdict {
                    allowed: true,
                    reasons: vec!["No KYC level required".to_string()],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KycStatus, UserContext, VenueId};

    #[test]
    fn test_kyc_policy_creation() {
        let config = KycPolicyConfig {
            required_for_venues: vec![VenueId("binance".to_string())],
            venue_kyc_requirements: HashMap::new(),
        };

        let policy = KycPolicy::new(config);
        assert!(policy.is_kyc_required_for_venue(&VenueId("binance".to_string())));
        assert!(!policy.is_kyc_required_for_venue(&VenueId("coinbase".to_string())));
    }

    #[test]
    fn test_kyc_level_ordering() {
        assert!(KycLevel::Basic < KycLevel::Enhanced);
        assert!(KycLevel::Enhanced < KycLevel::Full);
        assert!(KycLevel::Basic < KycLevel::Full);
    }

    #[test]
    fn test_kyc_evaluation_verified_user() {
        let config = KycPolicyConfig {
            required_for_venues: vec![VenueId("binance".to_string())],
            venue_kyc_requirements: HashMap::new(),
        };

        let policy = KycPolicy::new(config);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(crate::GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate_user_kyc(&context);
        assert!(verdict.allowed);
        assert_eq!(verdict.reasons, vec!["KYC verified"]);
    }

    #[test]
    fn test_kyc_evaluation_pending_user() {
        let config = KycPolicyConfig {
            required_for_venues: vec![VenueId("binance".to_string())],
            venue_kyc_requirements: HashMap::new(),
        };

        let policy = KycPolicy::new(config);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(crate::GeoRegion("US".to_string())),
            kyc_status: KycStatus::Pending,
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate_user_kyc(&context);
        assert!(!verdict.allowed);
        assert_eq!(verdict.reasons, vec!["KYC verification pending"]);
    }

    #[test]
    fn test_kyc_evaluation_with_levels() {
        let mut venue_kyc_requirements = HashMap::new();
        venue_kyc_requirements.insert(VenueId("binance".to_string()), KycLevel::Enhanced);

        let config = KycPolicyConfig {
            required_for_venues: vec![VenueId("binance".to_string())],
            venue_kyc_requirements,
        };

        let policy = KycPolicy::new(config);

        let context = UserContext {
            user_id: "user1".to_string(),
            ip_address: Some("1.2.3.4".to_string()),
            geo_region: Some(crate::GeoRegion("US".to_string())),
            kyc_status: KycStatus::Verified, // Should be treated as Full level
            venue_id: VenueId("binance".to_string()),
        };

        let verdict = policy.evaluate_user_kyc(&context);
        assert!(verdict.allowed);
        assert!(verdict.reasons[0].contains("meets required level"));
    }
}
