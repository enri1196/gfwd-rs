#[derive(Debug, Default, Clone, PartialEq)]
pub struct IcmpType {
    pub name: String,
    pub description: String,
}

impl IcmpType {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}
