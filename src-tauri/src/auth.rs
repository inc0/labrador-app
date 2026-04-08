/// GitHub OAuth Device Flow authentication.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::sleep;

use crate::keychain;

const KEYCHAIN_KEY: &str = "github_token";

// Register a GitHub OAuth App at https://github.com/settings/developers and enable Device Flow.
// Pass your client ID at build time: GITHUB_CLIENT_ID=<your-id> cargo tauri build
const CLIENT_ID: &str = match option_env!("GITHUB_CLIENT_ID") {
    Some(id) => id,
    None => "",
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub async fn request_device_code() -> Result<DeviceCodeResponse> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", "repo")])
        .send()
        .await?
        .json::<DeviceCodeResponse>()
        .await?;
    Ok(resp)
}

pub async fn poll_for_token(app: &AppHandle, device_code: &str, interval_secs: u64) -> Result<String> {
    let client = reqwest::Client::new();
    loop {
        sleep(Duration::from_secs(interval_secs)).await;
        let resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        match resp.error.as_deref() {
            None => {
                let token = resp.access_token.ok_or_else(|| anyhow!("no token in response"))?;
                keychain::store(app, KEYCHAIN_KEY, &token)?;
                return Ok(token);
            }
            Some("authorization_pending") => continue,
            Some("slow_down") => sleep(Duration::from_secs(5)).await,
            Some(e) => {
                return Err(anyhow!(
                    "{}: {}",
                    e,
                    resp.error_description.unwrap_or_default()
                ))
            }
        }
    }
}

pub fn get_token(app: &AppHandle) -> Result<Option<String>> {
    keychain::load(app, KEYCHAIN_KEY)
}

pub fn revoke_token(app: &AppHandle) -> Result<()> {
    keychain::delete(app, KEYCHAIN_KEY)
}
