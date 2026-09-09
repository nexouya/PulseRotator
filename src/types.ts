export interface PublicIpInfo {
  ip: string;
  country: string;
  country_code: string;
  city: string;
  org: string;
}

export interface RotatorMetrics {
  total_nodes: number;
  live_nodes: number;
  sleeping_nodes: number;
  dead_nodes: number;
  current_node: string | null;
  current_ip: PublicIpInfo;
  current_latency_ms: number | null;
  seconds_remaining: number;
  is_running: boolean;
  is_tun_active: boolean;
}

export interface AppLogEntry {
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'SUCCESS';
  message: string;
}

export interface ProxyNode {
  name: string;
  protocol: string;
  health: 'Live' | 'Sleeping' | 'Dead' | 'Untested';
  latency_ms: number | null;
  server: string;
  port: number;
}
