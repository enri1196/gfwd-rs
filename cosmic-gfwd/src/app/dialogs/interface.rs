use cosmic::iced::Length;
use cosmic::widget::{self, dropdown, settings};

use crate::core::validate_interface_name;
use crate::fl;

use super::{DialogMessage, localized_validation_error};

#[derive(Debug, Clone, Default)]
pub struct InterfaceFormState {
    pub interface: String,
    pub error: Option<String>,
}

pub(super) fn selected(state: &mut InterfaceFormState, index: usize, interfaces: &[String]) {
    if index == 0 {
        state.interface.clear();
        state.error = None;
    } else if let Some(interface) = interfaces.get(index - 1) {
        state.interface = interface.clone();
        validate(state);
    }
}

pub(super) fn name_changed(state: &mut InterfaceFormState, value: String) {
    state.interface = value;
    validate(state);
}

pub(super) fn validate(state: &mut InterfaceFormState) -> bool {
    match validate_interface_name(state.interface.trim()) {
        Ok(()) => {
            state.error = None;
            true
        }
        Err(error) => {
            state.error = Some(localized_validation_error(error));
            false
        }
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
