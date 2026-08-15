use cosmic::iced::Length;
use cosmic::widget::{self, dropdown, settings};

use crate::core::{RichRuleAction, RichRuleElement, RichRuleError, RichRuleFamily, RichRuleSpec};
use crate::fl;

use super::port::PORT_PROTOCOLS;
use super::{protocol_from_index, protocol_index};

#[derive(Debug, Clone)]
pub enum Message {
    RawModeToggled(bool),
    RawChanged(String),
    FamilySelected(usize),
    SourceChanged(String),
    SourceInvertToggled(bool),
    DestinationChanged(String),
    DestinationInvertToggled(bool),
    ElementSelected(usize),
    ElementValueChanged(String),
    PortProtocolSelected(usize),
    ActionSelected(usize),
    RejectTypeChanged(String),
    MarkChanged(String),
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

pub(super) fn update(state: &mut RichRuleFormState, message: Message) {
    match message {
        Message::RawModeToggled(value) => state.raw_mode = value,
        Message::RawChanged(value) => state.raw_rule = value,
        Message::FamilySelected(value) => state.family = value,
        Message::SourceChanged(value) => state.source = value,
        Message::SourceInvertToggled(value) => state.source_invert = value,
        Message::DestinationChanged(value) => state.destination = value,
        Message::DestinationInvertToggled(value) => state.destination_invert = value,
        Message::ElementSelected(value) => {
            state.element = value;
            state.element_value.clear();
        }
        Message::ElementValueChanged(value) => state.element_value = value,
        Message::PortProtocolSelected(value) => {
            state.port_protocol = protocol_from_index(value);
        }
        Message::ActionSelected(value) => state.action = value,
        Message::RejectTypeChanged(value) => state.reject_type = value,
        Message::MarkChanged(value) => state.mark = value,
    }
}

pub fn rich_rule_drawer<'a>(state: &'a RichRuleFormState) -> cosmic::Element<'a, Message> {
    let mode = settings::section()
        .title(fl!("dialog-rich-rule-mode-section"))
        .add(
            settings::item::builder(fl!("dialog-rich-rule-raw-mode"))
                .description(fl!("dialog-rich-rule-raw-description"))
                .control(widget::toggler(state.raw_mode).on_toggle(Message::RawModeToggled)),
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
                    .on_input(Message::RawChanged)
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
                dropdown(family_labels, Some(state.family), Message::FamilySelected)
                    .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-source")).control(
                widget::text_input::text_input(fl!("rich-rule-address-placeholder"), &state.source)
                    .on_input(Message::SourceChanged)
                    .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-source-invert")).control(
                widget::toggler(state.source_invert).on_toggle(Message::SourceInvertToggled),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-destination")).control(
                widget::text_input::text_input(
                    fl!("rich-rule-address-placeholder"),
                    &state.destination,
                )
                .on_input(Message::DestinationChanged)
                .width(Length::Fill),
            ),
        )
        .add(
            settings::item::builder(fl!("rich-rule-destination-invert")).control(
                widget::toggler(state.destination_invert)
                    .on_toggle(Message::DestinationInvertToggled),
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
                    Message::ElementSelected,
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
                .on_input(Message::ElementValueChanged)
                .width(Length::Fill),
            ),
        );
    if state.element == 1 {
        element = element.add(
            settings::item::builder(fl!("dialog-port-protocol-label")).control(
                dropdown(
                    &PORT_PROTOCOLS,
                    protocol_index(&state.port_protocol),
                    Message::PortProtocolSelected,
                )
                .width(Length::Fill),
            ),
        );
    }

    let mut action = settings::section()
        .title(fl!("rich-rule-action-section"))
        .add(
            settings::item::builder(fl!("rich-rule-action")).control(
                dropdown(action_labels, Some(state.action), Message::ActionSelected)
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
                .on_input(Message::RejectTypeChanged)
                .width(Length::Fill),
            ),
        );
    } else if state.action == 3 {
        action = action.add(
            settings::item::builder(fl!("rich-rule-mark")).control(
                widget::text_input::text_input(fl!("rich-rule-mark-placeholder"), &state.mark)
                    .on_input(Message::MarkChanged)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_modes_preserves_raw_input() {
        let mut state = RichRuleFormState::default();
        update(&mut state, Message::RawModeToggled(true));
        update(
            &mut state,
            Message::RawChanged("  <rule><drop/></rule>  ".into()),
        );
        assert_eq!(state.generated_rule().unwrap(), "<rule><drop/></rule>");

        update(&mut state, Message::RawModeToggled(false));
        update(&mut state, Message::FamilySelected(1));
        update(&mut state, Message::SourceChanged("192.0.2.0/24".into()));
        update(&mut state, Message::SourceInvertToggled(true));
        update(
            &mut state,
            Message::DestinationChanged("198.51.100.1".into()),
        );
        update(&mut state, Message::DestinationInvertToggled(true));
        update(&mut state, Message::ElementSelected(0));
        update(&mut state, Message::ElementValueChanged("https".into()));
        update(&mut state, Message::PortProtocolSelected(1));
        update(&mut state, Message::ActionSelected(0));
        update(
            &mut state,
            Message::RejectTypeChanged("icmp-port-unreachable".into()),
        );
        update(&mut state, Message::MarkChanged("0x1".into()));

        let generated = state.generated_rule().unwrap();
        assert!(
            generated.contains("<service name=\"https\"/>")
                && generated.contains("family=\"ipv4\"")
        );

        update(&mut state, Message::RawModeToggled(true));
        assert_eq!(state.raw_rule, "  <rule><drop/></rule>  ");
    }
}
