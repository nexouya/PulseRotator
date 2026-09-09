import React from 'react';
import { Layers, Terminal, CheckCircle2, Moon, AlertTriangle } from 'lucide-react';
import { AppLogEntry } from '../types';

interface MetricsAndLogProps {
  totalNodes: number;
  liveNodes: number;
  sleepingNodes: number;
  deadNodes: number;
  logs: AppLogEntry[];
}

export const MetricsAndLog: React.FC<MetricsAndLogProps> = ({
  totalNodes,
  liveNodes,
  sleepingNodes,
  deadNodes,
  logs,
}) => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {/* Pool Health Metrics Pill Box */}
      <div className="bg-surface rounded-xl p-4 border border-surface-border shadow-xl flex flex-col justify-between">
        <div className="text-xs font-mono font-semibold tracking-wider text-slate-400 uppercase flex items-center gap-1.5 mb-3">
          <Layers className="w-4 h-4 text-accent-cyan" />
          Smart Node Pool
        </div>

        <div className="space-y-2.5">
          <div className="flex items-center justify-between p-2 rounded bg-background/50 border border-surface-border/40">
            <span className="text-xs text-slate-300 flex items-center gap-1.5">
              <CheckCircle2 className="w-3.5 h-3.5 text-accent-emerald" />
              Live Ready
            </span>
            <span className="text-xs font-mono font-bold text-accent-emerald">{liveNodes}</span>
          </div>

          <div className="flex items-center justify-between p-2 rounded bg-background/50 border border-surface-border/40">
            <span className="text-xs text-slate-300 flex items-center gap-1.5">
              <Moon className="w-3.5 h-3.5 text-accent-amber" />
              Quarantined (Sleeping)
            </span>
            <span className="text-xs font-mono font-bold text-accent-amber">{sleepingNodes}</span>
          </div>

          <div className="flex items-center justify-between p-2 rounded bg-background/50 border border-surface-border/40">
            <span className="text-xs text-slate-300 flex items-center gap-1.5">
              <AlertTriangle className="w-3.5 h-3.5 text-slate-500" />
              Total Detected
            </span>
            <span className="text-xs font-mono font-bold text-slate-300">{totalNodes}</span>
          </div>
        </div>

        <div className="text-[10px] text-slate-500 mt-2 font-mono text-center">
          Auto-Revival Routine: 10m Quarantine
        </div>
      </div>

      {/* Real-time Terminal Log Console */}
      <div className="md:col-span-2 bg-surface rounded-xl p-4 border border-surface-border shadow-xl flex flex-col">
        <div className="text-xs font-mono font-semibold tracking-wider text-slate-400 uppercase flex items-center justify-between mb-2">
          <span className="flex items-center gap-1.5">
            <Terminal className="w-4 h-4 text-primary-500" />
            Live Activity Journal
          </span>
          <span className="text-[10px] text-slate-500 font-mono">Real-time IPC</span>
        </div>

        <div className="flex-1 bg-background rounded-lg p-3 font-mono text-[11px] overflow-y-auto max-h-36 min-h-[140px] space-y-1.5 border border-surface-border/40 select-text">
          {logs.length === 0 ? (
            <div className="text-slate-600 italic">Waiting for service initiation...</div>
          ) : (
            logs.map((log, idx) => (
              <div key={idx} className="flex items-start gap-2 leading-tight">
                <span className="text-slate-500 shrink-0">[{log.timestamp}]</span>
                <span
                  className={`font-bold shrink-0 text-[10px] px-1 rounded ${
                    log.level === 'SUCCESS'
                      ? 'bg-accent-emerald/20 text-accent-emerald'
                      : log.level === 'WARN'
                      ? 'bg-accent-amber/20 text-accent-amber'
                      : log.level === 'ERROR'
                      ? 'bg-accent-rose/20 text-accent-rose'
                      : 'bg-primary-500/20 text-primary-500'
                  }`}
                >
                  {log.level}
                </span>
                <span className="text-slate-300 break-all">{log.message}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
