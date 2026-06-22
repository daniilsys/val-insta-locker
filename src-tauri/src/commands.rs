use tauri::{AppHandle, Emitter, State};
use crate::config::{save_config, AppConfig, MapMacro};
use crate::state::AppState;
use crate::valorant::{agents, api, lockfile};

// ─── Connection ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn connect(state: State<'_, AppState>, app: AppHandle) -> Result<serde_json::Value, String> {
    let lock = lockfile::read_lockfile().map_err(|e| e.to_string())?;

    let player = api::get_player_info(&lock).await.map_err(|e| e.to_string())?;
    let phase = api::get_current_phase(&lock).await.unwrap_or_else(|_| "menus".to_string());

    {
        let mut s = state.lock().await;
        s.lockfile = Some(lock);
        s.puuid = player.puuid.clone();
        s.username = player.game_name.clone();
        s.tag_line = player.tag_line.clone();
        s.current_phase = phase.clone();
    }

    let _ = app.emit("phase-changed", &phase);

    Ok(serde_json::json!({
        "username": player.game_name,
        "tagLine": player.tag_line,
        "puuid": player.puuid,
        "phase": phase,
    }))
}

// ─── Phase polling (called from watcher loop) ────────────────────────────────

#[tauri::command]
pub async fn poll_phase(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let lock = {
        let s = state.lock().await;
        match &s.lockfile {
            Some(l) => l.clone(),
            None => return Err("Not connected".to_string()),
        }
    };

    let phase = api::get_current_phase(&lock).await.unwrap_or_else(|_| "unknown".to_string());

    let old_phase = {
        let mut s = state.lock().await;
        let old = s.current_phase.clone();
        s.current_phase = phase.clone();
        old
    };

    if phase != old_phase {
        let _ = app.emit("phase-changed", &phase);

        // Phase went back to menus after pregame — reset lock state
        if phase == "menus" || phase == "unknown" {
            let mut s = state.lock().await;
            s.is_locked = false;
            s.pregame_match_id.clear();
            s.current_map.clear();
            let _ = app.emit("lock-status", false);
        }
    }

    // If we're in pregame and instalocker is running, attempt to lock
    if phase == "pregame" {
        let (is_running, is_locked, puuid, config) = {
            let s = state.lock().await;
            (s.is_running, s.is_locked, s.puuid.clone(), s.config.clone())
        };

        if is_running && !is_locked && !puuid.is_empty() {
            let match_id = api::get_pregame_match_id(&lock, &puuid).await.unwrap_or_default();
            if !match_id.is_empty() {
                // Detect map for macro
                let map = api::get_pregame_map(&lock, &match_id).await.unwrap_or_default();
                {
                    let mut s = state.lock().await;
                    s.pregame_match_id = match_id.clone();
                    s.current_map = map.clone();
                }
                let _ = app.emit("map-detected", &map);

                // Determine agent + mode from macro or default config
                let (agent_id, lock_mode) = if config.macro_enabled && !map.is_empty() {
                    if let Some(m) = config.map_macros.get(&map) {
                        (m.agent_id.clone(), m.lock_mode.clone())
                    } else {
                        (config.selected_agent_id.clone(), config.lock_mode.clone())
                    }
                } else {
                    (config.selected_agent_id.clone(), config.lock_mode.clone())
                };

                if agent_id.is_empty() {
                    return Ok(phase);
                }

                // Delay before locking
                if config.delay_ms > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(config.delay_ms)).await;
                }

                // Double-check cancel wasn't requested during delay
                let cancelled = {
                    let s = state.lock().await;
                    s.cancel_requested
                };

                if !cancelled {
                    let result = if lock_mode == "select" {
                        api::select_agent(&lock, &match_id, &agent_id).await
                    } else {
                        api::lock_agent(&lock, &match_id, &agent_id).await
                    };

                    match result {
                        Ok(_) => {
                            let mut s = state.lock().await;
                            s.is_locked = true;
                            s.is_running = false;
                            let _ = app.emit("lock-status", true);
                            let _ = app.emit("agent-locked", &agent_id);
                        }
                        Err(e) => {
                            let _ = app.emit("lock-error", e.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(phase)
}

// ─── Instalocker control ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_instalocker(state: State<'_, AppState>) -> Result<(), String> {
    let mut s = state.lock().await;
    if s.config.selected_agent_id.is_empty() && !s.config.macro_enabled {
        return Err("No agent selected".to_string());
    }
    s.is_running = true;
    s.is_locked = false;
    s.cancel_requested = false;
    Ok(())
}

#[tauri::command]
pub async fn cancel_instalocker(state: State<'_, AppState>) -> Result<(), String> {
    let mut s = state.lock().await;
    s.is_running = false;
    s.is_locked = false;
    s.cancel_requested = true;
    Ok(())
}

#[tauri::command]
pub async fn quit_pregame(state: State<'_, AppState>) -> Result<(), String> {
    let (lock, match_id) = {
        let s = state.lock().await;
        let lock = s.lockfile.clone().ok_or("Not connected")?;
        let mid = s.pregame_match_id.clone();
        (lock, mid)
    };

    if match_id.is_empty() {
        return Err("No active pregame match".to_string());
    }

    api::quit_pregame(&lock, &match_id).await.map_err(|e| e.to_string())
}

// ─── Agents ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_agents() -> Result<Vec<agents::Agent>, String> {
    agents::fetch_agents().await.map_err(|e| e.to_string())
}

// ─── Config ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let s = state.lock().await;
    Ok(s.config.clone())
}

#[tauri::command]
pub async fn save_config_cmd(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.config = config.clone();
    save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_map_macro(
    state: State<'_, AppState>,
    map: String,
    macro_data: MapMacro,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.config.map_macros.insert(map, macro_data);
    let config = s.config.clone();
    drop(s);
    save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let s = state.lock().await;
    Ok(serde_json::json!({
        "isRunning": s.is_running,
        "isLocked": s.is_locked,
        "phase": s.current_phase,
        "map": s.current_map,
        "username": s.username,
        "tagLine": s.tag_line,
    }))
}
