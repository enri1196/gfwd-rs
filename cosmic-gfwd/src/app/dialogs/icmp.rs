use cosmic::iced::Length;
use cosmic::widget::{self, button, settings};

use crate::fl;
use crate::models::IcmpTypeInfo;

use super::{Effect, Outcome, Request, Submission, selected_zone, submit};

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    Selected(String),
}

#[derive(Debug, Clone, Default)]
pub struct IcmpFormState {
    /// Case-insensitive name and description filter.
    pub search: String,
}

pub(super) struct Context<'a> {
    pub(super) selected_zone: Option<&'a str>,
    pub(super) blocked_icmp: &'a [String],
    pub(super) operation_error: &'a mut Option<String>,
}

pub(super) fn update(
    state: &mut IcmpFormState,
    message: Message,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match message {
        Message::SearchChanged(value) => state.search = value,
        Message::Selected(icmp) => {
            let Some(zone) = selected_zone(context.operation_error, context.selected_zone) else {
                return Outcome::default();
            };
            if context.blocked_icmp.contains(&icmp) {
                *context.operation_error = Some(fl!("error-icmp-already-blocked"));
                return Outcome::default();
            }
            return submit(Submission::Icmp { zone, icmp });
        }
    }
    Outcome::default()
}

pub fn icmp_drawer<'a>(
    state: &'a IcmpFormState,
    types: &'a [IcmpTypeInfo],
    blocked: &'a [String],
    loading: bool,
    error: Option<&'a str>,
) -> cosmic::Element<'a, Message> {
    let filter = state.search.trim().to_lowercase();
    let mut section = settings::section().title(fl!("dialog-icmp-section")).add(
        widget::text_input::text_input(fl!("dialog-icmp-search-placeholder"), &state.search)
            .on_input(Message::SearchChanged)
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
                        (!is_blocked).then(|| Message::Selected(icmp.name.clone())),
                    )),
            );
        }
        if visible == 0 {
            section = section.add(widget::text::caption(fl!("dialog-icmp-empty")));
        }
    }

    settings::view_column(vec![section.into()]).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_duplicate_detection_and_submission_are_preserved() {
        let mut state = IcmpFormState::default();
        let mut operation_error = None;

        update(
            &mut state,
            Message::SearchChanged("echo".into()),
            Context {
                selected_zone: Some("public"),
                blocked_icmp: &[],
                operation_error: &mut operation_error,
            },
        );
        assert_eq!(state.search, "echo");

        let blocked = ["echo-request".to_string()];
        let outcome = update(
            &mut state,
            Message::Selected("echo-request".into()),
            Context {
                selected_zone: Some("public"),
                blocked_icmp: &blocked,
                operation_error: &mut operation_error,
            },
        );
        assert!(outcome.requests.is_empty());
        assert_eq!(operation_error, Some(fl!("error-icmp-already-blocked")));

        operation_error = None;
        let outcome = update(
            &mut state,
            Message::Selected("destination-unreachable".into()),
            Context {
                selected_zone: Some("public"),
                blocked_icmp: &blocked,
                operation_error: &mut operation_error,
            },
        );
        assert!(matches!(
            outcome.requests.as_slice(),
            [Request::Submit(Submission::Icmp { zone, icmp })]
                if zone == "public" && icmp == "destination-unreachable"
        ));
        assert!(operation_error.is_none());
    }
}
