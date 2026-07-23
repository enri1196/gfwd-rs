use super::{ValidationError, validate_port_protocol, validate_port_spec, validate_source};

/// Address family constraint for a structured rich rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RichRuleFamily {
    /// Match IPv4 traffic.
    Ipv4,
    /// Match IPv6 traffic.
    Ipv6,
}

/// The single match element of a structured rich rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RichRuleElement {
    /// Match a configured service name.
    Service(String),
    /// Match a port or inclusive range and transport protocol.
    Port { port: String, protocol: String },
    /// Match an IP protocol name or number.
    Protocol(String),
}

/// Terminal action of a structured rich rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RichRuleAction {
    /// Accept matching traffic.
    Accept,
    /// Reject matching traffic, optionally with a reject type.
    Reject(Option<String>),
    /// Silently drop matching traffic.
    Drop,
    /// Set a validated packet mark.
    Mark(String),
}

/// Typed input used to validate and generate one rich-rule XML string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RichRuleSpec {
    /// Optional address family.
    pub family: Option<RichRuleFamily>,
    /// Optional source address/network and inversion flag.
    pub source: Option<(String, bool)>,
    /// Optional destination address/network and inversion flag.
    pub destination: Option<(String, bool)>,
    /// Exactly one rule element.
    pub element: RichRuleElement,
    /// Exactly one terminal action.
    pub action: RichRuleAction,
}

/// A structured rich-rule validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RichRuleError {
    /// A required element value is empty.
    MissingElement,
    /// An address or CIDR network is invalid.
    InvalidAddress,
    /// The port or range is invalid.
    InvalidPort,
    /// The selected port protocol is invalid.
    InvalidPortProtocol,
    /// A protocol name/number contains unsupported characters.
    InvalidProtocol,
    /// A service or reject type contains unsupported characters.
    InvalidIdentifier,
    /// A packet mark is neither decimal nor hexadecimal, optionally with a mask.
    InvalidMark,
}

impl RichRuleSpec {
    /// Validates the spec and returns firewalld rich-rule XML.
    pub fn to_xml(&self) -> Result<String, RichRuleError> {
        validate_optional_address(&self.source)?;
        validate_optional_address(&self.destination)?;
        validate_element(&self.element)?;
        validate_action(&self.action)?;

        let mut rule = String::from("<rule");
        if let Some(family) = self.family {
            rule.push_str(match family {
                RichRuleFamily::Ipv4 => " family=\"ipv4\"",
                RichRuleFamily::Ipv6 => " family=\"ipv6\"",
            });
        }
        rule.push('>');

        if let Some((address, invert)) = &self.source {
            push_address(&mut rule, "source", address, *invert);
        }
        if let Some((address, invert)) = &self.destination {
            push_address(&mut rule, "destination", address, *invert);
        }
        match &self.element {
            RichRuleElement::Service(name) => {
                rule.push_str("<service name=\"");
                rule.push_str(&escape_attribute(name));
                rule.push_str("\"/>");
            }
            RichRuleElement::Port { port, protocol } => {
                rule.push_str("<port port=\"");
                rule.push_str(&escape_attribute(port));
                rule.push_str("\" protocol=\"");
                rule.push_str(&escape_attribute(protocol));
                rule.push_str("\"/>");
            }
            RichRuleElement::Protocol(value) => {
                rule.push_str("<protocol value=\"");
                rule.push_str(&escape_attribute(value));
                rule.push_str("\"/>");
            }
        }
        match &self.action {
            RichRuleAction::Accept => rule.push_str("<accept/>"),
            RichRuleAction::Reject(None) => rule.push_str("<reject/>"),
            RichRuleAction::Reject(Some(kind)) => {
                rule.push_str("<reject type=\"");
                rule.push_str(&escape_attribute(kind));
                rule.push_str("\"/>");
            }
            RichRuleAction::Drop => rule.push_str("<drop/>"),
            RichRuleAction::Mark(mark) => {
                rule.push_str("<mark set=\"");
                rule.push_str(&escape_attribute(mark));
                rule.push_str("\"/>");
            }
        }
        rule.push_str("</rule>");
        Ok(rule)
    }
}

fn validate_optional_address(address: &Option<(String, bool)>) -> Result<(), RichRuleError> {
    if let Some((address, _)) = address {
        validate_source(address).map_err(|_| RichRuleError::InvalidAddress)?;
    }
    Ok(())
}

fn validate_element(element: &RichRuleElement) -> Result<(), RichRuleError> {
    match element {
        RichRuleElement::Service(name) if name.trim().is_empty() => {
            Err(RichRuleError::MissingElement)
        }
        RichRuleElement::Service(name) => validate_identifier(name),
        RichRuleElement::Port { port, protocol } => {
            validate_port_spec(port).map_err(|error| match error {
                ValidationError::Required => RichRuleError::MissingElement,
                _ => RichRuleError::InvalidPort,
            })?;
            validate_port_protocol(protocol).map_err(|_| RichRuleError::InvalidPortProtocol)
        }
        RichRuleElement::Protocol(value) if value.trim().is_empty() => {
            Err(RichRuleError::MissingElement)
        }
        RichRuleElement::Protocol(value)
            if value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-') =>
        {
            Ok(())
        }
        RichRuleElement::Protocol(_) => Err(RichRuleError::InvalidProtocol),
    }
}

fn validate_action(action: &RichRuleAction) -> Result<(), RichRuleError> {
    match action {
        RichRuleAction::Reject(Some(kind)) if !kind.trim().is_empty() => validate_identifier(kind),
        RichRuleAction::Mark(value) if !valid_mark(value) => Err(RichRuleError::InvalidMark),
        _ => Ok(()),
    }
}

fn validate_identifier(value: &str) -> Result<(), RichRuleError> {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        Ok(())
    } else {
        Err(RichRuleError::InvalidIdentifier)
    }
}

fn valid_mark(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    let mut parts = value.split('/');
    let valid = parts.next().is_some_and(valid_mark_number)
        && parts.next().is_none_or(valid_mark_number)
        && parts.next().is_none();
    valid
}

fn valid_mark_number(value: &str) -> bool {
    if let Some(hex) = value.strip_prefix("0x") {
        !hex.is_empty() && hex.chars().all(|character| character.is_ascii_hexdigit())
    } else {
        !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
    }
}

fn push_address(rule: &mut String, element: &str, address: &str, invert: bool) {
    rule.push('<');
    rule.push_str(element);
    if invert {
        rule.push_str(" invert=\"true\"");
    }
    rule.push_str(" address=\"");
    rule.push_str(&escape_attribute(address));
    rule.push_str("\"/>");
}

fn escape_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_service_rule_with_addresses() {
        let spec = RichRuleSpec {
            family: Some(RichRuleFamily::Ipv4),
            source: Some(("192.0.2.0/24".to_string(), true)),
            destination: Some(("198.51.100.2".to_string(), false)),
            element: RichRuleElement::Service("https".to_string()),
            action: RichRuleAction::Accept,
        };
        assert_eq!(
            spec.to_xml().unwrap(),
            "<rule family=\"ipv4\"><source invert=\"true\" address=\"192.0.2.0/24\"/><destination address=\"198.51.100.2\"/><service name=\"https\"/><accept/></rule>"
        );
    }

    #[test]
    fn generates_port_reject_and_mark_rules() {
        let port = RichRuleSpec {
            family: None,
            source: None,
            destination: None,
            element: RichRuleElement::Port {
                port: "1000-2000".to_string(),
                protocol: "tcp".to_string(),
            },
            action: RichRuleAction::Reject(Some("icmp-port-unreachable".to_string())),
        };
        assert!(
            port.to_xml()
                .unwrap()
                .contains("<reject type=\"icmp-port-unreachable\"/>")
        );

        let mark = RichRuleSpec {
            action: RichRuleAction::Mark("0x1/0xff".to_string()),
            ..port
        };
        assert!(mark.to_xml().unwrap().contains("<mark set=\"0x1/0xff\"/>"));
    }

    #[test]
    fn rejects_invalid_cross_field_values() {
        let missing = RichRuleSpec {
            family: None,
            source: None,
            destination: None,
            element: RichRuleElement::Service(String::new()),
            action: RichRuleAction::Accept,
        };
        assert_eq!(missing.to_xml(), Err(RichRuleError::MissingElement));

        let invalid_mark = RichRuleSpec {
            element: RichRuleElement::Protocol("icmp".to_string()),
            action: RichRuleAction::Mark("xyz".to_string()),
            ..missing
        };
        assert_eq!(invalid_mark.to_xml(), Err(RichRuleError::InvalidMark));
    }
}
