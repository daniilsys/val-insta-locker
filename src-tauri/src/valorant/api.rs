use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use super::lockfile::LockfileData;

fn build_client() -> Result<Client> {
    Ok(Client::builder()
        .danger_accept_invalid_certs(true) // Valorant uses self-signed cert on localhost
        .build()?)
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerInfo {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

pub async fn get_player_info(lock: &LockfileData) -> Result<PlayerInfo> {
    let client = build_client()?;
    let resp = client
        .get(format!("{}/chat/v1/session", lock.base_url()))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("chat/v1/session returned {}", resp.status());
    }
    let v: Value = resp.json().await?;
    Ok(PlayerInfo {
        puuid: v["puuid"].as_str().unwrap_or("").to_string(),
        game_name: v["game_name"].as_str().unwrap_or("Player").to_string(),
        tag_line: v["game_tag"].as_str().unwrap_or("").to_string(),
    })
}

pub async fn get_current_phase(lock: &LockfileData) -> Result<String> {
    let client = build_client()?;
    let resp = client
        .get(format!(
            "{}/product-session/v1/external-sessions",
            lock.base_url()
        ))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok("unknown".to_string());
    }
    let v: Value = resp.json().await?;
    if let Some(obj) = v.as_object() {
        for (_, session) in obj {
            if session["productId"].as_str() == Some("valorant") {
                let raw = session["phase"].as_str().unwrap_or("unknown").to_lowercase();
                // Normalize Valorant phase strings to our known set
                let phase = match raw.as_str() {
                    "menus" | "lobby" | "mainmenu" | "home" => "menus",
                    "pregame" | "agent_select" | "agentselect" => "pregame",
                    "ingame" | "in_game" | "inprogress" | "ingameclient" | "ıngame" => "ingame",
                    _ => "unknown",
                };
                return Ok(phase.to_string());
            }
        }
    }
    Ok("menus".to_string())
}

pub async fn get_pregame_match_id(lock: &LockfileData, puuid: &str) -> Result<String> {
    let client = build_client()?;
    let resp = client
        .get(format!(
            "{}/pregame/v1/players/{}",
            lock.base_url(),
            puuid
        ))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("pregame/v1/players returned {}", resp.status());
    }
    let v: Value = resp.json().await?;
    let match_id = v["MatchID"].as_str().unwrap_or("").to_string();
    if match_id.is_empty() {
        bail!("No pregame match found");
    }
    Ok(match_id)
}

pub async fn get_pregame_map(lock: &LockfileData, match_id: &str) -> Result<String> {
    let client = build_client()?;
    let resp = client
        .get(format!(
            "{}/pregame/v1/matches/{}",
            lock.base_url(),
            match_id
        ))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(String::new());
    }
    let v: Value = resp.json().await?;
    // MapID looks like /Game/Maps/Ascent/Ascent — extract last segment
    let map_id = v["MapID"].as_str().unwrap_or("").to_string();
    let map_name = map_id.split('/').last().unwrap_or("").to_string();
    Ok(map_name)
}

pub async fn select_agent(lock: &LockfileData, match_id: &str, agent_id: &str) -> Result<()> {
    let client = build_client()?;
    let resp = client
        .post(format!(
            "{}/pregame/v1/matches/{}/select/{}",
            lock.base_url(),
            match_id,
            agent_id
        ))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("select agent returned {}", resp.status());
    }
    Ok(())
}

pub async fn lock_agent(lock: &LockfileData, match_id: &str, agent_id: &str) -> Result<()> {
    let client = build_client()?;
    let resp = client
        .post(format!(
            "{}/pregame/v1/matches/{}/lock/{}",
            lock.base_url(),
            match_id,
            agent_id
        ))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("lock agent returned {}", resp.status());
    }
    Ok(())
}

pub async fn quit_pregame(lock: &LockfileData, match_id: &str) -> Result<()> {
    let client = build_client()?;
    let resp = client
        .post(format!(
            "{}/pregame/v1/matches/{}/quit",
            lock.base_url(),
            match_id
        ))
        .header("Authorization", format!("Basic {}", lock.basic_auth()))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("quit pregame returned {}", resp.status());
    }
    Ok(())
}
