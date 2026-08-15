use cosmic::iced::Length;
use cosmic::widget::{self, button, settings};

use crate::fl;
use crate::models::IcmpTypeInfo;

use super::DialogMessage;

#[derive(Debug, Clone, Default)]
pub struct IcmpFormState {
    /// Case-insensitive name and description filter.
    pub search: String,
}

pub(super) fn search_changed(state: &mut IcmpFormState, value: String) {
    state.search = value;
}

pub fn icmp_drawer<'a>(
    state: &'a IcmpFormState,
    types: &'a [IcmpTypeInfo],
    blocked: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, DialogMessage> {
    let filter = state.search.trim().to_lowercase();
    let mut section = settings::section().title(fl!("dialog-icmp-section")).add(
        widget::text_input::text_input(fl!("dialog-icmp-search-placeholder"), &state.search)
            .on_input(DialogMessage::IcmpSearchChanged)
            .width(Length::Fill),
    );

    if loading {
        section = section.add(widget::text::caption(fl!("dialog-icmp-loading")));
    } else if let Some(error) = error {
        section = section.add(widget::text::caption(error));
    } else {
        let mut visible = 0;
        for icmp in types.iter().filter(|icmp| {
            filter.is_empty()
                || icmp.name.to_lowercase().contains(&filter)
                || icmp.description.to_lowercase().contains(&filter)
        }) {
            visible += 1;
            let is_blocked = blocked.contains(&icmp.name);
            let label = if is_blocked {
                fl!("dialog-icmp-blocked", name = icmp.name.as_str())
            } else {
                fl!("dialog-icmp-add", name = icmp.name.as_str())
            };
            section = section.add(
                settings::item::builder(icmp.name.as_str())
                    .description(icmp.description.as_str())
                    .control(button::standard(label).on_press_maybe(
                        (!is_blocked).then(|| DialogMessage::IcmpSelected(icmp.name.clone())),
                    )),
            );
        }
        if visible == 0 {
            section = section.add(widget::text::caption(fl!("dialog-icmp-empty")));
        }
    }

    settings::view_column(vec![section.into()]).into()
}
