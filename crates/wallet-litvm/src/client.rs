use std::path::{Path, PathBuf};

use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use zeroize::Zeroize;

use crate::address::{format_address, parse_evm_address};
use crate::amount::{format_zkltc, parse_zkltc};
use crate::dto::{
    LitVmHistoryTx, LitVmProbe, LitVmReplaceRequest, LitVmSendPreview, LitVmSendRequest,
    LitVmSendResult, LitVmSummary, UpdateLitVmSettingsRequest,
};
use crate::network::LitVmSettings;
use crate::error::LitVmError;
use crate::history::fetch_txlist;
use crate::network::{
    LitVmNetwork, LitVmSettingsFile, GAS_PAD_BPS, MAX_FEE_GWEI, MAX_GAS_LIMIT, MAX_PRIORITY_GWEI,
};
use crate::settings::{load_settings_file, save_settings_file};


pub struct LitVmClient {
    runtime: tokio::runtime::Runtime,
    data_dir: PathBuf,
    signer: PrivateKeySigner,
    address: Address,
    network: LitVmNetwork,
    rpc_http: String,
    rpc_override: Option<String>,
    signing_enabled: bool,
}

impl LitVmClient {
    pub fn open(data_dir: &Path, mut secret: [u8; 32]) -> Result<Self, LitVmError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| LitVmError::Rpc(e.to_string()))?;
        let file = load_settings_file(data_dir)?;
        let settings = LitVmSettings::from_file(&file, false)
            .map_err(LitVmError::Settings)?;
        let signer = PrivateKeySigner::from_slice(&secret)
            .map_err(|e| LitVmError::Rpc(e.to_string()))?
            .with_chain_id(Some(settings.network.chain_id));
        secret.zeroize();
        let address = signer.address();
        Ok(Self {
            runtime,
            data_dir: data_dir.to_path_buf(),
            signer,
            address,
            rpc_http: settings.rpc_http,
            rpc_override: settings.rpc_http_override,
            network: settings.network,
            signing_enabled: false,
        })
    }

    pub fn address(&self) -> String {
        format_address(self.address)
    }

    pub fn settings(&self) -> LitVmSettings {
        LitVmSettings {
            network: self.network.clone(),
            rpc_http: self.rpc_http.clone(),
            rpc_http_override: self.rpc_override.clone(),
            signing_enabled: self.signing_enabled,
        }
    }

    pub fn update_settings(
        &mut self,
        req: UpdateLitVmSettingsRequest,
    ) -> Result<LitVmSettings, LitVmError> {
        let file = LitVmSettingsFile {
            network_id: self.network.id.clone(),
            rpc_http_override: req
                .rpc_http_override
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        };
        save_settings_file(&self.data_dir, &file)?;
        self.rpc_override = file.rpc_http_override.clone();
        self.rpc_http = self
            .rpc_override
            .clone()
            .unwrap_or_else(|| self.network.rpc_http.clone());
        self.signing_enabled = false;
        Ok(self.settings())
    }

    pub fn explorer_tx_url(&self, txid: &str) -> String {
        format!(
            "{}/tx/{}",
            self.network.explorer.trim_end_matches('/'),
            txid.trim()
        )
    }

    pub fn summary(&mut self) -> Result<LitVmSummary, LitVmError> {
        let _ = self.probe();
        let balance = match self.balance_wei() {
            Ok(wei) => format_zkltc(wei),
            Err(_) => "0".into(),
        };
        Ok(LitVmSummary {
            address: self.address(),
            balance_zkltc: balance,
            network_id: self.network.id.clone(),
            network_name: self.network.display_name.clone(),
            chain_id: self.network.chain_id,
            symbol: self.network.symbol.clone(),
            faucet_url: self.network.faucet_url.clone(),
            explorer: self.network.explorer.clone(),
            rpc_http: self.rpc_http.clone(),
            signing_enabled: self.signing_enabled,
            seed_note: None,
        })
    }

    pub fn probe(&mut self) -> LitVmProbe {
        match self.rpc_chain_id() {
            Ok(id) => {
                let matches = id == self.network.chain_id;
                self.signing_enabled = matches;
                LitVmProbe {
                    expected_chain_id: self.network.chain_id,
                    rpc_chain_id: Some(id),
                    rpc_http: self.rpc_http.clone(),
                    matches,
                    error: if matches {
                        None
                    } else {
                        Some(format!(
                            "RPC chain ID {id} does not match preset {}",
                            self.network.chain_id
                        ))
                    },
                }
            }
            Err(e) => {
                self.signing_enabled = false;
                LitVmProbe {
                    expected_chain_id: self.network.chain_id,
                    rpc_chain_id: None,
                    rpc_http: self.rpc_http.clone(),
                    matches: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    pub fn preview_send(&self, req: &LitVmSendRequest) -> Result<LitVmSendPreview, LitVmError> {
        self.ensure_signing()?;
        let to = parse_evm_address(&req.address)?;
        let value = parse_zkltc(&req.amount_zkltc)?;
        if value.is_zero() {
            return Err(LitVmError::InvalidAmount("amount must be greater than 0".into()));
        }
        let built = self.runtime.block_on(self.build_tx(to, value, None))?;
        Ok(LitVmSendPreview {
            from: self.address(),
            to: format_address(to),
            amount_zkltc: format_zkltc(value),
            fee_zkltc: format_zkltc(built.fee),
            max_fee_zkltc: format_zkltc(built.max_fee),
            nonce: built.nonce,
            gas_limit: built.gas_limit,
        })
    }

    pub fn send(&self, req: &LitVmSendRequest) -> Result<LitVmSendResult, LitVmError> {
        self.ensure_signing()?;
        let to = parse_evm_address(&req.address)?;
        let value = parse_zkltc(&req.amount_zkltc)?;
        if value.is_zero() {
            return Err(LitVmError::InvalidAmount("amount must be greater than 0".into()));
        }
        let built = self.runtime.block_on(self.build_tx(to, value, None))?;
        let txid = self.runtime.block_on(self.broadcast(built.request))?;
        Ok(LitVmSendResult {
            txid,
            fee_zkltc: format_zkltc(built.fee),
        })
    }

    pub fn replace_tx(&self, req: &LitVmReplaceRequest) -> Result<LitVmSendResult, LitVmError> {
        self.ensure_signing()?;
        let to = parse_evm_address(&req.address)?;
        let value = parse_zkltc(&req.amount_zkltc)?;
        let built = self
            .runtime
            .block_on(self.build_tx(to, value, Some(req.nonce)))?;
        let bumped = self.runtime.block_on(self.bump_fees(built))?;
        let txid = self.runtime.block_on(self.broadcast(bumped.request))?;
        Ok(LitVmSendResult {
            txid,
            fee_zkltc: format_zkltc(bumped.fee),
        })
    }

    pub fn history(&self) -> Result<Vec<LitVmHistoryTx>, LitVmError> {
        fetch_txlist(&self.network, &self.address())
    }

    fn ensure_signing(&self) -> Result<(), LitVmError> {
        if self.signing_enabled {
            Ok(())
        } else {
            Err(LitVmError::SigningDisabled(self.network.chain_id))
        }
    }

    fn provider(&self) -> Result<impl Provider + Clone, LitVmError> {
        let url = self
            .rpc_http
            .parse()
            .map_err(|e| LitVmError::Rpc(format!("invalid RPC URL: {e}")))?;
        Ok(ProviderBuilder::new().connect_http(url))
    }

    fn rpc_chain_id(&self) -> Result<u64, LitVmError> {
        let provider = self.provider()?;
        self.runtime
            .block_on(async move {
                provider
                    .get_chain_id()
                    .await
                    .map_err(|e| LitVmError::Rpc(e.to_string()))
            })
    }

    fn balance_wei(&self) -> Result<U256, LitVmError> {
        let provider = self.provider()?;
        let addr = self.address;
        self.runtime.block_on(async move {
            provider
                .get_balance(addr)
                .await
                .map_err(|e| LitVmError::Rpc(e.to_string()))
        })
    }

    async fn build_tx(
        &self,
        to: Address,
        value: U256,
        nonce_override: Option<u64>,
    ) -> Result<BuiltTx, LitVmError> {
        let provider = self.provider()?;
        let nonce = match nonce_override {
            Some(n) => n,
            None => provider
                .get_transaction_count(self.address)
                .await
                .map_err(|e| LitVmError::Rpc(e.to_string()))?,
        };
        let fees = provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| LitVmError::Rpc(e.to_string()))?;
        let max_priority = cap_gwei(fees.max_priority_fee_per_gas, MAX_PRIORITY_GWEI)?;
        let max_fee = cap_gwei(fees.max_fee_per_gas, MAX_FEE_GWEI)?;
        if max_fee < max_priority {
            return Err(LitVmError::FeeCap(
                "max fee is below the priority fee".into(),
            ));
        }

        let draft = TransactionRequest::default()
            .with_from(self.address)
            .with_to(to)
            .with_value(value)
            .with_nonce(nonce)
            .with_chain_id(self.network.chain_id)
            .with_max_fee_per_gas(max_fee)
            .with_max_priority_fee_per_gas(max_priority);
        let gas = provider
            .estimate_gas(draft.clone())
            .await
            .map_err(|e| LitVmError::Rpc(e.to_string()))?;
        let gas_limit = std::cmp::min(
            gas.saturating_add(gas.saturating_mul(GAS_PAD_BPS) / 10_000),
            MAX_GAS_LIMIT,
        );
        if gas_limit == 0 {
            return Err(LitVmError::Rpc("gas estimate was 0".into()));
        }
        let request = draft.with_gas_limit(gas_limit);
        let fee = U256::from(gas_limit).saturating_mul(U256::from(max_fee));
        let max_fee_total = fee;
        Ok(BuiltTx {
            request,
            nonce,
            gas_limit,
            fee,
            max_fee: max_fee_total,
        })
    }

    async fn bump_fees(&self, mut built: BuiltTx) -> Result<BuiltTx, LitVmError> {
        let max_fee = built
            .request
            .max_fee_per_gas
            .ok_or_else(|| LitVmError::Rpc("missing max fee".into()))?;
        let tip = built
            .request
            .max_priority_fee_per_gas
            .ok_or_else(|| LitVmError::Rpc("missing priority fee".into()))?;
        let new_tip = cap_gwei(tip.saturating_mul(2), MAX_PRIORITY_GWEI)?;
        let new_max = cap_gwei(max_fee.saturating_mul(2), MAX_FEE_GWEI)?;
        built.request.max_priority_fee_per_gas = Some(new_tip);
        built.request.max_fee_per_gas = Some(new_max);
        built.fee = U256::from(built.gas_limit).saturating_mul(U256::from(new_max));
        built.max_fee = built.fee;
        Ok(built)
    }

    async fn broadcast(&self, request: TransactionRequest) -> Result<String, LitVmError> {
        let wallet = EthereumWallet::from(self.signer.clone());
        let tx = request
            .build(&wallet)
            .await
            .map_err(|e| LitVmError::Rpc(e.to_string()))?;
        let provider = self.provider()?;
        let pending = provider
            .send_tx_envelope(tx)
            .await
            .map_err(|e| LitVmError::Rpc(e.to_string()))?;
        Ok(format!("{:#x}", pending.tx_hash()))
    }
}

struct BuiltTx {
    request: TransactionRequest,
    nonce: u64,
    gas_limit: u64,
    fee: U256,
    max_fee: U256,
}

fn cap_gwei(value: u128, cap_gwei: u128) -> Result<u128, LitVmError> {
    let cap = cap_gwei.saturating_mul(1_000_000_000);
    if value > cap {
        return Err(LitVmError::FeeCap(format!(
            "{value} wei/gas > {cap_gwei} gwei cap"
        )));
    }
    Ok(value)
}
