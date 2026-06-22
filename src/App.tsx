import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore, GamePhase } from "./store";
import { Sidebar }      from "./components/Sidebar/Sidebar";
import { StatusBar }    from "./components/StatusBar/StatusBar";
import { AgentGrid }    from "./components/AgentGrid/AgentGrid";
import { ControlPanel } from "./components/ControlPanel/ControlPanel";
import { Settings }     from "./components/Settings/Settings";
import { MacroPanel }   from "./components/MacroPanel/MacroPanel";
import { Logs }         from "./components/Logs/Logs";

export default function App() {
  const {
    activeView,
    setConnected, setUsername, setPhase, setCurrentMap,
    setRunning, setLocked,
    setAgents, setAgentsLoaded,
    setConfig, addLog,
  } = useStore();

  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const cfg = await invoke<any>("get_config");
        setConfig(cfg);
      } catch {}

      try {
        const agents = await invoke<any[]>("get_agents");
        setAgents(agents);
        setAgentsLoaded(true);
        addLog(`Loaded ${agents.length} agents`);
      } catch (e) {
        addLog(`Failed to load agents: ${e}`);
        setAgentsLoaded(true);
      }

      try {
        const r = await invoke<{ username: string; tagLine: string; phase: string }>("connect");
        setConnected(true);
        setUsername(r.username, r.tagLine);
        setPhase(r.phase as GamePhase);
        addLog(`Connected as ${r.username}#${r.tagLine} — phase: ${r.phase}`);
      } catch (e) {
        addLog(`Valorant not running: ${e}`);
        setConnected(false);
      }
    })();
  }, []);

  useEffect(() => {
    const unsubs: (() => void)[] = [];

    listen<string>("phase-changed", e => {
      setPhase(e.payload as GamePhase);
      addLog(`Phase: ${e.payload}`);
    }).then(u => unsubs.push(u));

    listen<string>("map-detected", e => {
      if (e.payload) { setCurrentMap(e.payload); addLog(`Map: ${e.payload}`); }
    }).then(u => unsubs.push(u));

    listen<boolean>("lock-status", e => {
      setLocked(e.payload);
      if (!e.payload) setRunning(false);
    }).then(u => unsubs.push(u));

    listen<string>("agent-locked", e => {
      addLog(`Locked: ${e.payload}`);
    }).then(u => unsubs.push(u));

    listen<string>("lock-error", e => {
      addLog(`Error: ${e.payload}`);
    }).then(u => unsubs.push(u));

    return () => unsubs.forEach(u => u());
  }, []);

  useEffect(() => {
    pollRef.current = setInterval(async () => {
      try { await invoke("poll_phase"); } catch {}
    }, 600);
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, []);

  return (
    <div style={{ display:"flex", flexDirection:"column", height:"100vh", overflow:"hidden" }}>
      <StatusBar />
      <div style={{ display:"flex", flex:1, overflow:"hidden" }}>
        <Sidebar />

        {/* Main content */}
        <div style={{ flex:1, display:"flex", overflow:"hidden" }}>
          {activeView === "dashboard" && (
            <>
              <AgentGrid />
              <ControlPanel />
            </>
          )}
          {activeView === "settings" && <Settings />}
          {activeView === "macros"   && <MacroPanel />}
          {activeView === "logs"     && <Logs />}
        </div>
      </div>
    </div>
  );
}
