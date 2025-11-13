//! Integration tests for risk management enhancements
//!
//! This file contains integration tests that verify the enhanced risk management
//! components work together correctly.

use sniper_risk::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationConfig};
use sniper_risk::market_condition_risk::{MarketConditionRiskAdjuster, MarketConditionRiskConfig, MarketConditions, MarketTrend};
use sniper_risk::unified_risk::{UnifiedRiskEngine, UnifiedRiskConfig};
use sniper_risk::decide::{RiskDecisionEngine, RiskDecisionConfig};
use sniper_risk::honeypot::{HoneypotDetector, HoneypotDetectionConfig};
use sniper_risk::limits::{TradingLimitsEnforcer, TradingLimitsConfig, PortfolioState};
use sniper_risk::owner_powers::{OwnerPowerMonitor, OwnerPowerConfig};
use sniper_risk::lp_quality::{LpQualityAssessor, LpQualityConfig};
use sniper_risk::enhanced_correlation::EnhancedCorrelationAnalyzer;
use sniper_risk::market_condition_risk::MarketConditionRiskAdjuster;
use sniper_core::types::{TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};

#[test]
fn test_enhanced_correlation_module_exists() {
    // Check that the enhanced correlation module can be instantiated
    let config = EnhancedCorrelationConfig::default();
    let analyzer = EnhancedCorrelationAnalyzer::new(config);
    assert_eq!(analyzer.config.max_correlation, 0.8);
}

#[test]
fn test_market_condition_risk_module_exists() {
    // Check that the market condition risk module can be instantiated
    let config = MarketConditionRiskConfig::default();
    let adjuster = MarketConditionRiskAdjuster::new(config);
    assert!(adjuster.config.enabled);
}

#[test]
fn test_unified_risk_engine_module_exists() {
    // Check that the unified risk engine can be instantiated with all components
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
    let correlation_analyzer = EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
    let market_risk_adjuster = MarketConditionRiskAdjuster::new(MarketConditionRiskConfig::default());
    
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
fn test_enhanced_correlation_functionality() {
    // Test the enhanced correlation analyzer functionality
    let config = EnhancedCorrelationConfig::default();
    let mut analyzer = EnhancedCorrelationAnalyzer::new(config);
    
    // Add some historical price data
    let prices_a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
    let prices_b = vec![200.0, 220.0, 240.0, 260.0, 280.0];
    let timestamps = vec![1, 2, 3, 4, 5];
    
    analyzer.update_historical_prices("ETH", prices_a, timestamps.clone());
    analyzer.update_historical_prices("BTC", prices_b, timestamps);
    
    // Calculate correlation
    let result = analyzer.calculate_enhanced_correlation("ETH", "BTC");
    
    // Should have some correlation value
    assert!(result.correlation >= 0.0);
    assert!(result.correlation <= 1.0);
    assert!(result.confidence > 0.0);
}

#[test]
fn test_market_condition_risk_functionality() {
    // Test the market condition risk adjuster functionality
    let config = MarketConditionRiskConfig::default();
    let mut adjuster = MarketConditionRiskAdjuster::new(config);
    
    // Add market conditions data
    let conditions = MarketConditions {
        volatility: 0.15,
        normal_volatility: 0.1,
        trend: MarketTrend::Bull,
        liquidity: 1000000.0,
        normal_liquidity: 800000.0,
        timestamp: 1234567890,
    };
    
    adjuster.update_market_conditions(conditions);
    
    // Calculate risk multiplier
    let result = adjuster.calculate_risk_multiplier(0.0, 5.0);
    
    // Should have a valid multiplier
    assert!(result.multiplier > 0.0);
    assert!(!result.components.is_empty());
}

#[test]
fn test_unified_risk_engine_functionality() {
    // Test the unified risk engine functionality
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
    let correlation_analyzer = EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
    let market_risk_adjuster = MarketConditionRiskAdjuster::new(MarketConditionRiskConfig::default());
    
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
    
    // Create a trade plan
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
    
    // Assess risk
    let result = engine.assess_risk(&trade_plan, 100000.0).unwrap();
    
    // Should have a valid result
    assert!(result.overall_score <= 100);
    assert!(!result.reasons.is_empty());
}

#[test]
fn test_module_integration() {
    // Test that all modules can work together
    let correlation_config = EnhancedCorrelationConfig::default();
    let mut correlation_analyzer = EnhancedCorrelationAnalyzer::new(correlation_config);
    
    // Add some data to the correlation analyzer
    let prices_a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
    let prices_b = vec![200.0, 220.0, 240.0, 260.0, 280.0];
    let timestamps = vec![1, 2, 3, 4, 5];
    
    correlation_analyzer.update_historical_prices("ETH", prices_a, timestamps.clone());
    correlation_analyzer.update_historical_prices("BTC", prices_b, timestamps);
    
    let correlation_result = correlation_analyzer.calculate_enhanced_correlation("ETH", "BTC");
    assert!(correlation_result.correlation >= 0.0);
    
    let market_config = MarketConditionRiskConfig::default();
    let mut market_adjuster = MarketConditionRiskAdjuster::new(market_config);
    
    let conditions = MarketConditions {
        volatility: 0.15,
        normal_volatility: 0.1,
        trend: MarketTrend::Bull,
        liquidity: 1000000.0,
        normal_liquidity: 800000.0,
        timestamp: 1234567890,
    };
    
    market_adjuster.update_market_conditions(conditions);
    let market_result = market_adjuster.calculate_risk_multiplier(0.0, 5.0);
    assert!(market_result.multiplier > 0.0);
}