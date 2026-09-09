import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Header } from './components/Header';
import { StatusCard } from './components/StatusCard';
import { ControlPanel } from './components/ControlPanel';
import { MetricsAndLog } from './components/MetricsAndLog';
import { RotatorMetrics, AppLogEntry, PublicIpInfo, ProxyNode } from './types';
import { RefreshCw, ListFilter, SlidersHorizontal, Info } from 'lucide-react';

export default function App() {
  const [isAdmin, setIsAdmin] = useState(true);
  const [subUrl, setSubUrl] = useState('');
  const [interval, setIntervalVal] = useState(30);
  const [enableTun, setEnableTun] = useState(true);
  const [isFetchingSub, setIsFetchingSub] = useState(false);
  const [activeTab, setActiveTab] = useState<'rotator' | 'proxies' | 'settings'>('rotator');

  const [metrics, setMetrics] = useState<RotatorMetrics>({
    total_nodes: 0,
    live_nodes: 0,
    sleeping_nodes: 0,
    dead_nodes: 0,
    current_node: null,
    current_ip: { ip: '', country: '', country_code: '', city: '', org: '' },
    current_latency_ms: null,
    seconds_remaining: 30,
    is_running: false,
    is_tun_active: true,
  });

  const [logs, setLogs] = useState<AppLogEntry[]>([
    {
      timestamp: new Date().toLocaleTimeString(),
      level: 'INFO',
      message: 'PulseRotator initialized. Decoupled Core ready.',
    },
  ]);

  // Check Admin privileges on load & setup event listeners
  useEffect(() => {
    invoke<boolean>('cmd_check_admin')
      .then((elevated) => setIsAdmin(elevated))
      .catch(() => setIsAdmin(false));

    // Listen for metrics updates from Rust Rotator loop
    const unlistenMetrics = listen<RotatorMetrics>('metrics-update', (event) => {
      setMetrics(event.payload);
    });

    // Listen for real-time activity log entries
    const unlistenLogs = listen<AppLogEntry>('app-log', (event) => {
      setLogs((prev) => [event.payload, ...prev.slice(0, 49)]);
    });

    return () => {
      unlistenMetrics.then((fn) => fn());
      unlistenLogs.then((fn) => fn());
    };
  }, []);

  const handleStart = async () => {
    if (!subUrl) return;
    try {
      addLog('INFO', `Initializing Core with ${interval}s interval (TUN: ${enableTun})...`);
      await invoke('cmd_start_rotation', {
        subUrl,
        intervalSecs: interval,
        enableTun,
      });
      addLog('SUCCESS', 'PulseRotator Core successfully started and tunneling active!');
    } catch (err: any) {
      addLog('ERROR', `Start failed: ${err.toString()}`);
    }
  };

  const handleStop = async () => {
    try {
      addLog('WARN', 'Stopping PulseRotator and restoring OS network routes...');
      await invoke('cmd_stop_rotation');
      addLog('INFO', 'Rotator stopped cleanly. DNS cache flushed.');
    } catch (err: any) {
      addLog('ERROR', `Stop failed: ${err.toString()}`);
    }
  };

  const handleForceSwitch = async () => {
    try {
      await invoke('cmd_force_switch_now');
      addLog('INFO', 'Force switch signal sent to backend.');
    } catch (err: any) {
      addLog('ERROR', `Force switch failed: ${err.toString()}`);
    }
  };

  const handleFetchSub = async () => {
    if (!subUrl) return;
    setIsFetchingSub(true);
    try {
      addLog('INFO', `Parsing subscription links from source...`);
      const nodes = await invoke<ProxyNode[]>('cmd_fetch_sub', { subUrl });
      addLog('SUCCESS', `Successfully detected ${nodes.length} nodes from subscription!`);
    } catch (err: any) {
      addLog('ERROR', `Failed to parse subscription: ${err.toString()}`);
    } finally {
      setIsFetchingSub(false);
    }
  };

  const addLog = (level: AppLogEntry['level'], message: string) => {
    setLogs((prev) => [
      {
        timestamp: new Date().toLocaleTimeString(),
        level,
        message,
      },
      ...prev.slice(0, 49),
    ]);
  };

  return (
    <div className="flex flex-col h-screen bg-background text-slate-100 font-sans select-none">
      {/* 1. Header Bar */}
      <Header
        isRunning={metrics.is_running}
        isTunActive={metrics.is_tun_active}
        isAdmin={isAdmin}
      />

      {/* 2. Modular Tab Navigation Bar (Extensible for Full Client Future) */}
      <div className="flex items-center px-6 pt-3 border-b border-surface-border bg-surface/30 gap-2">
        <button
          onClick={() => setActiveTab('rotator')}
          className={`flex items-center gap-1.5 px-3 py-2 text-xs font-semibold rounded-t-lg border-b-2 transition-all ${
            activeTab === 'rotator'
              ? 'border-primary-500 text-white bg-surface/80'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <RefreshCw className="w-3.5 h-3.5 text-primary-500" />
          <span>IP Rotator Engine</span>
        </button>

        <button
          onClick={() => setActiveTab('proxies')}
          className={`flex items-center gap-1.5 px-3 py-2 text-xs font-semibold rounded-t-lg border-b-2 transition-all ${
            activeTab === 'proxies'
              ? 'border-primary-500 text-white bg-surface/80'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <ListFilter className="w-3.5 h-3.5 text-slate-500" />
          <span>Proxy Nodes <span className="text-[9px] px-1 py-0.2 rounded bg-surface-border text-slate-400">Extensible</span></span>
        </button>

        <button
          onClick={() => setActiveTab('settings')}
          className={`flex items-center gap-1.5 px-3 py-2 text-xs font-semibold rounded-t-lg border-b-2 transition-all ${
            activeTab === 'settings'
              ? 'border-primary-500 text-white bg-surface/80'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <SlidersHorizontal className="w-3.5 h-3.5 text-slate-500" />
          <span>Core Config</span>
        </button>
      </div>

      {/* 3. Main Body Scrollable View */}
      <main className="flex-1 overflow-y-auto p-6 space-y-5">
        {activeTab === 'rotator' && (
          <>
            {/* Live Network State Banner */}
            <StatusCard
              currentIp={metrics.current_ip}
              currentNode={metrics.current_node}
              secondsRemaining={metrics.seconds_remaining}
              totalInterval={interval}
              isRunning={metrics.is_running}
              onForceSwitch={handleForceSwitch}
            />

            {/* Controls and Inputs */}
            <ControlPanel
              subUrl={subUrl}
              setSubUrl={setSubUrl}
              interval={interval}
              setInterval={setIntervalVal}
              enableTun={enableTun}
              setEnableTun={setEnableTun}
              isRunning={metrics.is_running}
              onStart={handleStart}
              onStop={handleStop}
              onFetchSub={handleFetchSub}
              isFetchingSub={isFetchingSub}
            />

            {/* Smart Pool Statistics & Real-time Console Log */}
            <MetricsAndLog
              totalNodes={metrics.total_nodes}
              liveNodes={metrics.live_nodes}
              sleepingNodes={metrics.sleeping_nodes}
              deadNodes={metrics.dead_nodes}
              logs={logs}
            />
          </>
        )}

        {activeTab === 'proxies' && (
          <div className="bg-surface rounded-xl p-8 border border-surface-border text-center space-y-3">
            <ListFilter className="w-10 h-10 text-primary-500 mx-auto opacity-75" />
            <h3 className="text-base font-bold text-white">Modular Proxy List Extension Ready</h3>
            <p className="text-xs text-slate-400 max-w-md mx-auto">
              This panel is pre-wired to the Rust <code className="text-accent-cyan font-mono">CoreController::query_proxies</code> trait.
              You can effortlessly add manual node selection, grouping, or speed-testing tiles here whenever you want to expand this into a standalone client!
            </p>
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="bg-surface rounded-xl p-6 border border-surface-border space-y-4">
            <h3 className="text-sm font-bold text-white flex items-center gap-2">
              <Info className="w-4 h-4 text-accent-cyan" />
              PulseRotator Core Engine Diagnostics
            </h3>
            <div className="grid grid-cols-2 gap-3 text-xs font-mono">
              <div className="p-3 bg-background rounded border border-surface-border/50">
                <span className="text-slate-500 block mb-1">LOCAL PROXY PORT</span>
                <span className="text-white font-bold">127.0.0.1:7890 (Mixed SOCKS5/HTTP)</span>
              </div>
              <div className="p-3 bg-background rounded border border-surface-border/50">
                <span className="text-slate-500 block mb-1">EXTERNAL CONTROLLER</span>
                <span className="text-white font-bold">127.0.0.1:9090 (REST API)</span>
              </div>
              <div className="p-3 bg-background rounded border border-surface-border/50">
                <span className="text-slate-500 block mb-1">ROUTING GROUP</span>
                <span className="text-white font-bold">ROTATOR (Selector)</span>
              </div>
              <div className="p-3 bg-background rounded border border-surface-border/50">
                <span className="text-slate-500 block mb-1">WINTUN ADAPTER DRIVER</span>
                <span className="text-accent-emerald font-bold">wintun.dll bundled x64</span>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
