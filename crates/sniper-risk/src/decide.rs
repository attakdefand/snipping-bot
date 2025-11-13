//! Risk decision engine implementation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sniper_core::types::{Decision, TradePlan};
use std::collections::HashMap;
use tracing::{debug, info};

/// Risk decision configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecisionConfig {
    /// Maximum allowed price impact (percentage)
    pub max_price_impact: f64,
    /// Maximum allowed slippage (percentage)
    pub max_slippage: f64,
    /// Minimum liquidity required (in USD)
    pub min_liquidity: f64,
    /// Maximum number of hops in a trade route
    pub max_hops: usize,
    /// Enable/disable risk checks
    pub enabled: bool,
    /// Custom risk rules
    pub custom_rules: Vec<CustomRiskRule>,
}

impl Default for RiskDecisionConfig {
    fn default() -> Self {
        Self {
            max_price_impact: 5.0, // 5% price impact
            max_slippage: 3.0,     // 3% slippage
            min_liquidity: 1000.0, // $1000 minimum liquidity
            max_hops: 3,           // Maximum 3 hops
            enabled: true,
            custom_rules: Vec::new(),
        }
    }
}

/// Custom risk rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRiskRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule condition (in a real implementation, this would be more complex)
    pub condition: String,
    /// Risk score impact (-100 to 100)
    pub score_impact: i32,
}

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk score (0-100, where 0 is highest risk)
    pub score: u32,
    /// Individual risk factors
    pub factors: Vec<RiskFactor>,
    /// Reasons for the assessment
    pub reasons: Vec<String>,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor name
    pub name: String,
    /// Factor score (0-100)
    pub score: u32,
    /// Weight of this factor in the overall score
    pub weight: f64,
}

/// Risk decision engine
pub struct RiskDecisionEngine {
    /// Configuration
    config: RiskDecisionConfig,
    /// Risk assessment cache
    assessment_cache: HashMap<String, RiskAssessment>,
}

impl RiskDecisionEngine {
    /// Create a new risk decision engine
    pub fn new(config: RiskDecisionConfig) -> Self {
        Self {
            config,
            assessment_cache: HashMap::new(),
        }
    }

    /// Evaluate a trade plan and make a risk decision
    ///
    /// # Arguments
    /// * `plan` - Trade plan to evaluate
    ///
    /// # Returns
    /// * `Result<Decision>` - Decision indicating whether the trade should proceed
    pub fn evaluate_trade(&mut self, plan: &TradePlan) -> Result<Decision> {
        debug!("Evaluating trade plan: {}", plan.idem_key);

        if !self.config.enabled {
            return Ok(Decision {
                allow: true,
                reasons: vec!["Risk checks disabled".to_string()],
            });
        }

        // Perform risk assessment
        let assessment = self.assess_trade_risk(plan)?;

        // Cache the assessment
        self.assessment_cache
            .insert(plan.idem_key.clone(), assessment.clone());

        // Make decision based on assessment
        let allow = assessment.score >= 70; // Minimum acceptable risk score

        let mut reasons = assessment.reasons.clone();
        if allow {
            reasons.push(format!("Risk score acceptable: {}", assessment.score));
        } else {
            reasons.push(format!("Risk score too low: {}", assessment.score));
        }

        info!(
            "Trade decision for {}: allow = {}, score = {}",
            plan.idem_key, allow, assessment.score
        );

        Ok(Decision { allow, reasons })
    }

    /// Assess the risk of a trade plan
    ///
    /// # Arguments
    /// * `plan` - Trade plan to assess
    ///
    /// # Returns
    /// * `Result<RiskAssessment>` - Risk assessment result
    pub fn assess_trade_risk(&self, plan: &TradePlan) -> Result<RiskAssessment> {
        let mut factors = Vec::new();
        let mut reasons = Vec::new();
        let mut total_score: f64 = 0.0;
        let mut total_weight: f64 = 0.0;

        // Price impact factor (weight: 0.3)
        let price_impact_factor = self.assess_price_impact(plan);
        factors.push(price_impact_factor.clone());
        total_score += price_impact_factor.score as f64 * price_impact_factor.weight;
        total_weight += price_impact_factor.weight;

        if price_impact_factor.score < 70 {
            reasons.push("High price impact detected".to_string());
        }

        // Slippage factor (weight: 0.25)
        let slippage_factor = self.assess_slippage(plan);
        factors.push(slippage_factor.clone());
        total_score += slippage_factor.score as f64 * slippage_factor.weight;
        total_weight += slippage_factor.weight;

        if slippage_factor.score < 70 {
            reasons.push("High slippage detected".to_string());
        }

        // Liquidity factor (weight: 0.25)
        let liquidity_factor = self.assess_liquidity(plan);
        factors.push(liquidity_factor.clone());
        total_score += liquidity_factor.score as f64 * liquidity_factor.weight;
        total_weight += liquidity_factor.weight;

        if liquidity_factor.score < 70 {
            reasons.push("Insufficient liquidity".to_string());
        }

        // Complexity factor (weight: 0.1)
        let complexity_factor = self.assess_complexity(plan);
        factors.push(complexity_factor.clone());
        total_score += complexity_factor.score as f64 * complexity_factor.weight;
        total_weight += complexity_factor.weight;

        if complexity_factor.score < 70 {
            reasons.push("High trade complexity".to_string());
        }

        // Custom rules factor (weight: 0.1)
        let custom_rules_factor = self.assess_custom_rules(plan);
        factors.push(custom_rules_factor.clone());
        total_score += custom_rules_factor.score as f64 * custom_rules_factor.weight;
        total_weight += custom_rules_factor.weight;

        if custom_rules_factor.score < 70 {
            reasons.push("Custom risk rules triggered".to_string());
        }

        // Calculate overall score
        let overall_score = if total_weight > 0.0 {
            (total_score / total_weight) as u32
        } else {
            100
        };

        // Add overall reasons
        if overall_score >= 90 {
            reasons.push("Low risk trade".to_string());
        } else if overall_score >= 70 {
            reasons.push("Moderate risk trade".to_string());
        } else {
            reasons.push("High risk trade".to_string());
        }

        Ok(RiskAssessment {
            score: overall_score,
            factors,
            reasons,
        })
    }

    /// Assess price impact risk
    fn assess_price_impact(&self, plan: &TradePlan) -> RiskFactor {
        // In a real implementation, this would calculate actual price impact
        // For this implementation, we'll simulate based on the plan
        let price_impact = self.calculate_simulated_price_impact(plan);

        let score = if price_impact <= self.config.max_price_impact * 0.5 {
            100
        } else if price_impact <= self.config.max_price_impact {
            (100.0 - (price_impact / self.config.max_price_impact) * 30.0) as u32
        } else {
            (70.0 - (price_impact / self.config.max_price_impact) * 30.0).max(0.0) as u32
        };

        RiskFactor {
            name: "Price Impact".to_string(),
            score,
            weight: 0.3,
        }
    }

    /// Assess slippage risk
    fn assess_slippage(&self, plan: &TradePlan) -> RiskFactor {
        // In a real implementation, this would calculate actual slippage
        // For this implementation, we'll simulate based on the plan
        let slippage = self.calculate_simulated_slippage(plan);

        let score = if slippage <= self.config.max_slippage * 0.5 {
            100
        } else if slippage <= self.config.max_slippage {
            (100.0 - (slippage / self.config.max_slippage) * 30.0) as u32
        } else {
            (70.0 - (slippage / self.config.max_slippage) * 30.0).max(0.0) as u32
        };

        RiskFactor {
            name: "Slippage".to_string(),
            score,
            weight: 0.25,
        }
    }

    /// Assess liquidity risk
    fn assess_liquidity(&self, plan: &TradePlan) -> RiskFactor {
        // In a real implementation, this would check actual liquidity
        // For this implementation, we'll simulate based on the plan
        let liquidity = self.calculate_simulated_liquidity(plan);

        let score = if liquidity >= self.config.min_liquidity * 2.0 {
            100
        } else if liquidity >= self.config.min_liquidity {
            (70.0 + (liquidity / self.config.min_liquidity - 1.0) * 30.0) as u32
        } else {
            (70.0 * (liquidity / self.config.min_liquidity)).max(0.0) as u32
        };

        RiskFactor {
            name: "Liquidity".to_string(),
            score,
            weight: 0.25,
        }
    }

    /// Assess trade complexity risk
    fn assess_complexity(&self, plan: &TradePlan) -> RiskFactor {
        // In a real implementation, this would analyze the trade route complexity
        // For this implementation, we'll simulate based on the plan
        let hops = self.calculate_simulated_hops(plan);

        let score = if hops <= self.config.max_hops / 2 {
            100
        } else if hops <= self.config.max_hops {
            (100.0 - (hops as f64 / self.config.max_hops as f64) * 30.0) as u32
        } else {
            (70.0 - (hops as f64 / self.config.max_hops as f64) * 30.0).max(0.0) as u32
        };

        RiskFactor {
            name: "Complexity".to_string(),
            score,
            weight: 0.1,
        }
    }

    /// Assess custom rules risk
    fn assess_custom_rules(&self, _plan: &TradePlan) -> RiskFactor {
        // In a real implementation, this would evaluate custom rules
        // For this implementation, we'll use a simple scoring

        let mut total_impact: i32 = 0;
        for rule in &self.config.custom_rules {
            // In a real implementation, we would evaluate the rule condition
            // For this implementation, we'll just apply the score impact
            total_impact += rule.score_impact;
        }

        let score = (100i32 + total_impact).clamp(0, 100) as u32;

        RiskFactor {
            name: "Custom Rules".to_string(),
            score,
            weight: 0.1,
        }
    }

    /// Calculate simulated price impact
    fn calculate_simulated_price_impact(&self, plan: &TradePlan) -> f64 {
        // Simulate price impact calculation
        // In a real implementation, this would use actual pool data
        let amount_ratio = plan.amount_in as f64 / 1000000000000000000.0; // Ratio to 1 ETH
        amount_ratio * 0.5 // Simulate 0.5% price impact per ETH
    }

    /// Calculate simulated slippage
    fn calculate_simulated_slippage(&self, plan: &TradePlan) -> f64 {
        // Simulate slippage calculation
        let price_impact = self.calculate_simulated_price_impact(plan);
        price_impact * 1.2 // Slippage is typically higher than price impact
    }

    /// Calculate simulated liquidity
    fn calculate_simulated_liquidity(&self, _plan: &TradePlan) -> f64 {
        // Simulate liquidity calculation
        // In a real implementation, this would query the pool's liquidity
        5000.0 // Simulate $5000 liquidity
    }

    /// Calculate simulated number of hops
    fn calculate_simulated_hops(&self, _plan: &TradePlan) -> usize {
        // Simulate hop calculation
        // In a real implementation, this would analyze the trade route
        2 // Simulate 2-hop trade
    }

    /// Get cached risk assessment for a trade
    ///
    /// # Arguments
    /// * `trade_id` - Trade identifier
    ///
    /// # Returns
    /// * `Option<&RiskAssessment>` - Cached risk assessment or None
    pub fn get_cached_assessment(&self, trade_id: &str) -> Option<&RiskAssessment> {
        self.assessment_cache.get(trade_id)
    }

    /// Clear cached assessments
    pub fn clear_cache(&mut self) {
        self.assessment_cache.clear();
    }

    /// Update configuration
    ///
    /// # Arguments
    /// * `config` - New configuration
    pub fn update_config(&mut self, config: RiskDecisionConfig) {
        self.config = config;
    }

    /// Add a custom risk rule
    ///
    /// # Arguments
    /// * `rule` - Custom risk rule to add
    pub fn add_custom_rule(&mut self, rule: CustomRiskRule) {
        self.config.custom_rules.push(rule);
    }
}

/// Advanced risk decision engine with machine learning capabilities
pub struct AdvancedRiskDecisionEngine {
    /// Base risk decision engine
    base_engine: RiskDecisionEngine,
    /// Historical decision data for learning
    historical_data: Vec<TradeOutcome>,
    /// Learning rate for model updates
    learning_rate: f64,
}

/// Trade outcome for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOutcome {
    /// Trade identifier
    pub trade_id: String,
    /// Actual profit/loss
    pub pnl: f64,
    /// Risk score at time of decision
    pub risk_score: u32,
    /// Timestamp
    pub timestamp: u64,
}

impl AdvancedRiskDecisionEngine {
    /// Create a new advanced risk decision engine
    pub fn new(base_engine: RiskDecisionEngine) -> Self {
        Self {
            base_engine,
            historical_data: Vec::new(),
            learning_rate: 0.01,
        }
    }

    /// Evaluate a trade with learning capabilities
    ///
    /// # Arguments
    /// * `plan` - Trade plan to evaluate
    ///
    /// # Returns
    /// * `Result<Decision>` - Decision with learning-enhanced accuracy
    pub fn evaluate_trade_with_learning(&mut self, plan: &TradePlan) -> Result<Decision> {
        // Get base decision
        let mut decision = self.base_engine.evaluate_trade(plan)?;

        // Apply learning adjustments
        if let Some(adjusted_score) = self.adjust_risk_score(plan) {
            // Update decision based on learning
            decision.allow = adjusted_score >= 70;

            if decision.allow && adjusted_score < 70 {
                decision
                    .reasons
                    .push("Learning model suggests higher risk".to_string());
            } else if !decision.allow && adjusted_score >= 70 {
                decision
                    .reasons
                    .push("Learning model suggests lower risk".to_string());
            }
        }

        Ok(decision)
    }

    /// Adjust risk score based on historical data
    fn adjust_risk_score(&self, plan: &TradePlan) -> Option<u32> {
        // In a real implementation, this would use ML models
        // For this implementation, we'll simulate with a simple approach

        // Get base assessment
        let assessment = match self.base_engine.assess_trade_risk(plan) {
            Ok(a) => a,
            Err(_) => return None,
        };

        // Simple adjustment based on historical patterns
        let mut adjustment: i32 = 0;

        // If we have historical data, adjust based on patterns
        if !self.historical_data.is_empty() {
            let avg_pnl: f64 = self.historical_data.iter().map(|d| d.pnl).sum::<f64>()
                / self.historical_data.len() as f64;

            // If average PnL is negative, increase risk score requirement
            if avg_pnl < 0.0 {
                adjustment -= 10;
            } else if avg_pnl > 0.05 {
                adjustment += 5;
            }
        }

        let adjusted_score = (assessment.score as i32 + adjustment).clamp(0, 100) as u32;
        Some(adjusted_score)
    }

    /// Record trade outcome for learning
    ///
    /// # Arguments
    /// * `outcome` - Trade outcome data
    pub fn record_trade_outcome(&mut self, outcome: TradeOutcome) {
        self.historical_data.push(outcome);

        // Keep only recent data (last 1000 trades)
        if self.historical_data.len() > 1000 {
            self.historical_data
                .drain(0..self.historical_data.len() - 1000);
        }
    }

    /// Update learning rate
    ///
    /// # Arguments
    /// * `rate` - New learning rate
    pub fn update_learning_rate(&mut self, rate: f64) {
        self.learning_rate = rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExecMode, ExitRules, GasPolicy};

    #[test]
    fn test_risk_decision_config() {
        let config = RiskDecisionConfig::default();
        assert_eq!(config.max_price_impact, 5.0);
        assert_eq!(config.max_slippage, 3.0);
        assert_eq!(config.min_liquidity, 1000.0);
        assert_eq!(config.max_hops, 3);
        assert!(config.enabled);
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_risk_decision_engine_creation() {
        let config = RiskDecisionConfig::default();
        let engine = RiskDecisionEngine::new(config);
        assert!(engine.assessment_cache.is_empty());
    }

    #[test]
    fn test_trade_evaluation_allowed() {
        let config = RiskDecisionConfig::default();
        let mut engine = RiskDecisionEngine::new(config);

        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in: 1000000000000000000, // 1 ETH
            min_out: 900000000000000000,    // 0.9 ETH worth of tokens
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules {
                take_profit_pct: Some(10.0),
                stop_loss_pct: Some(5.0),
                trailing_pct: Some(2.0),
            },
            idem_key: "test-trade-1".to_string(),
        };

        let decision = engine.evaluate_trade(&plan).unwrap();
        assert!(decision.allow);
        assert!(!decision.reasons.is_empty());
    }

    #[test]
    fn test_risk_assessment() {
        let config = RiskDecisionConfig::default();
        let engine = RiskDecisionEngine::new(config);

        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in: 1000000000000000000, // 1 ETH
            min_out: 900000000000000000,    // 0.9 ETH worth of tokens
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules {
                take_profit_pct: Some(10.0),
                stop_loss_pct: Some(5.0),
                trailing_pct: Some(2.0),
            },
            idem_key: "test-trade-2".to_string(),
        };

        let assessment = engine.assess_trade_risk(&plan).unwrap();
        assert!(assessment.score > 0);
        assert!(!assessment.factors.is_empty());
        assert!(!assessment.reasons.is_empty());
    }

    #[test]
    fn test_disabled_risk_checks() {
        let config = RiskDecisionConfig {
            enabled: false,
            ..Default::default()
        };
        let mut engine = RiskDecisionEngine::new(config);

        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in: 1000000000000000000, // 1 ETH
            min_out: 900000000000000000,    // 0.9 ETH worth of tokens
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules {
                take_profit_pct: Some(10.0),
                stop_loss_pct: Some(5.0),
                trailing_pct: Some(2.0),
            },
            idem_key: "test-trade-3".to_string(),
        };

        let decision = engine.evaluate_trade(&plan).unwrap();
        assert!(decision.allow);
        assert_eq!(decision.reasons, vec!["Risk checks disabled".to_string()]);
    }

    #[test]
    fn test_custom_rules() {
        let mut config = RiskDecisionConfig::default();
        config.custom_rules.push(CustomRiskRule {
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            condition: "test".to_string(),
            score_impact: -20,
        });

        let engine = RiskDecisionEngine::new(config);
        assert_eq!(engine.config.custom_rules.len(), 1);
    }

    #[test]
    fn test_advanced_risk_engine() {
        let config = RiskDecisionConfig::default();
        let base_engine = RiskDecisionEngine::new(config);
        let mut advanced_engine = AdvancedRiskDecisionEngine::new(base_engine);

        let outcome = TradeOutcome {
            trade_id: "test-trade".to_string(),
            pnl: 0.05,
            risk_score: 80,
            timestamp: 1234567890,
        };

        advanced_engine.record_trade_outcome(outcome);
        assert_eq!(advanced_engine.historical_data.len(), 1);
    }
}
