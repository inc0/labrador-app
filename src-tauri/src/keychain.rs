/// Persistent key-value credential store backed by tauri-plugin-store.
///
/// Data is written to `labrador_secrets.json` in the app's private data
/// directory — survives app updates, wiped only on uninstall.
use anyhow::Result;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "labrador_secrets.json";

pub fn store(app: &AppHandle, key: &str, value: &str) -> Result<()> {
    let s = app.store(STORE_FILE)?;
    s.set(key, serde_json::json!(value));
    s.save()?;
    Ok(())
}

pub fn load(app: &AppHandle, key: &str) -> Result<Option<String>> {
    let s = app.store(STORE_FILE)?;
    Ok(s.get(key).and_then(|v| v.as_str().map(str::to_string)))
}

pub fn delete(app: &AppHandle, key: &str) -> Result<()> {
    let s = app.store(STORE_FILE)?;
    s.delete(key);
    s.save()?;
    Ok(())
}
