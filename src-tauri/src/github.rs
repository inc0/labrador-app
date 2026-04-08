/// GitHub REST API client — branch, commit, PR operations.
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub owner: String,
    pub repo: String,
    pub base_branch: String, // usually "main"
}

pub struct GitHubClient {
    token: String,
    http: reqwest::Client,
}

impl GitHubClient {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            http: reqwest::Client::new(),
        }
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn api(&self, path: &str) -> String {
        format!("https://api.github.com{path}")
    }

    /// Resolve the SHA of the tip of `base_branch`.
    async fn branch_sha(&self, cfg: &RepoConfig, branch: &str) -> Result<String> {
        let url = self.api(&format!(
            "/repos/{}/{}/git/ref/heads/{}",
            cfg.owner, cfg.repo, branch
        ));
        let resp: serde_json::Value = self
            .http
            .get(&url)
            .header("Authorization", self.auth())
            .header("User-Agent", "labrador-app")
            .send()
            .await?
            .json()
            .await?;
        resp["object"]["sha"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("could not resolve branch SHA for {branch}: {resp}"))
    }

    /// Create a new branch from the tip of `base_branch`.
    pub async fn create_branch(&self, cfg: &RepoConfig, new_branch: &str) -> Result<()> {
        let sha = self.branch_sha(cfg, &cfg.base_branch).await?;
        let url = self.api(&format!("/repos/{}/{}/git/refs", cfg.owner, cfg.repo));
        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth())
            .header("User-Agent", "labrador-app")
            .json(&json!({ "ref": format!("refs/heads/{new_branch}"), "sha": sha }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("create_branch failed: {body}"));
        }
        Ok(())
    }

    /// Upload a file (raw bytes) to the repo on `branch`.
    /// Returns the blob SHA.
    pub async fn upload_file(
        &self,
        cfg: &RepoConfig,
        branch: &str,
        path: &str,       // e.g. "assets/abc123.jpg"
        content: &[u8],
        message: &str,
    ) -> Result<()> {
        let encoded = B64.encode(content);
        let url = self.api(&format!(
            "/repos/{}/{}/contents/{}",
            cfg.owner, cfg.repo, path
        ));
        let resp = self
            .http
            .put(&url)
            .header("Authorization", self.auth())
            .header("User-Agent", "labrador-app")
            .json(&json!({
                "message": message,
                "content": encoded,
                "branch": branch,
            }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("upload_file failed for {path}: {body}"));
        }
        Ok(())
    }

    /// Create a pull request and return its HTML URL.
    pub async fn create_pr(
        &self,
        cfg: &RepoConfig,
        branch: &str,
        title: &str,
        body: &str,
    ) -> Result<String> {
        let url = self.api(&format!("/repos/{}/{}/pulls", cfg.owner, cfg.repo));
        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth())
            .header("User-Agent", "labrador-app")
            .json(&json!({
                "title": title,
                "body": body,
                "head": branch,
                "base": cfg.base_branch,
            }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("create_pr failed: {body}"));
        }
        let pr: serde_json::Value = resp.json().await?;
        pr["html_url"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("no html_url in PR response"))
    }

    /// List open PRs whose head branch starts with `prefix`.
    pub async fn list_open_prs(
        &self,
        cfg: &RepoConfig,
        prefix: &str,
    ) -> Result<Vec<PrSummary>> {
        let url = self.api(&format!(
            "/repos/{}/{}/pulls?state=open&per_page=50",
            cfg.owner, cfg.repo
        ));
        let prs: Vec<serde_json::Value> = self
            .http
            .get(&url)
            .header("Authorization", self.auth())
            .header("User-Agent", "labrador-app")
            .send()
            .await?
            .json()
            .await?;
        Ok(prs
            .into_iter()
            .filter(|pr| {
                pr["head"]["ref"]
                    .as_str()
                    .map(|r| r.starts_with(prefix))
                    .unwrap_or(false)
            })
            .filter_map(|pr| {
                Some(PrSummary {
                    number: pr["number"].as_u64()? as u32,
                    title: pr["title"].as_str()?.to_string(),
                    url: pr["html_url"].as_str()?.to_string(),
                    branch: pr["head"]["ref"].as_str()?.to_string(),
                    created_at: pr["created_at"].as_str()?.to_string(),
                })
            })
            .collect())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrSummary {
    pub number: u32,
    pub title: String,
    pub url: String,
    pub branch: String,
    pub created_at: String,
}
