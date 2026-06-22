import { invoke } from "@tauri-apps/api/core";
import { Map, Trash2 } from "lucide-react";
import { useState } from "react";
import { useStore, MapMacro } from "../../store";

const MAPS = [
  "Ascent", "Bind", "Haven", "Split", "Fracture",
  "Icebox", "Breeze", "Pearl", "Lotus", "Sunset", "Abyss",
];

export function MacroPanel() {
  const { config, agents, patchConfig, addLog } = useStore();
  const [selectedMap, setSelectedMap] = useState<string | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState("");
  const [lockMode, setLockMode] = useState<"lock" | "select">("lock");

  const handleSaveMacro = async () => {
    if (!selectedMap || !selectedAgentId) return;
    const agent = agents.find((a) => a.uuid === selectedAgentId);
    if (!agent) return;

    const macro: MapMacro = {
      agentId: selectedAgentId,
      agentName: agent.display_name,
      lockMode,
    };

    const newMacros = { ...config.mapMacros, [selectedMap]: macro };
    patchConfig({ mapMacros: newMacros });

    try {
      await invoke("set_map_macro", {
        map: selectedMap,
        macroData: {
          agent_id: selectedAgentId,
          agent_name: agent.display_name,
          lock_mode: lockMode,
        },
      });
      addLog(`Macro set: ${selectedMap} → ${agent.display_name} (${lockMode})`);
      setSelectedMap(null);
    } catch (e) {
      addLog(`Failed to save macro: ${e}`);
    }
  };

  const handleDeleteMacro = async (map: string) => {
    const newMacros = { ...config.mapMacros };
    delete newMacros[map];
    patchConfig({ mapMacros: newMacros });
    const newConfig = { ...config, mapMacros: newMacros };
    try {
      await invoke("save_config_cmd", { config: newConfig });
      addLog(`Macro removed for ${map}`);
    } catch {}
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-4 animate-slide-up">
      <div>
        <h2 className="text-xs font-bold uppercase tracking-widest text-[#7b7b7b] mb-1">Map Macros</h2>
        <p className="text-[11px] text-[#555]">Assign a specific agent + mode for each map.</p>
      </div>

      {!config.macroEnabled && (
        <div className="panel p-3 border-[#facc15]/30 bg-[#facc15]/5">
          <p className="text-xs text-[#facc15] font-semibold">
            Map Macros are disabled — enable them in Settings.
          </p>
        </div>
      )}

      {/* Existing macros */}
      {Object.entries(config.mapMacros).length > 0 && (
        <div className="space-y-2">
          <p className="text-[10px] font-bold uppercase tracking-widest text-[#555]">Active Macros</p>
          {Object.entries(config.mapMacros).map(([map, macro]) => (
            <div key={map} className="panel p-3 flex items-center gap-3">
              <Map size={14} className="text-[#FF4655] flex-shrink-0" />
              <div className="flex-1 min-w-0">
                <span className="text-xs font-bold text-[#ece8e1] uppercase tracking-wide">{map}</span>
                <span className="text-[#7b7b7b] text-xs"> → </span>
                <span className="text-xs font-bold text-[#FF4655]">{macro.agentName}</span>
                <span className="ml-1 text-[10px] text-[#555] uppercase tracking-widest">({macro.lockMode})</span>
              </div>
              <button
                onClick={() => handleDeleteMacro(map)}
                className="text-[#555] hover:text-[#FF4655] transition-colors"
              >
                <Trash2 size={13} />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Add new macro */}
      <div className="panel p-4 space-y-3">
        <p className="text-[10px] font-bold uppercase tracking-widest text-[#555]">Add / Edit Macro</p>

        {/* Map selector */}
        <div>
          <p className="text-[10px] text-[#7b7b7b] uppercase tracking-widest mb-1.5">Map</p>
          <div className="grid grid-cols-4 gap-1.5">
            {MAPS.map((map) => (
              <button
                key={map}
                onClick={() => setSelectedMap(map)}
                className={`py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all border ${
                  selectedMap === map
                    ? "bg-[#FF4655] border-[#FF4655] text-white"
                    : "bg-transparent border-[#2a2a2a] text-[#7b7b7b] hover:border-[#FF4655]/40 hover:text-[#ece8e1]"
                }`}
                style={{ clipPath: "polygon(0 0, calc(100% - 5px) 0, 100% 5px, 100% 100%, 5px 100%, 0 calc(100% - 5px))" }}
              >
                {map}
              </button>
            ))}
          </div>
        </div>

        {/* Agent selector */}
        <div>
          <p className="text-[10px] text-[#7b7b7b] uppercase tracking-widest mb-1.5">Agent</p>
          <select
            value={selectedAgentId}
            onChange={(e) => setSelectedAgentId(e.target.value)}
            className="w-full bg-[#111] border border-[#2a2a2a] text-[#ece8e1] text-xs py-2 px-3 outline-none focus:border-[#FF4655]/60"
          >
            <option value="">— Select Agent —</option>
            {agents.map((a) => (
              <option key={a.uuid} value={a.uuid}>
                {a.display_name} ({a.role})
              </option>
            ))}
          </select>
        </div>

        {/* Mode */}
        <div className="flex gap-2">
          {(["lock", "select"] as const).map((m) => (
            <button
              key={m}
              onClick={() => setLockMode(m)}
              className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-widest border transition-all ${
                lockMode === m
                  ? "bg-[#FF4655] border-[#FF4655] text-white"
                  : "bg-transparent border-[#2a2a2a] text-[#7b7b7b] hover:border-[#FF4655]/40"
              }`}
              style={{ clipPath: "polygon(0 0, calc(100% - 5px) 0, 100% 5px, 100% 100%, 5px 100%, 0 calc(100% - 5px))" }}
            >
              {m}
            </button>
          ))}
        </div>

        <button
          onClick={handleSaveMacro}
          disabled={!selectedMap || !selectedAgentId}
          className="val-btn-primary w-full disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Save Macro
        </button>
      </div>
    </div>
  );
}
