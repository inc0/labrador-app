mod auth;
mod gemini;
mod github;
mod keychain;

use anyhow::Result;
use base64::Engine;
use chrono::Local;
use gemini::{GeminiClient, ImageKind, LinkedNote};
use github::{GitHubClient, PrSummary, RepoConfig};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use uuid::Uuid;

// ── app state ─────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct AppState {
    device_code: Mutex<Option<auth::DeviceCodeResponse>>,
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn gemini(app: &AppHandle) -> Result<GeminiClient> {
    let key = keychain::load(app, "gemini_api_key")?
        .ok_or_else(|| anyhow::anyhow!("Gemini API key not set"))?;
    Ok(GeminiClient::new(&key, "gemini-2.5-flash"))
}

fn github_client(app: &AppHandle) -> Result<GitHubClient> {
    let token = auth::get_token(app)?
        .ok_or_else(|| anyhow::anyhow!("Not authenticated with GitHub"))?;
    Ok(GitHubClient::new(&token))
}

fn repo_cfg(app: &AppHandle) -> Result<RepoConfig> {
    let raw = keychain::load(app, "repo_config")?
        .ok_or_else(|| anyhow::anyhow!("GitHub repo not configured"))?;
    Ok(serde_json::from_str(&raw)?)
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// --- Auth ---

#[tauri::command]
async fn auth_start(state: State<'_, AppState>) -> Result<auth::DeviceCodeResponse, String> {
    let dc = auth::request_device_code().await.map_err(|e| e.to_string())?;
    *state.device_code.lock().await = Some(dc.clone());
    Ok(dc)
}

#[tauri::command]
async fn auth_poll(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let guard = state.device_code.lock().await;
    let dc = guard.as_ref().ok_or("No device code flow in progress")?;
    let device_code = dc.device_code.clone();
    let interval = dc.interval;
    drop(guard);
    auth::poll_for_token(&app, &device_code, interval)
        .await
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn auth_status(app: AppHandle) -> Result<bool, String> {
    auth::get_token(&app).map(|t| t.is_some()).map_err(|e| e.to_string())
}

#[tauri::command]
fn auth_logout(app: AppHandle) -> Result<(), String> {
    auth::revoke_token(&app).map_err(|e| e.to_string())
}

/// --- Config ---

#[tauri::command]
fn set_gemini_key(app: AppHandle, key: String) -> Result<(), String> {
    keychain::store(&app, "gemini_api_key", &key).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_gemini_key(app: AppHandle) -> Result<Option<String>, String> {
    keychain::load(&app, "gemini_api_key").map_err(|e| e.to_string())
}

#[tauri::command]
fn set_repo_config(app: AppHandle, owner: String, repo: String, base_branch: String) -> Result<(), String> {
    let cfg = RepoConfig { owner, repo, base_branch };
    let raw = serde_json::to_string(&cfg).map_err(|e| e.to_string())?;
    keychain::store(&app, "repo_config", &raw).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_repo_config(app: AppHandle) -> Result<Option<RepoConfig>, String> {
    let raw = keychain::load(&app, "repo_config").map_err(|e| e.to_string())?;
    match raw {
        None => Ok(None),
        Some(s) => serde_json::from_str(&s).map(Some).map_err(|e| e.to_string()),
    }
}

/// --- Processing ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputImage {
    pub data: String,
    pub mime: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessedNote {
    pub note: LinkedNote,
    pub note_source_paths: Vec<String>,
    pub supplemental: Vec<(String, String)>,
    pub markdown: String,
    pub branch: String,
}

#[tauri::command]
async fn process_images(app: AppHandle, images: Vec<InputImage>) -> Result<ProcessedNote, String> {
    let ai = gemini(&app).map_err(|e| e.to_string())?;
    let branch = format!("lab-notes/{}", Local::now().format("%Y-%m-%d-%H%M%S"));

    let mut note_texts: Vec<String> = vec![];
    let mut note_source_paths: Vec<String> = vec![];
    let mut supplemental: Vec<(String, String)> = vec![];

    for img in &images {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&img.data)
            .map_err(|e| e.to_string())?;

        let kind = ai.classify(&bytes, &img.mime).await.map_err(|e| e.to_string())?;
        let ext = img.mime.split('/').nth(1).unwrap_or("jpg");
        let filename = format!("assets/{}.{}", Uuid::new_v4().simple(), ext);

        match kind {
            ImageKind::Notes => {
                let text = ai.transcribe(&bytes, &img.mime).await.map_err(|e| e.to_string())?;
                note_texts.push(text);
                note_source_paths.push(filename);
            }
            ImageKind::Photo => {
                let alt = ai.alt_text(&bytes, &img.mime).await.map_err(|e| e.to_string())?;
                supplemental.push((filename, alt));
            }
        }
    }

    let raw_text = if note_texts.is_empty() {
        "[No handwritten notes — see attached lab images below.]".to_string()
    } else {
        note_texts.join("\n\n")
    };

    let note = ai.link(&raw_text, 10).await.map_err(|e| e.to_string())?;
    let markdown = build_markdown(&note, &note_source_paths, &supplemental);

    Ok(ProcessedNote { note, note_source_paths, supplemental, markdown, branch })
}

fn build_markdown(
    note: &LinkedNote,
    note_sources: &[String],
    supplemental: &[(String, String)],
) -> String {
    let mut lines: Vec<String> = vec![];

    if !note.tags.is_empty() {
        let tags = note.tags.iter().map(|t| format!("[[{t}]]")).collect::<Vec<_>>().join(", ");
        lines.push(format!("tags:: {tags}"));
    }

    for line in note.linked_text.lines() {
        let s = line.trim_end();
        if s.starts_with("- ") || s.is_empty() {
            lines.push(s.to_string());
        } else {
            lines.push(format!("- {s}"));
        }
    }

    if !note_sources.is_empty() {
        lines.push("- **Note sources**".to_string());
        for path in note_sources {
            lines.push(format!("  - ![note source]({path})"));
        }
    }

    if !supplemental.is_empty() {
        lines.push("- **Supplemental images**".to_string());
        for (path, alt) in supplemental {
            lines.push(format!("  - ![{alt}]({path})"));
        }
    }

    lines.join("\n") + "\n"
}

/// --- GitHub PR ---

#[derive(Serialize, Deserialize)]
pub struct SubmitPayload {
    pub branch: String,
    pub title: String,
    pub markdown: String,
    pub images: Vec<InputImage>,
    pub note_source_paths: Vec<String>,
    pub supplemental: Vec<(String, String)>,
}

#[tauri::command]
async fn submit_pr(app: AppHandle, payload: SubmitPayload) -> Result<String, String> {
    let gh = github_client(&app).map_err(|e| e.to_string())?;
    let cfg = repo_cfg(&app).map_err(|e| e.to_string())?;

    gh.create_branch(&cfg, &payload.branch)
        .await
        .map_err(|e| e.to_string())?;

    let paths: Vec<&str> = payload.note_source_paths
        .iter()
        .map(String::as_str)
        .chain(payload.supplemental.iter().map(|(p, _)| p.as_str()))
        .collect();

    for (img, path) in payload.images.iter().zip(paths.iter()) {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&img.data)
            .map_err(|e| e.to_string())?;
        gh.upload_file(&cfg, &payload.branch, path, &bytes, "add: lab image")
            .await
            .map_err(|e| e.to_string())?;
    }

    let safe_title: String = payload.title
        .chars()
        .map(|c| if r#"/\:*?"<>|"#.contains(c) { '-' } else { c })
        .collect();
    let note_path = format!("pages/{safe_title}.md");

    gh.upload_file(
        &cfg, &payload.branch, &note_path,
        payload.markdown.as_bytes(),
        &format!("add: {}", payload.title),
    )
    .await
    .map_err(|e| e.to_string())?;

    let pr_body = format!(
        "## {}\n\n```\n{}\n```\n\n---\n*Created by Labrador*",
        payload.title, payload.markdown
    );
    gh.create_pr(&cfg, &payload.branch, &payload.title, &pr_body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_prs(app: AppHandle) -> Result<Vec<PrSummary>, String> {
    let gh = github_client(&app).map_err(|e| e.to_string())?;
    let cfg = repo_cfg(&app).map_err(|e| e.to_string())?;
    gh.list_open_prs(&cfg, "lab-notes/")
        .await
        .map_err(|e| e.to_string())
}

// ── entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            auth_start,
            auth_poll,
            auth_status,
            auth_logout,
            set_gemini_key,
            get_gemini_key,
            set_repo_config,
            get_repo_config,
            process_images,
            submit_pr,
            list_prs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
