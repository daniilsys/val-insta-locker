use anyhow::{bail, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LockfileData {
    pub port: u16,
    pub password: String,
}

impl LockfileData {
    pub fn basic_auth(&self) -> String {
        let raw = format!("riot:{}", self.password);
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, raw.as_bytes())
    }

    pub fn base_url(&self) -> String {
        format!("https://127.0.0.1:{}", self.port)
    }
}

fn lockfile_path() -> PathBuf {
    // Windows: %LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
    // macOS (used during dev/testing): ~/Library/Application Support/Riot Games/...
    #[cfg(target_os = "windows")]
    {
        let local = dirs::data_local_dir().unwrap_or_default();
        local
            .join("Riot Games")
            .join("Riot Client")
            .join("Config")
            .join("lockfile")
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback for dev on macOS
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library")
            .join("Application Support")
            .join("Riot Games")
            .join("Riot Client")
            .join("Config")
            .join("lockfile")
    }
}

pub fn read_lockfile() -> Result<LockfileData> {
    let path = lockfile_path();
    if !path.exists() {
        bail!("Lockfile not found — is Valorant running?");
    }
    let contents = std::fs::read_to_string(&path)?;
    // format: name:pid:port:password:protocol
    let parts: Vec<&str> = contents.trim().split(':').collect();
    if parts.len() < 5 {
        bail!("Lockfile format unexpected: {}", contents);
    }
    let port = parts[2].parse::<u16>()?;
    let password = parts[3].to_string();
    Ok(LockfileData { port, password })
}
