// Application constants
pub const APP_ID: &str = "com.github.Gfwd";
pub const APP_NAME: &str = "GFWD";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Window dimensions
pub const DEFAULT_WINDOW_WIDTH: i32 = 1280;
pub const DEFAULT_WINDOW_HEIGHT: i32 = 720;
pub const SIDEBAR_WIDTH: i32 = 250;

// Validation limits
pub const MAX_ZONE_NAME_LENGTH: usize = 17;

// Timeouts (in seconds)
pub const TOAST_TIMEOUT_SUCCESS: u32 = 3;
pub const TOAST_TIMEOUT_ERROR: u32 = 8;
pub const TOAST_TIMEOUT_WARNING: u32 = 5;

// Protocols
pub const SUPPORTED_PROTOCOLS: &[&str] = &["tcp", "udp", "sctp", "dccp"];
