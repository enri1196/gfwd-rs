use cosmic::iced::Length;
use cosmic::widget::{self, settings};

use crate::core::validate_source;
use crate::fl;

use super::localized_validation_error;

#[derive(Debug, Clone)]
pub enum Message {
    AddressChanged(String),
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

pub(super) fn update(state: &mut SourceFormState, message: Message) {
    match message {
        Message::AddressChanged(value) => {
            state.source = value;
            state.touched = true;
        }
    }
}

pub(super) fn touch(state: &mut SourceFormState) {
    state.touched = true;
}

pub fn source_drawer<'a>(state: &'a SourceFormState) -> cosmic::Element<'a, Message> {
    let mut section = settings::section()
        .title(fl!("dialog-source-section"))
        .add(widget::text::caption(fl!("dialog-source-format-hint")))
        .add(
            settings::item::builder(fl!("dialog-source-label")).control(
                widget::text_input::text_input(fl!("dialog-source-placeholder"), &state.source)
                    .on_input(Message::AddressChanged)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_touch_the_field_and_recompute_validity() {
        let mut state = SourceFormState::default();

        update(&mut state, Message::AddressChanged("not-a-source".into()));
        assert!(state.touched);
        assert!(!state.is_valid());

        update(&mut state, Message::AddressChanged("192.0.2.0/24".into()));
        assert!(state.touched);
        assert!(state.is_valid());
    }
}
