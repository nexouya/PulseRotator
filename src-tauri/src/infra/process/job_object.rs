// ============================================================================
// File: src-tauri/src/infra/process/job_object.rs
// Purpose: Windows Job Object implementation to guarantee ZERO zombie processes
// Note: When PulseRotator exits, Windows OS kernel automatically kills the child.
// ============================================================================

#[cfg(windows)]
pub mod win {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
        JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    pub struct ProcessJob {
        handle: HANDLE,
    }

    // Windows HANDLE contains *mut c_void, so we explicitly mark ProcessJob as Send + Sync
    // It is completely safe because access is synchronized via Mutex inside SidecarManager.
    unsafe impl Send for ProcessJob {}
    unsafe impl Sync for ProcessJob {}

    impl ProcessJob {
        pub fn new() -> Result<Self, String> {
            unsafe {
                let handle = CreateJobObjectW(None, None)
                    .map_err(|e| format!("CreateJobObjectW failed: {:?}", e))?;

                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
                .map_err(|e| format!("SetInformationJobObject failed: {:?}", e))?;

                Ok(Self { handle })
            }
        }

        pub fn assign_child(&self, child: &Child) -> Result<(), String> {
            unsafe {
                let raw_handle = HANDLE(child.as_raw_handle() as *mut _);
                AssignProcessToJobObject(self.handle, raw_handle)
                    .map_err(|e| format!("AssignProcessToJobObject failed: {:?}", e))?;
                Ok(())
            }
        }
    }

    impl Drop for ProcessJob {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}
