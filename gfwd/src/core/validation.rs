use crate::core::error::GfwdError;

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

/// Validates a port number or range string
pub fn validate_port(port: &str) -> Result<String, GfwdError> {
    let port = port.trim();
    
    if port.is_empty() {
        return Err(GfwdError::Validation("Port cannot be empty".to_string()));
    }
    
    // Check if it's a range (contains dash)
    if let Some((start, end)) = port.split_once('-') {
        let start_port = parse_single_port(start.trim())?;
        let end_port = parse_single_port(end.trim())?;
        
        if start_port > end_port {
            return Err(GfwdError::Validation(
                "Start port cannot be greater than end port".to_string()
            ));
        }
        
        Ok(format!("{}-{}", start_port, end_port))
    } else {
        // Single port
        let port_num = parse_single_port(port)?;
        Ok(port_num.to_string())
    }
}

/// Validates a protocol string
pub fn validate_protocol(protocol: &str) -> Result<String, GfwdError> {
    let protocol = protocol.trim().to_lowercase();
    match protocol.as_str() {
        "tcp" | "udp" | "sctp" | "dccp" => Ok(protocol),
        _ => Err(GfwdError::Validation(
            format!("Invalid protocol '{}'. Must be tcp, udp, sctp, or dccp", protocol)
        ))
    }
}

/// Helper function to parse a single port number
fn parse_single_port(port: &str) -> Result<u16, GfwdError> {
    let port_num = port.parse::<u16>()
        .map_err(|_| GfwdError::Validation(format!("Invalid port number: {}", port)))?;
    
    if port_num == 0 {
        return Err(GfwdError::Validation("Port cannot be 0".to_string()));
    }
    
    Ok(port_num)
}