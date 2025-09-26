#[derive(Debug, Default, Clone, PartialEq)]
pub struct ZoneSettings {
    pub version: String,
    pub name: String,
    pub description: String,
    pub unused: bool,
    pub target: ZoneTarget,
    pub services: Vec<String>,
    pub ports: Vec<(String, String)>,
    pub icmp_blocks: Vec<String>,
    pub masquerade: bool,
    pub forward_ports: Vec<(String, String, String, String)>,
    pub interfaces: Vec<String>,
    pub sources: Vec<String>,
    pub rich_rules: Vec<String>,
    pub protocols: Vec<String>,
    pub source_ports: Vec<(String, String)>,
}

#[derive(Debug, Default, derive_more::Display, Clone, PartialEq)]
#[allow(unused)]
pub enum ZoneTarget {
    #[default]
    #[display("default")]
    Default,
    #[display("ACCEPT")]
    Accept,
    #[display("DROP")]
    Drop,
    #[display("REJECT")]
    Reject,
}