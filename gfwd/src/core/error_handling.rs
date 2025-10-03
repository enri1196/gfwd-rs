use crate::core::error::GfwdError;

/// Helper functions for creating specific error types with consistent messaging

impl GfwdError {
    // Zone Error Helpers (currently used)
    pub fn zone_already_exists(zone: &str) -> Self {
        Self::Zone(format!("Zone '{}' already exists", zone))
    }
}

/// Utility functions for error handling in async operations
pub mod async_helpers {
    use super::*;
    // Removed unused toast imports

    // Removed unused async helper functions - use log_error directly instead

    /// Log an error with appropriate log level based on error category
    pub fn log_error(error: &GfwdError, context: &str) {
        use relm4::gtk::glib::{self, LogLevel};

        let log_level = match error.category() {
            crate::core::error::ErrorCategory::System => LogLevel::Critical,
            crate::core::error::ErrorCategory::Operation => LogLevel::Error,
            crate::core::error::ErrorCategory::Validation => LogLevel::Warning,
        };

        glib::g_log!(log_level, "{}: {}", context, error);
    }
}

/// Validation error helpers for consistent error messages
pub mod validation_helpers {
    use super::*;

    pub fn invalid_port_range(range: &str) -> GfwdError {
        GfwdError::Validation(format!(
            "Invalid port range '{}'. Use format 'start-end' where both are valid port numbers",
            range
        ))
    }

    pub fn invalid_protocol(protocol: &str) -> GfwdError {
        GfwdError::Validation(format!(
            "Invalid protocol '{}'. Supported protocols are: tcp, udp, sctp, dccp",
            protocol
        ))
    }

    pub fn invalid_ip_address(address: &str) -> GfwdError {
        GfwdError::Validation(format!(
            "Invalid IP address '{}'. Must be a valid IPv4 or IPv6 address",
            address
        ))
    }

    pub fn empty_field(field_name: &str) -> GfwdError {
        GfwdError::Validation(format!("{} cannot be empty", field_name))
    }

    pub fn field_too_long(field_name: &str, max_length: usize) -> GfwdError {
        GfwdError::Validation(format!(
            "{} cannot be longer than {} characters",
            field_name, max_length
        ))
    }

    pub fn invalid_characters(field_name: &str, allowed: &str) -> GfwdError {
        GfwdError::Validation(format!(
            "{} contains invalid characters. Allowed characters: {}",
            field_name, allowed
        ))
    }
}
