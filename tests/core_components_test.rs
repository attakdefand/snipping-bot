//! Core components integration test
//!
//! This test verifies that all the newly implemented components work together correctly

use sniper_core::types::{ChainRef, TradePlan, ExecMode, GasPolicy, ExitRules};
use sniper_exec::{exec_mempool, exec_private, gas};
use sniper_keys::mpc;

#[tokio::test]
async fn test_all_components_integration() {
    println!("Testing integration of all newly implemented components...");
    
    // 1. Test gas estimation
    println!("1. Testing gas estimation...");
    let estimator = gas::GasEstimator::new(25, 2);
    assert_eq!(estimator.base_fee_gwei, 25);
    assert_eq!(estimator.priority_fee_gwei, 2);
    
    // 2. Test trade plan creation
    println!("2. Testing trade plan creation...");
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
        idem_key: "core_components_test".to_string(),
    };
    
    // 3. Test gas estimation with the plan
    println!("3. Testing gas estimation with trade plan...");
    let gas_policy = estimator.estimate_gas(&plan);
    assert_eq!(gas_policy.max_fee_gwei, 27); // 25 + 2
    assert_eq!(gas_policy.max_priority_gwei, 2);
    
    // 4. Test mempool execution
    println!("4. Testing mempool execution...");
    let mempool_result = exec_mempool::execute_via_mempool(&plan).await;
    assert!(mempool_result.is_ok());
    let mempool_receipt = mempool_result.unwrap();
    assert!(mempool_receipt.success);
    assert!(mempool_receipt.tx_hash.starts_with("0x"));
    assert_eq!(mempool_receipt.gas_used, 120000);
    
    // 5. Test private execution
    println!("5. Testing private execution...");
    let private_result = exec_private::execute_via_private(&plan).await;
    assert!(private_result.is_ok());
    let private_receipt = private_result.unwrap();
    assert!(private_receipt.success);
    assert!(private_receipt.tx_hash.starts_with("0x"));
    assert_eq!(private_receipt.gas_used, 110000);
    
    // 6. Test MPC key management
    println!("6. Testing MPC key management...");
    let manager = mpc::MpcKeyManager::new("core-test-participant".to_string(), 2, 3);
    assert!(manager.can_sign());
    
    let key_result = manager.generate_key_share().await;
    assert!(key_result.is_ok());
    let key_id = key_result.unwrap();
    assert!(key_id.starts_with("mpc-key-"));
    
    let transaction_data = b"core components test transaction";
    let signature_result = manager.sign_transaction(&key_id, transaction_data).await;
    assert!(signature_result.is_ok());
    let signature = signature_result.unwrap();
    assert!(!signature.is_empty());
    
    // 7. Test network condition adjustment
    println!("7. Testing network condition adjustment...");
    let mut adjusted_estimator = gas::GasEstimator::new(20, 1);
    adjusted_estimator.adjust_for_network_conditions(0.5); // 50% congestion
    assert_eq!(adjusted_estimator.base_fee_gwei, 40);
    assert_eq!(adjusted_estimator.priority_fee_gwei, 2);
    
    println!("All core components integration tests passed!");
}