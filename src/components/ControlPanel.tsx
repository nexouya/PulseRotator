import React from 'react';
import { Sliders, Link, Play, Square, ShieldCheck, DownloadCloud } from 'lucide-react';

interface ControlPanelProps {
  subUrl: string;
  setSubUrl: (val: string) => void;
  interval: number;
  setInterval: (val: number) => void;
  enableTun: boolean;
  setEnableTun: (val: boolean) => void;
  isRunning: boolean;
  onStart: () => void;
  onStop: () => void;
  onFetchSub: () => void;
  isFetchingSub: boolean;
}

export const ControlPanel: React.FC<ControlPanelProps> = ({
  subUrl,
  setSubUrl,
  interval,
  setInterval,
  enableTun,
  setEnableTun,
  isRunning,
  onStart,
  onStop,
  onFetchSub,
  isFetchingSub,
}) => {
  return (
    <div className="bg-surface rounded-xl p-5 border border-surface-border shadow-xl space-y-4">
      <div className="text-xs font-mono font-semibold tracking-wider text-slate-400 uppercase flex items-center gap-1.5">
        <Sliders className="w-4 h-4 text-primary-500" />
        Rotation & Tunnel Configuration
      </div>

      {/* Subscription Input */}
      <div>
        <label className="block text-xs font-medium text-slate-300 mb-1.5 flex items-center gap-1">
          <Link className="w-3.5 h-3.5 text-slate-400" />
          Subscription Link or Raw Nodes (VLESS / VMess / Reality / SS)
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            placeholder="https://example.com/api/v1/client/subscribe?token=..."
            value={subUrl}
            onChange={(e) => setSubUrl(e.target.value)}
            disabled={isRunning}
            className="flex-1 bg-background border border-surface-border rounded-lg px-3 py-2 text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none focus:border-primary-500 disabled:opacity-50"
          />
          <button
            onClick={onFetchSub}
            disabled={isRunning || !subUrl || isFetchingSub}
            className="flex items-center gap-1.5 px-3 py-2 bg-surface-hover hover:bg-surface-border text-xs font-semibold rounded-lg border border-slate-700 text-slate-300 disabled:opacity-40 transition-colors"
          >
            <DownloadCloud className={`w-3.5 h-3.5 ${isFetchingSub ? 'animate-bounce' : ''}`} />
            <span>{isFetchingSub ? 'Checking...' : 'Inspect'}</span>
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-1">
        {/* Interval Slider / Input */}
        <div>
          <div className="flex justify-between items-center mb-1.5">
            <span className="text-xs font-medium text-slate-300">Rotation Interval</span>
            <span className="text-xs font-mono text-accent-cyan font-bold">{interval}s</span>
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min="5"
              max="300"
              step="5"
              value={interval}
              onChange={(e) => setInterval(Number(e.target.value))}
              disabled={isRunning}
              className="flex-1 accent-primary-500 cursor-pointer disabled:opacity-50"
            />
            <input
              type="number"
              min="5"
              max="3600"
              value={interval}
              onChange={(e) => setInterval(Number(e.target.value))}
              disabled={isRunning}
              className="w-16 bg-background border border-surface-border rounded px-2 py-1 text-xs font-mono text-center text-slate-200 disabled:opacity-50"
            />
          </div>
        </div>

        {/* Full Tunnel (TUN) Toggle */}
        <div className="flex items-center justify-between p-3 rounded-lg bg-background/50 border border-surface-border/60">
          <div className="flex items-center gap-2.5">
            <ShieldCheck className="w-5 h-5 text-primary-500" />
            <div>
              <div className="text-xs font-semibold text-slate-200">Full Tunnel (TUN Mode)</div>
              <div className="text-[10px] text-slate-400">Routes entire Windows network via Wintun</div>
            </div>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={enableTun}
              onChange={(e) => setEnableTun(e.target.checked)}
              disabled={isRunning}
              className="sr-only peer"
            />
            <div className="w-9 h-5 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary-600 peer-disabled:opacity-50"></div>
          </label>
        </div>
      </div>

      {/* Main Action Button */}
      <div className="pt-2">
        {!isRunning ? (
          <button
            onClick={onStart}
            disabled={!subUrl}
            className="w-full py-3 px-4 rounded-xl bg-gradient-to-r from-primary-600 to-accent-cyan hover:from-primary-500 hover:to-accent-cyan/90 text-white font-bold text-sm tracking-wide flex items-center justify-center gap-2 shadow-lg shadow-primary-500/25 disabled:opacity-40 disabled:cursor-not-allowed transition-all"
          >
            <Play className="w-4 h-4 fill-white" />
            <span>START PULSEROTATOR</span>
          </button>
        ) : (
          <button
            onClick={onStop}
            className="w-full py-3 px-4 rounded-xl bg-accent-rose/90 hover:bg-accent-rose text-white font-bold text-sm tracking-wide flex items-center justify-center gap-2 shadow-lg shadow-accent-rose/25 transition-all"
          >
            <Square className="w-4 h-4 fill-white" />
            <span>STOP & RESTORE NETWORK</span>
          </button>
        )}
      </div>
    </div>
  );
};
