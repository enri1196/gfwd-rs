#[derive(Debug, Default, Clone, PartialEq)]
pub struct DirectRule {
    pub table: String,
    pub chain: String,
    pub priority: i32,
    pub args: Vec<String>,
}

impl DirectRule {
    pub fn new(table: String, chain: String, priority: i32, args: Vec<String>) -> Self {
        Self {
            table,
            chain,
            priority,
            args,
        }
    }

    /// Get a human-readable representation of the rule
    pub fn display_rule(&self) -> String {
        format!("{} {} [{}] {}", 
            self.table, 
            self.chain, 
            self.priority, 
            self.args.join(" ")
        )
    }
}