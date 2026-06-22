use tauri::{AppHandle, Emitter, State};
use crate::config::{save_config, AppConfig, MapMacro};
use crate::state::AppState;
use crate::valorant::{agents, api, lockfile};

fn log(app: &AppHandle, msg: impl Into<String>) {
    let _ = app.emit("log-entry", msg.into());
}

async fn build_remote_auth(lock: &lockfile::LockfileData, app: &AppHandle) -> Option<api::RemoteAuth> {
    match api::get_remote_auth(lock).await {
        Ok(auth) => {
            log(app, format!("Remote auth OK (GLZ: {})", glz_hostname(&auth.glz_url)));
            Some(auth)
        }
        Err(e) => {
            log(app, format!("Remote auth failed: {e}"));
            None
        }
    }
}

fn glz_hostname(url: &str) -> &str {
    url.trim_start_matches("https://").split('.').next().unwrap_or("?")
}

// ─── Connection ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn connect(state: State<'_, AppState>, app: AppHandle) -> Result<serde_json::Value, String> {
    let lock = lockfile::read_lockfile().map_err(|e| e.to_string())?;

    let player = api::get_player_info(&lock).await.map_err(|e| e.to_string())?;
    let (phase, raw_phase) = api::get_current_phase(&lock).await
        .unwrap_or_else(|_| ("menus".to_string(), "error".to_string()));

    log(&app, format!("Connected: {} | phase: {}", player.game_name, phase));

    let remote_auth = build_remote_auth(&lock, &app).await;

    {
        let mut s = state.lock().await;
        s.lockfile = Some(lock);
        s.puuid = player.puuid.clone();
        s.username = player.game_name.clone();
        s.tag_line = player.tag_line.clone();
        s.current_phase = phase.clone();
        s.remote_auth = remote_auth;
    }

    let _ = app.emit("phase-changed", &phase);

    Ok(serde_json::json!({
        "username": player.game_name,
        "tagLine": player.tag_line,
        "puuid": player.puuid,
        "phase": phase,
    }))
}

// ─── Phase polling ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn poll_phase(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    // Auto-reconnect if no lockfile stored
    let lock = {
        let s = state.lock().await;
        if let Some(l) = &s.lockfile {
            l.clone()
        } else {
            drop(s);
            match lockfile::read_lockfile() {
                Ok(new_lock) => {
                    log(&app, "Lockfile found — auto-connecting...");
                    let player = api::get_player_info(&new_lock).await;
                    let remote_auth = build_remote_auth(&new_lock, &app).await;
                    let mut s = state.lock().await;
                    s.lockfile = Some(new_lock.clone());
                    s.remote_auth = remote_auth;
                    if let Ok(p) = player {
                        s.username = p.game_name.clone();
                        s.tag_line = p.tag_line.clone();
                        s.puuid = p.puuid;
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

    let (phase, raw_phase) = match api::get_current_phase(&lock).await {
        Ok(p) => p,
        Err(e) => {
            log(&app, format!("Phase poll network error (Valorant closed?): {e}"));
            let mut s = state.lock().await;
            s.lockfile = None;
            s.remote_auth = None;
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
        log(&app, format!("Phase: {} → {}", old_phase, phase));
        let _ = app.emit("phase-changed", &phase);

        // Reset lock state only when back in menus (game over / dodge)
        // so we don't re-poll a match we already locked
        if phase == "menus" {
            let mut s = state.lock().await;
            s.is_locked = false;
            s.pregame_match_id.clear();
            s.current_map.clear();
        }

        // When entering pregame (Gameplay phase), refresh remote auth to have latest tokens
        if phase == "pregame" {
            let remote = build_remote_auth(&lock, &app).await;
            let mut s = state.lock().await;
            if let Some(auth) = remote {
                s.puuid = auth.puuid.clone();
                s.remote_auth = Some(auth);
            }
        }
    }

    // Attempt to lock: when not ingame and armed, poll pregame API via GLZ
    if phase != "ingame" {
        let (is_running, is_locked, config, remote_auth) = {
            let s = state.lock().await;
            (s.is_running, s.is_locked, s.config.clone(), s.remote_auth.clone())
        };

        if is_running && !is_locked {
            let Some(auth) = remote_auth else {
                // No remote auth yet — try to build it
                log(&app, "No remote auth — retrying...");
                let remote = build_remote_auth(&lock, &app).await;
                let mut s = state.lock().await;
                if let Some(a) = remote {
                    s.puuid = a.puuid.clone();
                    s.remote_auth = Some(a);
                }
                return Ok(phase);
            };

            if auth.glz_url.is_empty() {
                log(&app, "GLZ URL empty — waiting for Valorant to connect to game server");
                return Ok(phase);
            }

            match api::get_pregame_match_id(&auth).await {
                Err(_) => {
                    // Not in agent select yet — silent retry
                }
                Ok(match_id) if match_id.is_empty() => {}
                Ok(match_id) => {
                    let map = api::get_pregame_map(&auth, &match_id).await.unwrap_or_default();
                    {
                        let mut s = state.lock().await;
                        s.pregame_match_id = match_id.clone();
                        s.current_map = map.clone();
                    }
                    if !map.is_empty() {
                        let _ = app.emit("map-detected", &map);
                    }

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
                        log(&app, "Cannot lock: no agent selected");
                        return Ok(phase);
                    }

                    log(&app, format!(
                        "Agent select — map={} agent={:.8}... mode={} delay={}ms",
                        if map.is_empty() { "unknown" } else { &map },
                        &agent_id, lock_mode, config.delay_ms
                    ));

                    if config.delay_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(config.delay_ms)).await;
                    }

                    let cancelled = { state.lock().await.cancel_requested };
                    if cancelled {
                        log(&app, "Lock cancelled during delay");
                        return Ok(phase);
                    }

                    let select_result = api::select_agent(&auth, &match_id, &agent_id).await;
                    if let Err(e) = &select_result {
                        log(&app, format!("Select ERR: {e}"));
                    }

                    let final_result = if lock_mode == "lock" {
                        let r = api::lock_agent(&auth, &match_id, &agent_id).await;
                        if let Err(e) = &r { log(&app, format!("Lock ERR: {e}")); }
                        r
                    } else {
                        select_result
                    };

                    match final_result {
                        Ok(_) => {
                            log(&app, "Locked ✓ — armed for next game");
                            let mut s = state.lock().await;
                            s.is_locked = true;
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
pub async fn start_instalocker(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let mut s = state.lock().await;
    if s.config.selected_agent_id.is_empty() && !s.config.macro_enabled {
        return Err("No agent selected".to_string());
    }
    s.is_running = true;
    s.is_locked = false;
    s.cancel_requested = false;
    log(&app, format!(
        "Armed — agent={} mode={} delay={}ms",
        if s.config.selected_agent_id.is_empty() { "macro" } else { &s.config.selected_agent_name },
        s.config.lock_mode,
        s.config.delay_ms
    ));
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
    let (auth, match_id) = {
        let s = state.lock().await;
        let auth = s.remote_auth.clone().ok_or("Not connected (no remote auth)")?;
        let mid = s.pregame_match_id.clone();
        (auth, mid)
    };

    if match_id.is_empty() {
        return Err("No active pregame match".to_string());
    }

    api::quit_pregame(&auth, &match_id).await.map_err(|e| e.to_string())
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
pub async fn save_config_cmd(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
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
