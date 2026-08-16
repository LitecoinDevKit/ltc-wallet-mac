//! LitVM EVM sidecar. No BDK / litecoin types.

mod address;
mod amount;
mod client;
mod dto;
mod error;
mod history;
mod network;
mod settings;

pub use client::LitVmClient;
pub use dto::{
    LitVmHistoryTx, LitVmProbe, LitVmReplaceRequest, LitVmSendPreview, LitVmSendRequest,
    LitVmSendResult, LitVmSummary, UpdateLitVmSettingsRequest,
};
pub use network::LitVmSettings;
pub use error::LitVmError;
pub use network::{LitVmNetwork, LITEFORGE_CHAIN_ID, LITEFORGE_FAUCET, LITEFORGE_RPC_HTTP};
pub use settings::wipe_settings_file;

pub fn is_evm_address(input: &str) -> bool {
    address::parse_evm_address(input).is_ok()
}

pub fn is_litecoin_like(input: &str) -> bool {
    address::is_litecoin_like(input)
}

#[cfg(test)]
mod tests {
    use alloy::signers::local::PrivateKeySigner;

    use super::*;

    #[test]
    fn anvil_mnemonic_secret_maps_to_known_address() {
        let secret =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        let signer = PrivateKeySigner::from_slice(&secret).unwrap();
        assert_eq!(
            signer.address().to_checksum(None),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }

    #[test]
    fn liteforge_preset_chain_id() {
        assert_eq!(LitVmNetwork::liteforge().chain_id, 4441);
    }
}
