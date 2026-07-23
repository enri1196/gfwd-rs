use std::collections::{BTreeSet, HashMap};
use std::fmt;

use zvariant::OwnedValue;

/// A complete typed view of the zone settings understood by this application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneSettingsSnapshot {
    /// User-facing short name.
    pub short_name: String,
    /// User-facing description.
    pub description: String,
    /// Packet target, such as `default`, `ACCEPT`, `REJECT`, or `DROP`.
    pub target: String,
    /// Enabled service names.
    pub services: Vec<String>,
    /// Destination port and protocol pairs.
    pub ports: Vec<(String, String)>,
    /// Enabled IP protocols.
    pub protocols: Vec<String>,
    /// Source port and protocol pairs.
    pub source_ports: Vec<(String, String)>,
    /// Blocked ICMP type names.
    pub icmp_blocks: Vec<String>,
    /// Whether ICMP block semantics are inverted.
    pub icmp_block_inversion: bool,
    /// Whether address masquerading is enabled.
    pub masquerade: bool,
    /// Port, protocol, destination port, and destination address tuples.
    pub forward_ports: Vec<(String, String, String, String)>,
    /// Bound network interfaces.
    pub interfaces: Vec<String>,
    /// Bound source addresses or networks.
    pub sources: Vec<String>,
    /// Rich rules in their exact textual representation.
    pub rich_rules: Vec<String>,
    /// Whether forwarding through the zone is enabled.
    pub forward: bool,
    /// Zone ingress priority.
    pub ingress_priority: i32,
    /// Zone egress priority.
    pub egress_priority: i32,
    /// Dictionary keys not understood by this application.
    pub unknown_keys: BTreeSet<String>,
}

impl Default for ZoneSettingsSnapshot {
    fn default() -> Self {
        Self {
            short_name: String::new(),
            description: String::new(),
            target: "default".to_string(),
            services: Vec::new(),
            ports: Vec::new(),
            protocols: Vec::new(),
            source_ports: Vec::new(),
            icmp_blocks: Vec::new(),
            icmp_block_inversion: false,
            masquerade: false,
            forward_ports: Vec::new(),
            interfaces: Vec::new(),
            sources: Vec::new(),
            rich_rules: Vec::new(),
            forward: true,
            ingress_priority: 0,
            egress_priority: 0,
            unknown_keys: BTreeSet::new(),
        }
    }
}

impl ZoneSettingsSnapshot {
    /// Decode a firewalld `getSettings2` dictionary.
    ///
    /// Empty values may be omitted by firewalld. Missing keys therefore use
    /// firewalld's zone defaults.
    pub fn from_settings(
        mut settings: HashMap<String, OwnedValue>,
    ) -> Result<Self, ZoneSettingsParseError> {
        // `version` is known metadata but is intentionally outside the
        // reconciliation surface described by ZoneSettingsSnapshot.
        settings.remove("version");

        let short_name = take(&mut settings, "short", "string")?.unwrap_or_default();
        let description = take(&mut settings, "description", "string")?.unwrap_or_default();
        let target =
            take(&mut settings, "target", "string")?.unwrap_or_else(|| "default".to_string());
        let services = take(&mut settings, "services", "array of strings")?.unwrap_or_default();
        let ports = take(&mut settings, "ports", "array of string pairs")?.unwrap_or_default();
        let protocols = take(&mut settings, "protocols", "array of strings")?.unwrap_or_default();
        let source_ports =
            take(&mut settings, "source_ports", "array of string pairs")?.unwrap_or_default();
        let icmp_blocks =
            take(&mut settings, "icmp_blocks", "array of strings")?.unwrap_or_default();
        let icmp_block_inversion =
            take(&mut settings, "icmp_block_inversion", "boolean")?.unwrap_or(false);
        let masquerade = take(&mut settings, "masquerade", "boolean")?.unwrap_or(false);
        let forward_ports = take(
            &mut settings,
            "forward_ports",
            "array of four-string tuples",
        )?
        .unwrap_or_default();
        let interfaces = take(&mut settings, "interfaces", "array of strings")?.unwrap_or_default();
        let sources = take(&mut settings, "sources", "array of strings")?.unwrap_or_default();
        let rich_rules = take(&mut settings, "rules_str", "array of strings")?.unwrap_or_default();
        let forward = take(&mut settings, "forward", "boolean")?.unwrap_or(true);
        let ingress_priority =
            take(&mut settings, "ingress_priority", "32-bit integer")?.unwrap_or(0);
        let egress_priority =
            take(&mut settings, "egress_priority", "32-bit integer")?.unwrap_or(0);

        Ok(Self {
            short_name,
            description,
            target,
            services,
            ports,
            protocols,
            source_ports,
            icmp_blocks,
            icmp_block_inversion,
            masquerade,
            forward_ports,
            interfaces,
            sources,
            rich_rules,
            forward,
            ingress_priority,
            egress_priority,
            unknown_keys: settings.into_keys().collect(),
        })
    }
}

fn take<T>(
    settings: &mut HashMap<String, OwnedValue>,
    key: &'static str,
    expected: &'static str,
) -> Result<Option<T>, ZoneSettingsParseError>
where
    T: TryFrom<OwnedValue, Error = zvariant::Error>,
{
    let Some(value) = settings.remove(key) else {
        return Ok(None);
    };
    let actual = value.value_signature().to_string();
    T::try_from(value)
        .map(Some)
        .map_err(|_| ZoneSettingsParseError::InvalidType {
            key,
            expected,
            actual,
        })
}

/// A structured zone settings dictionary decoding error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZoneSettingsParseError {
    /// A known key carried a value with an unexpected D-Bus type.
    InvalidType {
        /// Dictionary key that failed to decode.
        key: &'static str,
        /// Human-readable expected Rust/D-Bus shape.
        expected: &'static str,
        /// Actual D-Bus signature.
        actual: String,
    },
}

impl fmt::Display for ZoneSettingsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType {
                key,
                expected,
                actual,
            } => write!(
                f,
                "zone setting `{key}` expected {expected}, but received D-Bus type `{actual}`"
            ),
        }
    }
}

impl std::error::Error for ZoneSettingsParseError {}

/// A setting represented in a reconciliation difference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneSetting {
    /// User-facing short name.
    ShortName,
    /// User-facing description.
    Description,
    /// Packet target.
    Target,
    /// Enabled services.
    Services,
    /// Destination ports.
    Ports,
    /// Enabled protocols.
    Protocols,
    /// Source ports.
    SourcePorts,
    /// Blocked ICMP types.
    IcmpBlocks,
    /// ICMP block inversion.
    IcmpBlockInversion,
    /// Address masquerading.
    Masquerade,
    /// Forwarded ports.
    ForwardPorts,
    /// Bound interfaces.
    Interfaces,
    /// Bound sources.
    Sources,
    /// Rich rules.
    RichRules,
    /// Zone forwarding.
    Forward,
    /// Ingress priority.
    IngressPriority,
    /// Egress priority.
    EgressPriority,
}

/// A scalar value retained on both sides of a difference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScalarValue {
    /// Text setting.
    Text(String),
    /// Boolean setting.
    Boolean(bool),
    /// Signed integer setting.
    Integer(i32),
}

/// An exact member of a collection-valued zone setting.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollectionValue {
    /// A single string entry.
    Text(String),
    /// A port and protocol pair.
    Pair(String, String),
    /// A port, protocol, destination port, and destination address tuple.
    ForwardPort(String, String, String, String),
}

/// One typed difference between permanent and runtime zone settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZoneSettingDifference {
    /// A scalar whose permanent and runtime values differ.
    Scalar {
        /// Setting being compared.
        setting: ZoneSetting,
        /// Permanent value.
        permanent: ScalarValue,
        /// Runtime value.
        runtime: ScalarValue,
    },
    /// An order-insensitive collection whose membership differs.
    Collection {
        /// Setting being compared.
        setting: ZoneSetting,
        /// Entries present only in permanent configuration.
        permanent_only: Vec<CollectionValue>,
        /// Entries present only in runtime configuration.
        runtime_only: Vec<CollectionValue>,
    },
}

/// Whether the comparison covered every dictionary key returned by firewalld.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComparisonCompleteness {
    /// Every returned key was understood.
    Complete,
    /// One or both dictionaries contained future or unsupported keys.
    Incomplete {
        /// Unknown permanent dictionary keys.
        permanent_unknown: BTreeSet<String>,
        /// Unknown runtime dictionary keys.
        runtime_unknown: BTreeSet<String>,
    },
}

/// The pure comparison of permanent and runtime settings for one zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneReconciliation {
    /// Typed known-setting differences.
    pub differences: Vec<ZoneSettingDifference>,
    /// Whether all settings returned by firewalld were understood.
    pub completeness: ComparisonCompleteness,
}

/// Snapshots and their computed reconciliation for one selected zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneReconciliationData {
    /// Permanent zone settings.
    pub permanent: ZoneSettingsSnapshot,
    /// Runtime zone settings.
    pub runtime: ZoneSettingsSnapshot,
    /// Typed comparison of the snapshots.
    pub reconciliation: ZoneReconciliation,
}

impl ZoneReconciliationData {
    /// Build combined reconciliation data from two decoded snapshots.
    pub fn new(permanent: ZoneSettingsSnapshot, runtime: ZoneSettingsSnapshot) -> Self {
        let reconciliation = ZoneReconciliation::compare(&permanent, &runtime);
        Self {
            permanent,
            runtime,
            reconciliation,
        }
    }
}

/// Independently loaded reconciliation state for the selected zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZoneReconciliationState {
    /// The comparison request is in flight.
    Loading {
        /// Zone being compared.
        zone: String,
    },
    /// Every known setting matches and all dictionary keys were understood.
    InSync {
        /// Zone that was compared.
        zone: String,
        /// Loaded snapshots and comparison.
        data: Box<ZoneReconciliationData>,
    },
    /// Known permanent and runtime settings differ.
    Different {
        /// Zone that was compared.
        zone: String,
        /// Loaded snapshots and comparison.
        data: Box<ZoneReconciliationData>,
    },
    /// Known settings were compared, but unknown keys prevent a definitive result.
    Incomplete {
        /// Zone that was compared.
        zone: String,
        /// Loaded snapshots and comparison.
        data: Box<ZoneReconciliationData>,
    },
    /// Runtime comparison is not currently available.
    Unavailable {
        /// Selected zone, when one exists.
        zone: Option<String>,
    },
    /// A comparison request failed without invalidating permanent zone content.
    Error {
        /// Zone whose comparison failed.
        zone: String,
        /// Actionable loading error.
        message: String,
    },
}

impl Default for ZoneReconciliationState {
    fn default() -> Self {
        Self::Unavailable { zone: None }
    }
}

impl ZoneReconciliationState {
    /// Classify successfully loaded reconciliation data.
    pub fn from_data(zone: String, data: ZoneReconciliationData) -> Self {
        if data.reconciliation.is_in_sync() {
            Self::InSync {
                zone,
                data: Box::new(data),
            }
        } else if matches!(
            data.reconciliation.completeness,
            ComparisonCompleteness::Incomplete { .. }
        ) {
            Self::Incomplete {
                zone,
                data: Box::new(data),
            }
        } else {
            Self::Different {
                zone,
                data: Box::new(data),
            }
        }
    }

    /// Access loaded comparison data for any successful state.
    pub fn data(&self) -> Option<&ZoneReconciliationData> {
        match self {
            Self::InSync { data, .. }
            | Self::Different { data, .. }
            | Self::Incomplete { data, .. } => Some(data),
            Self::Loading { .. } | Self::Unavailable { .. } | Self::Error { .. } => None,
        }
    }
}

impl ZoneReconciliation {
    /// Compare two snapshots without depending on collection ordering.
    pub fn compare(permanent: &ZoneSettingsSnapshot, runtime: &ZoneSettingsSnapshot) -> Self {
        let mut differences = Vec::new();

        scalar(
            &mut differences,
            ZoneSetting::ShortName,
            &permanent.short_name,
            &runtime.short_name,
        );
        scalar(
            &mut differences,
            ZoneSetting::Description,
            &permanent.description,
            &runtime.description,
        );
        scalar(
            &mut differences,
            ZoneSetting::Target,
            &permanent.target,
            &runtime.target,
        );
        collection_text(
            &mut differences,
            ZoneSetting::Services,
            &permanent.services,
            &runtime.services,
        );
        collection_pairs(
            &mut differences,
            ZoneSetting::Ports,
            &permanent.ports,
            &runtime.ports,
        );
        collection_text(
            &mut differences,
            ZoneSetting::Protocols,
            &permanent.protocols,
            &runtime.protocols,
        );
        collection_pairs(
            &mut differences,
            ZoneSetting::SourcePorts,
            &permanent.source_ports,
            &runtime.source_ports,
        );
        collection_text(
            &mut differences,
            ZoneSetting::IcmpBlocks,
            &permanent.icmp_blocks,
            &runtime.icmp_blocks,
        );
        scalar(
            &mut differences,
            ZoneSetting::IcmpBlockInversion,
            permanent.icmp_block_inversion,
            runtime.icmp_block_inversion,
        );
        scalar(
            &mut differences,
            ZoneSetting::Masquerade,
            permanent.masquerade,
            runtime.masquerade,
        );
        collection_forward_ports(
            &mut differences,
            &permanent.forward_ports,
            &runtime.forward_ports,
        );
        collection_text(
            &mut differences,
            ZoneSetting::Interfaces,
            &permanent.interfaces,
            &runtime.interfaces,
        );
        collection_text(
            &mut differences,
            ZoneSetting::Sources,
            &permanent.sources,
            &runtime.sources,
        );
        collection_text(
            &mut differences,
            ZoneSetting::RichRules,
            &permanent.rich_rules,
            &runtime.rich_rules,
        );
        scalar(
            &mut differences,
            ZoneSetting::Forward,
            permanent.forward,
            runtime.forward,
        );
        scalar(
            &mut differences,
            ZoneSetting::IngressPriority,
            permanent.ingress_priority,
            runtime.ingress_priority,
        );
        scalar(
            &mut differences,
            ZoneSetting::EgressPriority,
            permanent.egress_priority,
            runtime.egress_priority,
        );

        let completeness = if permanent.unknown_keys.is_empty() && runtime.unknown_keys.is_empty() {
            ComparisonCompleteness::Complete
        } else {
            ComparisonCompleteness::Incomplete {
                permanent_unknown: permanent.unknown_keys.clone(),
                runtime_unknown: runtime.unknown_keys.clone(),
            }
        };

        Self {
            differences,
            completeness,
        }
    }

    /// Return true only when all keys were understood and no known value differs.
    pub fn is_in_sync(&self) -> bool {
        self.differences.is_empty() && self.completeness == ComparisonCompleteness::Complete
    }

    /// Return whether persisting runtime could change permanent known settings.
    pub fn has_runtime_only_differences(&self) -> bool {
        self.differences.iter().any(|difference| match difference {
            ZoneSettingDifference::Scalar { .. } => true,
            ZoneSettingDifference::Collection { runtime_only, .. } => !runtime_only.is_empty(),
        })
    }
}

trait IntoScalarValue {
    fn into_scalar_value(self) -> ScalarValue;
}

impl IntoScalarValue for &String {
    fn into_scalar_value(self) -> ScalarValue {
        ScalarValue::Text(self.clone())
    }
}

impl IntoScalarValue for bool {
    fn into_scalar_value(self) -> ScalarValue {
        ScalarValue::Boolean(self)
    }
}

impl IntoScalarValue for i32 {
    fn into_scalar_value(self) -> ScalarValue {
        ScalarValue::Integer(self)
    }
}

fn scalar<T>(
    differences: &mut Vec<ZoneSettingDifference>,
    setting: ZoneSetting,
    permanent: T,
    runtime: T,
) where
    T: Copy + PartialEq + IntoScalarValue,
{
    if permanent != runtime {
        differences.push(ZoneSettingDifference::Scalar {
            setting,
            permanent: permanent.into_scalar_value(),
            runtime: runtime.into_scalar_value(),
        });
    }
}

fn collection(
    differences: &mut Vec<ZoneSettingDifference>,
    setting: ZoneSetting,
    permanent: BTreeSet<CollectionValue>,
    runtime: BTreeSet<CollectionValue>,
) {
    if permanent != runtime {
        differences.push(ZoneSettingDifference::Collection {
            setting,
            permanent_only: permanent.difference(&runtime).cloned().collect(),
            runtime_only: runtime.difference(&permanent).cloned().collect(),
        });
    }
}

fn collection_text(
    differences: &mut Vec<ZoneSettingDifference>,
    setting: ZoneSetting,
    permanent: &[String],
    runtime: &[String],
) {
    collection(
        differences,
        setting,
        permanent
            .iter()
            .cloned()
            .map(CollectionValue::Text)
            .collect(),
        runtime.iter().cloned().map(CollectionValue::Text).collect(),
    );
}

fn collection_pairs(
    differences: &mut Vec<ZoneSettingDifference>,
    setting: ZoneSetting,
    permanent: &[(String, String)],
    runtime: &[(String, String)],
) {
    let values = |items: &[(String, String)]| {
        items
            .iter()
            .cloned()
            .map(|(first, second)| CollectionValue::Pair(first, second))
            .collect()
    };
    collection(differences, setting, values(permanent), values(runtime));
}

fn collection_forward_ports(
    differences: &mut Vec<ZoneSettingDifference>,
    permanent: &[(String, String, String, String)],
    runtime: &[(String, String, String, String)],
) {
    let values = |items: &[(String, String, String, String)]| {
        items
            .iter()
            .cloned()
            .map(|(port, protocol, to_port, to_address)| {
                CollectionValue::ForwardPort(port, protocol, to_port, to_address)
            })
            .collect()
    };
    collection(
        differences,
        ZoneSetting::ForwardPorts,
        values(permanent),
        values(runtime),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use zvariant::Value;

    fn owned<T>(value: T) -> OwnedValue
    where
        T: Into<Value<'static>>,
    {
        OwnedValue::try_from(value.into()).expect("test value should become owned")
    }

    #[test]
    fn omitted_keys_use_firewalld_defaults() {
        let snapshot = ZoneSettingsSnapshot::from_settings(HashMap::new()).unwrap();

        assert_eq!(snapshot.target, "default");
        assert!(snapshot.forward);
        assert_eq!(snapshot.ingress_priority, 0);
        assert_eq!(snapshot.egress_priority, 0);
        assert!(snapshot.services.is_empty());
        assert!(snapshot.unknown_keys.is_empty());
    }

    #[test]
    fn decodes_every_known_dictionary_shape() {
        let settings = HashMap::from([
            ("short".into(), owned("Work".to_string())),
            ("description".into(), owned("Office network".to_string())),
            ("target".into(), owned("DROP".to_string())),
            ("services".into(), owned(vec!["ssh".to_string()])),
            (
                "ports".into(),
                owned(vec![("8443".to_string(), "tcp".to_string())]),
            ),
            ("protocols".into(), owned(vec!["gre".to_string()])),
            (
                "source_ports".into(),
                owned(vec![("1024-65535".to_string(), "udp".to_string())]),
            ),
            (
                "icmp_blocks".into(),
                owned(vec!["echo-request".to_string()]),
            ),
            ("icmp_block_inversion".into(), owned(true)),
            ("masquerade".into(), owned(true)),
            (
                "forward_ports".into(),
                owned(vec![(
                    "443".to_string(),
                    "tcp".to_string(),
                    "8443".to_string(),
                    "192.0.2.5".to_string(),
                )]),
            ),
            ("interfaces".into(), owned(vec!["eth0".to_string()])),
            ("sources".into(), owned(vec!["192.0.2.0/24".to_string()])),
            (
                "rules_str".into(),
                owned(vec!["rule family=\"ipv4\" accept".to_string()]),
            ),
            ("forward".into(), owned(false)),
            ("ingress_priority".into(), owned(-10_i32)),
            ("egress_priority".into(), owned(20_i32)),
        ]);

        let snapshot = ZoneSettingsSnapshot::from_settings(settings).unwrap();

        assert_eq!(snapshot.short_name, "Work");
        assert_eq!(snapshot.ports, [("8443".into(), "tcp".into())]);
        assert_eq!(snapshot.forward_ports[0].3, "192.0.2.5");
        assert!(!snapshot.forward);
        assert_eq!(snapshot.ingress_priority, -10);
        assert_eq!(snapshot.egress_priority, 20);
    }

    #[test]
    fn rejects_incorrect_known_value_types() {
        let error = ZoneSettingsSnapshot::from_settings(HashMap::from([(
            "masquerade".into(),
            owned("yes".to_string()),
        )]))
        .unwrap_err();

        assert_eq!(
            error,
            ZoneSettingsParseError::InvalidType {
                key: "masquerade",
                expected: "boolean",
                actual: "s".into(),
            }
        );
    }

    #[test]
    fn preserves_unknown_keys_and_marks_comparison_incomplete() {
        let permanent = ZoneSettingsSnapshot::from_settings(HashMap::from([(
            "future_setting".into(),
            owned(true),
        )]))
        .unwrap();
        let runtime = ZoneSettingsSnapshot::default();

        let reconciliation = ZoneReconciliation::compare(&permanent, &runtime);

        assert_eq!(
            reconciliation.completeness,
            ComparisonCompleteness::Incomplete {
                permanent_unknown: BTreeSet::from(["future_setting".into()]),
                runtime_unknown: BTreeSet::new(),
            }
        );
        assert!(!reconciliation.is_in_sync());
    }

    #[test]
    fn collection_comparison_is_order_insensitive_and_exact() {
        let permanent = ZoneSettingsSnapshot {
            services: vec!["ssh".into(), "https".into()],
            ports: vec![("443".into(), "tcp".into())],
            rich_rules: vec!["rule accept".into()],
            ..ZoneSettingsSnapshot::default()
        };
        let runtime = ZoneSettingsSnapshot {
            services: vec!["https".into(), "ssh".into()],
            ports: vec![("443".into(), "udp".into())],
            rich_rules: vec!["rule  accept".into()],
            ..ZoneSettingsSnapshot::default()
        };

        let reconciliation = ZoneReconciliation::compare(&permanent, &runtime);

        assert_eq!(reconciliation.differences.len(), 2);
        assert!(
            !reconciliation.differences.iter().any(|difference| matches!(
                difference,
                ZoneSettingDifference::Collection {
                    setting: ZoneSetting::Services,
                    ..
                }
            ))
        );
        assert!(reconciliation.differences.iter().any(|difference| matches!(
            difference,
            ZoneSettingDifference::Collection {
                setting: ZoneSetting::RichRules,
                permanent_only,
                runtime_only,
            } if permanent_only == &[CollectionValue::Text("rule accept".into())]
                && runtime_only == &[CollectionValue::Text("rule  accept".into())]
        )));
    }

    #[test]
    fn scalar_differences_retain_both_values() {
        let permanent = ZoneSettingsSnapshot {
            target: "DROP".into(),
            masquerade: true,
            ingress_priority: -5,
            ..ZoneSettingsSnapshot::default()
        };
        let runtime = ZoneSettingsSnapshot {
            target: "ACCEPT".into(),
            ingress_priority: 5,
            ..ZoneSettingsSnapshot::default()
        };

        let reconciliation = ZoneReconciliation::compare(&permanent, &runtime);

        assert!(
            reconciliation
                .differences
                .contains(&ZoneSettingDifference::Scalar {
                    setting: ZoneSetting::Target,
                    permanent: ScalarValue::Text("DROP".into()),
                    runtime: ScalarValue::Text("ACCEPT".into()),
                })
        );
        assert!(
            reconciliation
                .differences
                .contains(&ZoneSettingDifference::Scalar {
                    setting: ZoneSetting::IngressPriority,
                    permanent: ScalarValue::Integer(-5),
                    runtime: ScalarValue::Integer(5),
                })
        );
    }
}
