//! Wipeable sidecar of frozen MWEB output ids (same privacy class as UTXO labels).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::WalletError;

pub const MWEB_FROZEN_FILE: &str = "mweb_frozen.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MwebFrozenFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub ids: BTreeSet<String>,
}

fn default_version() -> u32 {
    1
}

pub fn frozen_path(data_dir: &Path) -> PathBuf {
    data_dir.join(MWEB_FROZEN_FILE)
}

pub fn is_output_id_hex(raw: &str) -> bool {
    let s = raw.trim();
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn normalize_output_id(raw: &str) -> Result<String, WalletError> {
    let s = raw.trim().to_ascii_lowercase();
    if !is_output_id_hex(&s) {
        return Err(WalletError::Meta(
            "MWEB output id must be 32-byte hex (64 characters)".into(),
        ));
    }
    Ok(s)
}

pub fn read_ids(data_dir: &Path) -> Result<MwebFrozenFile, WalletError> {
    let path = frozen_path(data_dir);
    if !path.is_file() {
        return Ok(MwebFrozenFile {
            version: 1,
            ids: BTreeSet::new(),
        });
    }
    let bytes = fs::read_to_string(&path).map_err(WalletError::Io)?;
    serde_json::from_str(&bytes).map_err(|e| WalletError::Meta(e.to_string()))
}

pub fn write_ids(data_dir: &Path, file: &MwebFrozenFile) -> Result<(), WalletError> {
    fs::create_dir_all(data_dir)?;
    if file.ids.is_empty() {
        match fs::remove_file(frozen_path(data_dir)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(WalletError::Io(e)),
        }
    } else {
        let json =
            serde_json::to_string_pretty(file).map_err(|e| WalletError::Meta(e.to_string()))?;
        fs::write(frozen_path(data_dir), json)?;
        Ok(())
    }
}

pub fn set_locked(data_dir: &Path, output_id: &str, locked: bool) -> Result<(), WalletError> {
    let id = normalize_output_id(output_id)?;
    let mut file = read_ids(data_dir)?;
    if locked {
        file.ids.insert(id);
    } else {
        file.ids.remove(&id);
    }
    write_ids(data_dir, &file)
}

#[allow(dead_code)]
pub fn contains(data_dir: &Path, output_id: &str) -> bool {
    let Ok(id) = normalize_output_id(output_id) else {
        return false;
    };
    read_ids(data_dir)
        .map(|f| f.ids.contains(&id))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip() {
        let dir = tempdir().unwrap();
        let id = "ab".repeat(32);
        set_locked(dir.path(), &id, true).unwrap();
        assert!(contains(dir.path(), &id));
        assert!(contains(dir.path(), &id.to_uppercase()));
        set_locked(dir.path(), &id, false).unwrap();
        assert!(!contains(dir.path(), &id));
        assert!(!frozen_path(dir.path()).is_file());
    }

    #[test]
    fn rejects_bad_id() {
        let dir = tempdir().unwrap();
        let err = set_locked(dir.path(), "not-hex", true).unwrap_err();
        assert!(err.to_string().contains("32-byte hex"));
    }
}
