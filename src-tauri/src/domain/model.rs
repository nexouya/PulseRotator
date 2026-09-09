// ============================================================================
// File: src-tauri/src/domain/model.rs
// Purpose: Pure Domain Models for PulseRotator / NexuFlux
// Note: This layer is completely decoupled from UI, Mihomo, or OS frameworks.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Status of an individual proxy node within the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeHealth {
    /// Node is tested, responding with good latency, ready for rotation
    Live,
    /// Node timed out or errored; temporarily quarantined in sleep mode
    Sleeping,
    /// Permanently failed (e.g. malformed config or exhausted retries)
    Dead,
    /// Freshly loaded, waiting for initial delay test
    Untested,
}

/// Represents a single proxy node configuration in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    /// Unique identifier or name (e.g. from subscription: "US - Reality 01")
    pub name: String,
    /// Proxy protocol type (VLESS, VMess, Reality, SS, Trojan, Hysteria2)
    pub protocol: String,
    /// Current health classification
    pub health: NodeHealth,
    /// Last measured round-trip delay in milliseconds (None if unreachable)
    pub latency_ms: Option<u32>,
    /// Number of consecutive connection failures
    pub failure_count: u32,
    /// Total number of successful connections
    pub success_count: u32,
    /// Server IP or Domain destination
    pub server: String,
    /// Server Port
    pub port: u16,
    /// Timestamp until which this node remains in quarantine (ignored by serde)
    #[serde(skip)]
    pub sleep_until: Option<Instant>,
}

impl ProxyNode {
    pub fn new(name: String, protocol: String, server: String, port: u16) -> Self {
        Self {
            name,
            protocol,
            health: NodeHealth::Untested,
            latency_ms: None,
            failure_count: 0,
            success_count: 0,
            server,
            port,
            sleep_until: None,
        }
    }

    /// Mark node as failed and put into temporary quarantine (e.g. for 5-10 mins)
    pub fn put_to_sleep(&mut self, duration: std::time::Duration) {
        self.health = NodeHealth::Sleeping;
        self.failure_count += 1;
        self.latency_ms = None;
        self.sleep_until = Some(Instant::now() + duration);
    }

    /// Mark node as revived and healthy
    pub fn mark_alive(&mut self, latency: u32) {
        self.health = NodeHealth::Live;
        self.latency_ms = Some(latency);
        self.failure_count = 0;
        self.success_count += 1;
        self.sleep_until = None;
    }

    /// Check whether the node has finished its quarantine sleep
    pub fn is_ready_for_test(&self) -> bool {
        match self.sleep_until {
            Some(wake_time) => Instant::now() >= wake_time,
            None => true,
        }
    }
}

/// Information about the current outbound public IP
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicIpInfo {
    pub ip: String,
    pub country: String,
    pub country_code: String,
    pub city: String,
    pub org: String,
}

/// Statistics snapshot emitted to the frontend
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotatorMetrics {
    pub total_nodes: usize,
    pub live_nodes: usize,
    pub sleeping_nodes: usize,
    pub dead_nodes: usize,
    pub current_node: Option<String>,
    pub current_ip: PublicIpInfo,
    pub current_latency_ms: Option<u32>,
    pub seconds_remaining: u32,
    pub is_running: bool,
    pub is_tun_active: bool,
}

/// Log message emitted for UI terminal view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLogEntry {
    pub timestamp: String,
    pub level: String, // "INFO", "WARN", "ERROR", "SUCCESS"
    pub message: String,
}
