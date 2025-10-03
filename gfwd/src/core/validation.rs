use crate::core::error::GfwdError;
use crate::core::error_handling::validation_helpers;
use crate::utils::constants::{MAX_ZONE_NAME_LENGTH, SUPPORTED_PROTOCOLS};

/// Validates a zone name
pub fn validate_zone_name(name: &str) -> Result<String, GfwdError> {
    let name = name.trim();

    if name.is_empty() {
        return Err(validation_helpers::empty_field("Zone name"));
    }

    if name.len() > MAX_ZONE_NAME_LENGTH {
        return Err(validation_helpers::field_too_long("Zone name", MAX_ZONE_NAME_LENGTH));
    }

    // Check for valid characters (alphanumeric, dash, underscore)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(validation_helpers::invalid_characters("Zone name", "letters, numbers, dashes, and underscores"));
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
        return Err(validation_helpers::empty_field("Port"));
    }

    // Check if it's a range (contains dash)
    if let Some((start, end)) = port.split_once('-') {
        let start_port = parse_single_port(start.trim())?;
        let end_port = parse_single_port(end.trim())?;

        if start_port > end_port {
            return Err(validation_helpers::invalid_port_range(&format!("{}-{}", start, end)));
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
        Err(validation_helpers::invalid_protocol(&protocol))
    }
}

/// Validates a network interface name
pub fn validate_interface_name(interface: &str) -> Result<String, GfwdError> {
    let interface = interface.trim();

    if interface.is_empty() {
        return Err(validation_helpers::empty_field("Interface name"));
    }

    // Interface names should be reasonable length (Linux limit is typically 15 chars)
    if interface.len() > 15 {
        return Err(validation_helpers::field_too_long("Interface name", 15));
    }

    // Check for valid characters (alphanumeric, dash, underscore, dot, colon)
    if !interface
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':')
    {
        return Err(validation_helpers::invalid_characters("Interface name", "letters, numbers, dashes, underscores, dots, and colons"));
    }

    Ok(interface.to_string())
}

/// Validates a source address (IP address or network)
pub fn validate_source_address(source: &str) -> Result<String, GfwdError> {
    let source = source.trim();

    if source.is_empty() {
        return Err(validation_helpers::empty_field("Source address"));
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
            .map_err(|_| validation_helpers::invalid_ip_address(ip))?;
        // All values 0-255 are valid for octets
        let _ = octet;
    }

    Ok(())
}

/// Basic IPv6 address validation
fn validate_ipv6_address(ip: &str) -> Result<(), GfwdError> {
    // Basic IPv6 validation - check for valid characters and structure
    if ip.is_empty() {
        return Err(validation_helpers::empty_field("IPv6 address"));
    }

    // IPv6 addresses contain only hex digits, colons, and possibly dots (for IPv4-mapped)
    if !ip
        .chars()
        .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
    {
        return Err(validation_helpers::invalid_ip_address(ip));
    }

    // Must contain at least one colon
    if !ip.contains(':') {
        return Err(validation_helpers::invalid_ip_address(ip));
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

/// Validates an IP set name
pub fn validate_ipset_name(name: &str) -> Result<String, GfwdError> {
    let name = name.trim();

    if name.is_empty() {
        return Err(GfwdError::Validation(
            "IP set name cannot be empty".to_string(),
        ));
    }

    // IP set names should be reasonable length
    if name.len() > 31 {
        return Err(GfwdError::Validation(
            "IP set name cannot be longer than 31 characters".to_string(),
        ));
    }

    // Check for valid characters (alphanumeric, dash, underscore)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GfwdError::Validation(
            "IP set name can only contain letters, numbers, dashes, and underscores".to_string(),
        ));
    }

    // Cannot start with dash
    if name.starts_with('-') {
        return Err(GfwdError::Validation(
            "IP set name cannot start with a dash".to_string(),
        ));
    }

    Ok(name.to_string())
}

/// Validates an IP set type
pub fn validate_ipset_type(ipset_type: &str) -> Result<String, GfwdError> {
    let ipset_type = ipset_type.trim();
    
    // Common IP set types supported by firewalld
    let valid_types = [
        "hash:ip", "hash:net", "hash:ip,port", "hash:net,port",
        "hash:ip,port,ip", "hash:ip,port,net", "hash:net,port,net",
        "hash:net,iface", "hash:mac", "bitmap:ip", "bitmap:ip,mac",
        "bitmap:port", "list:set"
    ];

    if valid_types.contains(&ipset_type) {
        Ok(ipset_type.to_string())
    } else {
        Err(GfwdError::Validation(format!(
            "Invalid IP set type '{}'. Must be one of: {}",
            ipset_type,
            valid_types.join(", ")
        )))
    }
}

/// Validates an IP set entry based on the IP set type
pub fn validate_ipset_entry(entry: &str, ipset_type: &str) -> Result<String, GfwdError> {
    let entry = entry.trim();

    if entry.is_empty() {
        return Err(GfwdError::Validation(
            "IP set entry cannot be empty".to_string(),
        ));
    }

    match ipset_type {
        "hash:ip" => {
            // Single IP address
            if entry.contains(':') {
                validate_ipv6_address(entry)?;
            } else {
                validate_ipv4_address(entry)?;
            }
        }
        "hash:net" => {
            // Network address (IP/prefix)
            validate_source_address(entry)?;
        }
        "hash:ip,port" => {
            // IP address and port (e.g., "192.168.1.1,80")
            if let Some((ip, port)) = entry.split_once(',') {
                if ip.trim().contains(':') {
                    validate_ipv6_address(ip.trim())?;
                } else {
                    validate_ipv4_address(ip.trim())?;
                }
                validate_port(port.trim())?;
            } else {
                return Err(GfwdError::Validation(
                    "IP,port entry must contain comma-separated IP and port".to_string(),
                ));
            }
        }
        "hash:net,port" => {
            // Network and port (e.g., "192.168.1.0/24,80")
            if let Some((net, port)) = entry.split_once(',') {
                validate_source_address(net.trim())?;
                validate_port(port.trim())?;
            } else {
                return Err(GfwdError::Validation(
                    "Network,port entry must contain comma-separated network and port".to_string(),
                ));
            }
        }
        "hash:mac" => {
            // MAC address validation
            validate_mac_address(entry)?;
        }
        "bitmap:ip" | "bitmap:port" | "list:set" => {
            // For bitmap and list types, accept any non-empty string
            // More specific validation would require knowledge of the set's range/members
            if entry.is_empty() {
                return Err(GfwdError::Validation(
                    "Entry cannot be empty".to_string(),
                ));
            }
        }
        _ => {
            // For other types, perform basic validation
            if entry.contains(',') {
                // Multi-part entry, validate each part as IP or port
                let parts: Vec<&str> = entry.split(',').collect();
                for part in parts {
                    let part = part.trim();
                    if part.contains(':') {
                        validate_ipv6_address(part)?;
                    } else if part.contains('.') {
                        validate_ipv4_address(part)?;
                    } else if part.parse::<u16>().is_ok() {
                        validate_port(part)?;
                    }
                    // Allow other formats for complex types
                }
            } else if entry.contains(':') {
                validate_ipv6_address(entry)?;
            } else if entry.contains('.') {
                validate_ipv4_address(entry)?;
            }
            // Allow other formats for complex types
        }
    }

    Ok(entry.to_string())
}

/// Validates a MAC address
fn validate_mac_address(mac: &str) -> Result<(), GfwdError> {
    let mac = mac.trim();
    
    // MAC address should be in format XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX
    let parts: Vec<&str> = if mac.contains(':') {
        mac.split(':').collect()
    } else if mac.contains('-') {
        mac.split('-').collect()
    } else {
        return Err(GfwdError::Validation(format!(
            "Invalid MAC address format: {}. Use XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX",
            mac
        )));
    };

    if parts.len() != 6 {
        return Err(GfwdError::Validation(format!(
            "Invalid MAC address: {}. Must have 6 parts",
            mac
        )));
    }

    for part in parts {
        if part.len() != 2 {
            return Err(GfwdError::Validation(format!(
                "Invalid MAC address: {}. Each part must be 2 hex digits",
                mac
            )));
        }
        
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GfwdError::Validation(format!(
                "Invalid MAC address: {}. Must contain only hex digits",
                mac
            )));
        }
    }

    Ok(())
}

/// Validates a rich rule XML string
pub fn validate_rich_rule_xml(xml: &str) -> Result<String, GfwdError> {
    let xml = xml.trim();

    if xml.is_empty() {
        return Err(GfwdError::Validation(
            "Rich rule XML cannot be empty".to_string(),
        ));
    }

    // Basic XML structure validation
    if !xml.starts_with("<rule") {
        return Err(GfwdError::Validation(
            "Rich rule must start with <rule".to_string(),
        ));
    }

    if !xml.ends_with("</rule>") {
        return Err(GfwdError::Validation(
            "Rich rule must end with </rule>".to_string(),
        ));
    }

    // Check for required action element
    let has_action = xml.contains("<accept") || 
                    xml.contains("<reject") || 
                    xml.contains("<drop") || 
                    xml.contains("<mark");

    if !has_action {
        return Err(GfwdError::Validation(
            "Rich rule must contain an action (accept, reject, drop, or mark)".to_string(),
        ));
    }

    // Validate that XML is well-formed by checking basic structure
    let mut tag_stack = Vec::new();
    let mut in_tag = false;
    let mut current_tag = String::new();

    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                current_tag.clear();
            }
            '>' => {
                if in_tag {
                    in_tag = false;
                    if current_tag.starts_with('/') {
                        // Closing tag
                        let tag_name = &current_tag[1..];
                        if let Some(last_tag) = tag_stack.pop() {
                            if last_tag != tag_name {
                                return Err(GfwdError::Validation(format!(
                                    "Mismatched XML tags: expected </{}>", last_tag
                                )));
                            }
                        }
                    } else if !current_tag.ends_with('/') {
                        // Opening tag (not self-closing)
                        let tag_name = current_tag.split_whitespace().next().unwrap_or("");
                        if !tag_name.is_empty() {
                            tag_stack.push(tag_name.to_string());
                        }
                    }
                }
            }
            _ => {
                if in_tag {
                    current_tag.push(ch);
                }
            }
        }
    }

    if !tag_stack.is_empty() {
        return Err(GfwdError::Validation(
            "Rich rule XML has unclosed tags".to_string(),
        ));
    }

    Ok(xml.to_string())
}

/// Validates rich rule components for logical consistency
pub fn validate_rich_rule_logic(
    source: Option<&str>,
    destination: Option<&str>, 
    service: Option<&str>,
    port: Option<(&str, &str)>,
    protocol: Option<&str>,
) -> Result<(), GfwdError> {
    // Validate source address if provided
    if let Some(src) = source {
        validate_source_address(src)?;
    }

    // Validate destination address if provided
    if let Some(dest) = destination {
        validate_source_address(dest)?;
    }

    // Validate port if provided
    if let Some((port_str, protocol_str)) = port {
        validate_port(port_str)?;
        validate_protocol(protocol_str)?;
    }

    // Validate protocol if provided
    if let Some(proto) = protocol {
        validate_protocol(proto)?;
    }

    // Check for conflicting specifications
    if service.is_some() && port.is_some() {
        return Err(GfwdError::Validation(
            "Rich rule cannot specify both service and port".to_string(),
        ));
    }

    if service.is_some() && protocol.is_some() {
        return Err(GfwdError::Validation(
            "Rich rule cannot specify both service and protocol".to_string(),
        ));
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

    #[test]
    fn test_validate_ipset_name() {
        // Valid IP set names
        assert!(validate_ipset_name("my_ipset").is_ok());
        assert!(validate_ipset_name("ipset-1").is_ok());
        assert!(validate_ipset_name("test123").is_ok());
        assert!(validate_ipset_name("a").is_ok());

        // Invalid IP set names
        assert!(validate_ipset_name("").is_err());
        assert!(validate_ipset_name("   ").is_err());
        assert!(validate_ipset_name("-invalid").is_err());
        assert!(validate_ipset_name("invalid@name").is_err());
        assert!(validate_ipset_name("name with spaces").is_err());
        assert!(validate_ipset_name("very_long_ipset_name_that_exceeds_limit").is_err());
    }

    #[test]
    fn test_validate_ipset_type() {
        // Valid IP set types
        assert!(validate_ipset_type("hash:ip").is_ok());
        assert!(validate_ipset_type("hash:net").is_ok());
        assert!(validate_ipset_type("hash:ip,port").is_ok());
        assert!(validate_ipset_type("hash:mac").is_ok());
        assert!(validate_ipset_type("bitmap:ip").is_ok());
        assert!(validate_ipset_type("list:set").is_ok());

        // Invalid IP set types
        assert!(validate_ipset_type("invalid:type").is_err());
        assert!(validate_ipset_type("").is_err());
        assert!(validate_ipset_type("hash").is_err());
    }

    #[test]
    fn test_validate_ipset_entry() {
        // hash:ip entries
        assert!(validate_ipset_entry("192.168.1.1", "hash:ip").is_ok());
        assert!(validate_ipset_entry("::1", "hash:ip").is_ok());
        assert!(validate_ipset_entry("invalid", "hash:ip").is_err());

        // hash:net entries
        assert!(validate_ipset_entry("192.168.1.0/24", "hash:net").is_ok());
        assert!(validate_ipset_entry("10.0.0.0/8", "hash:net").is_ok());
        assert!(validate_ipset_entry("192.168.1.1", "hash:net").is_ok()); // Single IP is valid
        assert!(validate_ipset_entry("invalid/24", "hash:net").is_err());

        // hash:ip,port entries
        assert!(validate_ipset_entry("192.168.1.1,80", "hash:ip,port").is_ok());
        assert!(validate_ipset_entry("::1,443", "hash:ip,port").is_ok());
        assert!(validate_ipset_entry("192.168.1.1", "hash:ip,port").is_err()); // Missing port
        assert!(validate_ipset_entry("invalid,80", "hash:ip,port").is_err());

        // hash:mac entries
        assert!(validate_ipset_entry("aa:bb:cc:dd:ee:ff", "hash:mac").is_ok());
        assert!(validate_ipset_entry("AA-BB-CC-DD-EE-FF", "hash:mac").is_ok());
        assert!(validate_ipset_entry("invalid-mac", "hash:mac").is_err());
    }

    #[test]
    fn test_validate_mac_address() {
        // Valid MAC addresses
        assert!(validate_mac_address("aa:bb:cc:dd:ee:ff").is_ok());
        assert!(validate_mac_address("AA:BB:CC:DD:EE:FF").is_ok());
        assert!(validate_mac_address("00:11:22:33:44:55").is_ok());
        assert!(validate_mac_address("aa-bb-cc-dd-ee-ff").is_ok());

        // Invalid MAC addresses
        assert!(validate_mac_address("").is_err());
        assert!(validate_mac_address("aa:bb:cc:dd:ee").is_err()); // Too few parts
        assert!(validate_mac_address("aa:bb:cc:dd:ee:ff:gg").is_err()); // Too many parts
        assert!(validate_mac_address("aa:bb:cc:dd:ee:gg").is_err()); // Invalid hex
        assert!(validate_mac_address("a:bb:cc:dd:ee:ff").is_err()); // Part too short
        assert!(validate_mac_address("aaa:bb:cc:dd:ee:ff").is_err()); // Part too long
        assert!(validate_mac_address("aabbccddeeff").is_err()); // No separators
    }

    #[test]
    fn test_validate_rich_rule_xml() {
        // Valid rich rule XML
        assert!(validate_rich_rule_xml("<rule><accept/></rule>").is_ok());
        assert!(validate_rich_rule_xml("<rule family=\"ipv4\"><source address=\"192.168.1.0/24\"/><accept/></rule>").is_ok());
        assert!(validate_rich_rule_xml("<rule><port port=\"80\" protocol=\"tcp\"/><reject/></rule>").is_ok());
        assert!(validate_rich_rule_xml("<rule><service name=\"ssh\"/><drop/></rule>").is_ok());

        // Invalid rich rule XML
        assert!(validate_rich_rule_xml("").is_err());
        assert!(validate_rich_rule_xml("not xml").is_err());
        assert!(validate_rich_rule_xml("<rule>no action</rule>").is_err());
        assert!(validate_rich_rule_xml("<rule><accept/>").is_err()); // Unclosed
        assert!(validate_rich_rule_xml("rule><accept/></rule>").is_err()); // No opening <
        assert!(validate_rich_rule_xml("<rule><accept/></wrong>").is_err()); // Mismatched tags
    }

    #[test]
    fn test_validate_rich_rule_logic() {
        // Valid combinations
        assert!(validate_rich_rule_logic(
            Some("192.168.1.0/24"), 
            None, 
            Some("ssh"), 
            None, 
            None
        ).is_ok());
        
        assert!(validate_rich_rule_logic(
            None, 
            Some("10.0.0.1"), 
            None, 
            Some(("80", "tcp")), 
            None
        ).is_ok());

        assert!(validate_rich_rule_logic(
            None, 
            None, 
            None, 
            None, 
            Some("tcp")
        ).is_ok());

        // Invalid combinations
        assert!(validate_rich_rule_logic(
            None, 
            None, 
            Some("ssh"), 
            Some(("80", "tcp")), 
            None
        ).is_err()); // Service and port conflict

        assert!(validate_rich_rule_logic(
            None, 
            None, 
            Some("ssh"), 
            None, 
            Some("tcp")
        ).is_err()); // Service and protocol conflict

        // Invalid addresses
        assert!(validate_rich_rule_logic(
            Some("invalid-ip"), 
            None, 
            None, 
            None, 
            None
        ).is_err());
    }
}
