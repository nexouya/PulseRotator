// ============================================================================
// File: src-tauri/src/infra/process/sidecar.rs
// Purpose: Multiplatform Process Lifecycle Manager with Auto Cleanup
// ============================================================================

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, warn, error};
use crate::domain::error::AppError;

#[cfg(windows)]
use crate::infra::process::job_object::win::ProcessJob;

pub struct SidecarManager {
    child_process: Arc<Mutex<Option<Child>>>,
    binary_path: PathBuf,
    is_running: Arc<AtomicBool>,
    #[cfg(windows)]
    job_object: Arc<Mutex<Option<ProcessJob>>>,
}

impl SidecarManager {
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            child_process: Arc::new(Mutex::new(None)),
            binary_path,
            is_running: Arc::new(AtomicBool::new(false)),
            #[cfg(windows)]
            job_object: Arc::new(Mutex::new(None)),
        }
    }

    /// Spawn the mihomo sidecar with working directory and config file
    pub fn spawn(&self, work_dir: &Path, config_path: &Path) -> Result<(), AppError> {
        self.terminate()?;

        info!("Spawning Mihomo sidecar: {:?}", self.binary_path);
        info!("Working directory: {:?}", work_dir);
        info!("Config path: {:?}", config_path);

        if !self.binary_path.exists() {
            return Err(AppError::CoreProcess(format!(
                "Mihomo executable not found at: {:?}",
                self.binary_path
            )));
        }

        #[cfg(windows)]
        let job = ProcessJob::new().map_err(|e| AppError::CoreProcess(e))?;

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("-d").arg(work_dir);
        cmd.arg("-f").arg(config_path);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW = 0x08000000
            cmd.creation_flags(0x08000000);
        }

        let child = cmd.spawn().map_err(|e| {
            AppError::CoreProcess(format!("Failed to spawn mihomo process: {}", e))
        })?;

        #[cfg(windows)]
        {
            let _ = job.assign_child(&child);
            *self.job_object.lock() = Some(job);
        }

        *self.child_process.lock() = Some(child);
        self.is_running.store(true, Ordering::SeqCst);
        info!("Mihomo sidecar spawned successfully");

        Ok(())
    }

    /// Terminate running core process cleanly
    pub fn terminate(&self) -> Result<(), AppError> {
        let mut lock = self.child_process.lock();
        if let Some(mut child) = lock.take() {
            info!("Killing Mihomo core process...");
            let _ = child.kill();
            let _ = child.wait();
        }
        self.is_running.store(false, Ordering::SeqCst);
        #[cfg(windows)]
        {
            *self.job_object.lock() = None;
        }
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        let mut lock = self.child_process.lock();
        if let Some(child) = lock.as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                Ok(Some(status)) => {
                    warn!("Mihomo process exited with status: {}", status);
                    self.is_running.store(false, Ordering::SeqCst);
                    false
                }
                Err(e) => {
                    error!("Error checking mihomo process: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
