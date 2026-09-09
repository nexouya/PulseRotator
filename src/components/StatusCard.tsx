import React from 'react';
import { Globe, RefreshCw, Activity, MapPin } from 'lucide-react';
import { PublicIpInfo } from '../types';

interface StatusCardProps {
  currentIp: PublicIpInfo;
  currentNode: string | null;
  secondsRemaining: number;
  totalInterval: number;
  isRunning: boolean;
  onForceSwitch: () => void;
}

export const StatusCard: React.FC<StatusCardProps> = ({
  currentIp,
  currentNode,
  secondsRemaining,
  totalInterval,
  isRunning,
  onForceSwitch,
}) => {
  const percent = totalInterval > 0 ? Math.max(0, Math.min(100, ((totalInterval - secondsRemaining) / totalInterval) * 100)) : 0;

  return (
    <div className="bg-surface rounded-xl p-5 border border-surface-border shadow-xl relative overflow-hidden">
      {/* Subtle background glow */}
      <div className="absolute -right-16 -top-16 w-48 h-48 bg-primary-500/10 rounded-full blur-3xl pointer-events-none" />

      <div className="flex items-center justify-between mb-4">
        <span className="text-xs font-mono font-semibold tracking-wider text-slate-400 uppercase flex items-center gap-1.5">
          <Globe className="w-4 h-4 text-accent-cyan" />
          Live Network Routing
        </span>

        <button
          onClick={onForceSwitch}
          disabled={!isRunning}
          className="flex items-center space-x-1.5 text-xs font-semibold px-3 py-1.5 rounded-lg bg-surface-hover hover:bg-surface-border disabled:opacity-40 disabled:pointer-events-none text-slate-200 border border-slate-700 transition-colors"
        >
          <RefreshCw className="w-3.5 h-3.5 text-accent-cyan" />
          <span>Force Switch Now</span>
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Public IP Display */}
        <div className="p-4 rounded-lg bg-background/60 border border-surface-border/60">
          <div className="text-[11px] font-mono text-slate-400 mb-1">CURRENT OUTBOUND IP</div>
          <div className="text-2xl font-mono font-bold tracking-tight text-white flex items-center gap-2">
            <span>{currentIp.ip || (isRunning ? 'Resolving...' : '127.0.0.1')}</span>
            {currentIp.country_code && (
              <span className="text-xs px-2 py-0.5 rounded bg-primary-500/20 text-primary-500 font-sans">
                {currentIp.country_code}
              </span>
            )}
          </div>
          <div className="text-xs text-slate-400 mt-1 flex items-center gap-1">
            <MapPin className="w-3 h-3 text-slate-500" />
            <span>{currentIp.country ? `${currentIp.city || ''} ${currentIp.country}` : 'Local Route'}</span>
          </div>
        </div>

        {/* Connected Node Details */}
        <div className="p-4 rounded-lg bg-background/60 border border-surface-border/60 flex flex-col justify-between">
          <div>
            <div className="text-[11px] font-mono text-slate-400 mb-1">ACTIVE PROXY NODE</div>
            <div className="text-sm font-semibold text-slate-200 truncate" title={currentNode || 'None'}>
              {currentNode || (isRunning ? 'Selecting Node...' : 'Idle')}
            </div>
          </div>
          <div className="flex items-center justify-between text-xs text-slate-400 mt-2">
            <span className="flex items-center gap-1">
              <Activity className="w-3 h-3 text-accent-emerald" />
              Routing: {isRunning ? 'Direct Tunnel' : 'Standby'}
            </span>
          </div>
        </div>
      </div>

      {/* Countdown Progress Bar */}
      <div className="mt-5">
        <div className="flex justify-between items-center text-xs font-mono mb-1.5">
          <span className="text-slate-400">Next IP Switch in:</span>
          <span className="text-accent-cyan font-bold">{isRunning ? `${secondsRemaining}s` : '--'}</span>
        </div>
        <div className="w-full h-2 rounded-full bg-background overflow-hidden border border-surface-border">
          <div
            className="h-full bg-gradient-to-r from-primary-500 to-accent-cyan transition-all duration-1000 ease-linear"
            style={{ width: isRunning ? `${percent}%` : '0%' }}
          />
        </div>
      </div>
    </div>
  );
};
