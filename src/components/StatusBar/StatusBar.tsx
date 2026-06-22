import { useStore, GamePhase } from "../../store";
import { Wifi, WifiOff, Crosshair, Shield } from "lucide-react";

const PHASE_LABELS: Record<GamePhase, string> = {
  menus: "In Menus",
  pregame: "Agent Select",
  ingame: "In Game",
  unknown: "Waiting...",
};

const PHASE_COLORS: Record<GamePhase, string> = {
  menus: "#7b7b7b",
  pregame: "#FF4655",
  ingame: "#4ade80",
  unknown: "#7b7b7b",
};

export function StatusBar() {
  const { connected, username, tagLine, phase, currentMap, isRunning, isLocked } = useStore();

  return (
    <div className="flex items-center justify-between px-4 py-2 border-b border-[#2a2a2a] bg-[#111111]">
      {/* Left: connection status */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5">
          {connected ? (
            <Wifi size={14} className="text-[#4ade80]" />
          ) : (
            <WifiOff size={14} className="text-[#FF4655]" />
          )}
          <span className="text-xs font-bold uppercase tracking-widest" style={{ color: connected ? "#4ade80" : "#FF4655" }}>
            {connected ? "Connected" : "Disconnected"}
          </span>
        </div>

        {connected && username && (
          <>
            <div className="w-px h-3 bg-[#2a2a2a]" />
            <span className="text-xs text-[#ece8e1] font-semibold">
              {username}
              {tagLine && <span className="text-[#7b7b7b]">#{tagLine}</span>}
            </span>
          </>
        )}
      </div>

      {/* Center: phase indicator */}
      <div className="flex items-center gap-2">
        <div
          className="status-dot animate-pulse"
          style={{ backgroundColor: PHASE_COLORS[phase] }}
        />
        <span
          className="text-xs font-bold uppercase tracking-widest"
          style={{ color: PHASE_COLORS[phase] }}
        >
          {PHASE_LABELS[phase]}
        </span>
        {currentMap && phase === "pregame" && (
          <span className="text-xs text-[#7b7b7b] ml-1">— {currentMap}</span>
        )}
      </div>

      {/* Right: instalocker status */}
      <div className="flex items-center gap-2">
        {isLocked && (
          <div className="tag bg-[#FF4655]/20 text-[#FF4655] animate-lock-flash">
            <Crosshair size={10} />
            Locked
          </div>
        )}
        {isRunning && !isLocked && (
          <div className="tag bg-[#facc15]/10 text-[#facc15] animate-pulse-red">
            <Shield size={10} />
            Armed
          </div>
        )}
      </div>
    </div>
  );
}
