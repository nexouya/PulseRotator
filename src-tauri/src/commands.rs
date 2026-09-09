// ============================================================================
// File: src-tauri/src/commands.rs
// Purpose: Tauri IPC Invocation Handlers
// ============================================================================

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tracing::{info, warn, error};

use crate::app::quarantine::QuarantineManager;
use crate::app::rotator::RotatorEngine;
use crate::app::sub_parser::SubParser;
use crate::domain::error::AppError;
use crate::domain::model::{ProxyNode, RotatorMetrics};
use crate::domain::traits::CoreController;
use crate::infra::mihomo::config_gen::ConfigGenerator;
use crate::infra::system::os_ops::{flush_dns_cache, is_elevated};

pub struct AppState {
    pub core: Arc<dyn CoreController>,
    pub quarantine: Arc<QuarantineManager>,
    pub rotator: Arc<RotatorEngine>,
    pub config_dir: PathBuf,
}

#[tauri::command]
pub async fn cmd_check_admin() -> Result<bool, String> {
    Ok(is_elevated())
}

#[tauri::command]
pub async fn cmd_fetch_sub(sub_url: String) -> Result<Vec<ProxyNode>, String> {
    SubParser::fetch_and_parse(&sub_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_start_rotation(
    app: AppHandle,
    state: State<'_, AppState>,
    sub_url: String,
    interval_secs: u32,
    enable_tun: bool,
) -> Result<(), String> {
    info!("Command: start_rotation (sub: {}, interval: {}s, tun: {})", sub_url, interval_secs, enable_tun);

    // 1. Admin permission check if TUN is requested
    if enable_tun && !is_elevated() {
        return Err("Administrator privileges required to enable Wintun TUN mode on Windows!".to_string());
    }

    // 2. Generate and write config.yaml for Mihomo
    let config_path = state.config_dir.join("config.yaml");
    let yaml_content = ConfigGenerator::generate_yaml(&sub_url, enable_tun, "")
        .map_err(|e| e.to_string())?;

    ConfigGenerator::save_to_file(&config_path, &yaml_content)
        .map_err(|e| e.to_string())?;

    // 3. Start CoreController (Mihomo Sidecar)
    state.core.start(&config_path)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Configure and Launch Rotator Engine
    state.rotator.set_interval(interval_secs);
    state.rotator.set_tun(enable_tun);

    let rotator_arc = Arc::clone(&state.rotator);
    rotator_arc.start(app.clone());

    // 5. Initial health test on nodes
    let rotator_clone = Arc::clone(&state.rotator);
    let app_clone = app.clone();
    tokio::spawn(async move {
        // Wait 1 second for proxy providers to resolve
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        rotator_clone.test_all_nodes(&app_clone).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn cmd_stop_rotation(state: State<'_, AppState>) -> Result<(), String> {
    info!("Command: stop_rotation");
    state.rotator.stop();
    state.core.stop().await.map_err(|e| e.to_string())?;
    flush_dns_cache();
    Ok(())
}

#[tauri::command]
pub async fn cmd_force_switch_now(state: State<'_, AppState>) -> Result<(), String> {
    info!("Command: force_switch_now");
    state.rotator.force_switch();
    Ok(())
}

#[tauri::command]
pub async fn cmd_test_all_nodes(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    info!("Command: test_all_nodes");
    state.rotator.test_all_nodes(&app).await;
    Ok(())
}
