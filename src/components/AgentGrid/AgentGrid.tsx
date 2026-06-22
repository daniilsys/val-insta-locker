import { useStore, Agent } from "../../store";
import { invoke } from "@tauri-apps/api/core";
import clsx from "clsx";

const ROLES = ["All", "Duelist", "Initiator", "Controller", "Sentinel"];

const ROLE_COLORS: Record<string, string> = {
  Duelist: "#FF4655",
  Initiator: "#facc15",
  Controller: "#818cf8",
  Sentinel: "#4ade80",
};

export function AgentGrid() {
  const {
    agents,
    agentsLoaded,
    config,
    roleFilter,
    setRoleFilter,
    patchConfig,
    addLog,
  } = useStore();

  const filtered = agents.filter(
    (a) => roleFilter === "All" || a.role === roleFilter
  );

  const handleSelect = async (agent: Agent) => {
    const newConfig = {
      ...config,
      selectedAgentId: agent.uuid,
      selectedAgentName: agent.display_name,
    };
    patchConfig({ selectedAgentId: agent.uuid, selectedAgentName: agent.display_name });
    addLog(`Agent selected: ${agent.display_name}`);
    try {
      await invoke("save_config_cmd", { config: newConfig });
    } catch (e) {
      addLog(`Failed to save config: ${e}`);
    }
  };

  if (!agentsLoaded) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-3 text-[#7b7b7b]">
        <div className="w-8 h-8 border-2 border-[#FF4655] border-t-transparent rounded-full animate-spin" />
        <span className="text-xs uppercase tracking-widest">Loading agents...</span>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Role filter */}
      <div className="flex gap-1.5 px-4 py-3 border-b border-[#2a2a2a]">
        {ROLES.map((role) => (
          <button
            key={role}
            onClick={() => setRoleFilter(role)}
            className={clsx(
              "px-3 py-1 text-xs font-bold uppercase tracking-widest transition-all duration-150",
              "clip-path-[polygon(0_0,calc(100%-6px)_0,100%_6px,100%_100%,6px_100%,0_calc(100%-6px))]",
              roleFilter === role
                ? "bg-[#FF4655] text-white"
                : "bg-[#1a1a1a] border border-[#2a2a2a] text-[#7b7b7b] hover:text-[#ece8e1] hover:border-[#FF4655]/40"
            )}
            style={
              roleFilter === role
                ? {}
                : role !== "All"
                ? { borderColor: ROLE_COLORS[role] + "33" }
                : {}
            }
          >
            {role === "All" ? "All Agents" : role}
          </button>
        ))}
      </div>

      {/* Grid */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="grid grid-cols-6 gap-2">
          {filtered.map((agent) => {
            const isSelected = config.selectedAgentId === agent.uuid;
            return (
              <button
                key={agent.uuid}
                onClick={() => handleSelect(agent)}
                className={clsx("agent-card bg-[#1a1a1a] group", isSelected && "selected")}
                title={agent.display_name}
              >
                {/* Role accent line */}
                <div
                  className="absolute top-0 left-0 right-0 h-0.5 opacity-60"
                  style={{ backgroundColor: ROLE_COLORS[agent.role] || "#7b7b7b" }}
                />

                {/* Agent image */}
                <div className="relative aspect-square bg-[#111] overflow-hidden">
                  <img
                    src={agent.display_icon_small}
                    alt={agent.display_name}
                    className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
                    loading="lazy"
                  />
                  {isSelected && (
                    <div className="absolute inset-0 bg-[#FF4655]/10" />
                  )}
                </div>

                {/* Name */}
                <div className="px-1.5 py-1 text-center">
                  <p className={clsx(
                    "text-[9px] font-bold uppercase tracking-widest truncate leading-tight",
                    isSelected ? "text-[#FF4655]" : "text-[#ece8e1]"
                  )}>
                    {agent.display_name}
                  </p>
                </div>
              </button>
            );
          })}
        </div>

        {filtered.length === 0 && (
          <div className="flex items-center justify-center h-32 text-[#7b7b7b] text-xs uppercase tracking-widest">
            No agents found
          </div>
        )}
      </div>
    </div>
  );
}
