use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IpSetDetails {
    pub name: String,
    pub ipset_type: String,
    pub entries: Vec<String>,
    pub options: HashMap<String, String>,
}
