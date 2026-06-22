import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore, GamePhase } from "./store";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { StatusBar } from "./components/StatusBar/StatusBar";
import { AgentGrid } from "./components/AgentGrid/AgentGrid";
import { LockPanel } from "./components/LockPanel/LockPanel";
import { Settings } from "./components/Settings/Settings";
import { MacroPanel } from "./components/MacroPanel/MacroPanel";
import { Logs } from "./components/Logs/Logs";

export default function App() {
  const {
    activeView,
    setConnected,
    setUsername,
    setPhase,
    setCurrentMap,
    setRunning,
    setLocked,
    setAgents,
    setAgentsLoaded,
    setConfig,
    addLog,
  } = useStore();

  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Bootstrap
  useEffect(() => {
    (async () => {
      // Load config
      try {
        const cfg = await invoke<any>("get_config");
        setConfig(cfg);
      } catch {}

      // Fetch agents
      try {
        const agents = await invoke<any[]>("get_agents");
        setAgents(agents);
        setAgentsLoaded(true);
        addLog(`Loaded ${agents.length} agents`);
      } catch (e) {
        addLog(`Failed to load agents: ${e}`);
        setAgentsLoaded(true);
      }

      // Connect to Valorant
      try {
        const result = await invoke<{ username: string; tagLine: string; phase: string }>("connect");
        setConnected(true);
        setUsername(result.username, result.tagLine);
        setPhase(result.phase as GamePhase);
        addLog(`Connected as ${result.username}#${result.tagLine} — phase: ${result.phase}`);
      } catch (e) {
        addLog(`Valorant not running: ${e}`);
        setConnected(false);
      }
    })();
  }, []);

  // Tauri event listeners
  useEffect(() => {
    const unsubs: (() => void)[] = [];

    listen<string>("phase-changed", (e) => {
      setPhase(e.payload as GamePhase);
      addLog(`Phase changed: ${e.payload}`);
    }).then((u) => unsubs.push(u));

    listen<string>("map-detected", (e) => {
      if (e.payload) {
        setCurrentMap(e.payload);
        addLog(`Map detected: ${e.payload}`);
      }
    }).then((u) => unsubs.push(u));

    listen<boolean>("lock-status", (e) => {
      setLocked(e.payload);
      if (!e.payload) setRunning(false);
    }).then((u) => unsubs.push(u));

    listen<string>("agent-locked", (e) => {
      addLog(`Agent locked: ${e.payload}`);
    }).then((u) => unsubs.push(u));

    listen<string>("lock-error", (e) => {
      addLog(`Lock error: ${e.payload}`);
    }).then((u) => unsubs.push(u));

    return () => unsubs.forEach((u) => u());
  }, []);

  // Phase polling every 600ms
  useEffect(() => {
    pollRef.current = setInterval(async () => {
      try {
        await invoke("poll_phase");
      } catch {}
    }, 600);

    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-[#0f0f0f]">
      <StatusBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 flex flex-col overflow-hidden">
          {activeView === "dashboard" && (
            <>
              <AgentGrid />
              <LockPanel />
            </>
          )}
          {activeView === "settings" && <Settings />}
          {activeView === "macros" && <MacroPanel />}
          {activeView === "logs" && <Logs />}
        </main>
      </div>
    </div>
  );
}
