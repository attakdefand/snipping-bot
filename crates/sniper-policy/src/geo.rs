//! Geographic restriction policies implementation
use anyhow::Result;
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

/// Geographic region identifier (ISO 3166-1 alpha-2 country code)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GeoRegion(pub String);

/// IP geolocation database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpGeoEntry {
    /// IP address range start
    pub start: IpAddr,
    /// IP address range end
    pub end: IpAddr,
    /// Country code
    pub country: String,
    /// Region/state code
    pub region: Option<String>,
    /// City name
    pub city: Option<String>,
}

/// Geographic restriction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoRestrictionPolicy {
    /// Allowed regions (if empty, all regions are allowed except blocked ones)
    pub allowed_regions: Vec<GeoRegion>,
    /// Blocked regions (takes precedence over allowed regions)
    pub blocked_regions: Vec<GeoRegion>,
    /// Whether to block private IP addresses
    pub block_private_ips: bool,
    /// Whether to block unknown regions
    pub block_unknown_regions: bool,
}

impl Default for GeoRestrictionPolicy {
    fn default() -> Self {
        Self {
            allowed_regions: Vec::new(),
            blocked_regions: Vec::new(),
            block_private_ips: true,
            block_unknown_regions: false,
        }
    }
}

/// Geographic policy engine
#[derive(Debug, Clone)]
pub struct GeoPolicyEngine {
    /// Policy configuration
    policy: GeoRestrictionPolicy,
    /// IP geolocation database
    geo_db: HashMap<String, IpGeoEntry>,
}

impl GeoPolicyEngine {
    /// Create a new geographic policy engine
    pub fn new(policy: GeoRestrictionPolicy) -> Self {
        Self {
            policy,
            geo_db: HashMap::new(),
        }
    }

    /// Load IP geolocation database
    ///
    /// # Arguments
    /// * `db_entries` - Vector of IP geolocation entries
    pub fn load_geo_database(&mut self, db_entries: Vec<IpGeoEntry>) {
        for entry in db_entries {
            self.geo_db.insert(entry.country.clone(), entry);
        }
    }

    /// Get geographic region for an IP address
    ///
    /// # Arguments
    /// * `ip` - IP address to look up
    ///
    /// # Returns
    /// * `Option<GeoRegion>` - Geographic region or None if not found
    pub fn get_region_for_ip(&self, ip: &IpAddr) -> Option<GeoRegion> {
        // In a real implementation, this would do a proper IP range lookup
        // For this implementation, we'll simulate with a simple approach

        // Check if it's a private IP
        if self.is_private_ip(ip) {
            if self.policy.block_private_ips {
                return None;
            } else {
                return Some(GeoRegion("PRIVATE".to_string()));
            }
        }

        // Simulate database lookup
        // In a real implementation, this would check the IP against the geo_db ranges
        match ip.to_string().as_str() {
            "1.1.1.1" => Some(GeoRegion("AU".to_string())),
            "8.8.8.8" => Some(GeoRegion("US".to_string())),
            "192.168.1.1" => Some(GeoRegion("PRIVATE".to_string())),
            _ => {
                if self.policy.block_unknown_regions {
                    None
                } else {
                    Some(GeoRegion("UNKNOWN".to_string()))
                }
            }
        }
    }

    /// Check if an IP address is private
    ///
    /// # Arguments
    /// * `ip` - IP address to check
    ///
    /// # Returns
    /// * `bool` - True if IP is private, false otherwise
    pub fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private()
                    || ipv4.is_loopback()
                    || ipv4.is_link_local()
                    || ipv4.is_broadcast()
            }
            IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unspecified(),
        }
    }

    /// Evaluate if a geographic region is allowed
    ///
    /// # Arguments
    /// * `region` - Geographic region to evaluate
    ///
    /// # Returns
    /// * `Result<bool>` - True if allowed, false if blocked, error if evaluation fails
    pub fn is_region_allowed(&self, region: &GeoRegion) -> Result<bool> {
        // Check if the region is explicitly blocked
        if self.policy.blocked_regions.contains(region) {
            return Ok(false);
        }

        // If there are allowed regions specified, check if the region is in the list
        if !self.policy.allowed_regions.is_empty() && !self.policy.allowed_regions.contains(region)
        {
            return Ok(false);
        }

        Ok(true)
    }

    /// Evaluate if an IP address is allowed
    ///
    /// # Arguments
    /// * `ip` - IP address to evaluate
    ///
    /// # Returns
    /// * `Result<bool>` - True if allowed, false if blocked
    pub fn is_ip_allowed(&self, ip: &IpAddr) -> Result<bool> {
        // Check if it's a private IP and we're blocking them
        if self.is_private_ip(ip) && self.policy.block_private_ips {
            return Ok(false);
        }

        // Get region for IP
        let region = match self.get_region_for_ip(ip) {
            Some(region) => region,
            None => return Ok(false), // Unknown region and we're blocking them
        };

        self.is_region_allowed(&region)
    }

    /// Get detailed evaluation result for an IP address
    ///
    /// # Arguments
    /// * `ip` - IP address to evaluate
    ///
    /// # Returns
    /// * `Result<(bool, Vec<String>)>` - (allowed, reasons)
    pub fn evaluate_ip(&self, ip: &IpAddr) -> Result<(bool, Vec<String>)> {
        let mut reasons = Vec::new();

        // Check if it's a private IP
        if self.is_private_ip(ip) {
            if self.policy.block_private_ips {
                reasons.push("Private IP addresses are blocked".to_string());
                return Ok((false, reasons));
            } else {
                reasons.push("Private IP address detected".to_string());
            }
        }

        // Get region for IP
        let region = match self.get_region_for_ip(ip) {
            Some(region) => {
                reasons.push(format!("Region identified as: {}", region.0));
                region
            }
            None => {
                if self.policy.block_unknown_regions {
                    reasons.push("Unknown region and blocking is enabled".to_string());
                    return Ok((false, reasons));
                } else {
                    reasons.push("Unknown region".to_string());
                    GeoRegion("UNKNOWN".to_string())
                }
            }
        };

        // Check region restrictions
        if self.policy.blocked_regions.contains(&region) {
            reasons.push(format!("Region {} is explicitly blocked", region.0));
            return Ok((false, reasons));
        }

        if !self.policy.allowed_regions.is_empty() && !self.policy.allowed_regions.contains(&region)
        {
            reasons.push(format!("Region {} is not in allowed list", region.0));
            return Ok((false, reasons));
        }

        reasons.push("No restrictions apply".to_string());
        Ok((true, reasons))
    }

    /// Add an IP to the blocked list
    ///
    /// # Arguments
    /// * `ip` - IP address to block
    pub fn block_ip(&mut self, ip: &IpAddr) {
        // In a real implementation, this would add to a blocked IPs list
        // For this implementation, we'll just log it
        println!("Blocking IP: {}", ip);
    }

    /// Add a region to the blocked list
    ///
    /// # Arguments
    /// * `region` - Region to block
    pub fn block_region(&mut self, region: GeoRegion) {
        if !self.policy.blocked_regions.contains(&region) {
            self.policy.blocked_regions.push(region);
        }
    }

    /// Add a region to the allowed list
    ///
    /// # Arguments
    /// * `region` - Region to allow
    pub fn allow_region(&mut self, region: GeoRegion) {
        if !self.policy.allowed_regions.contains(&region) {
            self.policy.allowed_regions.push(region);
        }
    }
}

/// Advanced geographic policy with time-based restrictions
#[derive(Debug, Clone)]
pub struct AdvancedGeoPolicy {
    /// Base geographic policy
    base_policy: GeoPolicyEngine,
    /// Time-based restrictions by region
    time_restrictions: HashMap<GeoRegion, Vec<TimeRestriction>>,
}

/// Time-based restriction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    /// Day of week (0-6, where 0 is Sunday)
    pub day_of_week: u8,
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
}

impl AdvancedGeoPolicy {
    /// Create a new advanced geographic policy
    pub fn new(base_policy: GeoPolicyEngine) -> Self {
        Self {
            base_policy,
            time_restrictions: HashMap::new(),
        }
    }

    /// Add a time restriction for a region
    ///
    /// # Arguments
    /// * `region` - Geographic region
    /// * `restriction` - Time restriction to apply
    pub fn add_time_restriction(&mut self, region: GeoRegion, restriction: TimeRestriction) {
        self.time_restrictions
            .entry(region)
            .or_default()
            .push(restriction);
    }

    /// Evaluate if an IP is allowed with time restrictions
    ///
    /// # Arguments
    /// * `ip` - IP address to evaluate
    ///
    /// # Returns
    /// * `Result<bool>` - True if allowed, false if blocked
    pub fn is_ip_allowed_with_time(&self, ip: &IpAddr) -> Result<bool> {
        // First check base policy
        if !self.base_policy.is_ip_allowed(ip)? {
            return Ok(false);
        }

        // Get region for IP
        let region = match self.base_policy.get_region_for_ip(ip) {
            Some(region) => region,
            None => return Ok(true), // If we can't determine region, skip time restrictions
        };

        // Check time restrictions
        if let Some(restrictions) = self.time_restrictions.get(&region) {
            let now = chrono::Utc::now();
            let day_of_week = now.weekday().num_days_from_sunday() as u8;
            let hour = now.hour() as u8;

            for restriction in restrictions {
                if restriction.day_of_week == day_of_week
                    && hour >= restriction.start_hour
                    && hour <= restriction.end_hour
                {
                    return Ok(false); // Blocked due to time restriction
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_geo_region_creation() {
        let region = GeoRegion("US".to_string());
        assert_eq!(region.0, "US");
    }

    #[test]
    fn test_geo_policy_engine_creation() {
        let policy = GeoRestrictionPolicy::default();
        let engine = GeoPolicyEngine::new(policy);
        assert!(engine.policy.blocked_regions.is_empty());
        assert!(engine.policy.allowed_regions.is_empty());
        assert!(engine.policy.block_private_ips);
        assert!(!engine.policy.block_unknown_regions);
    }

    #[test]
    fn test_private_ip_detection() {
        let policy = GeoRestrictionPolicy::default();
        let engine = GeoPolicyEngine::new(policy);

        let private_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        assert!(engine.is_private_ip(&private_ip));

        let public_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        assert!(!engine.is_private_ip(&public_ip));
    }

    #[test]
    fn test_region_allowance() {
        let policy = GeoRestrictionPolicy {
            allowed_regions: vec![GeoRegion("US".to_string()), GeoRegion("CA".to_string())],
            blocked_regions: vec![GeoRegion("CN".to_string())],
            block_private_ips: true,
            block_unknown_regions: false,
        };

        let engine = GeoPolicyEngine::new(policy);

        // Test allowed region
        assert!(engine
            .is_region_allowed(&GeoRegion("US".to_string()))
            .unwrap());
        assert!(engine
            .is_region_allowed(&GeoRegion("CA".to_string()))
            .unwrap());

        // Test blocked region
        assert!(!engine
            .is_region_allowed(&GeoRegion("CN".to_string()))
            .unwrap());

        // Test region not in allowed list
        assert!(!engine
            .is_region_allowed(&GeoRegion("UK".to_string()))
            .unwrap());
    }

    #[test]
    fn test_ip_allowance() {
        let policy = GeoRestrictionPolicy::default();
        let engine = GeoPolicyEngine::new(policy);

        let public_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        assert!(engine.is_ip_allowed(&public_ip).unwrap());

        let private_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        assert!(!engine.is_ip_allowed(&private_ip).unwrap());
    }

    #[test]
    fn test_policy_modification() {
        let policy = GeoRestrictionPolicy::default();
        let mut engine = GeoPolicyEngine::new(policy);

        engine.block_region(GeoRegion("CN".to_string()));
        assert!(engine
            .policy
            .blocked_regions
            .contains(&GeoRegion("CN".to_string())));

        engine.allow_region(GeoRegion("US".to_string()));
        assert!(engine
            .policy
            .allowed_regions
            .contains(&GeoRegion("US".to_string())));
    }

    #[test]
    fn test_advanced_geo_policy() {
        let base_policy = GeoPolicyEngine::new(GeoRestrictionPolicy::default());
        let mut advanced_policy = AdvancedGeoPolicy::new(base_policy);

        let restriction = TimeRestriction {
            day_of_week: 0, // Sunday
            start_hour: 0,
            end_hour: 23,
        };

        advanced_policy.add_time_restriction(GeoRegion("US".to_string()), restriction);
        assert!(advanced_policy
            .time_restrictions
            .contains_key(&GeoRegion("US".to_string())));
    }
}
