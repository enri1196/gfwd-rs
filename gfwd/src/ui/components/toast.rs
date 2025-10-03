use relm4::{abstractions::Toaster, prelude::*};

use crate::utils::constants::{TOAST_TIMEOUT_ERROR, TOAST_TIMEOUT_SUCCESS};

#[derive(Debug, Clone)]
pub enum ToastType {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub message: String,
    pub toast_type: ToastType,
    pub timeout: Option<u32>, // timeout in seconds, None for no timeout
}

impl ToastMessage {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            toast_type: ToastType::Success,
            timeout: Some(TOAST_TIMEOUT_SUCCESS),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            toast_type: ToastType::Error,
            timeout: Some(TOAST_TIMEOUT_ERROR),
        }
    }
}

/// Helper function to create and show a toast with enhanced accessibility
pub fn show_toast(toaster: &Toaster, message: ToastMessage) {
    let mut toast_builder = adw::Toast::builder().title(&message.message);

    // Set timeout if specified
    if let Some(timeout) = message.timeout {
        toast_builder = toast_builder.timeout(timeout);
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
    }

    toaster.add_toast(toast);
}
