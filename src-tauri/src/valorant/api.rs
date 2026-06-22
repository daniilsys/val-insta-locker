use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use super::lockfile::LockfileData;

fn build_client() -> Result<Client> {
    Ok(Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?)
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerInfo {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

/// Fetch PUUID from the entitlements token endpoint — most reliable source.
pub async fn get_puuid(lock: &LockfileData) -> Result<String> {
    let client = build_client()?;
    let resp = client
        .get(format!("{}/entitlements/v1/token", lock.base_url()))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        bail!("entitlements/v1/token HTTP {}: {}", status, &body[..body.len().min(200)]);
    }
    let v: Value = serde_json::from_str(&body)?;
    let puuid = v["subject"].as_str().unwrap_or("").to_string();
    if puuid.is_empty() {
        bail!("entitlements/v1/token: 'subject' field missing. Body: {}", &body[..body.len().min(300)]);
    }
    Ok(puuid)
}

pub async fn get_player_info(lock: &LockfileData) -> Result<PlayerInfo> {
    let puuid = get_puuid(lock).await?;

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
                    let name = v["game_name"].as_str().unwrap_or("Player").to_string();
                    let tag = v["game_tag"].as_str().unwrap_or("").to_string();
                    (name, tag)
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

/// Returns (normalized_phase, raw_phase) so caller can log the raw string.
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
                    // "Gameplay" = agent select phase (confirmed from original repo)
                    "pregame" | "agent_select" | "agentselect" | "characterselect" | "gameplay" => "pregame",
                    // "ingame" / "ıngame" = actual in-game (match started)
                    "ingame" | "in_game" | "inprogress" | "ingameclient" | "ıngame" => "ingame",
                    _ => "unknown",
                };
                return Ok((normalized.to_string(), raw));
            }
        }
    }
    Ok(("menus".to_string(), "no_valorant_session".to_string()))
}

pub async fn get_pregame_match_id(lock: &LockfileData, puuid: &str) -> Result<String> {
    let client = build_client()?;
    let resp = client
        .get(format!("{}/pregame/v1/players/{}", lock.base_url(), puuid))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
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

pub async fn get_pregame_map(lock: &LockfileData, match_id: &str) -> Result<String> {
    let client = build_client()?;
    let resp = client
        .get(format!("{}/pregame/v1/matches/{}", lock.base_url(), match_id))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
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

pub async fn select_agent(lock: &LockfileData, match_id: &str, agent_id: &str) -> Result<()> {
    let client = build_client()?;
    let resp = client
        .post(format!("{}/pregame/v1/matches/{}/select/{}", lock.base_url(), match_id, agent_id))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("select HTTP {}: {}", status, &body[..body.len().min(300)]);
    }
    Ok(())
}

pub async fn lock_agent(lock: &LockfileData, match_id: &str, agent_id: &str) -> Result<()> {
    let client = build_client()?;
    let resp = client
        .post(format!("{}/pregame/v1/matches/{}/lock/{}", lock.base_url(), match_id, agent_id))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("lock HTTP {}: {}", status, &body[..body.len().min(300)]);
    }
    Ok(())
}

pub async fn quit_pregame(lock: &LockfileData, match_id: &str) -> Result<()> {
    let client = build_client()?;
    let resp = client
        .post(format!("{}/pregame/v1/matches/{}/quit", lock.base_url(), match_id))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("quit pregame returned {}", resp.status());
    }
    Ok(())
}
