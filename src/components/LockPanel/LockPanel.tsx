import { invoke } from "@tauri-apps/api/core";
import { Crosshair, X, LogOut, Shield, Lock, MousePointer } from "lucide-react";
import clsx from "clsx";
import { useStore } from "../../store";

export function LockPanel() {
  const {
    connected,
    config,
    isRunning,
    isLocked,
    phase,
    agents,
    patchConfig,
    setRunning,
    setLocked,
    addLog,
  } = useStore();

  const selectedAgent = agents.find((a) => a.uuid === config.selectedAgentId);

  const handleStart = async () => {
    if (!config.selectedAgentId && !config.macroEnabled) {
      addLog("Select an agent first");
      return;
    }
    try {
      await invoke("start_instalocker");
      setRunning(true);
      setLocked(false);
      addLog(`Instalocker armed — waiting for agent select... (${config.lockMode} mode)`);
    } catch (e) {
      addLog(`Error: ${e}`);
    }
  };

  const handleCancel = async () => {
    try {
      await invoke("cancel_instalocker");
      setRunning(false);
      setLocked(false);
      addLog("Instalocker cancelled");
    } catch (e) {
      addLog(`Error: ${e}`);
    }
  };

  const handleQuit = async () => {
    try {
      await invoke("quit_pregame");
      addLog("Match abandoned");
    } catch (e) {
      addLog(`Error: ${e}`);
    }
  };

  const toggleLockMode = async () => {
    const newMode = config.lockMode === "lock" ? "select" : "lock";
    patchConfig({ lockMode: newMode });
    try {
      await invoke("save_config_cmd", { config: { ...config, lockMode: newMode } });
      addLog(`Mode changed to: ${newMode}`);
    } catch {}
  };

  return (
    <div className="flex flex-col gap-3 p-4 border-t border-[#2a2a2a] bg-[#111111]">
      {/* Selected agent preview */}
      <div className="flex items-center gap-3">
        {selectedAgent ? (
          <>
            <div className="relative w-12 h-12 overflow-hidden bg-[#1a1a1a] flex-shrink-0"
              style={{ clipPath: "polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))" }}>
              <img
                src={selectedAgent.display_icon_small}
                alt={selectedAgent.display_name}
                className="w-full h-full object-cover"
              />
            </div>
            <div>
              <p className="text-xs text-[#7b7b7b] uppercase tracking-widest font-bold">Selected Agent</p>
              <p className="text-base font-bold text-[#ece8e1] uppercase tracking-wide leading-tight">
                {selectedAgent.display_name}
              </p>
            </div>
          </>
        ) : (
          <div className="flex items-center gap-2 text-[#7b7b7b]">
            <div className="w-12 h-12 border border-dashed border-[#2a2a2a] flex items-center justify-center"
              style={{ clipPath: "polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))" }}>
              <Crosshair size={16} className="text-[#2a2a2a]" />
            </div>
            <div>
              <p className="text-xs uppercase tracking-widest font-bold">No Agent Selected</p>
              <p className="text-xs text-[#555]">Click an agent above</p>
            </div>
          </div>
        )}

        {/* Lock mode toggle */}
        <button
          onClick={toggleLockMode}
          className={clsx(
            "ml-auto flex items-center gap-2 px-3 py-2 text-xs font-bold uppercase tracking-widest transition-all duration-150",
            "border",
            config.lockMode === "lock"
              ? "border-[#FF4655]/60 text-[#FF4655] bg-[#FF4655]/10"
              : "border-[#818cf8]/60 text-[#818cf8] bg-[#818cf8]/10"
          )}
          style={{ clipPath: "polygon(0 0, calc(100% - 6px) 0, 100% 6px, 100% 100%, 6px 100%, 0 calc(100% - 6px))" }}
          title="Toggle between Lock and Select-Only mode"
        >
          {config.lockMode === "lock" ? (
            <><Lock size={11} /> Lock</>
          ) : (
            <><MousePointer size={11} /> Select</>
          )}
        </button>
      </div>

      {/* Action buttons */}
      <div className="flex gap-2">
        {!isRunning && !isLocked && (
          <button
            onClick={handleStart}
            disabled={!connected || (!config.selectedAgentId && !config.macroEnabled)}
            className={clsx(
              "val-btn-primary flex-1 flex items-center justify-center gap-2",
              (!connected || (!config.selectedAgentId && !config.macroEnabled)) && "opacity-40 cursor-not-allowed"
            )}
          >
            <Shield size={14} />
            Arm Instalocker
          </button>
        )}

        {isRunning && !isLocked && (
          <button onClick={handleCancel} className="val-btn-ghost flex-1 flex items-center justify-center gap-2">
            <X size={14} />
            Cancel
          </button>
        )}

        {isLocked && (
          <div className="flex-1 flex items-center justify-center gap-2 text-[#4ade80] font-bold text-sm uppercase tracking-widest border border-[#4ade80]/40 py-2.5 animate-lock-flash"
            style={{ clipPath: "polygon(0 0, calc(100% - 10px) 0, 100% 10px, 100% 100%, 10px 100%, 0 calc(100% - 10px))" }}>
            <Crosshair size={14} />
            Agent Locked!
          </div>
        )}

        {/* Quit pregame — visible when in pregame */}
        {(phase === "pregame" || isLocked) && (
          <button
            onClick={handleQuit}
            className="val-btn-danger flex items-center gap-2 px-4"
            title="Abandon this match (standard penalties apply)"
          >
            <LogOut size={14} />
            Quit
          </button>
        )}
      </div>
    </div>
  );
}
