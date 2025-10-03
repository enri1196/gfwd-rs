use relm4::{abstractions::Toaster, prelude::*};

use crate::core::error::{GfwdError, ErrorCategory};
use crate::utils::constants::{TOAST_TIMEOUT_ERROR, TOAST_TIMEOUT_SUCCESS, TOAST_TIMEOUT_WARNING};

#[derive(Debug, Clone)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    // Removed unused Info variant
}

#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub message: String,
    pub toast_type: ToastType,
    pub timeout: Option<u32>, // timeout in seconds, None for no timeout
    pub action_label: Option<String>,
    pub action_target: Option<String>,
}

impl ToastMessage {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            toast_type: ToastType::Success,
            timeout: Some(TOAST_TIMEOUT_SUCCESS),
            action_label: None,
            action_target: None,
        }
    }

    // Removed unused toast constructors - use show_error_toast, show_success_toast, etc. instead

    /// Create a toast from a GfwdError with appropriate styling and recovery suggestions
    pub fn from_error(error: &GfwdError) -> Self {
        let mut toast = Self {
            message: error.user_message(),
            toast_type: match error.category() {
                ErrorCategory::System => ToastType::Error,
                ErrorCategory::Validation => ToastType::Warning,
                ErrorCategory::Operation => ToastType::Error,
            },
            timeout: Some(match error.category() {
                ErrorCategory::System | ErrorCategory::Operation => TOAST_TIMEOUT_ERROR,
                _ => TOAST_TIMEOUT_WARNING,
            }),
            action_label: None,
            action_target: None,
        };

        // Add recovery suggestion as action if available
        if let Some(suggestion) = error.recovery_suggestion() {
            toast.action_label = Some("Help".to_string());
            toast.action_target = Some(suggestion);
        }

        toast
    }

    // Removed unused toast modifier methods
}

/// Helper function to create and show a toast with enhanced accessibility
pub fn show_toast(toaster: &Toaster, message: ToastMessage) {
    let mut toast_builder = adw::Toast::builder().title(&message.message);

    // Set timeout if specified
    if let Some(timeout) = message.timeout {
        toast_builder = toast_builder.timeout(timeout);
    }

    // Add action button if specified
    if let Some(action_label) = &message.action_label {
        toast_builder = toast_builder.button_label(action_label);
        if let Some(action_target) = &message.action_target {
            toast_builder = toast_builder.action_name("toast.help");
            // Store the help text in the toast's action target
            toast_builder = toast_builder.action_target(&relm4::gtk::glib::Variant::from(action_target.as_str()));
        }
    }

    let toast = toast_builder.build();

    // Set priority based on type
    match message.toast_type {
        ToastType::Success => {
            toast.set_priority(adw::ToastPriority::Normal);
        }
        ToastType::Error => {
            toast.set_priority(adw::ToastPriority::High);
        }
        ToastType::Warning => {
            toast.set_priority(adw::ToastPriority::High);
        }
        // Removed unused Info case
    }

    toaster.add_toast(toast);
}

/// Show an error toast from a GfwdError with appropriate styling and recovery suggestions
pub fn show_error_toast(toaster: &Toaster, error: &GfwdError) {
    show_toast(toaster, ToastMessage::from_error(error));
}

/// Show a success toast for completed operations
pub fn show_success_toast(toaster: &Toaster, operation: &str, item: &str) {
    let message = format!("Successfully {} {}", operation, item);
    show_toast(toaster, ToastMessage::success(message));
}

// Removed unused toast helper functions
