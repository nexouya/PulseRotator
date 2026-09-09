// ============================================================================
// File: src-tauri/src/lib.rs
// Purpose: Main Library Entrypoint & Tauri App Builder
// ============================================================================

pub mod app;
pub mod commands;
pub mod domain;
pub mod infra;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::quarantine::QuarantineManager;
use crate::app::rotator::RotatorEngine;
use crate::commands::AppState;
use crate::infra::mihomo::client::MihomoController;
use crate::infra::process::sidecar::SidecarManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,nexuflux_lib=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Resolve application data & config paths
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("./data"));
            std::fs::create_dir_all(&app_data_dir).ok();

            // Locate the packaged Mihomo binary
            let current_exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));

            // Check standard binary search paths
            let candidate_paths = [
                current_exe_dir.join("binaries").join("mihomo-x86_64-pc-windows-msvc.exe"),
                current_exe_dir.join("binaries").join("mihomo.exe"),
                current_exe_dir.join("mihomo.exe"),
                PathBuf::from("src-tauri/binaries/mihomo-x86_64-pc-windows-msvc.exe"),
                PathBuf::from("src-tauri/binaries/mihomo.exe"),
            ];

            let binary_path = candidate_paths
                .iter()
                .find(|p| p.exists())
                .cloned()
                .unwrap_or_else(|| candidate_paths[0].clone());

            tracing::info!("Using Mihomo binary path: {:?}", binary_path);

            // Instantiate Subsystems
            let sidecar_mgr = Arc::new(SidecarManager::new(binary_path));
            let mihomo_client = Arc::new(MihomoController::new(
                "http://127.0.0.1:9090".to_string(),
                None,
                sidecar_mgr,
            ));

            // Default quarantine duration: 10 minutes for failing nodes
            let quarantine_mgr = Arc::new(QuarantineManager::new(Duration::from_secs(600)));
            let rotator_engine = Arc::new(RotatorEngine::new(
                Arc::clone(&mihomo_client) as Arc<dyn domain::traits::CoreController>,
                Arc::clone(&quarantine_mgr),
            ));

            app.manage(AppState {
                core: mihomo_client,
                quarantine: quarantine_mgr,
                rotator: rotator_engine,
                config_dir: app_data_dir,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::cmd_check_admin,
            commands::cmd_fetch_sub,
            commands::cmd_start_rotation,
            commands::cmd_stop_rotation,
            commands::cmd_force_switch_now,
            commands::cmd_test_all_nodes
        ])
        .run(tauri::generate_context!())
        .expect("Error while running PulseRotator application");
}
