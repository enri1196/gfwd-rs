use thiserror::Error;

#[cfg(feature = "dbus-backend")]
use zbus::Error as ZBusError;

#[derive(Error, Debug)]
pub enum GfwdError {
    #[cfg(feature = "dbus-backend")]
    #[error("D-Bus communication error: {0}")]
    ZBus(#[from] ZBusError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration validation error: {0}")]
    Validation(String),
}

impl GfwdError {
    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            #[cfg(feature = "dbus-backend")]
            GfwdError::ZBus(_) => "Failed to communicate with firewall service. Please check if firewalld is running.".to_string(),
            GfwdError::Validation(msg) => format!("Invalid input: {}", msg),
            GfwdError::Io(e) => format!("System error: {}", e),
        }
    }
}
