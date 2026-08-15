use cosmic::iced::Length;
use cosmic::widget::{self, dropdown, settings};

use crate::core::{IPSET_TYPES, validate_ipset_entry, validate_ipset_name, validate_ipset_type};
use crate::fl;

use super::{DialogMessage, localized_validation_error};

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

pub(super) fn name_changed(state: &mut IpSetFormState, value: String) {
    state.name = value;
    state.name_touched = true;
}

pub(super) fn type_selected(state: &mut IpSetFormState, index: usize) {
    state.ipset_type = ipset_from_index(index);
}

pub(super) fn entries_changed(state: &mut IpSetFormState, value: String) {
    state.entries = value;
    state.entries_touched = true;
}

pub(super) fn touch_submission_fields(state: &mut IpSetFormState) {
    state.name_touched = true;
    state.entries_touched = true;
}

pub(super) fn split_entries(entries: &str) -> Vec<String> {
    entries
        .lines()
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
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
    if state.name_touched
        && let Err(error) = validate_ipset_name(&state.name)
    {
        section = section.add(widget::text::caption(localized_validation_error(error)));
    }
    if state.entries_touched
        && let Some(error) = state
            .entries
            .lines()
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .find_map(|entry| validate_ipset_entry(entry, &state.ipset_type).err())
    {
        section = section.add(widget::text::caption(localized_validation_error(error)));
    }
    let content = settings::view_column(vec![section.into()]);

    content.into()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submission_entries_preserve_composite_tuple_commas() {
        assert_eq!(
            split_entries("192.0.2.1,443,198.51.100.2\n\n  2001:db8::1,53  \n"),
            ["192.0.2.1,443,198.51.100.2", "2001:db8::1,53"]
        );
    }
}
