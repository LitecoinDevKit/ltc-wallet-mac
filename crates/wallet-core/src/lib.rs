//! Wallet-core: BDK boundary for the Litecoin Mac wallet.
//!
//! Public API exposes serde DTOs only. BDK types remain private.

mod aezeed;
mod app;
mod descriptors;
mod discovery;
mod dto;
mod electrum;
mod error;
pub mod explorer;
mod contacts;
mod history_export;
mod insights;
mod labels;
mod meta;
mod metadata;
mod mweb;
mod mweb_history;
mod network;
mod rpc;
mod secrets;
mod seed;
mod utxo_labels;

pub use app::{MemoryBackedApp, WalletApp};
pub use seed::{derive_preview, DerivePreview, MasterSecret, MwebSchemePreview};
pub use dto::{
    AddressReuseHint, CombinedSummary, ContactKind, ContactRecord, CreateWalletRequest,
    CreateWalletResponse, DeleteContactRequest, ElectrumProbe, FeeEstimate, FeeLadder,
    MetricSeries, MigrateEncryptRequest, MwebBroadcastResult, MwebScheme, MwebSendPreview,
    MwebSendRequest, MwebSyncProgress, NetworkPulse, PeginPreview, PeginRequest, PeginResult,
    PegoutPreview, PegoutRequest, RestoreWalletRequest, SendPreview, SendRequest, SendResult,
    SetTxLabelRequest, SetUtxoLabelRequest, SetUtxoLockedRequest, SyncResult, TxEnrichment, TxIo,
    TxKind, TxRecord, TxStatus, UnlockRequest, UpdateSettingsRequest, UpsertContactRequest,
    UtxoRecord, WalletSettings, WalletSummary, DEFAULT_MWEB_FEE_SATS,
};
pub use explorer::{DEFAULT_EXPLORER_BASE_URL, is_chain_txid};
pub use error::{BroadcastFailureKind, WalletError};
pub use history_export::HistoryExportFormat;
pub use metadata::{MetadataBundle, MetadataImportResult};
pub use network::WalletNetwork;
pub use secrets::{
    EncryptedFileSecretStore, FileSecretStore, MemoryStore, SecretStore, UnlockableSecretStore,
};

/// Filename for the legacy plaintext mnemonic store.
pub const MNEMONIC_FILE: &str = "wallet.mnemonic";
/// Filename for the encrypted mnemonic store.
pub const MNEMONIC_ENC_FILE: &str = "wallet.mnemonic.enc";
