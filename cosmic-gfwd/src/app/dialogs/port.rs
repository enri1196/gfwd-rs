use cosmic::iced::Length;
use cosmic::widget::{self, dropdown, settings};

use crate::core::{validate_forward_address, validate_port_protocol, validate_port_spec};
use crate::fl;

use super::{DialogMessage, Submission, localized_validation_error};

pub(super) const PORT_PROTOCOLS: [&str; 4] = ["tcp", "udp", "sctp", "dccp"];

/// Semantic kind of permanent port rule being created.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum PortKind {
    /// A destination port accepted by the zone.
    #[default]
    Destination,
    /// A source port accepted by the zone.
    Source,
    /// A destination port forwarded to another address or port.
    Forward,
}

#[derive(Debug, Clone)]
pub struct PortFormState {
    /// Port number or inclusive port range.
    pub port: String,
    /// Transport protocol for the port rule.
    pub protocol: String,
    /// Semantic kind of port rule being created.
    pub kind: PortKind,
    /// Optional forwarding destination address.
    pub dest_ip: String,
    /// Required forwarding destination port.
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
            kind: PortKind::default(),
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
            && (self.kind != PortKind::Forward
                || (validate_port_spec(&self.dest_port).is_ok()
                    && validate_forward_address(&self.dest_ip).is_ok()))
    }
}

pub(super) fn number_changed(state: &mut PortFormState, value: String) {
    state.port = value;
    state.port_touched = true;
}

pub(super) fn protocol_selected(state: &mut PortFormState, index: usize) {
    state.protocol = protocol_from_index(index);
}

pub(super) fn destination_address_changed(state: &mut PortFormState, value: String) {
    state.dest_ip = value;
    state.dest_ip_touched = true;
}

pub(super) fn destination_port_changed(state: &mut PortFormState, value: String) {
    state.dest_port = value;
    state.dest_port_touched = true;
}

pub(super) fn touch_submission_fields(state: &mut PortFormState) {
    state.port_touched = true;
    if state.kind == PortKind::Forward {
        state.dest_ip_touched = true;
        state.dest_port_touched = true;
    }
}

pub(super) fn submission(state: &PortFormState, zone: String) -> Submission {
    let port = state.port.trim().to_string();
    let protocol = state.protocol.trim().to_string();
    match state.kind {
        PortKind::Destination => Submission::Port {
            zone,
            port,
            protocol,
        },
        PortKind::Source => Submission::SourcePort {
            zone,
            port,
            protocol,
        },
        PortKind::Forward => Submission::ForwardPort {
            zone,
            port,
            protocol,
            to_port: state.dest_port.trim().to_string(),
            to_addr: state.dest_ip.trim().to_string(),
        },
    }
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
    if state.port_touched
        && let Err(error) = validate_port_spec(&state.port)
    {
        port_section = port_section.add(widget::text::caption(localized_validation_error(error)));
    }
    if let Err(error) = validate_port_protocol(&state.protocol) {
        port_section = port_section.add(widget::text::caption(localized_validation_error(error)));
    }
    sections.push(port_section.into());

    if state.kind == PortKind::Forward {
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
        if state.dest_ip_touched
            && let Err(error) = validate_forward_address(&state.dest_ip)
        {
            destination_section =
                destination_section.add(widget::text::caption(localized_validation_error(error)));
        }
        if state.dest_port_touched
            && let Err(error) = validate_port_spec(&state.dest_port)
        {
            destination_section =
                destination_section.add(widget::text::caption(localized_validation_error(error)));
        }
        sections.push(destination_section.into());
    }

    let content = settings::view_column(sections);

    content.into()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_port_kind_shares_port_and_protocol_validation() {
        for kind in [PortKind::Destination, PortKind::Source, PortKind::Forward] {
            let valid = PortFormState {
                kind,
                port: "1000-2000".into(),
                protocol: "sctp".into(),
                dest_port: "8443".into(),
                ..PortFormState::default()
            };
            assert!(valid.is_valid(), "{kind:?} should accept shared fields");

            let invalid_port = PortFormState {
                port: "70000".into(),
                ..valid.clone()
            };
            assert!(
                !invalid_port.is_valid(),
                "{kind:?} should reject an invalid port"
            );

            let invalid_protocol = PortFormState {
                protocol: "icmp".into(),
                ..valid
            };
            assert!(
                !invalid_protocol.is_valid(),
                "{kind:?} should reject an invalid protocol"
            );
        }
    }

    #[test]
    fn only_forward_ports_validate_destination_fields() {
        for kind in [PortKind::Destination, PortKind::Source] {
            let state = PortFormState {
                kind,
                port: "443".into(),
                protocol: "tcp".into(),
                dest_ip: "not an address".into(),
                dest_port: "not a port".into(),
                ..PortFormState::default()
            };
            assert!(
                state.is_valid(),
                "{kind:?} should ignore forwarding-only fields"
            );
        }

        let missing_destination_port = PortFormState {
            kind: PortKind::Forward,
            port: "443".into(),
            protocol: "tcp".into(),
            ..PortFormState::default()
        };
        assert!(!missing_destination_port.is_valid());

        let optional_destination_address = PortFormState {
            dest_port: "8443".into(),
            ..missing_destination_port
        };
        assert!(optional_destination_address.is_valid());
    }
}
