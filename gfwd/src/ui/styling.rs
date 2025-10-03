// UI Styling and Accessibility Constants

/// Standard CSS classes for consistent styling across the application
pub mod css_classes {
    // Button styling
    pub const FLAT: &str = "flat";
    pub const SUGGESTED_ACTION: &str = "suggested-action";
    pub const DESTRUCTIVE_ACTION: &str = "destructive-action";

    // State styling
    pub const ACCENT: &str = "accent";
    pub const SUCCESS: &str = "success";
    pub const WARNING: &str = "warning";
    pub const ERROR: &str = "error";

    // Layout styling (available for future use)
    #[allow(dead_code)]
    pub const BOXED_LIST: &str = "boxed-list";
    #[allow(dead_code)]
    pub const CAPTION: &str = "caption";
    #[allow(dead_code)]
    pub const DIM_LABEL: &str = "dim-label";
    #[allow(dead_code)]
    pub const TAG: &str = "tag";
}

/// Standard icon names for consistent iconography
pub mod icons {
    // Network and connectivity
    pub const NETWORK_WIRED: &str = "network-wired-symbolic";
    #[allow(dead_code)]
    pub const NETWORK_SERVER: &str = "network-server-symbolic";
    #[allow(dead_code)]
    pub const NETWORK_WIRELESS: &str = "network-wireless-symbolic";
    #[allow(dead_code)]
    pub const NETWORK_CELLULAR: &str = "network-cellular-signal-good-symbolic";

    // Security and firewall
    pub const SECURITY_HIGH: &str = "security-high-symbolic";
    #[allow(dead_code)]
    pub const SECURITY_MEDIUM: &str = "security-medium-symbolic";
    #[allow(dead_code)]
    pub const SECURITY_LOW: &str = "security-low-symbolic";

    // Actions
    #[allow(dead_code)]
    pub const ADD: &str = "list-add-symbolic";
    pub const REMOVE: &str = "user-trash-symbolic";
    #[allow(dead_code)]
    pub const REFRESH: &str = "view-refresh-symbolic";
    pub const EDIT: &str = "document-edit-symbolic";

    // Status and feedback
    pub const OK: &str = "emblem-ok-symbolic";
    pub const WARNING: &str = "dialog-warning-symbolic";
    pub const ERROR: &str = "dialog-error-symbolic";
    pub const INFO: &str = "dialog-information-symbolic";

    // System and applications
    #[allow(dead_code)]
    pub const APPLICATIONS_SYSTEM: &str = "applications-system-symbolic";
    pub const PREFERENCES_SYSTEM: &str = "preferences-system-symbolic";
    #[allow(dead_code)]
    pub const PREFERENCES_NETWORK: &str = "preferences-system-network-symbolic";

    // Navigation and organization
    #[allow(dead_code)]
    pub const FOLDER: &str = "folder-symbolic";
    pub const GO_JUMP: &str = "go-jump-symbolic";
    pub const OBJECT_SELECT: &str = "object-select-symbolic";
    #[allow(dead_code)]
    pub const OBJECT_FLIP: &str = "object-flip-horizontal-symbolic";

    // Marks and emphasis
    #[allow(dead_code)]
    pub const EMBLEM_IMPORTANT: &str = "emblem-important-symbolic";
    #[allow(dead_code)]
    pub const MARK_LOCATION: &str = "mark-location-symbolic";
}
