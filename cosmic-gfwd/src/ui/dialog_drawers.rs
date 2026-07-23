use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, dropdown, settings};

use crate::core::{
    ValidationError, validate_forward_address, validate_port_protocol, validate_port_spec,
    validate_source,
};
use crate::fl;
use crate::models::IcmpTypeInfo;
use crate::models::ZoneTarget;

const PORT_PROTOCOLS: [&str; 4] = ["tcp", "udp", "sctp", "dccp"];
const IPSET_TYPES: [&str; 7] = [
    "hash:ip",
    "hash:net",
    "hash:ip,port",
    "hash:net,port",
    "hash:mac",
    "bitmap:ip",
    "list:set",
];

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
    RichRuleChanged(String),
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

#[derive(Debug, Clone, Default)]
pub struct RichRuleFormState {
    pub rule: String,
}

#[derive(Debug, Clone)]
pub struct IpSetFormState {
    pub name: String,
    pub ipset_type: String,
    pub entries: String,
}

impl Default for IpSetFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            ipset_type: IPSET_TYPES[0].to_string(),
            entries: String::new(),
        }
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
    let content = settings::view_column(vec![
        settings::section()
            .title(fl!("dialog-rich-rule-section"))
            .add(
                settings::item::builder(fl!("dialog-rich-rule-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-rich-rule-placeholder"),
                        &state.rule,
                    )
                    .on_input(DialogMessage::RichRuleChanged)
                    .width(Length::Fill),
                ),
            )
            .into(),
    ]);

    content.into()
}

pub fn ipset_drawer<'a>(state: &'a IpSetFormState) -> cosmic::Element<'a, DialogMessage> {
    let type_selected = ipset_index(&state.ipset_type);

    let content = settings::view_column(vec![
        settings::section()
            .title(fl!("dialog-ipset-section"))
            .add(
                settings::item::builder(fl!("dialog-ipset-name-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-ipset-name-placeholder"),
                        &state.name,
                    )
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
            )
            .into(),
    ]);

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
