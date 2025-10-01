use crate::core::error::GfwdError;
use crate::utils::constants::{MAX_ZONE_NAME_LENGTH, SUPPORTED_PROTOCOLS};

/// Validates a zone name
pub fn validate_zone_name(name: &str) -> Result<String, GfwdError> {
    let name = name.trim();

    if name.is_empty() {
        return Err(GfwdError::Validation(
            "Zone name cannot be empty".to_string(),
        ));
    }

    if name.len() > MAX_ZONE_NAME_LENGTH {
        return Err(GfwdError::Validation(format!(
            "Zone name cannot be longer than {} characters",
            MAX_ZONE_NAME_LENGTH
        )));
    }

    // Check for valid characters (alphanumeric, dash, underscore)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GfwdError::Validation(
            "Zone name can only contain letters, numbers, dashes, and underscores".to_string(),
        ));
    }

    // Cannot start with dash
    if name.starts_with('-') {
        return Err(GfwdError::Validation(
            "Zone name cannot start with a dash".to_string(),
        ));
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
                "Start port cannot be greater than end port".to_string(),
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
    if SUPPORTED_PROTOCOLS.contains(&protocol.as_str()) {
        Ok(protocol)
    } else {
        Err(GfwdError::Validation(format!(
            "Invalid protocol '{}'. Must be one of: {}",
            protocol,
            SUPPORTED_PROTOCOLS.join(", ")
        )))
    }
}

/// Validates a network interface name
pub fn validate_interface_name(interface: &str) -> Result<String, GfwdError> {
    let interface = interface.trim();

    if interface.is_empty() {
        return Err(GfwdError::Validation(
            "Interface name cannot be empty".to_string(),
        ));
    }

    // Interface names should be reasonable length (Linux limit is typically 15 chars)
    if interface.len() > 15 {
        return Err(GfwdError::Validation(
            "Interface name cannot be longer than 15 characters".to_string(),
        ));
    }

    // Check for valid characters (alphanumeric, dash, underscore, dot, colon)
    if !interface
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':')
    {
        return Err(GfwdError::Validation(
            "Interface name can only contain letters, numbers, dashes, underscores, dots, and colons".to_string(),
        ));
    }

    Ok(interface.to_string())
}

/// Validates a source address (IP address or network)
pub fn validate_source_address(source: &str) -> Result<String, GfwdError> {
    let source = source.trim();

    if source.is_empty() {
        return Err(GfwdError::Validation(
            "Source address cannot be empty".to_string(),
        ));
    }

    // Check if it's an IPv4 address or network
    if let Some((ip, prefix)) = source.split_once('/') {
        // Network format (e.g., 192.168.1.0/24)
        validate_ipv4_address(ip.trim())?;
        let prefix_num = prefix
            .trim()
            .parse::<u8>()
            .map_err(|_| GfwdError::Validation(format!("Invalid network prefix: {}", prefix)))?;
        if prefix_num > 32 {
            return Err(GfwdError::Validation(
                "IPv4 network prefix cannot be greater than 32".to_string(),
            ));
        }
        Ok(format!("{}/{}", ip.trim(), prefix_num))
    } else if source.contains(':') {
        // IPv6 address - basic validation
        validate_ipv6_address(source)?;
        Ok(source.to_string())
    } else {
        // Single IPv4 address
        validate_ipv4_address(source)?;
        Ok(source.to_string())
    }
}

/// Validates an IPv4 address
fn validate_ipv4_address(ip: &str) -> Result<(), GfwdError> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(GfwdError::Validation(format!(
            "Invalid IPv4 address: {}",
            ip
        )));
    }

    for part in parts {
        let octet = part
            .parse::<u8>()
            .map_err(|_| GfwdError::Validation(format!("Invalid IPv4 address: {}", ip)))?;
        // All values 0-255 are valid for octets
        let _ = octet;
    }

    Ok(())
}

/// Basic IPv6 address validation
fn validate_ipv6_address(ip: &str) -> Result<(), GfwdError> {
    // Basic IPv6 validation - check for valid characters and structure
    if ip.is_empty() {
        return Err(GfwdError::Validation(
            "IPv6 address cannot be empty".to_string(),
        ));
    }

    // IPv6 addresses contain only hex digits, colons, and possibly dots (for IPv4-mapped)
    if !ip
        .chars()
        .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
    {
        return Err(GfwdError::Validation(format!(
            "Invalid IPv6 address: {}",
            ip
        )));
    }

    // Must contain at least one colon
    if !ip.contains(':') {
        return Err(GfwdError::Validation(format!(
            "Invalid IPv6 address: {}",
            ip
        )));
    }

    // Cannot start or end with a single colon (unless it's :: for compression)
    if (ip.starts_with(':') && !ip.starts_with("::")) || (ip.ends_with(':') && !ip.ends_with("::"))
    {
        return Err(GfwdError::Validation(format!(
            "Invalid IPv6 address: {}",
            ip
        )));
    }

    // Check for invalid triple colon or more
    if ip.contains(":::") {
        return Err(GfwdError::Validation(format!(
            "Invalid IPv6 address: {}",
            ip
        )));
    }

    // Check for multiple :: (compression can only appear once)
    let double_colon_count = ip.matches("::").count();
    if double_colon_count > 1 {
        return Err(GfwdError::Validation(format!(
            "Invalid IPv6 address: {}",
            ip
        )));
    }

    Ok(())
}

/// Helper function to parse a single port number
fn parse_single_port(port: &str) -> Result<u16, GfwdError> {
    let port_num = port
        .parse::<u16>()
        .map_err(|_| GfwdError::Validation(format!("Invalid port number: {}", port)))?;

    if port_num == 0 {
        return Err(GfwdError::Validation("Port cannot be 0".to_string()));
    }

    Ok(port_num)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_interface_name() {
        // Valid interface names
        assert!(validate_interface_name("eth0").is_ok());
        assert!(validate_interface_name("wlan0").is_ok());
        assert!(validate_interface_name("br-123").is_ok());
        assert!(validate_interface_name("veth0.1").is_ok());
        assert!(validate_interface_name("docker0").is_ok());
        assert!(validate_interface_name("tap:1").is_ok());

        // Invalid interface names
        assert!(validate_interface_name("").is_err());
        assert!(validate_interface_name("   ").is_err());
        assert!(validate_interface_name("interface-name-too-long").is_err());
        assert!(validate_interface_name("eth@0").is_err());
        assert!(validate_interface_name("eth 0").is_err());
    }

    #[test]
    fn test_validate_source_address() {
        // Valid IPv4 addresses
        assert!(validate_source_address("192.168.1.1").is_ok());
        assert!(validate_source_address("10.0.0.0").is_ok());
        assert!(validate_source_address("255.255.255.255").is_ok());

        // Valid IPv4 networks
        assert!(validate_source_address("192.168.1.0/24").is_ok());
        assert!(validate_source_address("10.0.0.0/8").is_ok());
        assert!(validate_source_address("172.16.0.0/12").is_ok());

        // Valid IPv6 addresses
        assert!(validate_source_address("::1").is_ok());
        assert!(validate_source_address("2001:db8::1").is_ok());
        assert!(validate_source_address("fe80::1").is_ok());

        // Invalid addresses
        assert!(validate_source_address("").is_err());
        assert!(validate_source_address("   ").is_err());
        assert!(validate_source_address("256.1.1.1").is_err());
        assert!(validate_source_address("192.168.1.1/33").is_err());
        assert!(validate_source_address("192.168.1").is_err());
        assert!(validate_source_address("not-an-ip").is_err());
        assert!(validate_source_address(":").is_err());
        assert!(validate_source_address("::").is_ok()); // This should be valid
    }

    #[test]
    fn test_validate_ipv4_address() {
        // Valid IPv4 addresses
        assert!(validate_ipv4_address("0.0.0.0").is_ok());
        assert!(validate_ipv4_address("192.168.1.1").is_ok());
        assert!(validate_ipv4_address("255.255.255.255").is_ok());

        // Invalid IPv4 addresses
        assert!(validate_ipv4_address("256.1.1.1").is_err());
        assert!(validate_ipv4_address("192.168.1").is_err());
        assert!(validate_ipv4_address("192.168.1.1.1").is_err());
        assert!(validate_ipv4_address("192.168.a.1").is_err());
        assert!(validate_ipv4_address("").is_err());
    }

    #[test]
    fn test_validate_ipv6_address() {
        // Valid IPv6 addresses
        assert!(validate_ipv6_address("::1").is_ok());
        assert!(validate_ipv6_address("2001:db8::1").is_ok());
        assert!(validate_ipv6_address("fe80::1").is_ok());
        assert!(validate_ipv6_address("::").is_ok());
        assert!(validate_ipv6_address("2001:db8:85a3::8a2e:370:7334").is_ok());

        // Invalid IPv6 addresses
        assert!(validate_ipv6_address("").is_err());
        assert!(validate_ipv6_address(":").is_err());
        assert!(validate_ipv6_address(":::").is_err());
        assert!(validate_ipv6_address("2001:db8::1::2").is_err()); // Multiple :: not allowed
        assert!(validate_ipv6_address("2001:db8:85a3::8a2e:370g:7334").is_err()); // Invalid hex
        assert!(validate_ipv6_address("no-colons").is_err());
    }
}
