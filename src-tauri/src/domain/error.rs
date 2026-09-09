// ============================================================================
// File: src-tauri/src/domain/error.rs
// Purpose: Strongly-typed Domain and Application Error definitions
// ============================================================================

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Core process error: {0}")]
    CoreProcess(String),

    #[error("REST API error: {0}")]
    Api(String),

    #[error("Subscription parse error: {0}")]
    SubParse(String),

    #[error("All nodes are dead or sleeping. Standby circuit-breaker activated.")]
    CircuitBreakerTripped,

    #[error("Permission denied: Administrator privileges required for Wintun TUN adapter.")]
    AdminPrivilegeRequired,

    #[error("System I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

// Convert AppError to String for Tauri IPC result serialization
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
