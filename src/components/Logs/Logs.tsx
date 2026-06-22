import { useStore } from "../../store";

export function Logs() {
  const { logs } = useStore();

  return (
    <div className="anim-slide-up" style={{ flex:1, display:"flex", flexDirection:"column", overflow:"hidden", padding:"16px 20px" }}>
      <div style={{ display:"flex", alignItems:"center", justifyContent:"space-between", marginBottom:12 }}>
        <p className="section-label">Activity Log</p>
        <span style={{ fontSize:10, color:"var(--text3)" }}>{logs.length} entries</span>
      </div>

      <div style={{
        flex:1, overflowY:"auto",
        background:"var(--surface)", border:"1px solid var(--border)",
        padding:"10px 12px",
        clipPath:"polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 8px 100%, 0 calc(100% - 8px))",
        fontFamily:"'JetBrains Mono', 'Fira Code', ui-monospace, monospace",
        fontSize:11,
      }}>
        {logs.length === 0 ? (
          <p style={{ color:"var(--text3)", padding:"20px 0", textAlign:"center" }}>
            No activity yet — arm the instalocker to begin.
          </p>
        ) : (
          logs.map((log, i) => (
            <div key={i} style={{
              display:"flex", gap:8, padding:"2px 0",
              borderBottom: i < logs.length - 1 ? "1px solid var(--border)" : "none",
            }}>
              <span style={{ color:"var(--red)", opacity:0.5, flexShrink:0 }}>›</span>
              <span style={{ color:"var(--text2)", lineHeight:1.6, flex:1 }}>{log}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
