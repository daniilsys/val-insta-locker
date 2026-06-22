use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use super::lockfile::LockfileData;

// Base64-encoded platform JSON — hardcoded constant, same as RadiantConnect
const CLIENT_PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

fn build_client() -> Result<Client> {
    Ok(Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?)
}

/// Remote auth data needed for GLZ (game server) API calls.
/// Pregame endpoints live on the GLZ URL, not localhost.
#[derive(Debug, Clone, Default)]
pub struct RemoteAuth {
    pub glz_url: String,
    pub access_token: String,
    pub entitlement_token: String,
    pub client_version: String,
    pub puuid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerInfo {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

/// Read the GLZ game-server URL from the Valorant ShooterGame log file.
/// Valorant writes it there when it connects — RadiantConnect uses the same approach.
pub fn read_glz_url() -> Result<String> {
    #[cfg(target_os = "windows")]
    let log_path = {
        let local = dirs::data_local_dir().unwrap_or_default();
        local.join("Valorant").join("Saved").join("Logs").join("ShooterGame.log")
    };
    #[cfg(not(target_os = "windows"))]
    let log_path = std::path::PathBuf::from("/tmp/ShooterGame_nonexistent.log");

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| anyhow::anyhow!("Cannot read ShooterGame.log: {e}"))?;

    let mut last_glz: Option<String> = None;
    for line in content.lines() {
        if let Some(idx) = line.find("https://glz") {
            let rest = &line[idx..];
            // Stop at ".net" to get only the base URL, not any path after it
            let end = rest
                .find(".net")
                .map(|i| i + 4)
                .or_else(|| rest.find(|c: char| c.is_whitespace() || c == '"' || c == '\''))
                .unwrap_or(rest.len());
            let url = rest[..end].trim_end_matches('/');
            if url.starts_with("https://glz") {
                last_glz = Some(url.to_string());
            }
        }
    }

    last_glz.ok_or_else(|| {
        anyhow::anyhow!("GLZ URL not found in ShooterGame.log — queue into a game first")
    })
}

/// Fetch client version from valorant-api.com (best-effort, empty string on failure).
async fn get_client_version() -> String {
    let Ok(client) = reqwest::Client::builder().build() else { return String::new() };
    let Ok(resp) = client.get("https://valorant-api.com/v1/version").send().await else { return String::new() };
    let Ok(v) = resp.json::<Value>().await else { return String::new() };
    v["data"]["riotClientVersion"].as_str().unwrap_or("").to_string()
}

/// Build the full remote auth context needed to call GLZ (pregame) endpoints.
/// - Bearer token + Entitlement JWT from local /entitlements/v1/token
/// - GLZ URL from Valorant ShooterGame log file
pub async fn get_remote_auth(lock: &LockfileData) -> Result<RemoteAuth> {
    let client = build_client()?;
    let resp = client
        .get(format!("{}/entitlements/v1/token", lock.base_url()))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        bail!("entitlements/v1/token HTTP {}: {}", status, &body[..body.len().min(300)]);
    }

    let v: Value = serde_json::from_str(&body)?;
    let access_token = v["accessToken"].as_str().unwrap_or("").to_string();
    let entitlement_token = v["token"].as_str().unwrap_or("").to_string();
    let puuid = v["subject"].as_str().unwrap_or("").to_string();

    if access_token.is_empty() {
        bail!("entitlements: 'accessToken' missing. Body: {}", &body[..body.len().min(300)]);
    }
    if entitlement_token.is_empty() {
        bail!("entitlements: entitlement 'token' missing. Body: {}", &body[..body.len().min(300)]);
    }

    let glz_url = read_glz_url()?;
    let client_version = get_client_version().await;

    Ok(RemoteAuth { glz_url, access_token, entitlement_token, client_version, puuid })
}

/// Helper to build a request with all required remote headers.
fn remote_request(
    client: &Client,
    method: reqwest::Method,
    url: &str,
    auth: &RemoteAuth,
) -> reqwest::RequestBuilder {
    let mut req = client
        .request(method, url)
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .header("X-Riot-Entitlements-JWT", &auth.entitlement_token)
        .header("X-Riot-ClientPlatform", CLIENT_PLATFORM);
    if !auth.client_version.is_empty() {
        req = req.header("X-Riot-ClientVersion", &auth.client_version);
    }
    req
}

// ─── Local endpoints (127.0.0.1 + Basic auth) ────────────────────────────────

pub async fn get_player_info(lock: &LockfileData) -> Result<PlayerInfo> {
    let puuid = {
        let client = build_client()?;
        let resp = client
            .get(format!("{}/entitlements/v1/token", lock.base_url()))
            .header("Authorization", format!("Basic {}", lock.basic_auth()))
            .send()
            .await?;
        let v: Value = resp.json().await?;
        v["subject"].as_str().unwrap_or("").to_string()
    };

    let (game_name, tag_line) = {
        let client = build_client()?;
        if let Ok(resp) = client
            .get(format!("{}/chat/v1/session", lock.base_url()))
            .header("Authorization", format!("Basic {}", lock.basic_auth()))
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(v) = resp.json::<Value>().await {
                    (
                        v["game_name"].as_str().unwrap_or("Player").to_string(),
                        v["game_tag"].as_str().unwrap_or("").to_string(),
                    )
                } else {
                    ("Player".to_string(), "".to_string())
                }
            } else {
                ("Player".to_string(), "".to_string())
            }
        } else {
            ("Player".to_string(), "".to_string())
        }
    };

    Ok(PlayerInfo { puuid, game_name, tag_line })
}

/// Returns (normalized_phase, raw_phase).
pub async fn get_current_phase(lock: &LockfileData) -> Result<(String, String)> {
    let client = build_client()?;
    let resp = client
        .get(format!("{}/product-session/v1/external-sessions", lock.base_url()))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(("unknown".to_string(), format!("http_{}", resp.status())));
    }
    let v: Value = resp.json().await?;
    if let Some(obj) = v.as_object() {
        for (_, session) in obj {
            if session["productId"].as_str() == Some("valorant") {
                let raw = session["phase"].as_str().unwrap_or("unknown").to_string();
                let normalized = match raw.to_lowercase().as_str() {
                    "menus" | "lobby" | "mainmenu" | "home" | "idle" => "menus",
                    // "Gameplay" = agent select phase (confirmed from RadiantConnect source)
                    "pregame" | "agent_select" | "agentselect" | "characterselect" | "gameplay" => "pregame",
                    // "ingame" / "ıngame" = match actually started
                    "ingame" | "in_game" | "inprogress" | "ingameclient" | "ıngame" => "ingame",
                    _ => "unknown",
                };
                return Ok((normalized.to_string(), raw));
            }
        }
    }
    Ok(("menus".to_string(), "no_valorant_session".to_string()))
}

// ─── Remote endpoints (GLZ URL + Bearer auth) ────────────────────────────────

pub async fn get_pregame_match_id(auth: &RemoteAuth) -> Result<String> {
    let client = build_client()?;
    let url = format!("{}/pregame/v1/players/{}", auth.glz_url, auth.puuid);
    let resp = remote_request(&client, reqwest::Method::GET, &url, auth)
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        bail!("pregame/v1/players HTTP {}: {}", status, &body[..body.len().min(300)]);
    }
    let v: Value = serde_json::from_str(&body)?;
    let match_id = v["MatchID"].as_str().unwrap_or("").to_string();
    if match_id.is_empty() {
        bail!("pregame/v1/players: 'MatchID' missing in: {}", &body[..body.len().min(300)]);
    }
    Ok(match_id)
}

pub async fn get_pregame_map(auth: &RemoteAuth, match_id: &str) -> Result<String> {
    let client = build_client()?;
    let url = format!("{}/pregame/v1/matches/{}", auth.glz_url, match_id);
    let resp = remote_request(&client, reqwest::Method::GET, &url, auth)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(String::new());
    }
    let v: Value = resp.json().await?;
    let map_id = v["MapID"].as_str().unwrap_or("").to_string();
    let map_name = map_id.split('/').last().unwrap_or("").to_string();
    Ok(map_name)
}

pub async fn select_agent(auth: &RemoteAuth, match_id: &str, agent_id: &str) -> Result<()> {
    let client = build_client()?;
    let url = format!("{}/pregame/v1/matches/{}/select/{}", auth.glz_url, match_id, agent_id);
    let resp = remote_request(&client, reqwest::Method::POST, &url, auth)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("select HTTP {}: {}", status, &body[..body.len().min(300)]);
    }
    Ok(())
}

pub async fn lock_agent(auth: &RemoteAuth, match_id: &str, agent_id: &str) -> Result<()> {
    let client = build_client()?;
    let url = format!("{}/pregame/v1/matches/{}/lock/{}", auth.glz_url, match_id, agent_id);
    let resp = remote_request(&client, reqwest::Method::POST, &url, auth)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("lock HTTP {}: {}", status, &body[..body.len().min(300)]);
    }
    Ok(())
}

pub async fn quit_pregame(auth: &RemoteAuth, match_id: &str) -> Result<()> {
    let client = build_client()?;
    let url = format!("{}/pregame/v1/matches/{}/quit", auth.glz_url, match_id);
    let resp = remote_request(&client, reqwest::Method::POST, &url, auth)
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("quit pregame returned {}", resp.status());
    }
    Ok(())
}
