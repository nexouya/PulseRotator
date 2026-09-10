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

fn log_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("PulseRotator").join("logs")
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        let payload = info.payload();
        let detail = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic payload".into());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());
        let full = format!(
            "[{}] PANIC: {}\n  detail: {}\n  at: {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg,
            detail,
            location
        );

        let dir = log_dir();
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("panic.log"), &full);

        // Also try next to the executable (useful for portable runs)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let _ = std::fs::write(parent.join("panic.log"), &full);
            }
        }

        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
            let text: Vec<u16> = format!(
                "PulseRotator failed to start.\n\n{}\n\nLog: {}\\panic.log",
                full,
                dir.display()
            )
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
            let caption: Vec<u16> = "PulseRotator Error"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                let _ = MessageBoxW(
                    None,
                    windows::core::PCWSTR(text.as_ptr()),
                    windows::core::PCWSTR(caption.as_ptr()),
                    MB_OK | MB_ICONERROR,
                );
            }
        }

        eprintln!("{}", full);
    }));
}

fn resolve_mihomo_binary(resource_dir: Option<PathBuf>) -> PathBuf {
    let current_exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut candidates: Vec<PathBuf> = Vec::new();

    // Packaged externalBin / sidecar locations (Tauri puts these next to resources)
    candidates.push(current_exe_dir.join("mihomo-x86_64-pc-windows-msvc.exe"));
    candidates.push(current_exe_dir.join("mihomo.exe"));
    candidates.push(current_exe_dir.join("binaries").join("mihomo-x86_64-pc-windows-msvc.exe"));
    candidates.push(current_exe_dir.join("binaries").join("mihomo.exe"));

    if let Some(res) = &resource_dir {
        candidates.push(res.join("mihomo-x86_64-pc-windows-msvc.exe"));
        candidates.push(res.join("mihomo.exe"));
        candidates.push(res.join("binaries").join("mihomo-x86_64-pc-windows-msvc.exe"));
        candidates.push(res.join("binaries").join("mihomo.exe"));
    }

    // Dev from project root / src-tauri
    candidates.push(PathBuf::from("src-tauri/binaries/mihomo-x86_64-pc-windows-msvc.exe"));
    candidates.push(PathBuf::from("src-tauri/binaries/mihomo.exe"));
    candidates.push(current_exe_dir.join("../../src-tauri/binaries/mihomo.exe"));

    candidates
        .iter()
        .find(|p| p.is_file())
        .cloned()
        .unwrap_or_else(|| {
            candidates
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("mihomo.exe"))
        })
}

fn resolve_wintun_dll(mihomo_path: &std::path::Path, resource_dir: Option<PathBuf>) -> Option<PathBuf> {
    let current_exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))?;

    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.push(current_exe_dir.join("wintun.dll"));
    candidates.push(current_exe_dir.join("binaries").join("wintun.dll"));
    if let Some(res) = &resource_dir {
        candidates.push(res.join("wintun.dll"));
        candidates.push(res.join("binaries").join("wintun.dll"));
    }
    if let Some(mihomo_dir) = mihomo_path.parent() {
        candidates.push(mihomo_dir.join("wintun.dll"));
        candidates.push(mihomo_dir.join("binaries").join("wintun.dll"));
    }
    candidates.push(PathBuf::from("src-tauri/binaries/wintun.dll"));

    candidates.into_iter().find(|p| p.is_file())
}

fn init_file_logger() -> Option<PathBuf> {
    let dir = log_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let log_path = dir.join("app.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,nexuflux_lib=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(file))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Some(log_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_panic_hook();
    let log_path = init_file_logger();
    if let Some(p) = &log_path {
        tracing::info!("PulseRotator starting. Log file: {:?}", p);
    }

    let run_result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./data"));
            std::fs::create_dir_all(&app_data_dir).ok();

            let resource_dir = app.path().resource_dir().ok();
            let binary_path = resolve_mihomo_binary(resource_dir.clone());
            tracing::info!("Using Mihomo binary path: {:?}", binary_path);

            if let Some(wintun) = resolve_wintun_dll(&binary_path, resource_dir) {
                if let Some(mihomo_dir) = binary_path.parent() {
                    let target = mihomo_dir.join("wintun.dll");
                    if !target.exists() {
                        if let Err(e) = std::fs::copy(&wintun, &target) {
                            tracing::warn!("Could not copy wintun.dll next to mihomo: {}", e);
                        } else {
                            tracing::info!("Copied wintun.dll to {:?}", target);
                        }
                    }
                }
            } else {
                tracing::warn!("wintun.dll not found — TUN mode will fail until it is present");
            }

            if !binary_path.exists() {
                tracing::error!(
                    "Mihomo binary missing. App UI will open but rotation cannot start. Looked near {:?}",
                    binary_path
                );
            }

            let sidecar_mgr = Arc::new(SidecarManager::new(binary_path));
            let mihomo_client = Arc::new(MihomoController::new(
                "http://127.0.0.1:9090".to_string(),
                None,
                sidecar_mgr,
            ));

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
            commands::cmd_test_all_nodes,
            commands::cmd_get_diagnostics
        ])
        .run(tauri::generate_context!());

    if let Err(e) = run_result {
        let msg = format!("Failed to start PulseRotator window: {e}");
        tracing::error!("{}", msg);
        let dir = log_dir();
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            dir.join("panic.log"),
            format!("[{}] {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg),
        );
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
            let text: Vec<u16> = format!("{}\n\nLog: {}\\panic.log", msg, dir.display())
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let caption: Vec<u16> = "PulseRotator Error"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                let _ = MessageBoxW(
                    None,
                    windows::core::PCWSTR(text.as_ptr()),
                    windows::core::PCWSTR(caption.as_ptr()),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
        std::process::exit(1);
    }
}
