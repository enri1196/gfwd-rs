//! Dialog state, validation, and context-drawer views.

use cosmic::iced::Alignment;
use cosmic::widget::{self, button};

use crate::core::ValidationError;
use crate::fl;
use crate::models::ZoneTarget;

use super::outcome::Outcome;

mod icmp;
mod interface;
mod ipset;
mod port;
mod rich_rule;
mod service;
mod source;
mod zone;

pub use icmp::{IcmpFormState, icmp_drawer};
pub use interface::{InterfaceFormState, interface_drawer};
#[allow(unused_imports)]
pub use ipset::{IpSetFormState, ipset_drawer, ipset_from_index, ipset_index};
pub use port::{PortFormState, PortKind, port_drawer, protocol_from_index, protocol_index};
pub use rich_rule::{RichRuleFormState, rich_rule_drawer};
pub use service::{ServiceFormState, service_drawer};
pub use source::{SourceFormState, source_drawer};
#[allow(unused_imports)]
pub use zone::{ZoneFormState, target_from_index, target_index, zone_drawer};

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

#[derive(Debug, Clone, Default)]
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

impl DialogState {
    /// Reset one form while retaining the selected port kind for port forms.
    pub fn reset(&mut self, kind: DialogKind) {
        self.operation_error = None;
        match kind {
            DialogKind::Zone => self.zone = ZoneFormState::default(),
            DialogKind::Service => self.service = ServiceFormState::default(),
            DialogKind::Port => {
                let kind = self.port.kind;
                self.port = PortFormState {
                    kind,
                    ..PortFormState::default()
                };
            }
            DialogKind::Interface => self.interface = InterfaceFormState::default(),
            DialogKind::Source => self.source = SourceFormState::default(),
            DialogKind::Icmp => self.icmp = IcmpFormState::default(),
            DialogKind::RichRule => self.rich_rule = RichRuleFormState::default(),
            DialogKind::IpSet => self.ipset = IpSetFormState::default(),
        }
    }
}

/// Authoritative state for every feature form.
pub(crate) type State = DialogState;

/// Immutable feature data used by dialog reduction.
pub(crate) struct Context<'a> {
    pub(crate) selected_zone: Option<&'a str>,
    pub(crate) interfaces: &'a [String],
    pub(crate) enabled_services: &'a [String],
    pub(crate) blocked_icmp: &'a [String],
    pub(crate) mutation_pending: bool,
}

/// Dialogs do not own asynchronous work; submissions are routed to domain slices.
#[derive(Debug)]
pub(crate) enum Effect {}

/// Validated, localization-independent submission intent.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Submission {
    Zone {
        name: String,
        description: String,
        target: ZoneTarget,
    },
    Service {
        zone: String,
        service: String,
    },
    Port {
        zone: String,
        port: String,
        protocol: String,
    },
    SourcePort {
        zone: String,
        port: String,
        protocol: String,
    },
    ForwardPort {
        zone: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    },
    Interface {
        zone: String,
        interface: String,
    },
    Source {
        zone: String,
        source: String,
    },
    Icmp {
        zone: String,
        icmp: String,
    },
    RichRule {
        zone: String,
        rule: String,
    },
    IpSet {
        name: String,
        ipset_type: String,
        entries: Vec<String>,
    },
}

/// Root coordination emitted by dialog reduction.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Request {
    Submit(Submission),
    CloseDrawer,
}

/// Reduce form editing, validation, cancellation, and submission.
pub(crate) fn update(
    state: &mut State,
    message: DialogMessage,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    if context.mutation_pending && matches!(message, DialogMessage::Submit(_)) {
        return Outcome::default();
    }
    match message {
        DialogMessage::ZoneNameChanged(value) => zone::name_changed(&mut state.zone, value),
        DialogMessage::ZoneDescriptionChanged(value) => {
            zone::description_changed(&mut state.zone, value);
        }
        DialogMessage::ZoneTargetSelected(index) => zone::target_selected(&mut state.zone, index),
        DialogMessage::ServiceSearchChanged(value) => {
            service::search_changed(&mut state.service, value);
        }
        DialogMessage::ServiceSelected(service) => {
            let Some(zone) = selected_zone(state, context.selected_zone) else {
                return Outcome::default();
            };
            if context.enabled_services.contains(&service) {
                state.operation_error = Some(fl!("error-service-already-enabled"));
                return Outcome::default();
            }
            return submit(Submission::Service { zone, service });
        }
        DialogMessage::PortNumberChanged(value) => port::number_changed(&mut state.port, value),
        DialogMessage::PortProtocolSelected(index) => {
            port::protocol_selected(&mut state.port, index);
        }
        DialogMessage::PortForwardDestIpChanged(value) => {
            port::destination_address_changed(&mut state.port, value);
        }
        DialogMessage::PortForwardDestPortChanged(value) => {
            port::destination_port_changed(&mut state.port, value);
        }
        DialogMessage::InterfaceSelected(index) => {
            interface::selected(&mut state.interface, index, context.interfaces);
        }
        DialogMessage::InterfaceNameChanged(value) => {
            interface::name_changed(&mut state.interface, value);
        }
        DialogMessage::SourceAddressChanged(value) => {
            source::address_changed(&mut state.source, value);
        }
        DialogMessage::IcmpSearchChanged(value) => icmp::search_changed(&mut state.icmp, value),
        DialogMessage::IcmpSelected(icmp) => {
            let Some(zone) = selected_zone(state, context.selected_zone) else {
                return Outcome::default();
            };
            if context.blocked_icmp.contains(&icmp) {
                state.operation_error = Some(fl!("error-icmp-already-blocked"));
                return Outcome::default();
            }
            return submit(Submission::Icmp { zone, icmp });
        }
        DialogMessage::RichRuleRawModeToggled(value) => {
            rich_rule::raw_mode_toggled(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleRawChanged(value) => {
            rich_rule::raw_changed(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleFamilySelected(value) => {
            rich_rule::family_selected(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleSourceChanged(value) => {
            rich_rule::source_changed(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleSourceInvertToggled(value) => {
            rich_rule::source_invert_toggled(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleDestinationChanged(value) => {
            rich_rule::destination_changed(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleDestinationInvertToggled(value) => {
            rich_rule::destination_invert_toggled(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleElementSelected(value) => {
            rich_rule::element_selected(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleElementValueChanged(value) => {
            rich_rule::element_value_changed(&mut state.rich_rule, value);
        }
        DialogMessage::RichRulePortProtocolSelected(value) => {
            rich_rule::port_protocol_selected(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleActionSelected(value) => {
            rich_rule::action_selected(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleRejectTypeChanged(value) => {
            rich_rule::reject_type_changed(&mut state.rich_rule, value);
        }
        DialogMessage::RichRuleMarkChanged(value) => {
            rich_rule::mark_changed(&mut state.rich_rule, value);
        }
        DialogMessage::IpSetNameChanged(value) => ipset::name_changed(&mut state.ipset, value),
        DialogMessage::IpSetTypeSelected(index) => {
            ipset::type_selected(&mut state.ipset, index);
        }
        DialogMessage::IpSetEntriesChanged(value) => {
            ipset::entries_changed(&mut state.ipset, value);
        }
        DialogMessage::Submit(kind) => return submit_form(state, kind, context.selected_zone),
        DialogMessage::Cancel(kind) => {
            state.reset(kind);
            return Outcome::request(Request::CloseDrawer);
        }
    }
    Outcome::default()
}

fn submit_form(
    state: &mut State,
    kind: DialogKind,
    selected: Option<&str>,
) -> Outcome<Effect, Request> {
    match kind {
        DialogKind::Zone => {
            let name = state.zone.name.trim().to_string();
            if name.is_empty() {
                state.operation_error = Some(fl!("error-required-field"));
                return Outcome::default();
            }
            submit(Submission::Zone {
                name,
                description: state.zone.description.trim().to_string(),
                target: state.zone.target.clone(),
            })
        }
        DialogKind::Service | DialogKind::Icmp => Outcome::default(),
        DialogKind::Port => {
            port::touch_submission_fields(&mut state.port);
            if !state.port.is_valid() {
                state.operation_error = Some(fl!("validation-fix-fields"));
                return Outcome::default();
            }
            let Some(zone) = selected_zone(state, selected) else {
                return Outcome::default();
            };
            submit(port::submission(&state.port, zone))
        }
        DialogKind::Interface => {
            if !interface::validate(&mut state.interface) {
                return Outcome::default();
            }
            let Some(zone) = selected_zone(state, selected) else {
                return Outcome::default();
            };
            submit(Submission::Interface {
                zone,
                interface: state.interface.interface.trim().to_string(),
            })
        }
        DialogKind::Source => {
            source::touch(&mut state.source);
            if !state.source.is_valid() {
                state.operation_error = Some(fl!("validation-fix-fields"));
                return Outcome::default();
            }
            let Some(zone) = selected_zone(state, selected) else {
                return Outcome::default();
            };
            submit(Submission::Source {
                zone,
                source: state.source.source.trim().to_string(),
            })
        }
        DialogKind::RichRule => {
            let Ok(rule) = state.rich_rule.generated_rule() else {
                state.operation_error = Some(fl!("validation-fix-fields"));
                return Outcome::default();
            };
            let Some(zone) = selected_zone(state, selected) else {
                return Outcome::default();
            };
            submit(Submission::RichRule { zone, rule })
        }
        DialogKind::IpSet => {
            ipset::touch_submission_fields(&mut state.ipset);
            if !state.ipset.is_valid() {
                state.operation_error = Some(fl!("validation-fix-fields"));
                return Outcome::default();
            }
            submit(Submission::IpSet {
                name: state.ipset.name.trim().to_string(),
                ipset_type: state.ipset.ipset_type.trim().to_string(),
                entries: ipset::split_entries(&state.ipset.entries),
            })
        }
    }
}

fn selected_zone(state: &mut State, selected: Option<&str>) -> Option<String> {
    selected.map(str::to_string).or_else(|| {
        state.operation_error = Some(fl!("error-select-zone-first"));
        None
    })
}

fn submit(submission: Submission) -> Outcome<Effect, Request> {
    Outcome::request(Request::Submit(submission))
}

/// Exhaustively run a dialog effect. Dialogs currently emit no effects.
pub(crate) fn effects(effect: Effect) -> cosmic::Task<DialogMessage> {
    match effect {}
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

pub fn drawer_footer_with_submit(
    kind: DialogKind,
    can_submit: bool,
) -> cosmic::Element<'static, DialogMessage> {
    let spacing = cosmic::theme::spacing();
    let submit_label = submit_label(kind);
    let submit_message = can_submit.then_some(DialogMessage::Submit(kind));

    widget::row::with_capacity(3)
        .push(widget::space::horizontal())
        .push(button::text(fl!("dialog-cancel")).on_press(DialogMessage::Cancel(kind)))
        .push(button::suggested(submit_label).on_press_maybe(submit_message))
        .spacing(spacing.space_s)
        .align_y(Alignment::Center)
        .into()
}

/// Builds a footer for picker drawers whose rows are the primary actions.
pub fn drawer_cancel_footer(kind: DialogKind) -> cosmic::Element<'static, DialogMessage> {
    widget::row::with_capacity(2)
        .push(widget::space::horizontal())
        .push(button::text(fl!("dialog-cancel")).on_press(DialogMessage::Cancel(kind)))
        .into()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context<'static> {
        Context {
            selected_zone: Some("public"),
            interfaces: &[],
            enabled_services: &[],
            blocked_icmp: &[],
            mutation_pending: false,
        }
    }

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
    fn resetting_port_form_preserves_each_selected_kind() {
        let mut dialogs = DialogState::default();
        for kind in [PortKind::Destination, PortKind::Source, PortKind::Forward] {
            dialogs.port.kind = kind;
            dialogs.port.port = "443".into();
            dialogs.port.protocol = "udp".into();
            dialogs.port.dest_ip = "192.0.2.1".into();
            dialogs.port.dest_port = "8443".into();
            dialogs.port.port_touched = true;
            dialogs.operation_error = Some("failure".into());

            dialogs.reset(DialogKind::Port);

            assert_eq!(dialogs.port.kind, kind);
            assert!(dialogs.port.port.is_empty());
            assert_eq!(dialogs.port.protocol, "tcp");
            assert!(dialogs.port.dest_ip.is_empty());
            assert!(dialogs.port.dest_port.is_empty());
            assert!(!dialogs.port.port_touched);
            assert!(dialogs.operation_error.is_none());
        }
    }

    #[test]
    fn submissions_route_all_port_kinds_with_normalized_values() {
        for (kind, expected) in [
            (
                PortKind::Destination,
                Submission::Port {
                    zone: "public".into(),
                    port: "443".into(),
                    protocol: "tcp".into(),
                },
            ),
            (
                PortKind::Source,
                Submission::SourcePort {
                    zone: "public".into(),
                    port: "443".into(),
                    protocol: "tcp".into(),
                },
            ),
            (
                PortKind::Forward,
                Submission::ForwardPort {
                    zone: "public".into(),
                    port: "443".into(),
                    protocol: "tcp".into(),
                    to_port: "8443".into(),
                    to_addr: "192.0.2.1".into(),
                },
            ),
        ] {
            let mut state = DialogState {
                port: PortFormState {
                    kind,
                    port: "443".into(),
                    protocol: "tcp".into(),
                    dest_port: "8443".into(),
                    dest_ip: "192.0.2.1".into(),
                    ..PortFormState::default()
                },
                ..DialogState::default()
            };
            let outcome = update(
                &mut state,
                DialogMessage::Submit(DialogKind::Port),
                context(),
            );
            assert!(matches!(
                outcome.requests.as_slice(),
                [Request::Submit(actual)] if actual == &expected
            ));
        }
    }
}
