use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::AppConfig;
use crate::valorant::lockfile::LockfileData;
use crate::valorant::api::RemoteAuth;

#[derive(Default)]
pub struct InnerState {
    pub config: AppConfig,
    pub lockfile: Option<LockfileData>,
    pub remote_auth: Option<RemoteAuth>,
    pub puuid: String,
    pub username: String,
    pub tag_line: String,
    pub current_phase: String,
    pub current_map: String,
    pub pregame_match_id: String,
    pub is_running: bool,
    pub is_locked: bool,
    pub cancel_requested: bool,
}

pub type AppState = Arc<Mutex<InnerState>>;

pub fn new_state() -> AppState {
    Arc::new(Mutex::new(InnerState {
        config: crate::config::load_config(),
        ..Default::default()
    }))
}
