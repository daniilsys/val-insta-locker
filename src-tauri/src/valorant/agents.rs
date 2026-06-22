use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub uuid: String,
    pub display_name: String,
    pub role: String,
    pub display_icon_small: String,
    pub bust_portrait: String,
}

#[derive(Deserialize)]
struct ApiAgent {
    uuid: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "displayIconSmall")]
    display_icon_small: Option<String>,
    #[serde(rename = "bustPortrait")]
    bust_portrait: Option<String>,
    #[serde(rename = "isPlayableCharacter")]
    is_playable_character: bool,
    role: Option<ApiRole>,
}

#[derive(Deserialize)]
struct ApiRole {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    data: Vec<ApiAgent>,
}

pub async fn fetch_agents() -> Result<Vec<Agent>> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://valorant-api.com/v1/agents?isPlayableCharacter=true")
        .send()
        .await?;

    let api: ApiResponse = resp.json().await?;

    let mut agents: Vec<Agent> = api
        .data
        .into_iter()
        .filter(|a| a.is_playable_character)
        .map(|a| Agent {
            uuid: a.uuid,
            display_name: a.display_name,
            role: a.role.map(|r| r.display_name).unwrap_or_default(),
            display_icon_small: a.display_icon_small.unwrap_or_default(),
            bust_portrait: a.bust_portrait.unwrap_or_default(),
        })
        .collect();

    agents.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    Ok(agents)
}
