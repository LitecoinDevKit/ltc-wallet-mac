use std::path::Path;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};
use wallet_core::WalletApp;
use wallet_litvm::{
    parse_evm_address, wipe_settings_file, LitVmClient, LitVmHistoryPage, LitVmProbe,
    LitVmReplaceRequest, LitVmSendPreview, LitVmSendRequest, LitVmSendResult, LitVmSettings,
    LitVmSummary, UpdateLitVmSettingsRequest,
};

use crate::{data_dir, map_err};

pub struct LitVmHandle {
    client: Mutex<Option<LitVmClient>>,
}

impl LitVmHandle {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
        }
    }

    pub fn drop_client(&self) {
        if let Ok(mut guard) = self.client.lock() {
            *guard = None;
        }
    }
}

fn map_litvm(err: wallet_litvm::LitVmError) -> String {
    err.to_string()
}

pub fn attach(wallet: &WalletApp, handle: &LitVmHandle, data_dir: &Path) -> Result<(), String> {
    let secret = wallet.litvm_account_secret().map_err(map_err)?;
    let client = LitVmClient::open(data_dir, secret).map_err(map_litvm)?;
    let mut guard = handle
        .client
        .lock()
        .map_err(|_| "litvm lock poisoned".to_string())?;
    *guard = Some(client);
    Ok(())
}

fn with_client<T>(
    handle: &LitVmHandle,
    f: impl FnOnce(&mut LitVmClient) -> Result<T, wallet_litvm::LitVmError>,
) -> Result<T, String> {
    let mut guard = handle
        .client
        .lock()
        .map_err(|_| "litvm lock poisoned".to_string())?;
    let client = guard.as_mut().ok_or_else(|| {
        "LitVM is not ready — unlock the wallet first".to_string()
    })?;
    f(client).map_err(map_litvm)
}

#[tauri::command]
pub async fn litvm_summary(
    handle: State<'_, Arc<LitVmHandle>>,
) -> Result<LitVmSummary, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| c.summary()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn test_litvm_rpc(
    handle: State<'_, Arc<LitVmHandle>>,
) -> Result<LitVmProbe, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| Ok(c.probe())))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn preview_litvm_send(
    handle: State<'_, Arc<LitVmHandle>>,
    req: LitVmSendRequest,
) -> Result<LitVmSendPreview, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| c.preview_send(&req)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn send_litvm(
    handle: State<'_, Arc<LitVmHandle>>,
    req: LitVmSendRequest,
) -> Result<LitVmSendResult, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| c.send(&req)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn replace_litvm_tx(
    handle: State<'_, Arc<LitVmHandle>>,
    req: LitVmReplaceRequest,
) -> Result<LitVmSendResult, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| c.replace_tx(&req)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn litvm_history(
    handle: State<'_, Arc<LitVmHandle>>,
) -> Result<LitVmHistoryPage, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| c.history()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn validate_litvm_address(address: String) -> Result<String, String> {
    parse_evm_address(&address)
        .map(|addr| addr.to_checksum(None))
        .map_err(map_litvm)
}

#[tauri::command]
pub async fn get_litvm_settings(
    handle: State<'_, Arc<LitVmHandle>>,
) -> Result<LitVmSettings, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| Ok(c.settings())))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_litvm_settings(
    handle: State<'_, Arc<LitVmHandle>>,
    req: UpdateLitVmSettingsRequest,
) -> Result<LitVmSettings, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || with_client(&handle, |c| c.update_settings(req)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn litvm_explorer_tx_url(
    handle: State<'_, Arc<LitVmHandle>>,
    txid: String,
) -> Result<String, String> {
    let handle = Arc::clone(&handle);
    tauri::async_runtime::spawn_blocking(move || {
        with_client(&handle, |c| Ok(c.explorer_tx_url(&txid)))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn attach_from_app(
    app: &AppHandle,
    wallet: &WalletApp,
) -> Result<(), String> {
    let dir = data_dir(app)?;
    let handle = app.state::<Arc<LitVmHandle>>();
    attach(wallet, &handle, &dir)
}

pub fn drop_from_app(app: &AppHandle) {
    if let Some(handle) = app.try_state::<Arc<LitVmHandle>>() {
        handle.drop_client();
    }
}

pub fn wipe_sidecar(data_dir: &Path) {
    let _ = wipe_settings_file(data_dir);
}
