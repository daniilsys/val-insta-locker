import { useStore, GamePhase } from "../../store";
import { Wifi, WifiOff } from "lucide-react";

const PHASE_META: Record<GamePhase, { label: string; color: string }> = {
  menus:   { label: "In Menus",     color: "var(--text3)" },
  pregame: { label: "Agent Select", color: "var(--red)" },
  ingame:  { label: "In Game",      color: "var(--green)" },
  unknown: { label: "Waiting...",   color: "var(--text3)" },
};

export function StatusBar() {
  const { connected, username, tagLine, phase, currentMap, isRunning, isLocked } = useStore();
  const meta = PHASE_META[phase];

  return (
    <div style={{
      display: "flex", alignItems: "center", justifyContent: "space-between",
      padding: "0 16px", height: "32px",
      borderBottom: "1px solid var(--border)",
      background: "var(--surface)",
      flexShrink: 0,
    }}>
      {/* Left: connection */}
      <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "5px" }}>
          {connected
            ? <Wifi size={11} color="var(--green)" />
            : <WifiOff size={11} color="var(--red)" />}
          <span style={{
            fontSize: "10px", fontWeight: 700, letterSpacing: "0.12em",
            textTransform: "uppercase",
            color: connected ? "var(--green)" : "var(--red)",
          }}>
            {connected ? "Connected" : "Disconnected"}
          </span>
        </div>

        {connected && username && (
          <>
            <div style={{ width: 1, height: 10, background: "var(--border)" }} />
            <span style={{ fontSize: "11px", fontWeight: 600, color: "var(--text)" }}>
              {username}
              <span style={{ color: "var(--text3)" }}>#{tagLine}</span>
            </span>
          </>
        )}
      </div>

      {/* Center: phase */}
      <div style={{ display: "flex", alignItems: "center", gap: "7px" }}>
        <div style={{
          width: 6, height: 6, borderRadius: "50%",
          background: meta.color,
          boxShadow: phase !== "unknown" && phase !== "menus" ? `0 0 6px ${meta.color}` : "none",
        }} />
        <span style={{
          fontSize: "10px", fontWeight: 700, letterSpacing: "0.12em",
          textTransform: "uppercase", color: meta.color,
        }}>
          {meta.label}
        </span>
        {currentMap && phase === "pregame" && (
          <span style={{ fontSize: "10px", color: "var(--text3)" }}>— {currentMap}</span>
        )}
      </div>

      {/* Right: instalocker badge */}
      <div style={{ display: "flex", alignItems: "center", gap: "6px", width: "120px", justifyContent: "flex-end" }}>
        {isLocked && (
          <div className="anim-locked-in" style={{
            display: "flex", alignItems: "center", gap: "4px",
            padding: "2px 8px",
            background: "rgba(74,222,128,0.1)",
            border: "1px solid rgba(74,222,128,0.3)",
            clipPath: "polygon(0 0, calc(100% - 5px) 0, 100% 5px, 100% 100%, 0 100%)",
          }}>
            <div style={{ width:5, height:5, borderRadius:"50%", background:"var(--green)" }} />
            <span style={{ fontSize:"9px", fontWeight:700, letterSpacing:"0.12em", textTransform:"uppercase", color:"var(--green)" }}>
              LOCKED
            </span>
          </div>
        )}
        {isRunning && !isLocked && (
          <div style={{
            display: "flex", alignItems: "center", gap: "4px",
            padding: "2px 8px",
            background: "var(--red-dim)",
            border: "1px solid var(--red-border)",
            clipPath: "polygon(0 0, calc(100% - 5px) 0, 100% 5px, 100% 100%, 0 100%)",
          }}>
            <div style={{ width:5, height:5, borderRadius:"50%", background:"var(--red)", animation:"arm-pulse 1.5s infinite" }} />
            <span style={{ fontSize:"9px", fontWeight:700, letterSpacing:"0.12em", textTransform:"uppercase", color:"var(--red)" }}>
              ARMED
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
