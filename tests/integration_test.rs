//! Integration tests for the sniper bot components
//!
//! This file tests the integration between different components of the sniper bot
//! including execution modules, gas estimation, and key management.

use sniper_core::types::{ChainRef, TradePlan, ExecMode, GasPolicy, ExitRules};
use sniper_exec::{exec_mempool, exec_private, gas};
use sniper_keys::mpc;

/// Test integration between gas estimation and execution modules
#[tokio::test]
async fn test_gas_estimation_and_execution_integration() {
    // Create a gas estimator
    let estimator = gas::GasEstimator::new(30, 2);
    
    // Create a trade plan
    let plan = TradePlan {
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        router: "0xRouterAddress".to_string(),
        token_in: "0xWETH".to_string(),
        token_out: "0xToken".to_string(),
        amount_in: 1000000000000000000, // 1 ETH
        min_out: 900000000000000000,    // 0.9 tokens
        mode: ExecMode::Mempool,
        gas: GasPolicy {
            max_fee_gwei: 50,
            max_priority_gwei: 2,
        },
        exits: ExitRules::default(),
        idem_key: "integration_test_1".to_string(),
    };
    
    // Estimate gas
    let gas_policy = estimator.estimate_gas(&plan);
    assert_eq!(gas_policy.max_fee_gwei, 32); // 30 + 2
    assert_eq!(gas_policy.max_priority_gwei, 2);
    
    // Execute trade with mempool
    let receipt = exec_mempool::execute_via_mempool(&plan).await;
    assert!(receipt.is_ok());
    
    let receipt = receipt.unwrap();
    assert!(receipt.success);
    assert!(receipt.tx_hash.starts_with("0x"));
}

/// Test integration of MPC key management
#[tokio::test]
async fn test_mpc_key_management_integration() {
    // Create MPC manager
    let manager = mpc::MpcKeyManager::new("integration-test-participant".to_string(), 2, 3);
    
    // Verify it can sign
    assert!(manager.can_sign());
    
    // Generate a key share
    let key_result = manager.generate_key_share().await;
    assert!(key_result.is_ok());
    
    let key_id = key_result.unwrap();
    assert!(key_id.starts_with("mpc-key-"));
    
    // Sign a transaction
    let transaction_data = b"integration test transaction";
    let signature_result = manager.sign_transaction(&key_id, transaction_data).await;
    assert!(signature_result.is_ok());
    
    let signature = signature_result.unwrap();
    assert!(!signature.is_empty());
}

/// Test execution modes integration
#[tokio::test]
async fn test_execution_modes_integration() {
    let plan = TradePlan {
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        router: "0xRouterAddress".to_string(),
        token_in: "0xWETH".to_string(),
        token_out: "0xToken".to_string(),
        amount_in: 1000000000000000000, // 1 ETH
        min_out: 900000000000000000,    // 0.9 tokens
        mode: ExecMode::Mempool,
        gas: GasPolicy {
            max_fee_gwei: 50,
            max_priority_gwei: 2,
        },
        exits: ExitRules::default(),
        idem_key: "execution_modes_test".to_string(),
    };
    
    // Test mempool execution
    let mempool_result = exec_mempool::execute_via_mempool(&plan).await;
    assert!(mempool_result.is_ok());
    
    // Test private execution
    let private_result = exec_private::execute_via_private(&plan).await;
    assert!(private_result.is_ok());
    
    // Verify both produced valid receipts
    let mempool_receipt = mempool_result.unwrap();
    let private_receipt = private_result.unwrap();
    
    assert!(mempool_receipt.success);
    assert!(private_receipt.success);
    assert!(mempool_receipt.tx_hash.starts_with("0x"));
    assert!(private_receipt.tx_hash.starts_with("0x"));
}

/// Test enhanced correlation module integration
#[test]
fn test_risk_enhanced_correlation_module_integration() {
    use sniper_risk::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationConfig};
    
    // Check that the enhanced correlation module can be instantiated
    let config = EnhancedCorrelationConfig::default();
    let analyzer = EnhancedCorrelationAnalyzer::new(config);
    assert_eq!(analyzer.config.max_correlation, 0.8);
}

/// Test market condition risk module integration
#[test]
fn test_risk_market_condition_risk_module_integration() {
    use sniper_risk::market_condition_risk::{MarketConditionRiskAdjuster, MarketConditionRiskConfig};
    
    // Check that the market condition risk module can be instantiated
    let config = MarketConditionRiskConfig::default();
    let adjuster = MarketConditionRiskAdjuster::new(config);
    assert!(adjuster.config.enabled);
}

/// Test unified risk engine module integration
#[test]
fn test_risk_unified_risk_engine_module_integration() {
    use sniper_risk::unified_risk::{UnifiedRiskEngine, UnifiedRiskConfig};
    use sniper_risk::decide::{RiskDecisionEngine, RiskDecisionConfig};
    use sniper_risk::honeypot::{HoneypotDetector, HoneypotDetectionConfig};
    use sniper_risk::limits::{TradingLimitsEnforcer, TradingLimitsConfig, PortfolioState};
    use sniper_risk::owner_powers::{OwnerPowerMonitor, OwnerPowerConfig};
    use sniper_risk::lp_quality::{LpQualityAssessor, LpQualityConfig};
    use sniper_risk::enhanced_correlation::EnhancedCorrelationAnalyzer;
    use sniper_risk::market_condition_risk::MarketConditionRiskAdjuster;
    
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

/// Test enhanced correlation functionality integration
#[test]
fn test_risk_enhanced_correlation_functionality_integration() {
    use sniper_risk::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationConfig};
    
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

/// Test market condition risk functionality integration
#[test]
fn test_risk_market_condition_risk_functionality_integration() {
    use sniper_risk::market_condition_risk::{MarketConditionRiskAdjuster, MarketConditionRiskConfig, MarketConditions, MarketTrend};
    
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

/// Test unified risk engine functionality integration
#[test]
fn test_risk_unified_risk_engine_functionality_integration() {
    use sniper_risk::unified_risk::{UnifiedRiskEngine, UnifiedRiskConfig};
    use sniper_risk::decide::{RiskDecisionEngine, RiskDecisionConfig};
    use sniper_risk::honeypot::{HoneypotDetector, HoneypotDetectionConfig};
    use sniper_risk::limits::{TradingLimitsEnforcer, TradingLimitsConfig, PortfolioState};
    use sniper_risk::owner_powers::{OwnerPowerMonitor, OwnerPowerConfig};
    use sniper_risk::lp_quality::{LpQualityAssessor, LpQualityConfig};
    use sniper_risk::enhanced_correlation::EnhancedCorrelationAnalyzer;
    use sniper_risk::market_condition_risk::MarketConditionRiskAdjuster;
    use sniper_core::types::{TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};
    
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

/// Test module integration
#[test]
fn test_risk_modules_integration() {
    use sniper_risk::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationConfig};
    use sniper_risk::market_condition_risk::{MarketConditionRiskAdjuster, MarketConditionRiskConfig, MarketConditions, MarketTrend};
    
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

/// Test all risk enhancements working together
#[test]
fn test_all_risk_enhancements_working_together() {
    use sniper_risk::enhanced_correlation::{EnhancedCorrelationAnalyzer, EnhancedCorrelationConfig};
    use sniper_risk::market_condition_risk::{MarketConditionRiskAdjuster, MarketConditionRiskConfig, MarketConditions, MarketTrend};
    use sniper_risk::unified_risk::{UnifiedRiskEngine, UnifiedRiskConfig};
    use sniper_risk::decide::{RiskDecisionEngine, RiskDecisionConfig};
    use sniper_risk::honeypot::{HoneypotDetector, HoneypotDetectionConfig};
    use sniper_risk::limits::{TradingLimitsEnforcer, TradingLimitsConfig, PortfolioState};
    use sniper_risk::owner_powers::{OwnerPowerMonitor, OwnerPowerConfig};
    use sniper_risk::lp_quality::{LpQualityAssessor, LpQualityConfig};
    use sniper_core::types::{TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};

    // 1. Enhanced Correlation Analysis
    let mut correlation_analyzer = EnhancedCorrelationAnalyzer::new(EnhancedCorrelationConfig::default());
    
    // Add data for correlation analysis
    let prices_a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
    let prices_b = vec![200.0, 220.0, 240.0, 260.0, 280.0];
    let timestamps = vec![1, 2, 3, 4, 5];
    
    correlation_analyzer.update_historical_prices("TOKEN_A", prices_a, timestamps.clone());
    correlation_analyzer.update_historical_prices("TOKEN_B", prices_b, timestamps.clone());
    
    let correlation_result = correlation_analyzer.calculate_enhanced_correlation("TOKEN_A", "TOKEN_B");
    assert!(correlation_result.correlation >= 0.0);
    
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
    assert!(market_result.multiplier > 0.0);
    
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
    assert!(unified_result.overall_score <= 100);
    assert!(!unified_result.reasons.is_empty());
}

/// Test compliance policy with disabled enforcement
#[test]
fn test_compliance_policy_disabled() {
    use sniper_policy::compliance::{CompliancePolicy, ComplianceConfig};
    
    // Create disabled compliance configuration
    let mut config = ComplianceConfig::default();
    config.enabled = false;
    
    // Create compliance policy engine
    let policy = CompliancePolicy::new(config);
    
    // All checks should pass when disabled
    let jurisdiction_result = policy.check_jurisdiction_compliance("US", "any_activity");
    assert!(jurisdiction_result.compliant);
    assert!(jurisdiction_result.reasons.contains(&"Compliance checking disabled".to_string()));
    
    let exchange_result = policy.check_exchange_tos_compliance("binance", 1000000.0, 10000, "ANY/PAIR");
    assert!(exchange_result.compliant);
    assert!(exchange_result.reasons.contains(&"Compliance checking disabled".to_string()));
    
    let retention_result = policy.check_data_retention_compliance();
    assert!(retention_result.compliant);
    assert!(retention_result.reasons.contains(&"Compliance checking disabled".to_string()));
}

/// Test compliance policy integration
#[test]
fn test_compliance_policy_integration() {
    use sniper_policy::compliance::{CompliancePolicy, ComplianceConfig, JurisdictionRules, ExchangeTosRules, ReportingConfig, DataRetentionEnforcement};
    use std::collections::HashMap;
    
    // Create compliance configuration
    let mut config = ComplianceConfig::default();
    
    // Set up jurisdiction rules
    let mut jurisdiction_rules = HashMap::new();
    jurisdiction_rules.insert("US".to_string(), JurisdictionRules {
        allowed_activities: vec!["spot_trading".to_string()],
        prohibited_activities: vec!["margin_trading".to_string(), "short_selling".to_string()],
        reporting_frequency_days: 30,
        compliance_officer: "compliance@firm.com".to_string(),
    });
    config.jurisdiction_rules = jurisdiction_rules;
    
    // Set up exchange TOS rules
    let mut exchange_rules = HashMap::new();
    exchange_rules.insert("binance".to_string(), ExchangeTosRules {
        max_daily_volume_usd: 100000.0,
        max_trades_per_day: 1000,
        prohibited_pairs: vec!["ETH/BTC".to_string()],
        cooldown_period_seconds: 60,
    });
    config.exchange_tos_rules = exchange_rules;
    
    // Set up reporting configuration
    config.reporting_config = ReportingConfig {
        enabled: true,
        frequency_hours: 24,
        recipients: vec!["compliance@firm.com".to_string()],
        format: "JSON".to_string(),
        detailed_findings: true,
    };
    
    // Set up data retention enforcement
    config.data_retention_enforcement = DataRetentionEnforcement {
        enabled: true,
        storage_paths: vec!["/var/log/snipping-bot".to_string()],
        verification_frequency_hours: 1,
        alert_on_violations: true,
    };
    
    // Create compliance policy engine
    let mut policy = CompliancePolicy::new(config);
    
    // Test jurisdiction compliance
    let us_result = policy.check_jurisdiction_compliance("US", "spot_trading");
    assert!(us_result.compliant);
    
    let us_prohibited_result = policy.check_jurisdiction_compliance("US", "margin_trading");
    assert!(!us_prohibited_result.compliant);
    
    // Test exchange TOS compliance
    let exchange_result = policy.check_exchange_tos_compliance("binance", 50000.0, 500, "ETH/USDT");
    assert!(exchange_result.compliant);
    
    let exchange_violation_result = policy.check_exchange_tos_compliance("binance", 150000.0, 1500, "ETH/BTC");
    assert!(!exchange_violation_result.compliant);
    
    // Test data retention compliance
    let retention_result = policy.check_data_retention_compliance();
    assert!(retention_result.compliant);
    
    // Test audit trail integrity verification
    let audit_integrity_result = policy.verify_audit_trail_integrity();
    assert!(audit_integrity_result.integrity_verified);
    
    // Test data retention verification
    let data_retention_result = policy.verify_data_retention_compliance();
    assert!(data_retention_result.retention_compliant);
    
    // Test audit logging
    let mut metadata = HashMap::new();
    metadata.insert("trade_id".to_string(), "12345".to_string());
    metadata.insert("user_id".to_string(), "user1".to_string());
    
    policy.log_audit_event("TRADE_EXECUTED", "user1", "Executed trade on binance", metadata);
    assert_eq!(policy.get_audit_log().len(), 1);
    
    // Test comprehensive compliance report generation
    let report = policy.generate_comprehensive_compliance_report();
    assert!(!report.summary.status.is_empty());
    assert!(report.summary.checks_performed > 0);
    
    // Test sending compliance report
    let send_result = policy.send_compliance_report();
    assert!(send_result.is_ok());
    
    // Test performing all compliance checks
    let all_checks = policy.perform_all_compliance_checks();
    assert_eq!(all_checks.len(), 3);
}
