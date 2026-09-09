// ============================================================================
// File: src-tauri/src/infra/system/os_ops.rs
// Purpose: OS-level utilities: Admin elevation check, DNS cache flush
// ============================================================================

use tracing::{info, warn};
use std::process::Command;

/// Check whether the current application process has elevated Administrator privileges
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        use windows::Win32::Foundation::HANDLE;

        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut ret_len = 0u32;
            let success = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut ret_len,
            );

            let _ = windows::Win32::Foundation::CloseHandle(token);

            if success.is_ok() {
                elevation.TokenIsElevated != 0
            } else {
                false
            }
        }
    }

    #[cfg(not(windows))]
    {
        // On Linux/macOS, check if uid == 0 (root)
        unsafe { libc::geteuid() == 0 }
    }
}

/// Flush system DNS cache to avoid stale lookups after IP/tunnel switches
pub fn flush_dns_cache() {
    info!("Flushing OS DNS cache...");
    #[cfg(windows)]
    {
        let _ = Command::new("ipconfig")
            .arg("/flushdns")
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("resolvectl")
            .arg("flush-caches")
            .output();
    }
}
