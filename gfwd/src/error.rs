use thiserror::Error;

#[derive(Error, Debug)]
pub enum GfwdError {
    #[error("ZBUS error: {0}")]
    ZBus(#[from] zbus::Error),
    
    // Add other error variants as needed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    // Add more error variants as your application grows
    // #[error("Configuration error: {0}")]
    // Config(String),
}
