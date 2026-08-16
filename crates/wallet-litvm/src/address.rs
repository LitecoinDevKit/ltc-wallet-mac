use alloy::primitives::Address;

use crate::error::LitVmError;

pub fn is_litecoin_like(input: &str) -> bool {
    let a = input.trim();
    let lower = a.to_ascii_lowercase();
    lower.starts_with("ltc1")
        || lower.starts_with("tltc1")
        || lower.starts_with("ltcmweb1")
        || lower.starts_with("tmweb1")
        || lower.starts_with("litecoin:")
        || matches!(
            a.chars().next(),
            Some('L' | 'M' | 'Q' | 'm' | '3')
        )
}

pub fn parse_evm_address(input: &str) -> Result<Address, LitVmError> {
    let a = input.trim();
    if is_litecoin_like(a) {
        return Err(LitVmError::LitecoinAddress);
    }
    let hex = a.strip_prefix("0x").or_else(|| a.strip_prefix("0X")).unwrap_or(a);
    if hex.len() != 40 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(LitVmError::InvalidAddress(
            "expected a 0x… LitVM address (40 hex chars)".into(),
        ));
    }
    hex.parse::<Address>()
        .map_err(|e| LitVmError::InvalidAddress(e.to_string()))
}

pub fn format_address(addr: Address) -> String {
    addr.to_checksum(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ltc_and_accepts_0x() {
        assert!(is_litecoin_like("ltc1qtest"));
        assert!(is_litecoin_like("Labc"));
        assert!(parse_evm_address("ltc1qtest").is_err());
        let a = parse_evm_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        assert_eq!(
            format_address(a).to_ascii_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }
}
