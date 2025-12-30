#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneDetails {
    pub name: String,
    pub description: String,
    pub target: ZoneTarget,
    pub masquerade: bool,
    pub icmp_block_inversion: bool,
    pub services: Vec<String>,
    pub ports: Vec<(String, String)>,
    pub forward_ports: Vec<(String, String, String, String)>,
    pub interfaces: Vec<String>,
    pub sources: Vec<String>,
    pub icmp_blocks: Vec<String>,
    pub rich_rules: Vec<String>,
    pub protocols: Vec<String>,
    pub source_ports: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneTarget {
    Default,
    Accept,
    Drop,
    Reject,
    Other(String),
}

impl ZoneTarget {
    pub fn from_raw(value: String) -> Self {
        match value.as_str() {
            "" => ZoneTarget::Default,
            "default" => ZoneTarget::Default,
            "DEFAULT" => ZoneTarget::Default,
            "ACCEPT" => ZoneTarget::Accept,
            "DROP" => ZoneTarget::Drop,
            "REJECT" => ZoneTarget::Reject,
            _ => ZoneTarget::Other(value),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ZoneTarget::Default => "default",
            ZoneTarget::Accept => "ACCEPT",
            ZoneTarget::Drop => "DROP",
            ZoneTarget::Reject => "REJECT",
            ZoneTarget::Other(value) => value.as_str(),
        }
    }
}

impl std::fmt::Display for ZoneTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
