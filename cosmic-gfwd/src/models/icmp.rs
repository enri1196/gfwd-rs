/// A configured firewalld ICMP type shown in the picker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcmpTypeInfo {
    /// Firewalld's stable ICMP type name.
    pub name: String,
    /// Human-readable description from the permanent configuration.
    pub description: String,
}
