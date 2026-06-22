import { Crosshair, Settings, Map, Terminal, RefreshCw, AlertTriangle } from "lucide-react";
import clsx from "clsx";
import { View, useStore } from "../../store";
import { invoke } from "@tauri-apps/api/core";

const NAV_ITEMS: { view: View; icon: typeof Crosshair; label: string }[] = [
  { view: "dashboard", icon: Crosshair, label: "Agents" },
  { view: "macros", icon: Map, label: "Macros" },
  { view: "settings", icon: Settings, label: "Settings" },
  { view: "logs", icon: Terminal, label: "Logs" },
];

export function Sidebar() {
  const { activeView, setActiveView, connected, setConnected, setUsername, addLog } = useStore();

  const handleReconnect = async () => {
    addLog("Reconnecting to Valorant...");
    try {
      const result = await invoke<{ username: string; tagLine: string; phase: string }>("connect");
      setConnected(true);
      setUsername(result.username, result.tagLine);
      addLog(`Connected as ${result.username}#${result.tagLine}`);
    } catch (e) {
      setConnected(false);
      addLog(`Connection failed: ${e}`);
    }
  };

  return (
    <div className="w-14 flex flex-col items-center py-3 gap-1 border-r border-[#2a2a2a] bg-[#0f0f0f] flex-shrink-0">
      {/* Logo */}
      <div className="mb-3 p-2">
        <div className="w-8 h-8 flex items-center justify-center">
          <Crosshair size={20} className="text-[#FF4655]" strokeWidth={2.5} />
        </div>
      </div>

      <div className="w-6 h-px bg-[#2a2a2a] mb-2" />

      {/* Nav */}
      {NAV_ITEMS.map(({ view, icon: Icon, label }) => (
        <button
          key={view}
          onClick={() => setActiveView(view)}
          title={label}
          className={clsx(
            "w-10 h-10 flex items-center justify-center transition-all duration-150 relative group",
            activeView === view
              ? "text-[#FF4655]"
              : "text-[#555] hover:text-[#ece8e1]"
          )}
        >
          {activeView === view && (
            <div className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 bg-[#FF4655] rounded-r" />
          )}
          <Icon size={16} strokeWidth={activeView === view ? 2.5 : 2} />
          {/* Tooltip */}
          <div className="absolute left-full ml-2 px-2 py-1 bg-[#1a1a1a] border border-[#2a2a2a] text-[10px] font-bold uppercase tracking-widest whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
            {label}
          </div>
        </button>
      ))}

      <div className="flex-1" />

      {/* Connection status + reconnect */}
      <div className="relative group">
        <button
          onClick={handleReconnect}
          className={clsx(
            "w-10 h-10 flex items-center justify-center transition-all duration-150",
            connected ? "text-[#4ade80] hover:text-[#86efac]" : "text-[#FF4655] hover:text-[#fca5a5] animate-pulse"
          )}
          title={connected ? "Connected — click to reconnect" : "Disconnected — click to connect"}
        >
          {connected ? <RefreshCw size={14} /> : <AlertTriangle size={14} />}
        </button>
        <div className="absolute left-full ml-2 px-2 py-1 bg-[#1a1a1a] border border-[#2a2a2a] text-[10px] font-bold uppercase tracking-widest whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
          {connected ? "Reconnect" : "Connect"}
        </div>
      </div>
    </div>
  );
}
