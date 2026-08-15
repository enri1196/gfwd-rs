use cosmic::iced::Length;
use cosmic::widget::{self, settings};

use crate::core::validate_source;
use crate::fl;

use super::{DialogMessage, localized_validation_error};

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

pub(super) fn address_changed(state: &mut SourceFormState, value: String) {
    state.source = value;
    state.touched = true;
}

pub(super) fn touch(state: &mut SourceFormState) {
    state.touched = true;
}

pub fn source_drawer<'a>(state: &'a SourceFormState) -> cosmic::Element<'a, DialogMessage> {
    let mut section = settings::section().title(fl!("dialog-source-section")).add(
        settings::item::builder(fl!("dialog-source-label")).control(
            widget::text_input::text_input(fl!("dialog-source-placeholder"), &state.source)
                .on_input(DialogMessage::SourceAddressChanged)
                .width(Length::Fill),
        ),
    );
    if state.touched
        && let Err(error) = validate_source(&state.source)
    {
        section = section.add(widget::text::caption(localized_validation_error(error)));
    }
    let content = settings::view_column(vec![section.into()]);

    content.into()
}
