#[derive(Debug, Default, Clone, PartialEq)]
#[allow(dead_code)]
pub struct PolicySettings {
    pub name: String,
    pub description: String,
    pub ingress_zones: Vec<String>,
    pub egress_zones: Vec<String>,
    pub priority: i32,
}

impl PolicySettings {
    #[allow(dead_code)]
    pub fn new(name: String, description: String, priority: i32) -> Self {
        Self {
            name,
            description,
            ingress_zones: Vec::new(),
            egress_zones: Vec::new(),
            priority,
        }
    }

    #[allow(dead_code)]
    pub fn with_ingress_zones(mut self, zones: Vec<String>) -> Self {
        self.ingress_zones = zones;
        self
    }

    #[allow(dead_code)]
    pub fn with_egress_zones(mut self, zones: Vec<String>) -> Self {
        self.egress_zones = zones;
        self
    }
}
