use std::net::IpAddr;

/// A client-side validation failure that can be localized by the UI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// A required value is empty.
    Required,
    /// A port is not an integer in `1..=65535`.
    InvalidPort,
    /// A port range has its start after its end.
    ReversedPortRange,
    /// A protocol is not supported by firewalld's port rules.
    InvalidProtocol,
    /// A value is not an IPv4 or IPv6 address.
    InvalidIpAddress,
    /// An interface name is longer than Linux permits.
    InterfaceNameTooLong,
    /// An interface name contains unsupported characters.
    InvalidInterfaceName,
    /// A source is not an IP address or network.
    InvalidSource,
    /// A CIDR prefix is outside the address family's valid range.
    InvalidCidrPrefix,
}

/// Validates a Linux network-interface name.
pub fn validate_interface_name(interface: &str) -> Result<(), ValidationError> {
    let interface = interface.trim();

    if interface.is_empty() {
        return Err(ValidationError::Required);
    }

    if interface.len() > 15 {
        return Err(ValidationError::InterfaceNameTooLong);
    }

    if !interface
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err(ValidationError::InvalidInterfaceName);
    }

    Ok(())
}

/// Validates an IPv4/IPv6 source address or CIDR network.
pub fn validate_source(value: &str) -> Result<(), ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::Required);
    }

    let Some((address, prefix)) = value.split_once('/') else {
        return value
            .parse::<IpAddr>()
            .map(|_| ())
            .map_err(|_| ValidationError::InvalidSource);
    };
    if prefix.contains('/') {
        return Err(ValidationError::InvalidSource);
    }
    let address = address
        .parse::<IpAddr>()
        .map_err(|_| ValidationError::InvalidSource)?;
    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| ValidationError::InvalidCidrPrefix)?;
    let maximum = if address.is_ipv4() { 32 } else { 128 };
    if prefix > maximum {
        return Err(ValidationError::InvalidCidrPrefix);
    }
    Ok(())
}

/// Validates a single port or inclusive `start-end` range.
pub fn validate_port_spec(value: &str) -> Result<(), ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::Required);
    }

    let mut parts = value.split('-');
    let start = parse_port(parts.next().unwrap_or_default())?;
    let Some(end_text) = parts.next() else {
        return Ok(());
    };
    if parts.next().is_some() {
        return Err(ValidationError::InvalidPort);
    }
    let end = parse_port(end_text)?;
    if start > end {
        return Err(ValidationError::ReversedPortRange);
    }
    Ok(())
}

/// Validates a protocol supported by firewalld port rules.
pub fn validate_port_protocol(value: &str) -> Result<(), ValidationError> {
    match value {
        "tcp" | "udp" | "sctp" | "dccp" => Ok(()),
        _ => Err(ValidationError::InvalidProtocol),
    }
}

/// Validates an optional forwarding address.
///
/// An empty address is deliberately accepted: firewalld uses it for a local
/// destination while still forwarding to the requested destination port.
pub fn validate_forward_address(value: &str) -> Result<(), ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(());
    }
    value
        .parse::<IpAddr>()
        .map(|_| ())
        .map_err(|_| ValidationError::InvalidIpAddress)
}

fn parse_port(value: &str) -> Result<u16, ValidationError> {
    let port = value
        .parse::<u16>()
        .map_err(|_| ValidationError::InvalidPort)?;
    if port == 0 {
        Err(ValidationError::InvalidPort)
    } else {
        Ok(port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ports_and_inclusive_ranges() {
        for value in ["1", "443", "65535", "1000-2000", "53-53"] {
            assert_eq!(validate_port_spec(value), Ok(()), "{value}");
        }
    }

    #[test]
    fn rejects_invalid_ports_and_reversed_ranges() {
        assert_eq!(validate_port_spec("0"), Err(ValidationError::InvalidPort));
        assert_eq!(
            validate_port_spec("65536"),
            Err(ValidationError::InvalidPort)
        );
        assert_eq!(
            validate_port_spec("2000-1000"),
            Err(ValidationError::ReversedPortRange)
        );
        assert_eq!(
            validate_port_spec("1-2-3"),
            Err(ValidationError::InvalidPort)
        );
    }

    #[test]
    fn limits_port_protocols() {
        for value in ["tcp", "udp", "sctp", "dccp"] {
            assert_eq!(validate_port_protocol(value), Ok(()));
        }
        assert_eq!(
            validate_port_protocol("icmp"),
            Err(ValidationError::InvalidProtocol)
        );
    }

    #[test]
    fn accepts_empty_or_literal_forwarding_addresses() {
        for value in ["", "192.0.2.1", "2001:db8::1"] {
            assert_eq!(validate_forward_address(value), Ok(()), "{value}");
        }
        assert_eq!(
            validate_forward_address("example.test"),
            Err(ValidationError::InvalidIpAddress)
        );
    }

    #[test]
    fn validates_interface_names() {
        for value in ["eth0", "wlan0", "enp0s31f6.100"] {
            assert_eq!(validate_interface_name(value), Ok(()), "{value}");
        }
        assert_eq!(validate_interface_name(""), Err(ValidationError::Required));
        assert_eq!(
            validate_interface_name("interface-name-too-long"),
            Err(ValidationError::InterfaceNameTooLong)
        );
        assert_eq!(
            validate_interface_name("eth 0"),
            Err(ValidationError::InvalidInterfaceName)
        );
    }

    #[test]
    fn accepts_ipv4_ipv6_and_cidr_sources() {
        for value in ["192.0.2.1", "192.0.2.0/24", "2001:db8::1", "2001:db8::/32"] {
            assert_eq!(validate_source(value), Ok(()), "{value}");
        }
    }

    #[test]
    fn rejects_invalid_sources_and_prefixes() {
        assert_eq!(
            validate_source("example.test"),
            Err(ValidationError::InvalidSource)
        );
        assert_eq!(
            validate_source("192.0.2.0/33"),
            Err(ValidationError::InvalidCidrPrefix)
        );
        assert_eq!(
            validate_source("2001:db8::/129"),
            Err(ValidationError::InvalidCidrPrefix)
        );
    }
}
