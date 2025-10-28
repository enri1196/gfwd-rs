#[derive(Debug, Default, Clone, PartialEq)]
pub struct RichRule {
    pub family: Option<String>,
    pub source: Option<RichRuleAddress>,
    pub destination: Option<RichRuleAddress>,
    pub service: Option<String>,
    pub port: Option<RichRulePort>,
    pub protocol: Option<String>,
    pub action: RichRuleAction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichRuleAddress {
    pub address: String,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichRulePort {
    pub port: String,
    pub protocol: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum RichRuleAction {
    #[default]
    Accept,
    Reject(Option<String>), // Optional reject type
    Drop,
    Mark(String), // Mark value
}

impl RichRule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_family(mut self, family: String) -> Self {
        self.family = Some(family);
        self
    }

    pub fn with_source(mut self, address: String, invert: bool) -> Self {
        self.source = Some(RichRuleAddress { address, invert });
        self
    }

    pub fn with_destination(mut self, address: String, invert: bool) -> Self {
        self.destination = Some(RichRuleAddress { address, invert });
        self
    }

    pub fn with_service(mut self, service: String) -> Self {
        self.service = Some(service);
        self
    }

    pub fn with_port(mut self, port: String, protocol: String) -> Self {
        self.port = Some(RichRulePort { port, protocol });
        self
    }

    pub fn with_protocol(mut self, protocol: String) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn with_action(mut self, action: RichRuleAction) -> Self {
        self.action = action;
        self
    }

    /// Generate XML representation of the rich rule
    pub fn to_xml(&self) -> String {
        let mut rule = String::from("<rule");

        if let Some(ref family) = self.family {
            rule.push_str(&format!(" family=\"{}\"", family));
        }

        rule.push('>');

        if let Some(ref source) = self.source {
            rule.push_str("<source");
            if source.invert {
                rule.push_str(" invert=\"true\"");
            }
            rule.push_str(&format!(" address=\"{}\"/>", source.address));
        }

        if let Some(ref destination) = self.destination {
            rule.push_str("<destination");
            if destination.invert {
                rule.push_str(" invert=\"true\"");
            }
            rule.push_str(&format!(" address=\"{}\"/>", destination.address));
        }

        if let Some(ref service) = self.service {
            rule.push_str(&format!("<service name=\"{}\"/>", service));
        }

        if let Some(ref port) = self.port {
            rule.push_str(&format!(
                "<port port=\"{}\" protocol=\"{}\"/>",
                port.port, port.protocol
            ));
        }

        if let Some(ref protocol) = self.protocol {
            rule.push_str(&format!("<protocol value=\"{}\"/>", protocol));
        }

        match &self.action {
            RichRuleAction::Accept => rule.push_str("<accept/>"),
            RichRuleAction::Reject(reject_type) => {
                if let Some(rtype) = reject_type {
                    rule.push_str(&format!("<reject type=\"{}\"/>", rtype));
                } else {
                    rule.push_str("<reject/>");
                }
            }
            RichRuleAction::Drop => rule.push_str("<drop/>"),
            RichRuleAction::Mark(mark) => rule.push_str(&format!("<mark set=\"{}\"/>", mark)),
        }

        rule.push_str("</rule>");
        rule
    }
}
