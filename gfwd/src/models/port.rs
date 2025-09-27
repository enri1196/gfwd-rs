#[derive(Debug, Clone, PartialEq)]
pub struct PortRule {
    pub port: String,
    pub protocol: String,
    pub forwarding: Option<ForwardingConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardingConfig {
    pub to_addr: String,
    pub to_port: String,
}

impl PortRule {
    pub fn new(port: String, protocol: String) -> Self {
        Self {
            port,
            protocol,
            forwarding: None,
        }
    }

    pub fn with_forwarding(port: String, protocol: String, forwarding: ForwardingConfig) -> Self {
        Self {
            port,
            protocol,
            forwarding: Some(forwarding),
        }
    }

    pub fn is_forwarded(&self) -> bool {
        self.forwarding.is_some()
    }
}
