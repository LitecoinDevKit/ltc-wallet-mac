use serde::Deserialize;

use crate::amount::format_zkltc;
use crate::dto::LitVmHistoryTx;
use crate::error::LitVmError;
use crate::network::LitVmNetwork;

#[derive(Debug, Deserialize)]
struct TxListResponse {
    status: Option<String>,
    message: Option<String>,
    result: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TxRow {
    hash: Option<String>,
    from: Option<String>,
    to: Option<String>,
    value: Option<String>,
    nonce: Option<String>,
    #[serde(rename = "timeStamp")]
    time_stamp: Option<String>,
    txreceipt_status: Option<String>,
}

pub fn fetch_txlist(
    network: &LitVmNetwork,
    address: &str,
) -> Result<Vec<LitVmHistoryTx>, LitVmError> {
    let base = network
        .history_api
        .as_deref()
        .unwrap_or(&network.explorer);
    let url = if base.contains("action=txlist") {
        format!("{base}&address={address}&sort=desc&page=1&offset=50")
    } else {
        format!(
            "{}/api?module=account&action=txlist&address={address}&sort=desc&page=1&offset=50",
            network.explorer.trim_end_matches('/')
        )
    };
    let body = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .call()
        .map_err(|e| LitVmError::History(e.to_string()))?
        .into_string()
        .map_err(|e| LitVmError::History(e.to_string()))?;
    parse_txlist(&body, address)
}

fn parse_txlist(body: &str, address: &str) -> Result<Vec<LitVmHistoryTx>, LitVmError> {
    let parsed: TxListResponse =
        serde_json::from_str(body).map_err(|e| LitVmError::History(e.to_string()))?;
    if parsed.status.as_deref() != Some("1") {
        let msg = parsed
            .message
            .unwrap_or_else(|| "history indexer unavailable".into());
        // Empty account is not an error.
        if msg.to_ascii_lowercase().contains("no transaction") {
            return Ok(Vec::new());
        }
        return Err(LitVmError::History(msg));
    }
    let rows: Vec<TxRow> = match parsed.result {
        Some(serde_json::Value::Array(arr)) => {
            serde_json::from_value(serde_json::Value::Array(arr))
                .map_err(|e| LitVmError::History(e.to_string()))?
        }
        _ => return Ok(Vec::new()),
    };
    let mine = address.to_ascii_lowercase();
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let txid = row.hash?;
            let from = row.from.unwrap_or_default();
            let to = row.to.unwrap_or_default();
            let value = row
                .value
                .as_deref()
                .and_then(|v| alloy::primitives::U256::from_str_radix(v, 10).ok())
                .unwrap_or_default();
            let nonce = row
                .nonce
                .as_deref()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            let timestamp = row.time_stamp.and_then(|t| t.parse().ok());
            let pending = row
                .txreceipt_status
                .as_deref()
                .map(|s| s != "1")
                .unwrap_or(false);
            Some(LitVmHistoryTx {
                incoming: to.to_ascii_lowercase() == mine,
                txid,
                from,
                to,
                amount_zkltc: format_zkltc(value),
                pending,
                nonce,
                timestamp,
            })
        })
        .collect())
}
