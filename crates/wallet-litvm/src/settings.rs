use std::fs;
use std::path::Path;

use crate::error::LitVmError;
use crate::network::{LitVmSettingsFile, SETTINGS_FILE};

pub fn load_settings_file(data_dir: &Path) -> Result<LitVmSettingsFile, LitVmError> {
    let path = data_dir.join(SETTINGS_FILE);
    if !path.exists() {
        return Ok(LitVmSettingsFile::default());
    }
    let raw = fs::read_to_string(&path)?;
    serde_json::from_str(&raw).map_err(|e| LitVmError::Settings(e.to_string()))
}

pub fn save_settings_file(data_dir: &Path, file: &LitVmSettingsFile) -> Result<(), LitVmError> {
    let path = data_dir.join(SETTINGS_FILE);
    let raw = serde_json::to_string_pretty(file)
        .map_err(|e| LitVmError::Settings(e.to_string()))?;
    fs::write(&path, raw)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn wipe_settings_file(data_dir: &Path) -> Result<(), LitVmError> {
    let path = data_dir.join(SETTINGS_FILE);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
