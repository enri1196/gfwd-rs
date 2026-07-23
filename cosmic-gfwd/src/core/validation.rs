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
    /// An IP-set name is longer than firewalld permits.
    IpSetNameTooLong,
    /// An IP-set name contains unsupported characters.
    InvalidIpSetName,
    /// An IP-set name starts with a dash.
    IpSetNameStartsWithDash,
    /// The requested IP-set type is unsupported.
    InvalidIpSetType,
    /// An IP-set entry has the wrong number or kind of components.
    InvalidIpSetEntry,
    /// A MAC-address component is invalid.
    InvalidMacAddress,
}

/// IP-set types supported by the creation and entry editors.
pub const IPSET_TYPES: [&str; 13] = [
    "hash:ip",
    "hash:net",
    "hash:ip,port",
    "hash:net,port",
    "hash:ip,port,ip",
    "hash:ip,port,net",
    "hash:net,port,net",
    "hash:net,iface",
    "hash:mac",
    "bitmap:ip",
    "bitmap:ip,mac",
    "bitmap:port",
    "list:set",
];

/// Validates a firewalld IP-set name.
pub fn validate_ipset_name(value: &str) -> Result<(), ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::Required);
    }
    if value.len() > 31 {
        return Err(ValidationError::IpSetNameTooLong);
    }
    if value.starts_with('-') {
        return Err(ValidationError::IpSetNameStartsWithDash);
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(ValidationError::InvalidIpSetName);
    }
    Ok(())
}

/// Validates that an IP-set type is supported by the COSMIC editor.
pub fn validate_ipset_type(value: &str) -> Result<(), ValidationError> {
    if IPSET_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::InvalidIpSetType)
    }
}

/// Validates an entry according to its selected IP-set type.
pub fn validate_ipset_entry(entry: &str, ipset_type: &str) -> Result<(), ValidationError> {
    validate_ipset_type(ipset_type)?;
    let entry = entry.trim();
    if entry.is_empty() {
        return Err(ValidationError::Required);
    }
    let parts = entry.split(',').map(str::trim).collect::<Vec<_>>();
    let ip = |value: &str| {
        value
            .parse::<IpAddr>()
            .map(|_| ())
            .map_err(|_| ValidationError::InvalidIpSetEntry)
    };
    let net = |value: &str| validate_source(value).map_err(|_| ValidationError::InvalidIpSetEntry);
    let port =
        |value: &str| validate_port_spec(value).map_err(|_| ValidationError::InvalidIpSetEntry);
    let iface = |value: &str| {
        validate_interface_name(value).map_err(|_| ValidationError::InvalidIpSetEntry)
    };

    match (ipset_type, parts.as_slice()) {
        ("hash:ip" | "bitmap:ip", [value]) => ip(value),
        ("hash:net", [value]) => net(value),
        ("hash:ip,port", [address, port_value]) => {
            ip(address)?;
            port(port_value)
        }
        ("hash:net,port", [network, port_value]) => {
            net(network)?;
            port(port_value)
        }
        ("hash:ip,port,ip", [address, port_value, destination]) => {
            ip(address)?;
            port(port_value)?;
            ip(destination)
        }
        ("hash:ip,port,net", [address, port_value, destination]) => {
            ip(address)?;
            port(port_value)?;
            net(destination)
        }
        ("hash:net,port,net", [network, port_value, destination]) => {
            net(network)?;
            port(port_value)?;
            net(destination)
        }
        ("hash:net,iface", [network, interface]) => {
            net(network)?;
            iface(interface)
        }
        ("hash:mac", [mac]) => validate_mac(mac),
        ("bitmap:ip,mac", [address, mac]) => {
            ip(address)?;
            validate_mac(mac)
        }
        ("bitmap:port", [value]) => port(value),
        ("list:set", [name]) => validate_ipset_name(name),
        _ => Err(ValidationError::InvalidIpSetEntry),
    }
}

fn validate_mac(value: &str) -> Result<(), ValidationError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() == 6
        && parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|character| character.is_ascii_hexdigit())
        })
    {
        Ok(())
    } else {
        Err(ValidationError::InvalidMacAddress)
    }
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

    #[test]
    fn validates_all_supported_ipset_entry_shapes() {
        let cases = [
            ("hash:ip", "192.0.2.1"),
            ("hash:net", "192.0.2.0/24"),
            ("hash:ip,port", "192.0.2.1,443"),
            ("hash:net,port", "192.0.2.0/24,443"),
            ("hash:ip,port,ip", "192.0.2.1,443,198.51.100.2"),
            ("hash:ip,port,net", "192.0.2.1,443,198.51.100.0/24"),
            ("hash:net,port,net", "192.0.2.0/24,443,198.51.100.0/24"),
            ("hash:net,iface", "192.0.2.0/24,eth0"),
            ("hash:mac", "02:00:5e:10:00:00"),
            ("bitmap:ip", "192.0.2.1"),
            ("bitmap:ip,mac", "192.0.2.1,02:00:5e:10:00:00"),
            ("bitmap:port", "1000-2000"),
            ("list:set", "trusted_networks"),
        ];
        for (kind, entry) in cases {
            assert_eq!(validate_ipset_entry(entry, kind), Ok(()), "{kind}: {entry}");
        }
    }

    #[test]
    fn rejects_wrong_ipset_entry_shapes() {
        assert_eq!(
            validate_ipset_entry("192.0.2.1", "hash:ip,port"),
            Err(ValidationError::InvalidIpSetEntry)
        );
        assert_eq!(
            validate_ipset_entry("not-a-mac", "hash:mac"),
            Err(ValidationError::InvalidMacAddress)
        );
        assert_eq!(
            validate_ipset_entry("65536", "bitmap:port"),
            Err(ValidationError::InvalidIpSetEntry)
        );
    }
}
