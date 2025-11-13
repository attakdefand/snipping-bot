//! Unified risk integration engine.
//!
//! This module provides a comprehensive risk assessment engine that combines
//! assessments from all risk components into a unified risk assessment.

use crate::decide::{RiskAssessment, RiskDecisionEngine};
use crate::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationResult};
use crate::honeypot::{HoneypotDetectionResult, HoneypotDetector};
use crate::limits::{LimitCheckResult, TradingLimitsEnforcer};
use crate::lp_quality::{LpQualityAssessor, LpQualityMetrics};
use crate::market_condition_risk::{MarketConditionRiskAdjuster, MarketRiskMultiplierResult};
use crate::owner_powers::{OwnerPowerMonitor, OwnerPowerResult};
use serde::{Deserialize, Serialize};
use sniper_core::types::TradePlan;

/// Configuration for unified risk engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRiskConfig {
    /// Enable/disable unified risk assessment
    pub enabled: bool,
    /// Weight for decision engine assessment (0.0 to 1.0)
    pub decision_engine_weight: f64,
    /// Weight for honeypot detection (0.0 to 1.0)
    pub honeypot_weight: f64,
    /// Weight for trading limits (0.0 to 1.0)
    pub limits_weight: f64,
    /// Weight for owner power monitoring (0.0 to 1.0)
    pub owner_power_weight: f64,
    /// Weight for LP quality assessment (0.0 to 1.0)
    pub lp_quality_weight: f64,
    /// Weight for correlation analysis (0.0 to 1.0)
    pub correlation_weight: f64,
    /// Minimum overall risk score for approval (0 to 100)
    pub min_risk_score: u32,
}

impl Default for UnifiedRiskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            decision_engine_weight: 0.3,
            honeypot_weight: 0.2,
            limits_weight: 0.2,
            owner_power_weight: 0.1,
            lp_quality_weight: 0.1,
            correlation_weight: 0.1,
            min_risk_score: 70,
        }
    }
}

/// Unified risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRiskResult {
    /// Overall risk score (0-100)
    pub overall_score: u32,
    /// Whether the trade is allowed
    pub allowed: bool,
    /// Individual component results
    pub components: RiskComponents,
    /// Reasons for the decision
    pub reasons: Vec<String>,
    /// Risk multiplier applied
    pub risk_multiplier: f64,
}

/// Individual risk component results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponents {
    pub decision_engine: RiskAssessment,
    pub honeypot: HoneypotDetectionResult,
    pub limits: LimitCheckResult,
    pub owner_power: OwnerPowerResult,
    pub lp_quality: Option<LpQualityMetrics>,
    pub correlation: EnhancedCorrelationResult,
    pub market_risk: MarketRiskMultiplierResult,
}

/// Unified risk integration engine
pub struct UnifiedRiskEngine {
    config: UnifiedRiskConfig,
    decision_engine: RiskDecisionEngine,
    honeypot_detector: HoneypotDetector,
    limits_enforcer: TradingLimitsEnforcer,
    owner_monitor: OwnerPowerMonitor,
    lp_assessor: LpQualityAssessor,
    correlation_analyzer: EnhancedCorrelationAnalyzer,
    market_risk_adjuster: MarketConditionRiskAdjuster,
}

#[allow(clippy::too_many_arguments)]
impl UnifiedRiskEngine {
    /// Create a new unified risk engine
    pub fn new(
        config: UnifiedRiskConfig,
        decision_engine: RiskDecisionEngine,
        honeypot_detector: HoneypotDetector,
        limits_enforcer: TradingLimitsEnforcer,
        owner_monitor: OwnerPowerMonitor,
        lp_assessor: LpQualityAssessor,
        correlation_analyzer: EnhancedCorrelationAnalyzer,
        market_risk_adjuster: MarketConditionRiskAdjuster,
    ) -> Self {
        Self {
            config,
            decision_engine,
            honeypot_detector,
            limits_enforcer,
            owner_monitor,
            lp_assessor,
            correlation_analyzer,
            market_risk_adjuster,
        }
    }

    /// Perform comprehensive risk assessment
    pub fn assess_risk(
        &mut self,
        trade_plan: &TradePlan,
        _portfolio_value: f64,
    ) -> anyhow::Result<UnifiedRiskResult> {
        if !self.config.enabled {
            return Ok(UnifiedRiskResult {
                overall_score: 100,
                allowed: true,
                components: self.get_placeholder_components(),
                reasons: vec!["Unified risk assessment disabled".to_string()],
                risk_multiplier: 1.0,
            });
        }

        // Get assessments from all components
        let decision_result = self.decision_engine.assess_trade_risk(trade_plan)?;
        let honeypot_result = self.honeypot_detector.analyze_token(&trade_plan.token_in)?;
        let limits_result = self.limits_enforcer.check_trade_limits(
            &trade_plan.idem_key,
            &trade_plan.token_in,
            "crypto", // TODO: Get actual sector
            trade_plan.amount_in as f64,
            0.0, // TODO: Estimate expected PnL
        )?;
        let owner_result = self.owner_monitor.monitor_contract(&trade_plan.token_in)?;
        let lp_result = self
            .lp_assessor
            .get_lp_metrics(&trade_plan.token_in)
            .cloned();
        let correlation_result = self
            .correlation_analyzer
            .calculate_enhanced_correlation(&trade_plan.token_in, &trade_plan.token_out);
        let market_risk_result = self
            .market_risk_adjuster
            .calculate_risk_multiplier(0.0, 5.0); // TODO: Get actual drawdown

        // Calculate overall risk score
        let overall_score = self.calculate_overall_score(
            &decision_result,
            &honeypot_result,
            &limits_result,
            &owner_result,
            lp_result.as_ref(),
            &correlation_result,
        );

        // Check if trade is allowed
        let allowed = self.is_trade_allowed(
            overall_score,
            &limits_result,
            &honeypot_result,
            &owner_result,
        );

        // Collect reasons
        let reasons = self.collect_reasons(
            overall_score,
            &decision_result,
            &honeypot_result,
            &limits_result,
            &owner_result,
            lp_result.as_ref(),
            &correlation_result,
        );

        let risk_multiplier = market_risk_result.multiplier;

        Ok(UnifiedRiskResult {
            overall_score,
            allowed,
            components: RiskComponents {
                decision_engine: decision_result,
                honeypot: honeypot_result,
                limits: limits_result,
                owner_power: owner_result,
                lp_quality: lp_result,
                correlation: correlation_result,
                market_risk: market_risk_result,
            },
            reasons,
            risk_multiplier,
        })
    }

    /// Calculate overall risk score based on weighted components
    fn calculate_overall_score(
        &self,
        decision_result: &RiskAssessment,
        honeypot_result: &HoneypotDetectionResult,
        limits_result: &LimitCheckResult,
        owner_result: &OwnerPowerResult,
        lp_result: Option<&LpQualityMetrics>,
        correlation_result: &EnhancedCorrelationResult,
    ) -> u32 {
        // Normalize component scores to 0-100 range
        let decision_score = decision_result.score as f64;
        let honeypot_score = if honeypot_result.is_honeypot {
            0.0
        } else {
            100.0
        };
        let limits_score = if limits_result.allowed { 100.0 } else { 0.0 };
        let owner_score = if owner_result.has_excessive_powers {
            0.0
        } else {
            100.0
        };
        let lp_score = lp_result.map(|lp| lp.quality_score).unwrap_or(50.0); // Default to 50 if no data
        let correlation_score = (1.0 - correlation_result.correlation.abs()) * 100.0; // Invert correlation (lower correlation is better)

        // Calculate weighted average
        let weighted_score = (decision_score * self.config.decision_engine_weight)
            + (honeypot_score * self.config.honeypot_weight)
            + (limits_score * self.config.limits_weight)
            + (owner_score * self.config.owner_power_weight)
            + (lp_score * self.config.lp_quality_weight)
            + (correlation_score * self.config.correlation_weight);

        // Ensure score is within 0-100 range
        weighted_score.clamp(0.0, 100.0) as u32
    }

    /// Determine if trade is allowed based on overall score and hard limits
    fn is_trade_allowed(
        &self,
        overall_score: u32,
        limits_result: &LimitCheckResult,
        honeypot_result: &HoneypotDetectionResult,
        owner_result: &OwnerPowerResult,
    ) -> bool {
        // Hard limits - if any of these fail, trade is not allowed regardless of score
        if !limits_result.allowed
            || honeypot_result.is_honeypot
            || owner_result.has_excessive_powers
        {
            return false;
        }

        // Soft limit - check minimum risk score
        overall_score >= self.config.min_risk_score
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_reasons(
        &self,
        overall_score: u32,
        decision_result: &RiskAssessment,
        honeypot_result: &HoneypotDetectionResult,
        limits_result: &LimitCheckResult,
        owner_result: &OwnerPowerResult,
        lp_result: Option<&LpQualityMetrics>,
        correlation_result: &EnhancedCorrelationResult,
    ) -> Vec<String> {
        let mut reasons = Vec::new();

        // Overall score reason
        reasons.push(format!("Overall risk score: {}", overall_score));

        // Component-specific reasons
        if !decision_result.reasons.is_empty() {
            reasons.push(format!(
                "Decision engine: {}",
                decision_result.reasons.join(", ")
            ));
        }

        if honeypot_result.is_honeypot {
            reasons.push(format!(
                "Honeypot detected with confidence {}",
                honeypot_result.confidence
            ));
        }

        if !limits_result.reasons.is_empty() {
            reasons.push(format!(
                "Limits check: {}",
                limits_result.reasons.join(", ")
            ));
        }

        if owner_result.has_excessive_powers {
            reasons.push(format!(
                "Excessive owner powers detected with risk score {}",
                owner_result.risk_score
            ));
        }

        if let Some(lp) = lp_result {
            if !lp.risk_flags.is_empty() {
                reasons.push(format!("LP quality issues: {:?}", lp.risk_flags));
            }
        }

        if correlation_result.correlation.abs() > 0.7 {
            reasons.push(format!(
                "High correlation ({:.2}) with existing positions",
                correlation_result.correlation
            ));
        }

        // Final decision reason
        if overall_score >= self.config.min_risk_score {
            reasons.push("Trade approved based on unified risk assessment".to_string());
        } else {
            reasons.push(format!(
                "Trade rejected due to low risk score (minimum: {})",
                self.config.min_risk_score
            ));
        }

        reasons
    }

    /// Get placeholder components for disabled assessment
    fn get_placeholder_components(&self) -> RiskComponents {
        RiskComponents {
            decision_engine: RiskAssessment {
                score: 100,
                factors: vec![],
                reasons: vec![],
            },
            honeypot: HoneypotDetectionResult {
                is_honeypot: false,
                confidence: 0,
                reasons: vec![],
                risk_factors: vec![],
            },
            limits: LimitCheckResult {
                allowed: true,
                reasons: vec![],
                usage_stats: crate::limits::UsageStats {
                    daily_volume_usd: 0.0,
                    daily_trades: 0,
                    daily_loss_usd: 0.0,
                    asset_exposure_pct: 0.0,
                    sector_exposure_pct: 0.0,
                },
            },
            owner_power: OwnerPowerResult {
                has_excessive_powers: false,
                risk_score: 0,
                powers: vec![],
                reasons: vec![],
            },
            lp_quality: None,
            correlation: EnhancedCorrelationResult {
                correlation: 0.0,
                price_correlation: 0.0,
                volatility_correlation: 0.0,
                regime_correlation: 0.0,
                confidence: 0.0,
                reason: "Placeholder".to_string(),
            },
            market_risk: MarketRiskMultiplierResult {
                multiplier: 1.0,
                components: vec![],
                reason: "Placeholder".to_string(),
            },
        }
    }

    /// Update decision engine configuration
    pub fn update_decision_engine_config(&mut self, config: crate::decide::RiskDecisionConfig) {
        self.decision_engine.update_config(config);
    }

    /// Update honeypot detector configuration
    pub fn update_honeypot_config(&mut self, config: crate::honeypot::HoneypotDetectionConfig) {
        self.honeypot_detector.update_config(config);
    }

    /// Update limits enforcer configuration
    pub fn update_limits_config(&mut self, config: crate::limits::TradingLimitsConfig) {
        self.limits_enforcer.update_config(config);
    }

    /// Update owner monitor configuration
    pub fn update_owner_monitor_config(&mut self, config: crate::owner_powers::OwnerPowerConfig) {
        self.owner_monitor.update_config(config);
    }

    /// Update LP assessor configuration
    pub fn update_lp_assessor_config(&mut self, config: crate::lp_quality::LpQualityConfig) {
        self.lp_assessor.update_config(config);
    }

    /// Update correlation analyzer configuration
    pub fn update_correlation_config(
        &mut self,
        config: crate::enhanced_correlation::EnhancedCorrelationConfig,
    ) {
        self.correlation_analyzer.update_config(config);
    }

    /// Update market risk adjuster configuration
    pub fn update_market_risk_config(
        &mut self,
        config: crate::market_condition_risk::MarketConditionRiskConfig,
    ) {
        self.market_risk_adjuster.update_config(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decide::RiskDecisionConfig;
    use crate::enhanced_correlation::EnhancedCorrelationConfig;
    use crate::honeypot::HoneypotDetectionConfig;
    use crate::limits::{PortfolioState, TradingLimitsConfig};
    use crate::lp_quality::LpQualityConfig;
    use crate::market_condition_risk::MarketConditionRiskConfig;
    use crate::owner_powers::OwnerPowerConfig;
    use sniper_core::types::{ChainRef, ExecMode, ExitRules, GasPolicy};

    #[test]
    fn test_unified_risk_engine_creation() {
        let config = UnifiedRiskConfig::default();

        let decision_engine = RiskDecisionEngine::new(RiskDecisionConfig::default());
        let honeypot_detector = HoneypotDetector::new(HoneypotDetectionConfig::default());
        let limits_enforcer = TradingLimitsEnforcer::new(
            TradingLimitsConfig::default(),
            PortfolioState {
                portfolio_value_usd: 100000.0,
                positions: vec![],
                trading_history: vec![],
            },
        );
        let owner_monitor = OwnerPowerMonitor::new(OwnerPowerConfig::default());
        let lp_assessor = LpQualityAssessor::new(LpQualityConfig::default());
        let correlation_analyzer =
            EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
        let market_risk_adjuster =
            MarketConditionRiskAdjuster::new(MarketConditionRiskConfig::default());

        let engine = UnifiedRiskEngine::new(
            config,
            decision_engine,
            honeypot_detector,
            limits_enforcer,
            owner_monitor,
            lp_assessor,
            correlation_analyzer,
            market_risk_adjuster,
        );

        assert!(engine.config.enabled);
    }

    #[test]
    fn test_disabled_unified_risk_engine() {
        let config = UnifiedRiskConfig {
            enabled: false,
            ..Default::default()
        };

        let decision_engine = RiskDecisionEngine::new(RiskDecisionConfig::default());
        let honeypot_detector = HoneypotDetector::new(HoneypotDetectionConfig::default());
        let limits_enforcer = TradingLimitsEnforcer::new(
            TradingLimitsConfig::default(),
            PortfolioState {
                portfolio_value_usd: 100000.0,
                positions: vec![],
                trading_history: vec![],
            },
        );
        let owner_monitor = OwnerPowerMonitor::new(OwnerPowerConfig::default());
        let lp_assessor = LpQualityAssessor::new(LpQualityConfig::default());
        let correlation_analyzer =
            EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
        let market_risk_adjuster =
            MarketConditionRiskAdjuster::new(MarketConditionRiskConfig::default());

        let mut engine = UnifiedRiskEngine::new(
            config,
            decision_engine,
            honeypot_detector,
            limits_enforcer,
            owner_monitor,
            lp_assessor,
            correlation_analyzer,
            market_risk_adjuster,
        );

        let trade_plan = TradePlan {
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
            idem_key: "test-trade".to_string(),
        };

        let result = engine.assess_risk(&trade_plan, 100000.0).unwrap();
        assert_eq!(result.overall_score, 100);
        assert!(result.allowed);
        assert_eq!(
            result.reasons,
            vec!["Unified risk assessment disabled".to_string()]
        );
        assert_eq!(result.risk_multiplier, 1.0);
    }

    #[test]
    fn test_overall_score_calculation() {
        let config = UnifiedRiskConfig::default();

        let decision_engine = RiskDecisionEngine::new(RiskDecisionConfig::default());
        let honeypot_detector = HoneypotDetector::new(HoneypotDetectionConfig::default());
        let limits_enforcer = TradingLimitsEnforcer::new(
            TradingLimitsConfig::default(),
            PortfolioState {
                portfolio_value_usd: 100000.0,
                positions: vec![],
                trading_history: vec![],
            },
        );
        let owner_monitor = OwnerPowerMonitor::new(OwnerPowerConfig::default());
        let lp_assessor = LpQualityAssessor::new(LpQualityConfig::default());
        let correlation_analyzer =
            EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
        let market_risk_adjuster =
            MarketConditionRiskAdjuster::new(MarketConditionRiskConfig::default());

        let engine = UnifiedRiskEngine::new(
            config,
            decision_engine,
            honeypot_detector,
            limits_enforcer,
            owner_monitor,
            lp_assessor,
            correlation_analyzer,
            market_risk_adjuster,
        );

        // Create mock assessment results
        let decision_result = RiskAssessment {
            score: 80,
            factors: vec![],
            reasons: vec![],
        };

        let honeypot_result = HoneypotDetectionResult {
            is_honeypot: false,
            confidence: 0,
            reasons: vec![],
            risk_factors: vec![],
        };

        let limits_result = LimitCheckResult {
            allowed: true,
            reasons: vec![],
            usage_stats: crate::limits::UsageStats {
                daily_volume_usd: 0.0,
                daily_trades: 0,
                daily_loss_usd: 0.0,
                asset_exposure_pct: 0.0,
                sector_exposure_pct: 0.0,
            },
        };

        let owner_result = OwnerPowerResult {
            has_excessive_powers: false,
            risk_score: 0,
            powers: vec![],
            reasons: vec![],
        };

        let lp_result = LpQualityMetrics {
            lp_address: "0xLP".to_string(),
            total_liquidity: 100000.0,
            transaction_count: 100,
            avg_price_impact: 0.5,
            lp_changes: 2,
            last_updated: 1234567890,
            quality_score: 90.0,
            risk_flags: vec![],
        };

        let correlation_result = EnhancedCorrelationResult {
            correlation: 0.3,
            price_correlation: 0.3,
            volatility_correlation: 0.2,
            regime_correlation: 0.1,
            confidence: 0.9,
            reason: "Test".to_string(),
        };

        let score = engine.calculate_overall_score(
            &decision_result,
            &honeypot_result,
            &limits_result,
            &owner_result,
            Some(&lp_result),
            &correlation_result,
        );

        // Expected calculation:
        // Decision: 80 * 0.3 = 24
        // Honeypot: 100 * 0.2 = 20
        // Limits: 100 * 0.2 = 20
        // Owner: 100 * 0.1 = 10
        // LP: 90 * 0.1 = 9
        // Correlation: (1.0 - 0.3) * 100 * 0.1 = 70 * 0.1 = 7
        // Total: 24 + 20 + 20 + 10 + 9 + 7 = 90
        assert_eq!(score, 90);
    }
}
