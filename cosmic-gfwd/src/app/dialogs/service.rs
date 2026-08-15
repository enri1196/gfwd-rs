use cosmic::iced::Length;
use cosmic::widget::{self, button, settings};

use crate::fl;

use super::DialogMessage;

/// Search state for the configured-service picker.
#[derive(Debug, Clone, Default)]
pub struct ServiceFormState {
    /// Case-insensitive service-name filter.
    pub search: String,
}

pub(super) fn search_changed(state: &mut ServiceFormState, value: String) {
    state.search = value;
}

/// Builds the searchable configured-service picker.
pub fn service_drawer<'a>(
    state: &'a ServiceFormState,
    services: &'a [String],
    enabled: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, DialogMessage> {
    let filter = state.search.trim().to_lowercase();
    let mut section = settings::section()
        .title(fl!("dialog-service-section"))
        .add(
            widget::text_input::text_input(fl!("dialog-service-search-placeholder"), &state.search)
                .on_input(DialogMessage::ServiceSearchChanged)
                .width(Length::Fill),
        );

    if loading {
        section = section.add(widget::text::caption(fl!("dialog-service-loading")));
    } else if let Some(error) = error {
        section = section.add(widget::text::caption(error));
    } else {
        let mut visible = 0;
        for service in services
            .iter()
            .filter(|service| filter.is_empty() || service.to_lowercase().contains(&filter))
        {
            visible += 1;
            let is_enabled = enabled.iter().any(|item| item == service);
            let label = if is_enabled {
                fl!("dialog-service-enabled", service = service)
            } else {
                service.clone()
            };
            let message = (!is_enabled).then(|| DialogMessage::ServiceSelected(service.clone()));
            section = section.add(
                button::standard(label)
                    .width(Length::Fill)
                    .on_press_maybe(message),
            );
        }
        if visible == 0 {
            section = section.add(widget::text::caption(fl!("dialog-service-empty")));
        }
    }

    settings::view_column(vec![section.into()]).into()
}
