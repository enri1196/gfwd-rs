use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, dropdown, settings};

use crate::core::{
    IPSET_TYPES, RichRuleAction, RichRuleElement, RichRuleError, RichRuleFamily, RichRuleSpec,
    ValidationError, validate_forward_address, validate_ipset_entry, validate_ipset_name,
    validate_ipset_type, validate_port_protocol, validate_port_spec, validate_source,
};
use crate::fl;
use crate::models::IcmpTypeInfo;
use crate::models::ZoneTarget;

const PORT_PROTOCOLS: [&str; 4] = ["tcp", "udp", "sctp", "dccp"];
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DialogKind {
    Zone,
    Service,
    Port,
    Interface,
    Source,
    Icmp,
    RichRule,
    IpSet,
}

#[derive(Debug, Clone)]
pub enum DialogMessage {
    ZoneNameChanged(String),
    ZoneDescriptionChanged(String),
    ZoneTargetSelected(usize),
    ServiceSearchChanged(String),
    ServiceSelected(String),
    PortNumberChanged(String),
    PortProtocolSelected(usize),
    PortForwardingToggled(bool),
    PortForwardDestIpChanged(String),
    PortForwardDestPortChanged(String),
    InterfaceSelected(usize),
    InterfaceNameChanged(String),
    SourceAddressChanged(String),
    IcmpSearchChanged(String),
    IcmpSelected(String),
    RichRuleRawModeToggled(bool),
    RichRuleRawChanged(String),
    RichRuleFamilySelected(usize),
    RichRuleSourceChanged(String),
    RichRuleSourceInvertToggled(bool),
    RichRuleDestinationChanged(String),
    RichRuleDestinationInvertToggled(bool),
    RichRuleElementSelected(usize),
    RichRuleElementValueChanged(String),
    RichRulePortProtocolSelected(usize),
    RichRuleActionSelected(usize),
    RichRuleRejectTypeChanged(String),
    RichRuleMarkChanged(String),
    IpSetNameChanged(String),
    IpSetTypeSelected(usize),
    IpSetEntriesChanged(String),
    Submit(DialogKind),
    Cancel(DialogKind),
}

#[derive(Debug, Clone)]
pub struct DialogState {
    /// Error returned by the current form submission.
    pub operation_error: Option<String>,
    pub zone: ZoneFormState,
    pub service: ServiceFormState,
    pub port: PortFormState,
    pub interface: InterfaceFormState,
    pub source: SourceFormState,
    pub icmp: IcmpFormState,
    pub rich_rule: RichRuleFormState,
    pub ipset: IpSetFormState,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            operation_error: None,
            zone: ZoneFormState::default(),
            service: ServiceFormState::default(),
            port: PortFormState::default(),
            interface: InterfaceFormState::default(),
            source: SourceFormState::default(),
            icmp: IcmpFormState::default(),
            rich_rule: RichRuleFormState::default(),
            ipset: IpSetFormState::default(),
        }
    }
}

impl DialogState {
    pub fn reset(&mut self, kind: DialogKind) {
        self.operation_error = None;
        match kind {
            DialogKind::Zone => self.zone = ZoneFormState::default(),
            DialogKind::Service => self.service = ServiceFormState::default(),
            DialogKind::Port => self.port = PortFormState::default(),
            DialogKind::Interface => self.interface = InterfaceFormState::default(),
            DialogKind::Source => self.source = SourceFormState::default(),
            DialogKind::Icmp => self.icmp = IcmpFormState::default(),
            DialogKind::RichRule => self.rich_rule = RichRuleFormState::default(),
            DialogKind::IpSet => self.ipset = IpSetFormState::default(),
        }
    }
}

/// Search state for the configured-service picker.
#[derive(Debug, Clone, Default)]
pub struct ServiceFormState {
    /// Case-insensitive service-name filter.
    pub search: String,
}

/// Adds a submission error above a drawer without introducing another scrollable.
pub fn drawer_with_error<'a>(
    content: cosmic::Element<'a, DialogMessage>,
    error: Option<&'a str>,
) -> cosmic::Element<'a, DialogMessage> {
    let mut column = widget::column::with_capacity(2).spacing(cosmic::theme::spacing().space_s);
    if let Some(error) = error {
        column = column.push(widget::text::caption(error));
    }
    column.push(content).into()
}

#[derive(Debug, Clone)]
pub struct ZoneFormState {
    pub name: String,
    pub description: String,
    pub target: ZoneTarget,
}

impl Default for ZoneFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            target: ZoneTarget::Default,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortFormState {
    pub port: String,
    pub protocol: String,
    pub forwarding: bool,
    pub dest_ip: String,
    pub dest_port: String,
    /// Whether the source port field has been edited.
    pub port_touched: bool,
    /// Whether the destination address field has been edited.
    pub dest_ip_touched: bool,
    /// Whether the destination port field has been edited.
    pub dest_port_touched: bool,
}

impl Default for PortFormState {
    fn default() -> Self {
        Self {
            port: String::new(),
            protocol: PORT_PROTOCOLS[0].to_string(),
            forwarding: false,
            dest_ip: String::new(),
            dest_port: String::new(),
            port_touched: false,
            dest_ip_touched: false,
            dest_port_touched: false,
        }
    }
}

impl PortFormState {
    /// Returns whether all currently visible port fields are valid.
    pub fn is_valid(&self) -> bool {
        validate_port_spec(&self.port).is_ok()
            && validate_port_protocol(&self.protocol).is_ok()
            && (!self.forwarding
                || (validate_port_spec(&self.dest_port).is_ok()
                    && validate_forward_address(&self.dest_ip).is_ok()))
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceFormState {
    pub interface: String,
    pub error: Option<String>,
}

impl Default for InterfaceFormState {
    fn default() -> Self {
        Self {
            interface: String::new(),
            error: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SourceFormState {
    pub source: String,
    /// Whether the source field has been edited.
    pub touched: bool,
}

impl SourceFormState {
    /// Returns whether the source is a valid IP address or CIDR network.
    pub fn is_valid(&self) -> bool {
        validate_source(&self.source).is_ok()
    }
}

#[derive(Debug, Clone, Default)]
pub struct IcmpFormState {
    /// Case-insensitive name and description filter.
    pub search: String,
}

/// State for structured generation and advanced raw rich-rule entry.
#[derive(Debug, Clone)]
pub struct RichRuleFormState {
    /// Whether advanced raw mode is active.
    pub raw_mode: bool,
    /// Lossless raw rule text.
    pub raw_rule: String,
    /// Family dropdown index: any, IPv4, or IPv6.
    pub family: usize,
    /// Optional source address/network.
    pub source: String,
    /// Whether the source match is inverted.
    pub source_invert: bool,
    /// Optional destination address/network.
    pub destination: String,
    /// Whether the destination match is inverted.
    pub destination_invert: bool,
    /// Element dropdown index: service, port, or protocol.
    pub element: usize,
    /// Service, port, or protocol value for the selected element.
    pub element_value: String,
    /// Protocol used by a port element.
    pub port_protocol: String,
    /// Action dropdown index: accept, reject, drop, or mark.
    pub action: usize,
    /// Optional reject type.
    pub reject_type: String,
    /// Mark value for the mark action.
    pub mark: String,
}

impl Default for RichRuleFormState {
    fn default() -> Self {
        Self {
            raw_mode: false,
            raw_rule: String::new(),
            family: 0,
            source: String::new(),
            source_invert: false,
            destination: String::new(),
            destination_invert: false,
            element: 0,
            element_value: String::new(),
            port_protocol: PORT_PROTOCOLS[0].to_string(),
            action: 0,
            reject_type: String::new(),
            mark: String::new(),
        }
    }
}

impl RichRuleFormState {
    /// Returns validated raw text or generated XML for submission.
    pub fn generated_rule(&self) -> Result<String, RichRuleError> {
        if self.raw_mode {
            return if self.raw_rule.trim().is_empty() {
                Err(RichRuleError::MissingElement)
            } else {
                Ok(self.raw_rule.trim().to_string())
            };
        }
        let optional_address = |value: &str, invert| {
            (!value.trim().is_empty()).then(|| (value.trim().to_string(), invert))
        };
        RichRuleSpec {
            family: match self.family {
                1 => Some(RichRuleFamily::Ipv4),
                2 => Some(RichRuleFamily::Ipv6),
                _ => None,
            },
            source: optional_address(&self.source, self.source_invert),
            destination: optional_address(&self.destination, self.destination_invert),
            element: match self.element {
                1 => RichRuleElement::Port {
                    port: self.element_value.trim().to_string(),
                    protocol: self.port_protocol.clone(),
                },
                2 => RichRuleElement::Protocol(self.element_value.trim().to_string()),
                _ => RichRuleElement::Service(self.element_value.trim().to_string()),
            },
            action: match self.action {
                1 => RichRuleAction::Reject(
                    (!self.reject_type.trim().is_empty())
                        .then(|| self.reject_type.trim().to_string()),
                ),
                2 => RichRuleAction::Drop,
                3 => RichRuleAction::Mark(self.mark.trim().to_string()),
                _ => RichRuleAction::Accept,
            },
        }
        .to_xml()
    }
}

#[derive(Debug, Clone)]
pub struct IpSetFormState {
    pub name: String,
    pub ipset_type: String,
    pub entries: String,
    /// Whether the name field has been edited.
    pub name_touched: bool,
    /// Whether the initial-entry field has been edited.
    pub entries_touched: bool,
}

impl Default for IpSetFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            ipset_type: IPSET_TYPES[0].to_string(),
            entries: String::new(),
            name_touched: false,
            entries_touched: false,
        }
    }
}

impl IpSetFormState {
    /// Returns whether the name, type, and every newline-separated entry are valid.
    pub fn is_valid(&self) -> bool {
        validate_ipset_name(&self.name).is_ok()
            && validate_ipset_type(&self.ipset_type).is_ok()
            && self
                .entries
                .lines()
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .all(|entry| validate_ipset_entry(entry, &self.ipset_type).is_ok())
    }
}

pub fn zone_drawer<'a>(state: &'a ZoneFormState) -> cosmic::Element<'a, DialogMessage> {
    let target_labels = target_labels();
    let target_selected = Some(target_index(&state.target));

    let content = settings::view_column(vec![
        settings::section()
            .title(fl!("dialog-zone-section-basic"))
            .add(
                settings::item::builder(fl!("dialog-zone-name-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-zone-name-placeholder"),
                        &state.name,
                    )
                    .on_input(DialogMessage::ZoneNameChanged)
                    .width(Length::Fill),
                ),
            )
            .add(
                settings::item::builder(fl!("dialog-zone-description-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-zone-description-placeholder"),
                        &state.description,
                    )
                    .on_input(DialogMessage::ZoneDescriptionChanged)
                    .width(Length::Fill),
                ),
            )
            .into(),
        settings::section()
            .title(fl!("dialog-zone-section-target"))
            .add(
                settings::item::builder(fl!("dialog-zone-target-label")).control(
                    dropdown(
                        target_labels,
                        target_selected,
                        DialogMessage::ZoneTargetSelected,
                    )
                    .width(Length::Fill),
                ),
            )
            .into(),
    ]);

    content.into()
}

/// Builds the searchable configured-service picker.
pub fn service_drawer<'a>(
    state: &'a ServiceFormState,
    services: &'a [String],
    enabled: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, DialogMessage> {
    let filter = state.search.trim().to_lowercase();
    let mut section = settings::section()
        .title(fl!("dialog-service-section"))
        .add(
            widget::text_input::text_input(fl!("dialog-service-search-placeholder"), &state.search)
                .on_input(DialogMessage::ServiceSearchChanged)
                .width(Length::Fill),
        );

    if loading {
        section = section.add(widget::text::caption(fl!("dialog-service-loading")));
    } else if let Some(error) = error {
        section = section.add(widget::text::caption(error));
    } else {
        let mut visible = 0;
        for service in services
            .iter()
            .filter(|service| filter.is_empty() || service.to_lowercase().contains(&filter))
        {
            visible += 1;
            let is_enabled = enabled.iter().any(|item| item == service);
            let label = if is_enabled {
                fl!("dialog-service-enabled", service = service)
            } else {
                service.clone()
            };
            let message = (!is_enabled).then(|| DialogMessage::ServiceSelected(service.clone()));
            section = section.add(
                button::standard(label)
                    .width(Length::Fill)
                    .on_press_maybe(message),
            );
        }
        if visible == 0 {
            section = section.add(widget::text::caption(fl!("dialog-service-empty")));
        }
    }

    settings::view_column(vec![section.into()]).into()
}

pub fn port_drawer<'a>(state: &'a PortFormState) -> cosmic::Element<'a, DialogMessage> {
    let protocol_selected = protocol_index(&state.protocol);

    let mut sections = Vec::new();
    let mut port_section = settings::section()
        .title(fl!("dialog-port-section"))
        .add(
            settings::item::builder(fl!("dialog-port-label")).control(
                widget::text_input::text_input(fl!("dialog-port-placeholder"), &state.port)
                    .on_input(DialogMessage::PortNumberChanged)
                    .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("dialog-port-protocol-label")).control(
                dropdown(
                    &PORT_PROTOCOLS,
                    protocol_selected,
                    DialogMessage::PortProtocolSelected,
                )
                .width(Length::Fill),
            ),
        );
    if state.port_touched {
        if let Err(error) = validate_port_spec(&state.port) {
            port_section =
                port_section.add(widget::text::caption(localized_validation_error(error)));
        }
    }
    if let Err(error) = validate_port_protocol(&state.protocol) {
        port_section = port_section.add(widget::text::caption(localized_validation_error(error)));
    }
    sections.push(port_section.into());

    sections.push(
        settings::section()
            .title(fl!("dialog-port-forwarding-section"))
            .add(
                settings::item::builder(fl!("dialog-port-forwarding-toggle")).control(
                    widget::toggler(state.forwarding)
                        .on_toggle(DialogMessage::PortForwardingToggled),
                ),
            )
            .into(),
    );

    if state.forwarding {
        let mut destination_section = settings::section()
            .title(fl!("dialog-port-forward-destination-section"))
            .add(
                settings::item::builder(fl!("dialog-port-dest-ip-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-port-dest-ip-placeholder"),
                        &state.dest_ip,
                    )
                    .on_input(DialogMessage::PortForwardDestIpChanged)
                    .width(Length::Fill),
                ),
            )
            .add(
                settings::item::builder(fl!("dialog-port-dest-port-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-port-dest-port-placeholder"),
                        &state.dest_port,
                    )
                    .on_input(DialogMessage::PortForwardDestPortChanged)
                    .width(Length::Fill),
                ),
            );
        if state.dest_ip_touched {
            if let Err(error) = validate_forward_address(&state.dest_ip) {
                destination_section = destination_section
                    .add(widget::text::caption(localized_validation_error(error)));
            }
        }
        if state.dest_port_touched {
            if let Err(error) = validate_port_spec(&state.dest_port) {
                destination_section = destination_section
                    .add(widget::text::caption(localized_validation_error(error)));
            }
        }
        sections.push(destination_section.into());
    }

    let content = settings::view_column(sections);

    content.into()
}

/// Converts a typed validation failure into localized user-facing text.
pub fn localized_validation_error(error: ValidationError) -> String {
    match error {
        ValidationError::Required => fl!("validation-required"),
        ValidationError::InvalidPort => fl!("validation-port"),
        ValidationError::ReversedPortRange => fl!("validation-port-range-order"),
        ValidationError::InvalidProtocol => fl!("validation-protocol"),
        ValidationError::InvalidIpAddress => fl!("validation-ip-address"),
        ValidationError::InterfaceNameTooLong => fl!("validation-interface-length"),
        ValidationError::InvalidInterfaceName => fl!("validation-interface-name"),
        ValidationError::InvalidSource => fl!("validation-source"),
        ValidationError::InvalidCidrPrefix => fl!("validation-cidr-prefix"),
        ValidationError::IpSetNameTooLong => fl!("validation-ipset-name-length"),
        ValidationError::InvalidIpSetName => fl!("validation-ipset-name"),
        ValidationError::IpSetNameStartsWithDash => fl!("validation-ipset-name-dash"),
        ValidationError::InvalidIpSetType => fl!("validation-ipset-type"),
        ValidationError::InvalidIpSetEntry => fl!("validation-ipset-entry"),
        ValidationError::InvalidMacAddress => fl!("validation-mac-address"),
    }
}

pub fn interface_drawer<'a>(
    state: &'a InterfaceFormState,
    interfaces: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, DialogMessage> {
    let mut section = settings::section().title(fl!("dialog-interface-section"));
    let show_manual_entry = interfaces.is_empty() && !loading;

    if loading {
        section = section.add(widget::text::caption(fl!("dialog-interface-loading")));
    } else if interfaces.is_empty() {
        section = section.add(widget::text::caption(fl!("dialog-interface-empty")));
    } else {
        section = section.add(widget::text::caption(fl!(
            "dialog-interface-count",
            count = interfaces.len()
        )));
    }

    if show_manual_entry {
        section = section.add(widget::text::caption(fl!("dialog-interface-manual-info")));
    }

    if let Some(error) = error {
        section = section.add(widget::text::caption(error));
    }

    let mut options = Vec::with_capacity(interfaces.len() + 1);
    let placeholder = if loading {
        fl!("dialog-interface-loading")
    } else if interfaces.is_empty() {
        fl!("dialog-interface-empty")
    } else {
        fl!("dialog-interface-select-placeholder")
    };
    options.push(placeholder.to_string());
    options.extend(interfaces.iter().cloned());

    let selected = if state.interface.is_empty() {
        Some(0)
    } else {
        interfaces
            .iter()
            .position(|iface| iface == &state.interface)
            .map(|index| index + 1)
            .or(Some(0))
    };

    section = section.add(
        settings::item::builder(fl!("dialog-interface-name-label")).control(
            dropdown(options, selected, DialogMessage::InterfaceSelected).width(Length::Fill),
        ),
    );

    if show_manual_entry {
        section = section.add(
            settings::item::builder(fl!("dialog-interface-manual-label")).control(
                widget::text_input::text_input(
                    fl!("dialog-interface-name-placeholder"),
                    &state.interface,
                )
                .on_input(DialogMessage::InterfaceNameChanged)
                .width(Length::Fill),
            ),
        );
    }

    let content = settings::view_column(vec![section.into()]);

    content.into()
}

pub fn source_drawer<'a>(state: &'a SourceFormState) -> cosmic::Element<'a, DialogMessage> {
    let mut section = settings::section().title(fl!("dialog-source-section")).add(
        settings::item::builder(fl!("dialog-source-label")).control(
            widget::text_input::text_input(fl!("dialog-source-placeholder"), &state.source)
                .on_input(DialogMessage::SourceAddressChanged)
                .width(Length::Fill),
        ),
    );
    if state.touched {
        if let Err(error) = validate_source(&state.source) {
            section = section.add(widget::text::caption(localized_validation_error(error)));
        }
    }
    let content = settings::view_column(vec![section.into()]);

    content.into()
}

pub fn icmp_drawer<'a>(
    state: &'a IcmpFormState,
    types: &'a [IcmpTypeInfo],
    blocked: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, DialogMessage> {
    let filter = state.search.trim().to_lowercase();
    let mut section = settings::section().title(fl!("dialog-icmp-section")).add(
        widget::text_input::text_input(fl!("dialog-icmp-search-placeholder"), &state.search)
            .on_input(DialogMessage::IcmpSearchChanged)
            .width(Length::Fill),
    );

    if loading {
        section = section.add(widget::text::caption(fl!("dialog-icmp-loading")));
    } else if let Some(error) = error {
        section = section.add(widget::text::caption(error));
    } else {
        let mut visible = 0;
        for icmp in types.iter().filter(|icmp| {
            filter.is_empty()
                || icmp.name.to_lowercase().contains(&filter)
                || icmp.description.to_lowercase().contains(&filter)
        }) {
            visible += 1;
            let is_blocked = blocked.contains(&icmp.name);
            let label = if is_blocked {
                fl!("dialog-icmp-blocked", name = icmp.name.as_str())
            } else {
                fl!("dialog-icmp-add", name = icmp.name.as_str())
            };
            section = section.add(
                settings::item::builder(icmp.name.as_str())
                    .description(icmp.description.as_str())
                    .control(button::standard(label).on_press_maybe(
                        (!is_blocked).then(|| DialogMessage::IcmpSelected(icmp.name.clone())),
                    )),
            );
        }
        if visible == 0 {
            section = section.add(widget::text::caption(fl!("dialog-icmp-empty")));
        }
    }

    settings::view_column(vec![section.into()]).into()
}

pub fn rich_rule_drawer<'a>(state: &'a RichRuleFormState) -> cosmic::Element<'a, DialogMessage> {
    let mode = settings::section()
        .title(fl!("dialog-rich-rule-mode-section"))
        .add(
            settings::item::builder(fl!("dialog-rich-rule-raw-mode"))
                .description(fl!("dialog-rich-rule-raw-description"))
                .control(
                    widget::toggler(state.raw_mode)
                        .on_toggle(DialogMessage::RichRuleRawModeToggled),
                ),
        );

    if state.raw_mode {
        let mut raw = settings::section()
            .title(fl!("dialog-rich-rule-raw-section"))
            .add(
                settings::item::builder(fl!("dialog-rich-rule-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-rich-rule-placeholder"),
                        &state.raw_rule,
                    )
                    .on_input(DialogMessage::RichRuleRawChanged)
                    .width(Length::Fill),
                ),
            );
        if state.raw_rule.trim().is_empty() {
            raw = raw.add(widget::text::caption(fl!("validation-rich-rule-raw")));
        }
        return settings::view_column(vec![mode.into(), raw.into()]).into();
    }

    let family_labels = vec![
        fl!("rich-rule-family-any"),
        fl!("rich-rule-family-ipv4"),
        fl!("rich-rule-family-ipv6"),
    ];
    let element_labels = vec![
        fl!("rich-rule-element-service"),
        fl!("rich-rule-element-port"),
        fl!("rich-rule-element-protocol"),
    ];
    let action_labels = vec![
        fl!("rich-rule-action-accept"),
        fl!("rich-rule-action-reject"),
        fl!("rich-rule-action-drop"),
        fl!("rich-rule-action-mark"),
    ];
    let addresses = settings::section()
        .title(fl!("rich-rule-addresses-section"))
        .add(
            settings::item::builder(fl!("rich-rule-family")).control(
                dropdown(
                    family_labels,
                    Some(state.family),
                    DialogMessage::RichRuleFamilySelected,
                )
                .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-source")).control(
                widget::text_input::text_input(fl!("rich-rule-address-placeholder"), &state.source)
                    .on_input(DialogMessage::RichRuleSourceChanged)
                    .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-source-invert")).control(
                widget::toggler(state.source_invert)
                    .on_toggle(DialogMessage::RichRuleSourceInvertToggled),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-destination")).control(
                widget::text_input::text_input(
                    fl!("rich-rule-address-placeholder"),
                    &state.destination,
                )
                .on_input(DialogMessage::RichRuleDestinationChanged)
                .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-destination-invert")).control(
                widget::toggler(state.destination_invert)
                    .on_toggle(DialogMessage::RichRuleDestinationInvertToggled),
            ),
        );

    let value_label = match state.element {
        1 => fl!("rich-rule-port"),
        2 => fl!("rich-rule-protocol"),
        _ => fl!("rich-rule-service"),
    };
    let mut element = settings::section()
        .title(fl!("rich-rule-element-section"))
        .add(
            settings::item::builder(fl!("rich-rule-element")).control(
                dropdown(
                    element_labels,
                    Some(state.element),
                    DialogMessage::RichRuleElementSelected,
                )
                .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(value_label).control(
                widget::text_input::text_input(
                    fl!("rich-rule-element-placeholder"),
                    &state.element_value,
                )
                .on_input(DialogMessage::RichRuleElementValueChanged)
                .width(Length::Fill),
            ),
        );
    if state.element == 1 {
        element = element.add(
            settings::item::builder(fl!("dialog-port-protocol-label")).control(
                dropdown(
                    &PORT_PROTOCOLS,
                    protocol_index(&state.port_protocol),
                    DialogMessage::RichRulePortProtocolSelected,
                )
                .width(Length::Fill),
            ),
        );
    }

    let mut action = settings::section()
        .title(fl!("rich-rule-action-section"))
        .add(
            settings::item::builder(fl!("rich-rule-action")).control(
                dropdown(
                    action_labels,
                    Some(state.action),
                    DialogMessage::RichRuleActionSelected,
                )
                .width(Length::Fill),
            ),
        );
    if state.action == 1 {
        action = action.add(
            settings::item::builder(fl!("rich-rule-reject-type")).control(
                widget::text_input::text_input(
                    fl!("rich-rule-reject-type-placeholder"),
                    &state.reject_type,
                )
                .on_input(DialogMessage::RichRuleRejectTypeChanged)
                .width(Length::Fill),
            ),
        );
    } else if state.action == 3 {
        action = action.add(
            settings::item::builder(fl!("rich-rule-mark")).control(
                widget::text_input::text_input(fl!("rich-rule-mark-placeholder"), &state.mark)
                    .on_input(DialogMessage::RichRuleMarkChanged)
                    .width(Length::Fill),
            ),
        );
    }

    let preview = match state.generated_rule() {
        Ok(rule) => settings::section()
            .title(fl!("rich-rule-preview-section"))
            .add(widget::text::body(rule)),
        Err(error) => settings::section()
            .title(fl!("rich-rule-preview-section"))
            .add(widget::text::caption(localized_rich_rule_error(error))),
    };

    settings::view_column(vec![
        mode.into(),
        addresses.into(),
        element.into(),
        action.into(),
        preview.into(),
    ])
    .into()
}

fn localized_rich_rule_error(error: RichRuleError) -> String {
    match error {
        RichRuleError::MissingElement => fl!("validation-rich-rule-element"),
        RichRuleError::InvalidAddress => fl!("validation-rich-rule-address"),
        RichRuleError::InvalidPort => fl!("validation-port"),
        RichRuleError::InvalidPortProtocol => fl!("validation-protocol"),
        RichRuleError::InvalidProtocol => fl!("validation-rich-rule-protocol"),
        RichRuleError::InvalidIdentifier => fl!("validation-rich-rule-identifier"),
        RichRuleError::InvalidMark => fl!("validation-rich-rule-mark"),
    }
}

pub fn ipset_drawer<'a>(state: &'a IpSetFormState) -> cosmic::Element<'a, DialogMessage> {
    let type_selected = ipset_index(&state.ipset_type);

    let mut section = settings::section()
        .title(fl!("dialog-ipset-section"))
        .add(
            settings::item::builder(fl!("dialog-ipset-name-label")).control(
                widget::text_input::text_input(fl!("dialog-ipset-name-placeholder"), &state.name)
                    .on_input(DialogMessage::IpSetNameChanged)
                    .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("dialog-ipset-type-label")).control(
                dropdown(
                    &IPSET_TYPES,
                    type_selected,
                    DialogMessage::IpSetTypeSelected,
                )
                .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("dialog-ipset-entries-label")).control(
                widget::text_input::text_input(
                    fl!("dialog-ipset-entries-placeholder"),
                    &state.entries,
                )
                .on_input(DialogMessage::IpSetEntriesChanged)
                .width(Length::Fill),
            ),
        );
    if state.name_touched {
        if let Err(error) = validate_ipset_name(&state.name) {
            section = section.add(widget::text::caption(localized_validation_error(error)));
        }
    }
    if state.entries_touched {
        if let Some(error) = state
            .entries
            .lines()
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .find_map(|entry| validate_ipset_entry(entry, &state.ipset_type).err())
        {
            section = section.add(widget::text::caption(localized_validation_error(error)));
        }
    }
    let content = settings::view_column(vec![section.into()]);

    content.into()
}

pub fn drawer_footer_with_submit(
    kind: DialogKind,
    can_submit: bool,
) -> cosmic::Element<'static, DialogMessage> {
    let spacing = cosmic::theme::spacing();
    let submit_label = submit_label(kind);
    let submit_message = can_submit.then_some(DialogMessage::Submit(kind));

    widget::row::with_capacity(3)
        .push(widget::horizontal_space())
        .push(button::text(fl!("dialog-cancel")).on_press(DialogMessage::Cancel(kind)))
        .push(button::suggested(submit_label).on_press_maybe(submit_message))
        .spacing(spacing.space_s)
        .align_y(Alignment::Center)
        .into()
}

/// Builds a footer for picker drawers whose rows are the primary actions.
pub fn drawer_cancel_footer(kind: DialogKind) -> cosmic::Element<'static, DialogMessage> {
    widget::row::with_capacity(2)
        .push(widget::horizontal_space())
        .push(button::text(fl!("dialog-cancel")).on_press(DialogMessage::Cancel(kind)))
        .into()
}

pub fn target_from_index(index: usize) -> ZoneTarget {
    match index {
        1 => ZoneTarget::Accept,
        2 => ZoneTarget::Drop,
        3 => ZoneTarget::Reject,
        _ => ZoneTarget::Default,
    }
}

pub fn target_index(target: &ZoneTarget) -> usize {
    match target {
        ZoneTarget::Default => 0,
        ZoneTarget::Accept => 1,
        ZoneTarget::Drop => 2,
        ZoneTarget::Reject => 3,
        ZoneTarget::Other(_) => 0,
    }
}

pub fn protocol_from_index(index: usize) -> String {
    PORT_PROTOCOLS
        .get(index)
        .unwrap_or(&PORT_PROTOCOLS[0])
        .to_string()
}

pub fn protocol_index(protocol: &str) -> Option<usize> {
    PORT_PROTOCOLS.iter().position(|value| *value == protocol)
}

pub fn ipset_from_index(index: usize) -> String {
    IPSET_TYPES
        .get(index)
        .unwrap_or(&IPSET_TYPES[0])
        .to_string()
}

pub fn ipset_index(ipset_type: &str) -> Option<usize> {
    IPSET_TYPES.iter().position(|value| *value == ipset_type)
}

fn submit_label(kind: DialogKind) -> String {
    match kind {
        DialogKind::Zone => fl!("dialog-submit-add-zone"),
        DialogKind::Service => fl!("dialog-submit-add-service"),
        DialogKind::Port => fl!("dialog-submit-add-port"),
        DialogKind::Interface => fl!("dialog-submit-add-interface"),
        DialogKind::Source => fl!("dialog-submit-add-source"),
        DialogKind::Icmp => fl!("dialog-submit-add-icmp"),
        DialogKind::RichRule => fl!("dialog-submit-add-rich-rule"),
        DialogKind::IpSet => fl!("dialog-submit-add-ipset"),
    }
}

fn target_labels() -> Vec<String> {
    vec![
        fl!("dialog-target-default"),
        fl!("dialog-target-accept"),
        fl!("dialog-target-drop"),
        fl!("dialog-target-reject"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resetting_one_drawer_preserves_other_form_state() {
        let mut dialogs = DialogState::default();
        dialogs.zone.name = "kept-zone".to_string();
        dialogs.rich_rule.raw_mode = true;
        dialogs.rich_rule.raw_rule = "<rule><accept/></rule>".to_string();
        dialogs.operation_error = Some("failure".to_string());

        dialogs.reset(DialogKind::RichRule);

        assert_eq!(dialogs.zone.name, "kept-zone");
        assert!(!dialogs.rich_rule.raw_mode);
        assert!(dialogs.rich_rule.raw_rule.is_empty());
        assert!(dialogs.operation_error.is_none());
    }

    #[test]
    fn switching_rich_rule_modes_preserves_raw_input() {
        let mut state = RichRuleFormState {
            raw_mode: true,
            raw_rule: "  <rule><drop/></rule>  ".to_string(),
            ..RichRuleFormState::default()
        };
        assert_eq!(
            state.generated_rule().unwrap(),
            "<rule><drop/></rule>".to_string()
        );

        state.raw_mode = false;
        state.element_value = "https".to_string();
        assert!(
            state
                .generated_rule()
                .unwrap()
                .contains("<service name=\"https\"/>")
        );

        state.raw_mode = true;
        assert_eq!(state.raw_rule, "  <rule><drop/></rule>  ");
    }
}
