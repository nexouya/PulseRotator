// ============================================================================
// File: src-tauri/src/domain/traits.rs
// Purpose: Core Controller Trait (Hexagonal Architecture Port)
// Note: Allows swapping Mihomo with Sing-box or Xray in the future seamlessly.
// ============================================================================

use async_trait::async_trait;
use std::path::Path;
use crate::domain::model::{ProxyNode, PublicIpInfo};
use crate::domain::error::AppError;

#[async_trait]
pub trait CoreController: Send + Sync {
    /// Start the underlying core process with given generated configuration
    async fn start(&self, config_path: &Path) -> Result<(), AppError>;

    /// Gracefully stop the core and teardown tunnels
    async fn stop(&self) -> Result<(), AppError>;

    /// Switch the active proxy node inside a named proxy-group (e.g. "ROTATOR")
    async fn select_proxy(&self, group: &str, node: &str) -> Result<(), AppError>;

    /// Test round-trip delay of a single node in milliseconds
    async fn test_delay(&self, node: &str, test_url: &str, timeout_ms: u32) -> Result<u32, AppError>;

    /// Query list of all loaded proxies from the core
    async fn query_proxies(&self) -> Result<Vec<ProxyNode>, AppError>;

    /// Query the current outbound public IP and geolocation
    async fn query_public_ip(&self) -> Result<PublicIpInfo, AppError>;

    /// Check if the core process is currently alive and responding
    async fn is_running(&self) -> bool;
}
