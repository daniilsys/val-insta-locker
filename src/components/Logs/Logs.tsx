import { useStore } from "../../store";

export function Logs() {
  const { logs } = useStore();

  return (
    <div className="flex-1 overflow-hidden flex flex-col p-4 animate-slide-up">
      <p className="text-[10px] font-bold uppercase tracking-widest text-[#555] mb-3">Activity Log</p>
      <div className="flex-1 overflow-y-auto font-mono text-[11px] space-y-1">
        {logs.length === 0 ? (
          <span className="text-[#555]">No activity yet.</span>
        ) : (
          logs.map((log, i) => (
            <div key={i} className="text-[#7b7b7b] leading-relaxed">
              <span className="text-[#FF4655]/70 mr-2">›</span>
              {log}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
