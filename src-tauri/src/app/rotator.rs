// ============================================================================
// File: src-tauri/src/app/rotator.rs
// Purpose: Resilient Async Rotation Engine & Worker Loops
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;
use tracing::{info, warn, error};

use crate::app::quarantine::QuarantineManager;
use crate::domain::model::{AppLogEntry, PublicIpInfo, RotatorMetrics};
use crate::domain::traits::CoreController;

pub struct RotatorEngine {
    core: Arc<dyn CoreController>,
    quarantine: Arc<QuarantineManager>,
    is_running: Arc<AtomicBool>,
    is_tun_active: Arc<AtomicBool>,
    interval_secs: Arc<AtomicU32>,
    seconds_remaining: Arc<AtomicU32>,
    force_switch_notify: Arc<Notify>,
    current_node: Arc<parking_lot::RwLock<Option<String>>>,
    current_ip: Arc<parking_lot::RwLock<PublicIpInfo>>,
}

impl RotatorEngine {
    pub fn new(core: Arc<dyn CoreController>, quarantine: Arc<QuarantineManager>) -> Self {
        Self {
            core,
            quarantine,
            is_running: Arc::new(AtomicBool::new(false)),
            is_tun_active: Arc::new(AtomicBool::new(true)),
            interval_secs: Arc::new(AtomicU32::new(30)),
            seconds_remaining: Arc::new(AtomicU32::new(30)),
            force_switch_notify: Arc::new(Notify::new()),
            current_node: Arc::new(parking_lot::RwLock::new(None)),
            current_ip: Arc::new(parking_lot::RwLock::new(PublicIpInfo::default())),
        }
    }

    pub fn set_interval(&self, secs: u32) {
        let val = if secs < 5 { 5 } else { secs };
        self.interval_secs.store(val, Ordering::SeqCst);
    }

    pub fn set_tun(&self, active: bool) {
        self.is_tun_active.store(active, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Trigger immediate rotation without waiting for countdown
    pub fn force_switch(&self) {
        self.force_switch_notify.notify_one();
    }

    /// Stop the rotator loops
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.force_switch_notify.notify_waiters();
    }

    /// Start the rotation loop and background health-checker in Tokio tasks
    pub fn start(self: Arc<Self>, app_handle: AppHandle) {
        self.is_running.store(true, Ordering::SeqCst);
        let interval = self.interval_secs.load(Ordering::SeqCst);
        self.seconds_remaining.store(interval, Ordering::SeqCst);

        // Spawn 1: Main Rotation Countdown Loop
        let self_clone1 = Arc::clone(&self);
        let app_handle1 = app_handle.clone();
        tokio::spawn(async move {
            self_clone1.run_rotation_loop(app_handle1).await;
        });

        // Spawn 2: Background Health-Check & Revival Loop
        let self_clone2 = Arc::clone(&self);
        let app_handle2 = app_handle.clone();
        tokio::spawn(async move {
            self_clone2.run_revival_loop(app_handle2).await;
        });
    }

    async fn run_rotation_loop(&self, app: AppHandle) {
        info!("Rotator countdown loop started");

        // Initial switch immediately on start
        self.execute_switch(&app).await;

        while self.is_running.load(Ordering::SeqCst) {
            let interval = self.interval_secs.load(Ordering::SeqCst);
            self.seconds_remaining.store(interval, Ordering::SeqCst);

            // 1-second countdown tick
            while self.seconds_remaining.load(Ordering::SeqCst) > 0 {
                if !self.is_running.load(Ordering::SeqCst) {
                    break;
                }

                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        self.seconds_remaining.fetch_sub(1, Ordering::SeqCst);
                        self.emit_metrics(&app);
                    }
                    _ = self.force_switch_notify.notified() => {
                        info!("Force switch triggered manually!");
                        break;
                    }
                }
            }

            if self.is_running.load(Ordering::SeqCst) {
                self.execute_switch(&app).await;
            }
        }

        info!("Rotator loop ended");
    }

    /// Perform a single node rotation
    async fn execute_switch(&self, app: &AppHandle) {
        // Pick next healthy node
        let next_node = self.quarantine.select_next_live_node();

        match next_node {
            Some(node) => {
                info!("Switching to next node: '{}'", node.name);
                self.emit_log(
                    app,
                    "INFO",
                    &format!("Switching to node '{}' (Ping: {:?}ms)", node.name, node.latency_ms),
                );

                // Call CoreController to switch the proxy in the ROTATOR group
                if let Err(e) = self.core.select_proxy("ROTATOR", &node.name).await {
                    error!("Failed to select proxy: {:?}", e);
                    self.quarantine.put_to_sleep(&node.name);
                    self.emit_log(app, "WARN", &format!("Node '{}' failed to connect -> Sent to quarantine", node.name));
                    return;
                }

                *self.current_node.write() = Some(node.name.clone());

                // Fetch new outbound IP in background
                let core_clone = Arc::clone(&self.core);
                let current_ip_store = Arc::clone(&self.current_ip);
                let app_clone = app.clone();
                let node_name = node.name.clone();

                tokio::spawn(async move {
                    // Small delay to allow new socket route to establish
                    tokio::time::sleep(Duration::from_millis(600)).await;
                    match core_clone.query_public_ip().await {
                        Ok(ip_info) => {
                            info!("Active Public IP: {} ({})", ip_info.ip, ip_info.country);
                            let log_msg = format!("IP Changed: {} | Loc: {} {}", ip_info.ip, ip_info.country, ip_info.country_code);
                            *current_ip_store.write() = ip_info;

                            let _ = app_clone.emit(
                                "app-log",
                                AppLogEntry {
                                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                                    level: "SUCCESS".to_string(),
                                    message: log_msg,
                                },
                            );
                        }
                        Err(e) => {
                            warn!("Failed to query outbound IP for {}: {:?}", node_name, e);
                        }
                    }
                });
            }
            None => {
                warn!("Circuit Breaker: No live nodes available!");
                self.emit_log(
                    app,
                    "WARN",
                    "No healthy nodes in pool! Standby mode active... Testing sleeping nodes.",
                );
                // Attempt quick test on all known nodes
                self.test_all_nodes(app).await;
            }
        }

        self.emit_metrics(app);
    }

    /// Background loop that periodically re-tests sleeping nodes
    async fn run_revival_loop(&self, app: AppHandle) {
        while self.is_running.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(60)).await;
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }

            let expired = self.quarantine.get_expired_sleeping_nodes();
            if !expired.is_empty() {
                info!("Testing {} expired sleeping nodes for revival...", expired.len());
                for name in expired {
                    let core = Arc::clone(&self.core);
                    let quarantine = Arc::clone(&self.quarantine);
                    let app_clone = app.clone();

                    tokio::spawn(async move {
                        if let Ok(latency) = core.test_delay(&name, "http://cp.cloudflare.com/generate_204", 2500).await {
                            quarantine.mark_alive(&name, latency);
                            let _ = app_clone.emit(
                                "app-log",
                                AppLogEntry {
                                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                                    level: "SUCCESS".to_string(),
                                    message: format!("Node '{}' revived! (Delay: {}ms)", name, latency),
                                },
                            );
                        }
                    });
                }
            }
        }
    }

    /// Fast test of all nodes (e.g. at startup or circuit-breaker event)
    pub async fn test_all_nodes(&self, app: &AppHandle) {
        if let Ok(nodes) = self.core.query_proxies().await {
            self.quarantine.update_nodes(nodes.clone());

            let mut tasks = Vec::new();
            for node in nodes {
                let core = Arc::clone(&self.core);
                let quarantine = Arc::clone(&self.quarantine);
                let name = node.name.clone();

                tasks.push(tokio::spawn(async move {
                    match core.test_delay(&name, "http://cp.cloudflare.com/generate_204", 2500).await {
                        Ok(delay) => {
                            quarantine.mark_alive(&name, delay);
                        }
                        Err(_) => {
                            quarantine.put_to_sleep(&name);
                        }
                    }
                }));
            }

            for t in tasks {
                let _ = t.await;
            }
            self.emit_metrics(app);
        }
    }

    fn emit_metrics(&self, app: &AppHandle) {
        let (total, live, sleeping, dead) = self.quarantine.get_stats();
        let metrics = RotatorMetrics {
            total_nodes: total,
            live_nodes: live,
            sleeping_nodes: sleeping,
            dead_nodes: dead,
            current_node: self.current_node.read().clone(),
            current_ip: self.current_ip.read().clone(),
            current_latency_ms: None,
            seconds_remaining: self.seconds_remaining.load(Ordering::SeqCst),
            is_running: self.is_running.load(Ordering::SeqCst),
            is_tun_active: self.is_tun_active.load(Ordering::SeqCst),
        };
        let _ = app.emit("metrics-update", metrics);
    }

    fn emit_log(&self, app: &AppHandle, level: &str, msg: &str) {
        let _ = app.emit(
            "app-log",
            AppLogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: level.to_string(),
                message: msg.to_string(),
            },
        );
    }
}
