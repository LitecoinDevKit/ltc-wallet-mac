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

fn hex_body(input: &str) -> Option<&str> {
    let a = input.trim();
    let hex = a.strip_prefix("0x").or_else(|| a.strip_prefix("0X")).unwrap_or(a);
    if hex.len() == 40 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex)
    } else {
        None
    }
}

fn is_mixed_case(hex: &str) -> bool {
    let has_upper = hex.chars().any(|c| c.is_ascii_alphabetic() && c.is_ascii_uppercase());
    let has_lower = hex.chars().any(|c| c.is_ascii_alphabetic() && c.is_ascii_lowercase());
    has_upper && has_lower
}

/// 40 hex chars after an optional 0x — used to route L1 vs LitVM, not to send.
pub fn looks_like_evm_address(input: &str) -> bool {
    hex_body(input).is_some()
}

pub fn parse_evm_address(input: &str) -> Result<Address, LitVmError> {
    let a = input.trim();
    if is_litecoin_like(a) {
        return Err(LitVmError::LitecoinAddress);
    }
    let Some(hex) = hex_body(a) else {
        return Err(LitVmError::InvalidAddress(
            "expected a 0x… LitVM address (40 hex chars)".into(),
        ));
    };
    if is_mixed_case(hex) {
        let with_prefix = if a.starts_with("0x") || a.starts_with("0X") {
            a.to_string()
        } else {
            format!("0x{hex}")
        };
        return Address::parse_checksummed(&with_prefix, None).map_err(|_| {
            LitVmError::InvalidAddress(
                "checksum failed — copy the address again (EIP-55)".into(),
            )
        });
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

    const GOOD: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    const LOWER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    #[test]
    fn rejects_ltc_and_accepts_0x() {
        assert!(is_litecoin_like("ltc1qtest"));
        assert!(is_litecoin_like("Labc"));
        assert!(parse_evm_address("ltc1qtest").is_err());
        let a = parse_evm_address(GOOD).unwrap();
        assert_eq!(format_address(a), GOOD);
    }

    #[test]
    fn all_lower_is_ok() {
        assert_eq!(
            format_address(parse_evm_address(LOWER).unwrap()),
            GOOD
        );
    }

    #[test]
    fn all_upper_is_ok() {
        let upper = format!("0x{}", &LOWER[2..].to_ascii_uppercase());
        assert_eq!(
            format_address(parse_evm_address(&upper).unwrap()),
            GOOD
        );
    }

    #[test]
    fn mixed_case_typo_is_rejected() {
        // Flip one checksum nibble's case vs the known-good EIP-55 string.
        let bad = "0xf39fd6e51aad88F6F4ce6aB8827279cffFb92266";
        assert!(is_mixed_case(&bad[2..]));
        let err = parse_evm_address(bad).unwrap_err().to_string();
        assert!(err.contains("checksum"), "{err}");
    }

    #[test]
    fn looks_like_does_not_require_checksum() {
        assert!(looks_like_evm_address(
            "0xf39fd6e51aad88F6F4ce6aB8827279cffFb92266"
        ));
    }
}
