import React from 'react';
import { Shield, ShieldAlert, Zap, Globe2 } from 'lucide-react';

interface HeaderProps {
  isRunning: boolean;
  isTunActive: boolean;
  isAdmin: boolean;
}

export const Header: React.FC<HeaderProps> = ({ isRunning, isTunActive, isAdmin }) => {
  return (
    <header className="flex items-center justify-between px-6 py-4 bg-surface/80 backdrop-blur-md border-b border-surface-border">
      <div className="flex items-center space-x-3">
        <div className="w-9 h-9 rounded-lg bg-gradient-to-tr from-primary-600 to-accent-cyan flex items-center justify-center shadow-lg shadow-primary-500/20">
          <Zap className="w-5 h-5 text-white animate-pulse" />
        </div>
        <div>
          <div className="flex items-center space-x-2">
            <h1 className="font-bold tracking-wider text-base text-white">PULSEROTATOR</h1>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-border font-mono text-slate-300">v0.1</span>
          </div>
          <p className="text-xs text-slate-400">High-Performance IP Rotator & Full Tunnel</p>
        </div>
      </div>

      <div className="flex items-center space-x-3">
        {/* Admin Elevation Status */}
        <div className={`flex items-center space-x-1.5 px-2.5 py-1 rounded-full text-xs font-mono border ${
          isAdmin
            ? 'bg-accent-emerald/10 text-accent-emerald border-accent-emerald/20'
            : 'bg-accent-amber/10 text-accent-amber border-accent-amber/20'
        }`}>
          {isAdmin ? <Shield className="w-3.5 h-3.5" /> : <ShieldAlert className="w-3.5 h-3.5" />}
          <span>{isAdmin ? 'ADMIN OK' : 'NON-ELEVATED'}</span>
        </div>

        {/* Global State Pill */}
        <div className={`flex items-center space-x-2 px-3 py-1 rounded-full text-xs font-semibold border transition-all ${
          isRunning
            ? 'bg-accent-cyan/15 text-accent-cyan border-accent-cyan/40 shadow-sm shadow-accent-cyan/20'
            : 'bg-slate-800 text-slate-400 border-slate-700'
        }`}>
          <span className={`w-2 h-2 rounded-full ${isRunning ? 'bg-accent-cyan animate-ping' : 'bg-slate-500'}`} />
          <span>{isRunning ? (isTunActive ? 'TUN ACTIVE (FULL)' : 'RUNNING (PROXY)') : 'DISCONNECTED'}</span>
        </div>
      </div>
    </header>
  );
};
