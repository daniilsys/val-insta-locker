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
    // Auto-reconnect: if no lockfile stored, try reading it now
    let lock = {
        let s = state.lock().await;
        if let Some(l) = &s.lockfile {
            l.clone()
        } else {
            drop(s);
            match lockfile::read_lockfile() {
                Ok(new_lock) => {
                    // Got the lockfile — fetch PUUID immediately
                    let puuid = api::get_puuid(&new_lock).await.unwrap_or_default();
                    let player = api::get_player_info(&new_lock).await;
                    let mut s = state.lock().await;
                    s.lockfile = Some(new_lock.clone());
                    if !puuid.is_empty() {
                        s.puuid = puuid;
                    }
                    if let Ok(p) = player {
                        s.username = p.game_name.clone();
                        s.tag_line = p.tag_line.clone();
                        if s.puuid.is_empty() {
                            s.puuid = p.puuid;
                        }
                        let _ = app.emit("connected", serde_json::json!({
                            "username": p.game_name,
                            "tagLine": p.tag_line,
                        }));
                    }
                    new_lock
                }
                Err(_) => return Ok("unknown".to_string()),
            }
        }
    };

    let phase = match api::get_current_phase(&lock).await {
        Ok(p) => p,
        Err(_) => {
            // Network error — Valorant likely restarted or closed. Clear stale lockfile.
            let mut s = state.lock().await;
            s.lockfile = None;
            s.puuid.clear();
            s.is_locked = false;
            s.pregame_match_id.clear();
            s.current_map.clear();
            let _ = app.emit("disconnected", ());
            let _ = app.emit("phase-changed", "unknown");
            return Ok("unknown".to_string());
        }
    };

    let old_phase = {
        let mut s = state.lock().await;
        let old = s.current_phase.clone();
        s.current_phase = phase.clone();
        old
    };

    if phase != old_phase {
        let _ = app.emit("phase-changed", &phase);

        // Phase went back to menus/unknown after pregame — reset lock state
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
        let (is_running, is_locked, mut puuid, config) = {
            let s = state.lock().await;
            (s.is_running, s.is_locked, s.puuid.clone(), s.config.clone())
        };

        // PUUID fallback — try entitlements if empty
        if puuid.is_empty() {
            if let Ok(p) = api::get_puuid(&lock).await {
                let _ = app.emit("log-entry", format!("PUUID recovered via entitlements"));
                let mut s = state.lock().await;
                s.puuid = p.clone();
                puuid = p;
            } else {
                let _ = app.emit("log-entry", "PUUID unavailable — cannot lock");
            }
        }

        if is_running && !is_locked && !puuid.is_empty() {
            match api::get_pregame_match_id(&lock, &puuid).await {
                Ok(match_id) if !match_id.is_empty() => {
                    // Detect map for macro
                    let map = api::get_pregame_map(&lock, &match_id).await.unwrap_or_default();
                    {
                        let mut s = state.lock().await;
                        s.pregame_match_id = match_id.clone();
                        s.current_map = map.clone();
                    }
                    if !map.is_empty() {
                        let _ = app.emit("map-detected", &map);
                    }

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
                        let _ = app.emit("log-entry", "No agent selected — cannot lock");
                        return Ok(phase);
                    }

                    let _ = app.emit("log-entry", format!("Pregame detected — match {}", &match_id[..8.min(match_id.len())]));

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
                        // Always select first, then lock if mode == "lock"
                        let select_result = api::select_agent(&lock, &match_id, &agent_id).await;
                        if let Err(ref e) = select_result {
                            let _ = app.emit("log-entry", format!("Select failed: {e}"));
                        }

                        let result = if lock_mode == "lock" {
                            api::lock_agent(&lock, &match_id, &agent_id).await
                        } else {
                            select_result.map_err(|e| e)
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
                Ok(_) => {} // empty match_id, not in pregame yet
                Err(_) => {} // not in pregame API yet, will retry next poll
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
