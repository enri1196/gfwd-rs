use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, dropdown, settings};

use crate::fl;
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
    PortNumberChanged(String),
    PortProtocolSelected(usize),
    PortForwardingToggled(bool),
    PortForwardDestIpChanged(String),
    PortForwardDestPortChanged(String),
    InterfaceSelected(usize),
    InterfaceNameChanged(String),
    SourceAddressChanged(String),
    IcmpTypeChanged(String),
    RichRuleChanged(String),
    IpSetNameChanged(String),
    IpSetTypeSelected(usize),
    IpSetEntriesChanged(String),
    Submit(DialogKind),
    Cancel(DialogKind),
}

#[derive(Debug, Clone)]
pub struct DialogState {
    pub zone: ZoneFormState,
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
            zone: ZoneFormState::default(),
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
        match kind {
            DialogKind::Zone => self.zone = ZoneFormState::default(),
            DialogKind::Port => self.port = PortFormState::default(),
            DialogKind::Interface => self.interface = InterfaceFormState::default(),
            DialogKind::Source => self.source = SourceFormState::default(),
            DialogKind::Icmp => self.icmp = IcmpFormState::default(),
            DialogKind::RichRule => self.rich_rule = RichRuleFormState::default(),
            DialogKind::IpSet => self.ipset = IpSetFormState::default(),
        }
    }
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
}

impl Default for PortFormState {
    fn default() -> Self {
        Self {
            port: String::new(),
            protocol: PORT_PROTOCOLS[0].to_string(),
            forwarding: false,
            dest_ip: String::new(),
            dest_port: String::new(),
        }
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
}

#[derive(Debug, Clone, Default)]
pub struct IcmpFormState {
    pub icmp_type: String,
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

pub fn port_drawer<'a>(state: &'a PortFormState) -> cosmic::Element<'a, DialogMessage> {
    let protocol_selected = protocol_index(&state.protocol);

    let mut sections = Vec::new();
    sections.push(
        settings::section()
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
            )
            .into(),
    );

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
        sections.push(
            settings::section()
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
                )
                .into(),
        );
    }

    let content = settings::view_column(sections);

    content.into()
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
    let content = settings::view_column(vec![
        settings::section()
            .title(fl!("dialog-source-section"))
            .add(
                settings::item::builder(fl!("dialog-source-label")).control(
                    widget::text_input::text_input(fl!("dialog-source-placeholder"), &state.source)
                        .on_input(DialogMessage::SourceAddressChanged)
                        .width(Length::Fill),
                ),
            )
            .into(),
    ]);

    content.into()
}

pub fn icmp_drawer<'a>(state: &'a IcmpFormState) -> cosmic::Element<'a, DialogMessage> {
    let content = settings::view_column(vec![
        settings::section()
            .title(fl!("dialog-icmp-section"))
            .add(
                settings::item::builder(fl!("dialog-icmp-type-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-icmp-type-placeholder"),
                        &state.icmp_type,
                    )
                    .on_input(DialogMessage::IcmpTypeChanged)
                    .width(Length::Fill),
                ),
            )
            .into(),
    ]);

    content.into()
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

pub fn drawer_footer(kind: DialogKind) -> cosmic::Element<'static, DialogMessage> {
    drawer_footer_with_submit(kind, true)
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
