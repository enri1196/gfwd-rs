use crate::error::GfwdError;

/// Validates a zone name
pub fn validate_zone_name(name: &str) -> Result<String, GfwdError> {
    let name = name.trim();
    
    if name.is_empty() {
        return Err(GfwdError::Validation("Zone name cannot be empty".to_string()));
    }
    
    if name.len() > 17 {
        return Err(GfwdError::Validation("Zone name cannot be longer than 17 characters".to_string()));
    }
    
    // Check for valid characters (alphanumeric, dash, underscore)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(GfwdError::Validation(
            "Zone name can only contain letters, numbers, dashes, and underscores".to_string()
        ));
    }
    
    // Cannot start with dash
    if name.starts_with('-') {
        return Err(GfwdError::Validation("Zone name cannot start with a dash".to_string()));
    }
    
    Ok(name.to_string())
}
