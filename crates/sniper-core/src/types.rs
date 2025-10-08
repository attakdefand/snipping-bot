use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRef {
    pub name: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub source: String, // dex|nft|cex|social
    pub kind: String,   // pair_created|trading_enabled|...
    pub chain: ChainRef,
    pub token0: Option<String>,
    pub token1: Option<String>,
    pub extra: serde_json::Value,
    pub seen_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecMode {
    Bundle,
    Private,
    Mempool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPolicy {
    pub max_fee_gwei: u64,
    pub max_priority_gwei: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExitRules {
    pub take_profit_pct: Option<f64>,
    pub stop_loss_pct: Option<f64>,
    pub trailing_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePlan {
    pub chain: ChainRef,
    pub router: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u128,
    pub min_out: u128,
    pub mode: ExecMode,
    pub gas: GasPolicy,
    pub exits: ExitRules,
    pub idem_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub allow: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecReceipt {
    pub tx_hash: String,
    pub success: bool,
    pub block: u64,
    pub gas_used: u64,
    pub fees_paid_wei: u128,
    pub failure_reason: Option<String>,
}
