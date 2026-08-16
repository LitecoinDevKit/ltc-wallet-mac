use thiserror::Error;

#[derive(Debug, Error)]
pub enum LitVmError {
    #[error("litvm is locked; unlock the wallet first")]
    Locked,

    #[error("invalid LitVM address: {0}")]
    InvalidAddress(String),

    #[error("that looks like a Litecoin address — switch to Send for L1 LTC")]
    LitecoinAddress,

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("signing is disabled until the RPC chain ID matches the preset ({0})")]
    SigningDisabled(u64),

    #[error("fee from RPC exceeds the safety cap ({0})")]
    FeeCap(String),

    #[error("{0}")]
    FeeCongested(String),

    #[error("that LitVM transaction already confirmed — no need to speed it up")]
    AlreadyConfirmed,

    #[error("settings error: {0}")]
    Settings(String),

    #[error("history error: {0}")]
    History(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
