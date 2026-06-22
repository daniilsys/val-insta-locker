import { useStore, Agent } from "../../store";
import { invoke } from "@tauri-apps/api/core";

const ROLES = ["All", "Duelist", "Initiator", "Controller", "Sentinel"];

const ROLE_COLOR: Record<string, string> = {
  Duelist:    "#FF4655",
  Initiator:  "#facc15",
  Controller: "#818cf8",
  Sentinel:   "#4ade80",
};

export function AgentGrid() {
  const { agents, agentsLoaded, config, roleFilter, setRoleFilter, patchConfig, addLog } = useStore();

  const filtered = roleFilter === "All" ? agents : agents.filter(a => a.role === roleFilter);

  const handleSelect = async (agent: Agent) => {
    const newConfig = { ...config, selectedAgentId: agent.uuid, selectedAgentName: agent.display_name };
    patchConfig({ selectedAgentId: agent.uuid, selectedAgentName: agent.display_name });
    addLog(`Agent selected: ${agent.display_name}`);
    try { await invoke("save_config_cmd", { config: newConfig }); } catch {}
  };

  if (!agentsLoaded) {
    return (
      <div style={{ flex:1, display:"flex", flexDirection:"column", alignItems:"center", justifyContent:"center", gap:10, color:"var(--text3)" }}>
        <div style={{
          width:24, height:24,
          border:"2px solid var(--red)", borderTopColor:"transparent",
          borderRadius:"50%", animation:"spin 0.7s linear infinite",
        }} />
        <span className="section-label">Loading agents...</span>
      </div>
    );
  }

  return (
    <div style={{ flex:1, display:"flex", flexDirection:"column", overflow:"hidden" }}>
      {/* Role tabs */}
      <div style={{
        display:"flex", gap:2, padding:"8px 10px",
        borderBottom:"1px solid var(--border)",
        flexShrink:0,
      }}>
        {ROLES.map(role => (
          <button
            key={role}
            onClick={() => setRoleFilter(role)}
            style={{
              padding: "4px 10px",
              fontSize: "10px", fontWeight:700, letterSpacing:"0.1em", textTransform:"uppercase",
              background: roleFilter === role ? "var(--red)" : "transparent",
              color: roleFilter === role ? "#fff" : "var(--text3)",
              border: `1px solid ${roleFilter === role ? "var(--red)" : role !== "All" ? (ROLE_COLOR[role]+"33") : "var(--border)"}`,
              cursor:"pointer", transition:"all 150ms",
              clipPath:"polygon(0 0, calc(100% - 4px) 0, 100% 4px, 100% 100%, 4px 100%, 0 calc(100% - 4px))",
            }}
          >
            {role === "All" ? "All" : role}
          </button>
        ))}
        <span style={{ marginLeft:"auto", fontSize:"10px", color:"var(--text3)", alignSelf:"center" }}>
          {filtered.length} agents
        </span>
      </div>

      {/* Grid */}
      <div style={{ flex:1, overflowY:"auto", padding:"8px 10px" }}>
        <div style={{
          display:"grid",
          gridTemplateColumns:"repeat(auto-fill, minmax(68px, 1fr))",
          gap:5,
        }}>
          {filtered.map(agent => {
            const isSelected = config.selectedAgentId === agent.uuid;
            return (
              <button
                key={agent.uuid}
                onClick={() => handleSelect(agent)}
                className={`agent-card ${isSelected ? "selected" : ""}`}
                title={`${agent.display_name} · ${agent.role}`}
              >
                {/* Role accent */}
                <div style={{
                  position:"absolute", top:0, left:0, right:0, height:2,
                  background: ROLE_COLOR[agent.role] || "var(--border)",
                  opacity: 0.7,
                }} />

                {/* Image */}
                <div style={{ aspectRatio:"1", background:"#111", overflow:"hidden" }}>
                  <img
                    src={agent.display_icon_small}
                    alt={agent.display_name}
                    style={{ width:"100%", height:"100%", objectFit:"cover", display:"block",
                      transition:"transform 200ms",
                    }}
                    loading="lazy"
                  />
                </div>

                {/* Name */}
                <div style={{ padding:"3px 4px 4px", textAlign:"center" }}>
                  <span style={{
                    fontSize:"8px", fontWeight:700, letterSpacing:"0.06em",
                    textTransform:"uppercase",
                    color: isSelected ? "var(--red)" : "var(--text2)",
                    display:"block", overflow:"hidden", textOverflow:"ellipsis", whiteSpace:"nowrap",
                  }}>
                    {agent.display_name}
                  </span>
                </div>

                {/* Selected checkmark */}
                {isSelected && (
                  <div style={{
                    position:"absolute", top:4, right:4,
                    width:12, height:12, borderRadius:"50%",
                    background:"var(--red)",
                    display:"flex", alignItems:"center", justifyContent:"center",
                  }}>
                    <svg width="7" height="7" viewBox="0 0 7 7" fill="none">
                      <path d="M1 3.5L2.8 5.5L6 1.5" stroke="#fff" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/>
                    </svg>
                  </div>
                )}
              </button>
            );
          })}
        </div>

        {filtered.length === 0 && (
          <div style={{ display:"flex", justifyContent:"center", padding:"40px 0", color:"var(--text3)" }}>
            <span className="section-label">No agents found</span>
          </div>
        )}
      </div>
    </div>
  );
}
