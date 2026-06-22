import { invoke } from "@tauri-apps/api/core";
import { Save } from "lucide-react";
import { useState } from "react";
import { useStore } from "../../store";

export function Settings() {
  const { config, patchConfig, addLog } = useStore();
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    try {
      await invoke("save_config_cmd", { config });
      addLog("Settings saved");
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      addLog(`Failed to save settings: ${e}`);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6 animate-slide-up">
      <div>
        <h2 className="text-xs font-bold uppercase tracking-widest text-[#7b7b7b] mb-4">Settings</h2>
      </div>

      {/* Lock mode */}
      <div className="panel p-4 space-y-3">
        <p className="text-xs font-bold uppercase tracking-widest text-[#7b7b7b]">Lock Mode</p>
        <div className="flex gap-2">
          {(["lock", "select"] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => patchConfig({ lockMode: mode })}
              className={`flex-1 py-2.5 text-xs font-bold uppercase tracking-widest transition-all duration-150 border ${
                config.lockMode === mode
                  ? "bg-[#FF4655] border-[#FF4655] text-white"
                  : "bg-transparent border-[#2a2a2a] text-[#7b7b7b] hover:border-[#FF4655]/40 hover:text-[#ece8e1]"
              }`}
              style={{ clipPath: "polygon(0 0, calc(100% - 6px) 0, 100% 6px, 100% 100%, 6px 100%, 0 calc(100% - 6px))" }}
            >
              {mode === "lock" ? "Lock (instant)" : "Select Only"}
            </button>
          ))}
        </div>
        <p className="text-[11px] text-[#7b7b7b] leading-relaxed">
          <span className="text-[#ece8e1] font-semibold">Lock</span> — immediately locks your agent. <br />
          <span className="text-[#ece8e1] font-semibold">Select Only</span> — hovers the agent, Valorant auto-locks when time runs out (safer for ranked).
        </p>
      </div>

      {/* Delay */}
      <div className="panel p-4 space-y-3">
        <div className="flex items-center justify-between">
          <p className="text-xs font-bold uppercase tracking-widest text-[#7b7b7b]">Lock Delay</p>
          <span className="text-sm font-bold text-[#FF4655]">{config.delayMs}ms</span>
        </div>
        <input
          type="range"
          min={0}
          max={2000}
          step={50}
          value={config.delayMs}
          onChange={(e) => patchConfig({ delayMs: Number(e.target.value) })}
          className="w-full accent-[#FF4655] cursor-pointer"
        />
        <div className="flex justify-between text-[10px] text-[#555] uppercase tracking-widest">
          <span>Instant</span>
          <span>Human-like</span>
          <span>2000ms</span>
        </div>
        <p className="text-[11px] text-[#7b7b7b]">
          Small delay makes the lock look less like a bot. 100–300ms recommended.
        </p>
      </div>

      {/* Macro mode toggle */}
      <div className="panel p-4 space-y-3">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs font-bold uppercase tracking-widest text-[#7b7b7b]">Map Macros</p>
            <p className="text-[11px] text-[#555] mt-0.5">Pick different agents per map automatically</p>
          </div>
          <button
            onClick={() => patchConfig({ macroEnabled: !config.macroEnabled })}
            className={`relative w-10 h-5 rounded-full transition-colors duration-200 ${
              config.macroEnabled ? "bg-[#FF4655]" : "bg-[#2a2a2a]"
            }`}
          >
            <div className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform duration-200 ${
              config.macroEnabled ? "translate-x-5" : "translate-x-0"
            }`} />
          </button>
        </div>
        {config.macroEnabled && (
          <p className="text-[11px] text-[#facc15]">
            Map macros active — go to the Macros tab to configure per-map agents.
          </p>
        )}
      </div>

      {/* Save button */}
      <button onClick={handleSave} className="val-btn-primary w-full flex items-center justify-center gap-2">
        <Save size={14} />
        {saved ? "Saved!" : "Save Settings"}
      </button>
    </div>
  );
}
