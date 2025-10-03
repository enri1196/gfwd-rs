use thiserror::Error;

#[derive(Error, Debug)]
pub enum GfwdError {
    #[error("D-Bus communication error: {0}")]
    ZBus(#[from] zbus::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration validation error: {0}")]
    Validation(String),

    // ICMP Management Errors
    #[error("ICMP type error: {0}")]
    IcmpType(String),

    #[error("ICMP block operation failed: {0}")]
    IcmpBlock(String),

    // IP Set Management Errors
    #[error("IP set error: {0}")]
    IPSet(String),

    // Zone Management Errors
    #[error("Zone operation error: {0}")]
    Zone(String),
}

impl GfwdError {
    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            // D-Bus and System Errors
            GfwdError::ZBus(e) => {
                if e.to_string()
                    .contains("org.freedesktop.DBus.Error.ServiceUnknown")
                {
                    "Firewall service is not available. Please start firewalld and try again."
                        .to_string()
                } else if e
                    .to_string()
                    .contains("org.freedesktop.DBus.Error.AccessDenied")
                {
                    "Permission denied. You may need administrator privileges to modify firewall settings.".to_string()
                } else {
                    "Failed to communicate with firewall service. Please check if firewalld is running.".to_string()
                }
            }
            GfwdError::Io(e) => format!("System error: {}", e),

            // Validation Errors
            GfwdError::Validation(msg) => format!("Invalid input: {}", msg),

            // ICMP Errors
            GfwdError::IcmpType(msg) => format!("ICMP type error: {}", msg),
            GfwdError::IcmpBlock(msg) => format!("Failed to manage ICMP block: {}", msg),

            // IP Set Errors
            GfwdError::IPSet(msg) => format!("IP set operation failed: {}", msg),

            // Zone Errors
            GfwdError::Zone(msg) => format!("Zone operation failed: {}", msg),
        }
    }

    /// Returns a recovery suggestion for the error
    pub fn recovery_suggestion(&self) -> Option<String> {
        match self {
            GfwdError::ZBus(e) => {
                if e.to_string()
                    .contains("org.freedesktop.DBus.Error.ServiceUnknown")
                {
                    Some(
                        "Try starting the firewalld service: sudo systemctl start firewalld"
                            .to_string(),
                    )
                } else if e
                    .to_string()
                    .contains("org.freedesktop.DBus.Error.AccessDenied")
                {
                    Some("Run the application with administrator privileges or add your user to the appropriate group".to_string())
                } else {
                    Some(
                        "Check if firewalld is running: sudo systemctl status firewalld"
                            .to_string(),
                    )
                }
            }
            GfwdError::Validation(_) => {
                Some("Check the input format and try again".to_string())
            }
            _ => None,
        }
    }

    /// Returns the error category for UI styling and handling
    pub fn category(&self) -> ErrorCategory {
        match self {
            GfwdError::ZBus(_) | GfwdError::Io(_) => ErrorCategory::System,
            GfwdError::Validation(_) => ErrorCategory::Validation,
            _ => ErrorCategory::Operation,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    System,
    Validation,
    Operation,
}

// Removed unused ErrorCategory methods since they're not being used
