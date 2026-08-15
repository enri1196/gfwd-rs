use cosmic::iced::Length;
use cosmic::widget::{self, button, settings};

use crate::fl;

use super::{Effect, Outcome, Request, Submission, selected_zone, submit};

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    Selected(String),
}

/// Search state for the configured-service picker.
#[derive(Debug, Clone, Default)]
pub struct ServiceFormState {
    /// Case-insensitive service-name filter.
    pub search: String,
}

pub(super) struct Context<'a> {
    pub(super) selected_zone: Option<&'a str>,
    pub(super) enabled_services: &'a [String],
    pub(super) operation_error: &'a mut Option<String>,
}

pub(super) fn update(
    state: &mut ServiceFormState,
    message: Message,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match message {
        Message::SearchChanged(value) => state.search = value,
        Message::Selected(service) => {
            let Some(zone) = selected_zone(context.operation_error, context.selected_zone) else {
                return Outcome::default();
            };
            if context.enabled_services.contains(&service) {
                *context.operation_error = Some(fl!("error-service-already-enabled"));
                return Outcome::default();
            }
            return submit(Submission::Service { zone, service });
        }
    }
    Outcome::default()
}

/// Builds the searchable configured-service picker.
pub fn service_drawer<'a>(
    state: &'a ServiceFormState,
    services: &'a [String],
    enabled: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, Message> {
    let filter = state.search.trim().to_lowercase();
    let mut section = settings::section()
        .title(fl!("dialog-service-section"))
        .add(
            widget::text_input::text_input(fl!("dialog-service-search-placeholder"), &state.search)
                .on_input(Message::SearchChanged)
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
            let message = (!is_enabled).then(|| Message::Selected(service.clone()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_and_selection_preserve_picker_behavior() {
        let mut state = ServiceFormState::default();
        let mut operation_error = None;

        let outcome = update(
            &mut state,
            Message::SearchChanged("ssh".into()),
            Context {
                selected_zone: Some("public"),
                enabled_services: &[],
                operation_error: &mut operation_error,
            },
        );
        assert_eq!(state.search, "ssh");
        assert!(outcome.requests.is_empty());

        let enabled = ["ssh".to_string()];
        let outcome = update(
            &mut state,
            Message::Selected("ssh".into()),
            Context {
                selected_zone: Some("public"),
                enabled_services: &enabled,
                operation_error: &mut operation_error,
            },
        );
        assert!(outcome.requests.is_empty());
        assert_eq!(operation_error, Some(fl!("error-service-already-enabled")));

        operation_error = None;
        let outcome = update(
            &mut state,
            Message::Selected("https".into()),
            Context {
                selected_zone: Some("public"),
                enabled_services: &enabled,
                operation_error: &mut operation_error,
            },
        );
        assert!(matches!(
            outcome.requests.as_slice(),
            [Request::Submit(Submission::Service { zone, service })]
                if zone == "public" && service == "https"
        ));
        assert!(operation_error.is_none());
    }
}
