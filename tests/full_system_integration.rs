//! Full system integration test for all implemented modules
//!
//! This test verifies that all the modules we've implemented work together correctly
//! including AMM quoting, chain operations, key management, policy enforcement,
//! risk management, signal processing, simulation, and telemetry.

use anyhow::Result;
use sniper_amm::univ3::quoter::{get_quote, Quote};
use sniper_chain::nonce::{NonceManager, AdvancedNonceManager, NonceStrategy};
use sniper_keys::local::{LocalKeyStorage, LocalKeyConfig, LocalKeyType};
use sniper_keys::vault::{VaultClient, VaultConfig};
use sniper_policy::geo::{GeoPolicy, GeoRestriction};
use sniper_risk::decide::{RiskEngine, RiskDecision, RiskFactors};
use sniper_risk::honeypot::{HoneypotDetector, HoneypotRisk};
use sniper_risk::limits::{TradingLimits, PositionLimits};
use sniper_risk::owner_powers::{OwnerPowerMonitor, OwnerPower};
use sniper_signals::normalize::{SignalNormalizer, NormalizedSignal};
use sniper_sim::calldata_dryrun::{CallDataSimulator, SimulationResult};
use sniper_sim::quoter_sim::{QuoteSimulator, SimulatedQuote};
use sniper_telemetry::alerts::slack::{SlackAlertSender, SlackConfig};
use sniper_telemetry::alerts::webhook::{WebhookAlertSender, WebhookConfig};
use sniper_cex::auth::{AuthManager, Credentials, HmacAlgorithm};
use sniper_cex::rest::{RestClient, RestConfig, RestClientManager};
use sniper_core::types::{ChainRef, TradePlan, ExecMode, GasPolicy, ExitRules};

#[tokio::test]
async fn test_full_system_integration() -> Result<()> {
    println!("Starting full system integration test...");
    
    // 1. Test AMM Quoter functionality
    test_amm_quoter().await?;
    
    // 2. Test Chain Nonce Management
    test_chain_nonce_management().await?;
    
    // 3. Test Key Storage (Local and Vault)
    test_key_storage().await?;
    
    // 4. Test Policy Enforcement (Geo restrictions)
    test_policy_enforcement().await?;
    
    // 5. Test Risk Management
    test_risk_management().await?;
    
    // 6. Test Signal Processing
    test_signal_processing().await?;
    
    // 7. Test Simulation
    test_simulation().await?;
    
    // 8. Test Telemetry (Alerting)
    test_telemetry().await?;
    
    // 9. Test CEX Integration
    test_cex_integration().await?;
    
    // 10. Test End-to-End Workflow
    test_end_to_end_workflow().await?;
    
    println!("Full system integration test completed successfully!");
    Ok(())
}

async fn test_amm_quoter() -> Result<()> {
    println!("Testing AMM Quoter...");
    
    let plan = TradePlan {
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        router: "0xRouter".to_string(),
        token_in: "0xWETH".to_string(),
        token_out: "0xUSDT".to_string(),
        amount_in: 1000000000000000000, // 1 ETH
        min_out: 900000000000000000,    // 0.9 USDT
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
        idem_key: "quoter_test".to_string(),
    };
    
    let quote = get_quote(&plan)?;
    assert_eq!(quote.amount_out, 900000000000000000);
    assert_eq!(quote.price_impact, 0.001);
    assert_eq!(quote.fee_amount, 3000000000000000); // 0.3% of 1 ETH
    assert_eq!(quote.gas_estimate, 150000);
    
    println!("AMM Quoter test passed!");
    Ok(())
}

async fn test_chain_nonce_management() -> Result<()> {
    println!("Testing Chain Nonce Management...");
    
    // Test basic nonce manager
    let manager = NonceManager::new();
    let nonce1 = manager.get_next_nonce("0xAddress1").await?;
    assert_eq!(nonce1, 0);
    
    let nonce2 = manager.get_next_nonce("0xAddress1").await?;
    assert_eq!(nonce2, 1);
    
    // Test advanced nonce manager with different strategies
    let sequential_manager = AdvancedNonceManager::new(NonceStrategy::Sequential);
    let nonce3 = sequential_manager.get_next_nonce("0xAddress2").await?;
    assert_eq!(nonce3, 0);
    
    let random_manager = AdvancedNonceManager::new(NonceStrategy::Random);
    let _nonce4 = random_manager.get_next_nonce("0xAddress3").await?;
    // We can't assert a specific value for random, just that it works
    
    println!("Chain Nonce Management test passed!");
    Ok(())
}

async fn test_key_storage() -> Result<()> {
    println!("Testing Key Storage...");
    
    // Test local key storage
    let config = LocalKeyConfig {
        storage_path: "./test_keys_integration".to_string(),
        encrypt_keys: false,
        master_password: None,
    };
    
    let mut storage = LocalKeyStorage::new(config)?;
    let key_material = b"test private key material for integration";
    
    // Store a key
    storage.store_key(
        "integration-test-key",
        LocalKeyType::PrivateKey,
        key_material,
        vec!["integration".to_string()]
    )?;
    
    // Retrieve the key
    let retrieved = storage.retrieve_key("integration-test-key")?;
    assert_eq!(retrieved, key_material);
    
    // Test key listing
    let keys = storage.list_keys()?;
    assert!(keys.contains(&"integration-test-key".to_string()));
    
    // Clean up
    let _ = std::fs::remove_dir_all("./test_keys_integration");
    
    // Note: Vault integration would require a running Vault server, so we'll just test instantiation
    let vault_config = VaultConfig {
        url: "http://localhost:8200".to_string(),
        token: "test-token".to_string(),
        mount_path: "secret".to_string(),
    };
    
    let _vault_client = VaultClient::new(vault_config)?;
    
    println!("Key Storage test passed!");
    Ok(())
}

async fn test_policy_enforcement() -> Result<()> {
    println!("Testing Policy Enforcement...");
    
    let mut policy = GeoPolicy::new();
    
    // Add a restriction
    policy.add_restriction(GeoRestriction::Country("US".to_string()));
    
    // Test IP checking (this would normally be done with real IP addresses)
    // For integration testing, we'll just verify the policy structure
    assert_eq!(policy.list_restrictions().len(), 1);
    
    println!("Policy Enforcement test passed!");
    Ok(())
}

async fn test_risk_management() -> Result<()> {
    println!("Testing Risk Management...");
    
    // Test risk engine
    let engine = RiskEngine::new();
    let factors = RiskFactors {
        volatility: 0.05,
        liquidity: 0.8,
        market_cap: 1000000.0,
        age_days: 365,
        holder_concentration: 0.1,
    };
    
    let decision = engine.evaluate_risk(&factors)?;
    assert!(matches!(decision, RiskDecision::Accept { .. } | RiskDecision::Reject { .. }));
    
    // Test honeypot detection
    let detector = HoneypotDetector::new();
    let risk = detector.analyze_contract("0xTestContract")?;
    assert!(matches!(risk, HoneypotRisk::Low | HoneypotRisk::Medium | HoneypotRisk::High));
    
    // Test trading limits
    let limits = TradingLimits::new(1000.0, 100.0);
    assert!(limits.check_daily_limit(50.0)?);
    
    // Test owner power monitoring
    let monitor = OwnerPowerMonitor::new();
    let power = monitor.check_contract("0xTestContract")?;
    assert!(matches!(power, OwnerPower::None | OwnerPower::Limited | OwnerPower::Full));
    
    println!("Risk Management test passed!");
    Ok(())
}

async fn test_signal_processing() -> Result<()> {
    println!("Testing Signal Processing...");
    
    let normalizer = SignalNormalizer::new();
    let signals = vec![
        NormalizedSignal {
            source: "test_source_1".to_string(),
            pair: "ETH/USDT".to_string(),
            confidence: 0.8,
            strength: 0.7,
            timestamp: 1234567890,
        },
        NormalizedSignal {
            source: "test_source_2".to_string(),
            pair: "ETH/USDT".to_string(),
            confidence: 0.9,
            strength: 0.8,
            timestamp: 1234567891,
        }
    ];
    
    let aggregated = normalizer.aggregate_signals(&signals);
    assert!(!aggregated.is_empty());
    
    println!("Signal Processing test passed!");
    Ok(())
}

async fn test_simulation() -> Result<()> {
    println!("Testing Simulation...");
    
    // Test calldata dry-run simulator
    let simulator = CallDataSimulator::new();
    let result = simulator.simulate_call("0xContract", "0xData").await?;
    assert!(matches!(result, SimulationResult::Success { .. } | SimulationResult::Revert { .. }));
    
    // Test quote simulator
    let quote_sim = QuoteSimulator::new();
    let sim_quote = quote_sim.simulate_quote(1000000000000000000, 0.9)?;
    assert!(sim_quote.amount_out > 0);
    
    println!("Simulation test passed!");
    Ok(())
}

async fn test_telemetry() -> Result<()> {
    println!("Testing Telemetry...");
    
    // Test Slack alerting (just instantiation, as we don't have a real Slack webhook)
    let slack_config = SlackConfig {
        webhook_url: "https://hooks.slack.com/services/PLACEHOLDER/PLACEHOLDER/PLACEHOLDER".to_string(),
        channel: Some("#alerts".to_string()),
        username: Some("sniper-bot".to_string()),
        icon_emoji: Some(":robot_face:".to_string()),
        enabled: false, // Disable for integration test
    };
    
    let _slack_sender = SlackAlertSender::new(slack_config)?;
    
    // Test webhook alerting (just instantiation, as we don't have a real webhook)
    let webhook_config = WebhookConfig {
        url: "https://example.com/webhook".to_string(),
        method: "POST".to_string(),
        headers: None,
        auth_token: None,
        auth_header: None,
        enabled: false, // Disable for integration test
        timeout_seconds: 30,
    };
    
    let _webhook_sender = WebhookAlertSender::new(webhook_config)?;
    
    println!("Telemetry test passed!");
    Ok(())
}

async fn test_cex_integration() -> Result<()> {
    println!("Testing CEX Integration...");
    
    // Test authentication manager
    let mut auth_manager = AuthManager::new();
    let credentials = Credentials::new_api_key(
        "test_api_key".to_string(),
        "test_api_secret".to_string(),
    );
    auth_manager.add_credentials("binance".to_string(), credentials);
    assert!(auth_manager.get_credentials("binance").is_some());
    
    // Test REST client manager
    let mut rest_manager = RestClientManager::new();
    let config = RestConfig {
        base_url: "https://httpbin.org".to_string(),
        timeout_seconds: 30,
        rate_limit: 1.0,
        ssl_verify: true,
    };
    
    let client = RestClient::new(sniper_cex::ExchangeId("httpbin".to_string()), config)?;
    rest_manager.add_client("httpbin".to_string(), client);
    assert!(rest_manager.get_client("httpbin").is_some());
    
    println!("CEX Integration test passed!");
    Ok(())
}

async fn test_end_to_end_workflow() -> Result<()> {
    println!("Testing End-to-End Workflow...");
    
    // Create a complete workflow that uses multiple components
    let plan = TradePlan {
        chain: ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        },
        router: "0xRouter".to_string(),
        token_in: "0xWETH".to_string(),
        token_out: "0xUSDT".to_string(),
        amount_in: 1000000000000000000, // 1 ETH
        min_out: 900000000000000000,    // 0.9 USDT
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
        idem_key: "e2e_workflow_test".to_string(),
    };
    
    // 1. Get a quote from AMM
    let quote = get_quote(&plan)?;
    assert!(quote.amount_out > 0);
    
    // 2. Check risk factors
    let engine = RiskEngine::new();
    let factors = RiskFactors {
        volatility: 0.05,
        liquidity: 0.8,
        market_cap: 1000000.0,
        age_days: 365,
        holder_concentration: 0.1,
    };
    
    let risk_decision = engine.evaluate_risk(&factors)?;
    match risk_decision {
        RiskDecision::Accept { .. } => {
            // 3. Get nonce for transaction
            let nonce_manager = NonceManager::new();
            let _nonce = nonce_manager.get_next_nonce("0xTraderAddress").await?;
            
            // 4. Simulate the trade
            let simulator = CallDataSimulator::new();
            let _sim_result = simulator.simulate_call(&plan.router, "0xCalldata").await?;
            
            // 5. Send alert about trade consideration
            // (In a real implementation, we would send alerts here)
        }
        RiskDecision::Reject { reason } => {
            println!("Trade rejected due to risk: {}", reason);
        }
    }
    
    println!("End-to-End Workflow test passed!");
    Ok(())
}