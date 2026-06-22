import { invoke } from "@tauri-apps/api/core";
import { Shield, X, LogOut, Crosshair, Map } from "lucide-react";
import { useStore } from "../../store";

function Toggle({ on, onToggle, label }: { on: boolean; onToggle: () => void; label?: string }) {
  return (
    <div style={{ display:"flex", alignItems:"center", gap:8 }}>
      <button
        className={`toggle ${on ? "on" : ""}`}
        onClick={onToggle}
        role="switch"
        aria-checked={on}
        aria-label={label}
      />
      <span style={{ fontSize:"11px", fontWeight:600, color: on ? "var(--text)" : "var(--text3)" }}>
        {on ? "ON" : "OFF"}
      </span>
    </div>
  );
}

export function ControlPanel() {
  const {
    connected, config, isRunning, isLocked, phase, agents,
    patchConfig, setRunning, setLocked, addLog,
    setActiveView,
  } = useStore();

  const selectedAgent = agents.find(a => a.uuid === config.selectedAgentId);
  const canArm = connected && (!!config.selectedAgentId || config.macroEnabled);

  const saveConfig = async (patch: Partial<typeof config>) => {
    const newConfig = { ...config, ...patch };
    patchConfig(patch);
    try { await invoke("save_config_cmd", { config: newConfig }); } catch {}
  };

  const handleArm = async () => {
    if (!canArm) return;
    try {
      await invoke("start_instalocker");
      setRunning(true);
      setLocked(false);
      addLog(`Armed — waiting for agent select (${config.lockMode} mode, ${config.delayMs}ms delay)`);
    } catch (e) { addLog(`Error: ${e}`); }
  };

  const handleCancel = async () => {
    try {
      await invoke("cancel_instalocker");
      setRunning(false);
      setLocked(false);
      addLog("Instalocker disarmed");
    } catch (e) { addLog(`Error: ${e}`); }
  };

  const handleQuit = async () => {
    try {
      await invoke("quit_pregame");
      addLog("Pregame abandoned");
    } catch (e) { addLog(`Error: ${e}`); }
  };

  return (
    <div style={{
      width: 220,
      flexShrink: 0,
      borderLeft: "1px solid var(--border)",
      background: "var(--surface)",
      display: "flex",
      flexDirection: "column",
      overflow: "hidden",
    }}>

      {/* ── Agent Preview ────────────────────────────────── */}
      <div style={{
        position: "relative",
        height: 160,
        background: "var(--bg)",
        overflow: "hidden",
        flexShrink: 0,
      }}>
        {selectedAgent ? (
          <>
            {/* Bust portrait as blurred background */}
            {selectedAgent.bust_portrait && (
              <img
                src={selectedAgent.bust_portrait}
                alt=""
                style={{
                  position:"absolute", inset:0, width:"100%", height:"100%",
                  objectFit:"cover", objectPosition:"top",
                  filter:"blur(2px) brightness(0.35)",
                  transform:"scale(1.05)",
                }}
              />
            )}
            {/* Icon centered */}
            <div style={{
              position:"relative", height:"100%",
              display:"flex", flexDirection:"column",
              alignItems:"center", justifyContent:"center",
              gap:8,
            }}>
              <div style={{
                width:72, height:72,
                clipPath:"polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))",
                overflow:"hidden", flexShrink:0,
                boxShadow:"0 0 20px rgba(255,70,85,0.3)",
              }}>
                <img
                  src={selectedAgent.display_icon_small}
                  alt={selectedAgent.display_name}
                  style={{ width:"100%", height:"100%", objectFit:"cover" }}
                />
              </div>
              <div style={{ textAlign:"center" }}>
                <p style={{ fontSize:15, fontWeight:800, letterSpacing:"0.08em", textTransform:"uppercase", color:"var(--text)", lineHeight:1.2 }}>
                  {selectedAgent.display_name}
                </p>
                <p style={{ fontSize:9, fontWeight:700, letterSpacing:"0.14em", textTransform:"uppercase", color:"var(--text3)", marginTop:2 }}>
                  {selectedAgent.role}
                </p>
              </div>
            </div>
          </>
        ) : (
          <div style={{
            height:"100%", display:"flex", flexDirection:"column",
            alignItems:"center", justifyContent:"center", gap:8,
          }}>
            <div style={{
              width:52, height:52,
              border:"1px dashed var(--border2)",
              clipPath:"polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))",
              display:"flex", alignItems:"center", justifyContent:"center",
            }}>
              <Crosshair size={18} color="var(--border2)" />
            </div>
            <div style={{ textAlign:"center" }}>
              <p style={{ fontSize:11, fontWeight:700, color:"var(--text3)", textTransform:"uppercase", letterSpacing:"0.08em" }}>
                No Agent
              </p>
              <p style={{ fontSize:10, color:"var(--text3)", marginTop:2 }}>Select from the grid</p>
            </div>
          </div>
        )}
      </div>

      {/* ── Controls ─────────────────────────────────────── */}
      <div style={{ flex:1, overflowY:"auto", padding:"14px 14px 12px" }}>

        {/* Lock mode */}
        <div style={{ marginBottom:16 }}>
          <p className="section-label" style={{ marginBottom:7 }}>Lock Mode</p>
          <div className="seg-control">
            <button
              className={`seg-btn ${config.lockMode === "lock" ? "active" : ""}`}
              onClick={() => saveConfig({ lockMode: "lock" })}
            >
              Lock
            </button>
            <button
              className={`seg-btn ${config.lockMode === "select" ? "active" : ""}`}
              onClick={() => saveConfig({ lockMode: "select" })}
            >
              Select
            </button>
          </div>
          <p style={{ fontSize:10, color:"var(--text3)", marginTop:5, lineHeight:1.5 }}>
            {config.lockMode === "lock"
              ? "Locks the agent instantly."
              : "Hovers only — Valorant auto-locks at timer end."}
          </p>
        </div>

        {/* Delay */}
        <div style={{ marginBottom:16 }}>
          <div style={{ display:"flex", justifyContent:"space-between", alignItems:"baseline", marginBottom:7 }}>
            <p className="section-label">Delay</p>
            <span style={{ fontSize:11, fontWeight:700, color:"var(--red)" }}>{config.delayMs}ms</span>
          </div>
          <input
            type="range" min={0} max={2000} step={50}
            value={config.delayMs}
            onChange={e => saveConfig({ delayMs: Number(e.target.value) })}
          />
          <div style={{ display:"flex", justifyContent:"space-between", marginTop:4 }}>
            <span style={{ fontSize:9, color:"var(--text3)", textTransform:"uppercase", letterSpacing:"0.08em" }}>Instant</span>
            <span style={{ fontSize:9, color:"var(--text3)", textTransform:"uppercase", letterSpacing:"0.08em" }}>2s</span>
          </div>
        </div>

        {/* Map macros toggle */}
        <div style={{ marginBottom:16 }}>
          <div style={{ display:"flex", alignItems:"center", justifyContent:"space-between", marginBottom:6 }}>
            <div>
              <p className="section-label">Map Macros</p>
              <p style={{ fontSize:10, color:"var(--text3)", marginTop:2 }}>Per-map agent</p>
            </div>
            <Toggle
              on={config.macroEnabled}
              onToggle={() => saveConfig({ macroEnabled: !config.macroEnabled })}
              label="Map macros"
            />
          </div>
          {config.macroEnabled && (
            <button
              onClick={() => setActiveView("macros")}
              style={{
                display:"flex", alignItems:"center", gap:5,
                fontSize:10, fontWeight:600, color:"var(--red)",
                background:"var(--red-dim)", border:"1px solid var(--red-border)",
                padding:"4px 10px", cursor:"pointer",
                clipPath:"polygon(0 0, calc(100% - 4px) 0, 100% 4px, 100% 100%, 4px 100%, 0 calc(100% - 4px))",
                width:"100%", marginTop:4,
              }}
            >
              <Map size={10} />
              Configure maps →
            </button>
          )}
        </div>

        {/* Divider */}
        <div style={{ height:1, background:"var(--border)", margin:"4px 0 14px" }} />

        {/* ARM / DISARM / LOCKED */}
        {!isRunning && (
          <button
            onClick={handleArm}
            disabled={!canArm}
            className="btn-primary"
          >
            <Shield size={13} />
            Arm Instalocker
          </button>
        )}

        {isRunning && !isLocked && (
          <div style={{ display:"flex", flexDirection:"column", gap:8 }}>
            <div style={{
              padding:"10px 14px",
              background:"var(--red-dim)",
              border:"1px solid var(--red-border)",
              clipPath:"polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))",
              textAlign:"center",
            }}>
              <p style={{ fontSize:10, fontWeight:700, color:"var(--red)", letterSpacing:"0.12em", textTransform:"uppercase" }}>
                Armed
              </p>
              <p style={{ fontSize:10, fontWeight:600, color:"var(--red)", letterSpacing:"0.06em", opacity:0.7, marginTop:2 }}>
                Waiting for agent select...
              </p>
            </div>
            <button onClick={handleCancel} className="btn-ghost" style={{ width:"100%" }}>
              <X size={12} />
              Disarm
            </button>
          </div>
        )}

        {isRunning && isLocked && (
          <div style={{ display:"flex", flexDirection:"column", gap:8 }}>
            <div
              className="anim-locked-in"
              style={{
                padding:"10px 14px",
                background:"rgba(74,222,128,0.08)",
                border:"1px solid rgba(74,222,128,0.3)",
                clipPath:"polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))",
                textAlign:"center",
              }}
            >
              <p style={{ fontSize:11, fontWeight:700, color:"var(--green)", letterSpacing:"0.12em", textTransform:"uppercase" }}>
                ✓ Agent Locked
              </p>
              <p style={{ fontSize:9, fontWeight:600, color:"var(--green)", opacity:0.6, marginTop:2, letterSpacing:"0.06em" }}>
                Armed for next game
              </p>
            </div>
            <button onClick={handleCancel} className="btn-ghost" style={{ width:"100%" }}>
              <X size={12} />
              Disarm
            </button>
          </div>
        )}

        {/* Quit pregame */}
        {(phase === "pregame" || isLocked) && (
          <button
            onClick={handleQuit}
            className="btn-danger"
            style={{ width:"100%", marginTop:8 }}
            title="Abandon this match — standard Valorant penalties apply"
          >
            <LogOut size={12} />
            Abandon Match
          </button>
        )}
      </div>

      {/* ── Bottom info ───────────────────────────────────── */}
      {!connected && (
        <div style={{
          padding:"8px 14px",
          borderTop:"1px solid var(--border)",
          background:"var(--red-dim)",
          flexShrink:0,
        }}>
          <p style={{ fontSize:10, color:"var(--red)", fontWeight:600, textAlign:"center" }}>
            Open Valorant to connect
          </p>
        </div>
      )}
    </div>
  );
}
