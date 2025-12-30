use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, settings};

use crate::fl;
use crate::models::IpSetDetails;

#[derive(Debug, Clone)]
pub struct IpSetViewState {
    pub ipsets: Vec<String>,
    pub selected: Option<String>,
    pub details: Option<IpSetDetails>,
    pub entry_input: String,
    pub entry_error: Option<String>,
    pub list_loading: bool,
    pub details_loading: bool,
}

impl Default for IpSetViewState {
    fn default() -> Self {
        Self {
            ipsets: Vec::new(),
            selected: None,
            details: None,
            entry_input: String::new(),
            entry_error: None,
            list_loading: false,
            details_loading: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IpSetViewAction {
    Select(String),
    EntryInputChanged(String),
    AddEntry,
}

pub fn view_ipset_content<'a, Message: 'static + Clone>(
    state: &'a IpSetViewState,
    map: impl Fn(IpSetViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let mut sections = Vec::new();

    let mut list_section = settings::section().title(fl!("ipset-section-list"));
    if state.list_loading {
        list_section = list_section.add(widget::text::caption(fl!("ipset-loading")));
    } else if state.ipsets.is_empty() {
        list_section = list_section.add(widget::text::caption(fl!("ipset-empty")));
    } else {
        for ipset in &state.ipsets {
            let is_selected = state.selected.as_deref() == Some(ipset.as_str());
            let button = if is_selected {
                button::suggested(ipset.as_str())
            } else {
                button::text(ipset.as_str())
            }
            .on_press(map(IpSetViewAction::Select(ipset.clone())));

            list_section = list_section.add(button);
        }
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
        sections.push(
            settings::section()
                .title(fl!("ipset-section-details"))
                .add(
                    settings::item::builder(fl!("ipset-detail-name")).control(widget::text(
                        details.name.as_str(),
                    )),
                )
                .add(
                    settings::item::builder(fl!("ipset-detail-type")).control(widget::text(
                        details.ipset_type.as_str(),
                    )),
                )
                .add(
                    settings::item::builder(fl!("ipset-detail-entries")).control(widget::text(
                        details.entries.len().to_string(),
                    )),
                )
                .add(
                    settings::item::builder(fl!("ipset-detail-options")).control(widget::text(
                        details.options.len().to_string(),
                    )),
                )
                .into(),
        );

        let can_add_entry = !state.entry_input.trim().is_empty() && state.selected.is_some();
        let add_message = can_add_entry.then_some(map(IpSetViewAction::AddEntry));
        let entry_row = widget::row::with_capacity(2)
            .push(
                widget::text_input::text_input(
                    fl!("ipset-entry-placeholder"),
                    &state.entry_input,
                )
                .on_input(move |value| map(IpSetViewAction::EntryInputChanged(value)))
                .width(Length::Fill),
            )
            .push(button::suggested(fl!("ipset-entry-add")).on_press_maybe(add_message))
            .spacing(cosmic::theme::spacing().space_s)
            .align_y(Alignment::Center);

        let mut entries_section = settings::section()
            .title(fl!("ipset-section-entries"))
            .add(entry_row);

        if let Some(error) = &state.entry_error {
            entries_section = entries_section.add(widget::text::caption(error.as_str()));
        }

        if details.entries.is_empty() {
            entries_section = entries_section.add(widget::text::caption(fl!("ipset-entry-empty")));
        } else {
            entries_section = entries_section.extend(
                details
                    .entries
                    .iter()
                    .map(|entry| widget::text::body(entry.as_str())),
            );
        }

        sections.push(entries_section.into());
    }

    let content = settings::view_column(sections);

    widget::scrollable::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
