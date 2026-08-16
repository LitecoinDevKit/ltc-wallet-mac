use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmSummary {
    pub address: String,
    pub balance_zkltc: String,
    pub network_id: String,
    pub network_name: String,
    pub chain_id: u64,
    pub symbol: String,
    pub faucet_url: Option<String>,
    pub explorer: String,
    pub rpc_http: String,
    pub signing_enabled: bool,
    pub seed_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmProbe {
    pub expected_chain_id: u64,
    pub rpc_chain_id: Option<u64>,
    pub rpc_http: String,
    pub matches: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmSendRequest {
    pub address: String,
    pub amount_zkltc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmSendPreview {
    pub from: String,
    pub to: String,
    pub amount_zkltc: String,
    pub fee_zkltc: String,
    pub max_fee_zkltc: String,
    pub nonce: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmSendResult {
    pub txid: String,
    pub fee_zkltc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmReplaceRequest {
    pub txid: String,
    pub nonce: u64,
    pub address: String,
    pub amount_zkltc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LitVmHistoryTx {
    pub txid: String,
    pub from: String,
    pub to: String,
    pub amount_zkltc: String,
    pub incoming: bool,
    pub pending: bool,
    pub nonce: u64,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLitVmSettingsRequest {
    pub rpc_http_override: Option<String>,
}
