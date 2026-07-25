//! IP-set list and detail view.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, icon, settings};

use crate::core::validate_ipset_entry;
use crate::fl;
use crate::models::IpSetDetails;

const MAX_LIST_ITEMS: usize = 5;
const LIST_ITEM_HEIGHT: f32 = 28.0;
const REMOVE_ICON: &str = "user-trash-symbolic";

#[derive(Debug, Clone, Default)]
pub struct IpSetViewState {
    pub ipsets: Vec<String>,
    pub selected: Option<String>,
    pub details: Option<IpSetDetails>,
    pub entry_input: String,
    pub entry_error: Option<String>,
    pub list_loading: bool,
    pub details_loading: bool,
}

#[derive(Debug, Clone)]
pub enum IpSetViewAction {
    Select(String),
    EntryInputChanged(String),
    AddEntry,
    RemoveEntry(String),
    /// Requests destructive confirmation for the selected IP set.
    DeleteSelected,
}

pub fn view_ipset_content<'a, Message: 'static + Clone>(
    state: &'a IpSetViewState,
    mutation_pending: bool,
    map: impl Fn(IpSetViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let mut sections = Vec::new();

    let mut list_section = settings::section().title(fl!("ipset-section-list"));
    if state.list_loading {
        list_section = list_section.add(widget::text::caption(fl!("ipset-loading")));
    } else if state.ipsets.is_empty() {
        list_section = list_section.add(widget::text::caption(fl!("ipset-empty")));
    } else {
        let spacing = cosmic::theme::spacing().space_xxs;
        let mut list = widget::column::with_capacity(state.ipsets.len())
            .spacing(spacing)
            .width(Length::Fill);

        for ipset in &state.ipsets {
            let is_selected = state.selected.as_deref() == Some(ipset.as_str());
            let button = if is_selected {
                button::suggested(ipset.as_str())
            } else {
                button::text(ipset.as_str())
            }
            .width(Length::Fill)
            .on_press(map(IpSetViewAction::Select(ipset.clone())));

            list = list.push(button);
        }

        let list_element = list_with_scroll(list.into(), state.ipsets.len());
        list_section = list_section.add(list_element);
    }

    sections.push(list_section.into());

    if state.details_loading {
        sections.push(
            settings::section()
                .title(fl!("ipset-section-details"))
                .add(widget::text::caption(fl!("ipset-details-loading")))
                .into(),
        );
    }

    if let Some(details) = &state.details {
        let mut options = details
            .options
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>();
        options.sort();
        sections.push(
            settings::section()
                .title(fl!("ipset-section-details"))
                .add(
                    settings::item::builder(fl!("ipset-detail-name"))
                        .control(widget::text(details.name.as_str())),
                )
                .add(
                    settings::item::builder(fl!("ipset-detail-type"))
                        .control(widget::text(details.ipset_type.as_str())),
                )
                .add(
                    settings::item::builder(fl!("ipset-detail-entries"))
                        .control(widget::text(details.entries.len().to_string())),
                )
                .add(
                    settings::item::builder(fl!("ipset-detail-options")).control(widget::text(
                        if options.is_empty() {
                            fl!("ipset-options-none")
                        } else {
                            options.join(", ")
                        },
                    )),
                )
                .add(settings::item::builder(fl!("ipset-delete-label")).control(
                    button::destructive(fl!("ipset-delete")).on_press_maybe(
                        (!mutation_pending).then_some(map(IpSetViewAction::DeleteSelected)),
                    ),
                ))
                .into(),
        );

        let can_add_entry = !mutation_pending
            && validate_ipset_entry(&state.entry_input, &details.ipset_type).is_ok()
            && state.selected.is_some();
        let add_message = can_add_entry.then_some(map(IpSetViewAction::AddEntry));
        let entry_input_row = widget::row::with_capacity(2)
            .push(
                widget::text_input::text_input(fl!("ipset-entry-placeholder"), &state.entry_input)
                    .on_input(move |value| map(IpSetViewAction::EntryInputChanged(value)))
                    .width(Length::Fill),
            )
            .push(button::suggested(fl!("ipset-entry-add")).on_press_maybe(add_message))
            .spacing(cosmic::theme::spacing().space_s)
            .align_y(Alignment::Center);

        let mut entries_section = settings::section()
            .title(fl!("ipset-section-entries"))
            .add(entry_input_row);

        if let Some(error) = &state.entry_error {
            entries_section = entries_section.add(widget::text::caption(error.as_str()));
        }

        if details.entries.is_empty() {
            entries_section = entries_section.add(widget::text::caption(fl!("ipset-entry-empty")));
        } else {
            let spacing = cosmic::theme::spacing().space_xxs;
            let entries_list = widget::column::with_capacity(details.entries.len())
                .spacing(spacing)
                .width(Length::Fill)
                .extend(
                    details
                        .entries
                        .iter()
                        .map(|entry| entry_item_row(entry, mutation_pending, map)),
                );
            let entries_element = list_with_scroll(entries_list.into(), details.entries.len());
            entries_section = entries_section.add(entries_element);
        }

        sections.push(entries_section.into());
    }

    let content = settings::view_column(sections);

    widget::scrollable::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn entry_item_row<'a, Message: 'static + Clone>(
    entry: &'a str,
    mutation_pending: bool,
    map: impl Fn(IpSetViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let remove = button::icon(icon::from_name(REMOVE_ICON))
        .tooltip(fl!("action-remove"))
        .extra_small()
        .on_press_maybe(
            (!mutation_pending).then_some(map(IpSetViewAction::RemoveEntry(entry.to_string()))),
        );

    widget::row::with_capacity(2)
        .push(widget::text::body(entry).width(Length::Fill))
        .push(remove)
        .spacing(spacing.space_s)
        .align_y(Alignment::Center)
        .into()
}

fn list_with_scroll<'a, Message: 'static>(
    list: cosmic::Element<'a, Message>,
    item_count: usize,
) -> cosmic::Element<'a, Message> {
    if item_count > MAX_LIST_ITEMS {
        let max_height = LIST_ITEM_HEIGHT * MAX_LIST_ITEMS as f32;
        widget::scrollable::scrollable(list)
            .height(Length::Fixed(max_height))
            .width(Length::Fill)
            .into()
    } else {
        list
    }
}
