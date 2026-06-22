use tauri::{AppHandle, Emitter, State};
use crate::config::{save_config, AppConfig, MapMacro};
use crate::state::AppState;
use crate::valorant::{agents, api, lockfile};

fn log(app: &AppHandle, msg: impl Into<String>) {
    let _ = app.emit("log-entry", msg.into());
}

// ─── Connection ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn connect(state: State<'_, AppState>, app: AppHandle) -> Result<serde_json::Value, String> {
    let lock = lockfile::read_lockfile().map_err(|e| e.to_string())?;

    let player = api::get_player_info(&lock).await.map_err(|e| e.to_string())?;
    let (phase, raw_phase) = api::get_current_phase(&lock).await
        .unwrap_or_else(|_| ("menus".to_string(), "error".to_string()));

    log(&app, format!("Connected: {} | PUUID: {}... | phase raw={} norm={}",
        player.game_name,
        &player.puuid[..player.puuid.len().min(8)],
        raw_phase, phase
    ));

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
                    log(&app, "Lockfile found — auto-connecting...");
                    let puuid = api::get_puuid(&new_lock).await.unwrap_or_default();
                    let player = api::get_player_info(&new_lock).await;
                    let mut s = state.lock().await;
                    s.lockfile = Some(new_lock.clone());
                    if !puuid.is_empty() {
                        s.puuid = puuid.clone();
                        log(&app, format!("PUUID (entitlements): {}...", &puuid[..puuid.len().min(8)]));
                    }
                    if let Ok(p) = player {
                        s.username = p.game_name.clone();
                        s.tag_line = p.tag_line.clone();
                        if s.puuid.is_empty() {
                            s.puuid = p.puuid.clone();
                            log(&app, format!("PUUID (fallback): {}...", &p.puuid[..p.puuid.len().min(8)]));
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

    let (phase, raw_phase) = match api::get_current_phase(&lock).await {
        Ok(p) => p,
        Err(e) => {
            log(&app, format!("Phase poll error (disconnected?): {e}"));
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
        log(&app, format!("Phase: {} → {} (raw: {})", old_phase, phase, raw_phase));
        let _ = app.emit("phase-changed", &phase);

        // Reset lock state only when the game actually ended (ingame → anything)
        // or when explicitly back in menus. Don't reset on "unknown" — that's also
        // what "Idle" maps to, which can appear during agent select.
        let game_ended = old_phase == "ingame" && phase != "ingame";
        let back_to_menus = phase == "menus";
        if game_ended || back_to_menus {
            let mut s = state.lock().await;
            s.is_locked = false;
            s.pregame_match_id.clear();
            s.current_map.clear();
            let _ = app.emit("lock-status", false);
        }
    }

    // Attempt to lock: poll pregame API directly regardless of reported phase.
    // external-sessions can return "Idle" during agent select, so we can't rely on phase == "pregame".
    // We skip only when already ingame (no point) or already locked.
    if phase != "ingame" {
        let (is_running, is_locked, mut puuid, config) = {
            let s = state.lock().await;
            (s.is_running, s.is_locked, s.puuid.clone(), s.config.clone())
        };

        // PUUID fallback
        if is_running && puuid.is_empty() {
            log(&app, "PUUID empty — fetching via entitlements...");
            match api::get_puuid(&lock).await {
                Ok(p) => {
                    log(&app, format!("PUUID recovered: {}...", &p[..p.len().min(8)]));
                    let mut s = state.lock().await;
                    s.puuid = p.clone();
                    puuid = p;
                }
                Err(e) => {
                    log(&app, format!("PUUID fetch failed: {e}"));
                }
            }
        }

        if !is_running {
            // Not armed — nothing to do
        } else if is_locked {
            // Already locked this game
        } else if puuid.is_empty() {
            log(&app, "Cannot lock: PUUID still empty");
        } else {
            // Try to get the pregame match ID — returns Err/404 when not in agent select
            match api::get_pregame_match_id(&lock, &puuid).await {
                Err(_) => {
                    // Not in agent select yet — silent, retry next poll
                }
                Ok(match_id) if match_id.is_empty() => {
                    // No match yet
                }
                Ok(match_id) => {
                    let map = api::get_pregame_map(&lock, &match_id).await.unwrap_or_default();
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
                        "Pregame match found. Map={} Agent={:.8} Mode={} Delay={}ms",
                        if map.is_empty() { "unknown" } else { &map },
                        &agent_id,
                        lock_mode,
                        config.delay_ms
                    ));

                    if config.delay_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(config.delay_ms)).await;
                    }

                    let cancelled = { state.lock().await.cancel_requested };

                    if cancelled {
                        log(&app, "Lock cancelled during delay");
                    } else {
                        log(&app, format!("Calling select on match {}...", &match_id[..match_id.len().min(8)]));
                        let select_result = api::select_agent(&lock, &match_id, &agent_id).await;
                        match &select_result {
                            Ok(_) => log(&app, "Select OK"),
                            Err(e) => log(&app, format!("Select ERR: {e}")),
                        }

                        let final_result = if lock_mode == "lock" {
                            log(&app, "Calling lock...");
                            let r = api::lock_agent(&lock, &match_id, &agent_id).await;
                            match &r {
                                Ok(_) => log(&app, "Lock OK"),
                                Err(e) => log(&app, format!("Lock ERR: {e}")),
                            }
                            r
                        } else {
                            select_result
                        };

                        match final_result {
                            Ok(_) => {
                                let mut s = state.lock().await;
                                s.is_locked = true;
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
    log(&app, format!("Armed — agent={} mode={} delay={}ms",
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
