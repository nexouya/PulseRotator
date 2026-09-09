// ============================================================================
// File: src-tauri/src/app/quarantine.rs
// Purpose: Thread-safe Smart Node Pool with Quarantine (Sleep) & Auto-Revival
// ============================================================================

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use crate::domain::model::{NodeHealth, ProxyNode};

pub struct QuarantineManager {
    nodes: Arc<RwLock<HashMap<String, ProxyNode>>>,
    active_cursor: Arc<RwLock<usize>>,
    quarantine_duration: Duration,
}

impl QuarantineManager {
    pub fn new(quarantine_duration: Duration) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            active_cursor: Arc::new(RwLock::new(0)),
            quarantine_duration,
        }
    }

    /// Load or update list of nodes (e.g. from subscription or core query)
    pub fn update_nodes(&self, new_nodes: Vec<ProxyNode>) {
        let mut map = self.nodes.write();
        for node in new_nodes {
            map.entry(node.name.clone())
                .and_modify(|existing| {
                    if existing.health == NodeHealth::Untested {
                        existing.health = node.health;
                    }
                })
                .or_insert(node);
        }
        info!("Quarantine pool updated. Total known nodes: {}", map.len());
    }

    /// Get list of live nodes ready for rotation
    pub fn get_live_nodes(&self) -> Vec<ProxyNode> {
        let map = self.nodes.read();
        map.values()
            .filter(|n| n.health == NodeHealth::Live)
            .cloned()
            .collect()
    }

    /// Mark a node as alive with reported latency
    pub fn mark_alive(&self, name: &str, latency: u32) {
        let mut map = self.nodes.write();
        if let Some(node) = map.get_mut(name) {
            node.mark_alive(latency);
        }
    }

    /// Put a failing node to sleep for the configured quarantine duration
    pub fn put_to_sleep(&self, name: &str) {
        let mut map = self.nodes.write();
        if let Some(node) = map.get_mut(name) {
            node.put_to_sleep(self.quarantine_duration);
            warn!(
                "Node '{}' put to sleep (quarantine) for {} secs",
                name,
                self.quarantine_duration.as_secs()
            );
        }
    }

    /// Select next live node in round-robin fashion
    pub fn select_next_live_node(&self) -> Option<ProxyNode> {
        let live_nodes = self.get_live_nodes();
        if live_nodes.is_empty() {
            return None;
        }

        let mut cursor = self.active_cursor.write();
        *cursor = (*cursor + 1) % live_nodes.len();
        Some(live_nodes[*cursor].clone())
    }

    /// Get nodes whose quarantine time has expired and need re-testing
    pub fn get_expired_sleeping_nodes(&self) -> Vec<String> {
        let map = self.nodes.read();
        map.values()
            .filter(|n| n.health == NodeHealth::Sleeping && n.is_ready_for_test())
            .map(|n| n.name.clone())
            .collect()
    }

    /// Statistics breakdown
    pub fn get_stats(&self) -> (usize, usize, usize, usize) {
        let map = self.nodes.read();
        let mut live = 0;
        let mut sleeping = 0;
        let mut dead = 0;
        let total = map.len();

        for n in map.values() {
            match n.health {
                NodeHealth::Live => live += 1,
                NodeHealth::Sleeping => sleeping += 1,
                NodeHealth::Dead => dead += 1,
                NodeHealth::Untested => {}
            }
        }
        (total, live, sleeping, dead)
    }
}
