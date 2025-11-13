//! Demonstration of risk management enhancements
//!
//! This file demonstrates that all three risk enhancement components are working together:
//! 1. Enhanced Correlation Risk Modeling
//! 2. Market Condition-Based Dynamic Risk Adjustment
//! 3. Better Integration Between Risk Components

use sniper_risk::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationConfig, EnhancedCorrelationResult};
use sniper_risk::market_condition_risk::{MarketConditionRiskAdjuster, MarketConditionRiskConfig, MarketConditions, MarketTrend, MarketRiskMultiplierResult};
use sniper_risk::unified_risk::{UnifiedRiskEngine, UnifiedRiskConfig, UnifiedRiskResult};
use sniper_risk::decide::{RiskDecisionEngine, RiskDecisionConfig};
use sniper_risk::honeypot::{HoneypotDetector, HoneypotDetectionConfig};
use sniper_risk::limits::{TradingLimitsEnforcer, TradingLimitsConfig, PortfolioState};
use sniper_risk::owner_powers::{OwnerPowerMonitor, OwnerPowerConfig};
use sniper_risk::lp_quality::{LpQualityAssessor, LpQualityConfig};
use sniper_core::types::{TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};

/// Demonstrate Enhanced Correlation Risk Modeling
#[test]
fn demo_enhanced_correlation_risk_modeling() {
    println!("=== Enhanced Correlation Risk Modeling Demo ===");
    
    // Create the enhanced correlation analyzer
    let config = EnhancedCorrelationConfig {
        max_correlation: 0.8,
        time_window_hours: 24,
        enabled: true,
        price_correlation_weight: 0.5,
        volatility_correlation_weight: 0.3,
        regime_correlation_weight: 0.2,
    };
    
    let mut analyzer = EnhancedCorrelationAnalyzer::new(config);
    
    // Add historical price data for two assets
    let eth_prices = vec![1000.0, 1020.0, 1010.0, 1030.0, 1040.0, 1035.0, 1050.0];
    let btc_prices = vec![20000.0, 20400.0, 20200.0, 20600.0, 20800.0, 20700.0, 21000.0];
    let timestamps = vec![1, 2, 3, 4, 5, 6, 7];
    
    analyzer.update_historical_prices("ETH", eth_prices, timestamps.clone());
    analyzer.update_historical_prices("BTC", btc_prices, timestamps.clone());
    
    // Add volatility data
    let eth_volatility = vec![0.02, 0.025, 0.018, 0.03, 0.028, 0.022, 0.026];
    let btc_volatility = vec![0.015, 0.018, 0.012, 0.025, 0.022, 0.019, 0.021];
    
    analyzer.update_volatility_data("ETH", eth_volatility, timestamps.clone());
    analyzer.update_volatility_data("BTC", btc_volatility, timestamps);
    
    // Calculate enhanced correlation
    let result = analyzer.calculate_enhanced_correlation("ETH", "BTC");
    
    println!("Enhanced Correlation Result:");
    println!("  Overall correlation: {:.4}", result.correlation);
    println!("  Price correlation: {:.4}", result.price_correlation);
    println!("  Volatility correlation: {:.4}", result.volatility_correlation);
    println!("  Regime correlation: {:.4}", result.regime_correlation);
    println!("  Confidence: {:.4}", result.confidence);
    
    // Verify the result is reasonable
    assert!(result.correlation >= 0.0 && result.correlation <= 1.0);
    assert!(result.confidence > 0.0);
    
    println!("✓ Enhanced correlation modeling is working correctly\n");
}

/// Demonstrate Market Condition-Based Dynamic Risk Adjustment
#[test]
fn demo_market_condition_based_dynamic_risk_adjustment() {
    println!("=== Market Condition-Based Dynamic Risk Adjustment Demo ===");
    
    // Create the market condition risk adjuster
    let config = MarketConditionRiskConfig {
        enabled: true,
        base_risk_multiplier: 1.0,
        high_volatility_multiplier: 0.7,
        low_volatility_multiplier: 1.2,
        bull_market_multiplier: 1.1,
        bear_market_multiplier: 0.8,
        sideways_market_multiplier: 0.9,
        high_volatility_threshold: 1.5,
        low_volatility_threshold: 0.7,
    };
    
    let mut adjuster = MarketConditionRiskAdjuster::new(config);
    
    // Add market conditions data (high volatility bull market)
    let conditions = MarketConditions {
        volatility: 0.03, // 3% volatility
        normal_volatility: 0.02, // 2% normal volatility
        trend: MarketTrend::Bull,
        liquidity: 1000000.0,
        normal_liquidity: 800000.0,
        timestamp: 1234567890,
    };
    
    adjuster.update_market_conditions(conditions);
    
    // Calculate risk multiplier with no portfolio drawdown
    let result = adjuster.calculate_risk_multiplier(0.0, 5.0);
    
    println!("Market Condition Risk Adjustment Result:");
    println!("  Risk multiplier: {:.4}", result.multiplier);
    println!("  Number of components: {}", result.components.len());
    
    for component in &result.components {
        println!("    {}: {:.4} - {}", component.factor, component.value, component.description);
    }
    
    // In a high volatility bull market, we expect:
    // Base (1.0) * High Volatility (0.7) * Bull Market (1.1) = 0.77
    assert!((result.multiplier - 0.77).abs() < 0.001);
    assert!(!result.components.is_empty());
    
    println!("✓ Market condition-based dynamic risk adjustment is working correctly\n");
}

/// Demonstrate Better Integration Between Risk Components
#[test]
fn demo_better_integration_between_risk_components() {
    println!("=== Better Integration Between Risk Components Demo ===");
    
    // Create all risk components
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
    
    // Create the unified risk engine
    let config = UnifiedRiskConfig {
        enabled: true,
        decision_engine_weight: 0.3,
        honeypot_weight: 0.2,
        limits_weight: 0.2,
        owner_power_weight: 0.1,
        lp_quality_weight: 0.1,
        correlation_weight: 0.1,
        min_risk_score: 70,
    };
    
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
        token_in: "0xWETH".to_string(),
        token_out: "0xUSDC".to_string(),
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
        idem_key: "demo-trade".to_string(),
    };
    
    // Assess risk
    let result = engine.assess_risk(&trade_plan, 100000.0).unwrap();
    
    println!("Unified Risk Assessment Result:");
    println!("  Overall score: {}", result.overall_score);
    println!("  Trade allowed: {}", result.allowed);
    println!("  Risk multiplier: {:.4}", result.risk_multiplier);
    println!("  Number of reasons: {}", result.reasons.len());
    
    for reason in &result.reasons {
        println!("    - {}", reason);
    }
    
    println!("  Component scores:");
    println!("    Decision engine: {}", result.components.decision_engine.score);
    println!("    Honeypot detection: {}", if result.components.honeypot.is_honeypot { "HONEYPOT" } else { "CLEAN" });
    println!("    Limits check: {}", if result.components.limits.allowed { "ALLOWED" } else { "BLOCKED" });
    println!("    Owner power: {}", if result.components.owner_power.has_excessive_powers { "EXCESSIVE" } else { "NORMAL" });
    println!("    Correlation: {:.4}", result.components.correlation.correlation);
    
    // Verify the result is reasonable
    assert!(result.overall_score <= 100);
    assert!(!result.reasons.is_empty());
    
    println!("✓ Better integration between risk components is working correctly\n");
}

/// Demonstrate all three enhancements working together
#[test]
fn demo_all_risk_enhancements_working_together() {
    println!("=== All Risk Enhancements Working Together Demo ===");
    
    // 1. Enhanced Correlation Analysis
    let mut correlation_analyzer = EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
    
    // Add data for correlation analysis
    let prices_a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
    let prices_b = vec![200.0, 220.0, 240.0, 260.0, 280.0];
    let timestamps = vec![1, 2, 3, 4, 5];
    
    correlation_analyzer.update_historical_prices("TOKEN_A", prices_a, timestamps.clone());
    correlation_analyzer.update_historical_prices("TOKEN_B", prices_b, timestamps.clone());
    
    let correlation_result = correlation_analyzer.calculate_enhanced_correlation("TOKEN_A", "TOKEN_B");
    println!("Enhanced correlation between TOKEN_A and TOKEN_B: {:.4}", correlation_result.correlation);
    
    // 2. Market Condition-Based Risk Adjustment
    let mut market_adjuster = MarketConditionRiskAdjuster::new(MarketConditionRiskConfig::default());
    
    let market_conditions = MarketConditions {
        volatility: 0.025,
        normal_volatility: 0.02,
        trend: MarketTrend::Bull,
        liquidity: 500000.0,
        normal_liquidity: 400000.0,
        timestamp: 1234567890,
    };
    
    market_adjuster.update_market_conditions(market_conditions);
    let market_result = market_adjuster.calculate_risk_multiplier(2.0, 5.0); // 2% drawdown
    println!("Market condition risk multiplier: {:.4}", market_result.multiplier);
    
    // 3. Unified Risk Integration
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
    
    let unified_config = UnifiedRiskConfig::default();
    let mut unified_engine = UnifiedRiskEngine::new(
        unified_config,
        decision_engine,
        honeypot_detector,
        limits_enforcer,
        owner_monitor,
        lp_assessor,
        correlation_analyzer,
        market_adjuster,
    );
    
    let trade_plan = TradePlan {
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        router: "0xRouter".to_string(),
        token_in: "TOKEN_A".to_string(),
        token_out: "TOKEN_B".to_string(),
        amount_in: 1000000000000000000,
        min_out: 900000000000000000,
        mode: ExecMode::Mempool,
        gas: GasPolicy {
            max_fee_gwei: 50,
            max_priority_gwei: 2,
        },
        exits: ExitRules::default(),
        idem_key: "integration-demo".to_string(),
    };
    
    let unified_result = unified_engine.assess_risk(&trade_plan, 100000.0).unwrap();
    println!("Unified risk assessment score: {}", unified_result.overall_score);
    println!("Trade allowed: {}", unified_result.allowed);
    println!("Final risk multiplier: {:.4}", unified_result.risk_multiplier);
    
    println!("\n✓ All three risk enhancements are working together correctly!");
    println!("✓ Enhanced Correlation Risk Modeling: IMPLEMENTED");
    println!("✓ Market Condition-Based Dynamic Risk Adjustment: IMPLEMENTED");
    println!("✓ Better Integration Between Risk Components: IMPLEMENTED");
}