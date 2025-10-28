use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct IPSetSettings {
    pub name: String,
    pub ipset_type: String,
    pub entries: Vec<String>,
    pub options: HashMap<String, String>,
}

impl IPSetSettings {
    #[allow(dead_code)]
    pub fn new(name: String, ipset_type: String) -> Self {
        Self {
            name,
            ipset_type,
            entries: Vec::new(),
            options: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_entries(mut self, entries: Vec<String>) -> Self {
        self.entries = entries;
        self
    }

    #[allow(dead_code)]
    pub fn with_options(mut self, options: HashMap<String, String>) -> Self {
        self.options = options;
        self
    }
}
