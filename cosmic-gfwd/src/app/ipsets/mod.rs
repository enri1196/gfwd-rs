//! IP-set MVU slice.

mod view;

use crate::{
    core::{BrokerError, FwdBroker, ValidationError, validate_ipset_entry},
    models::IpSetDetails,
};
use cosmic::Task;

use super::outcome::Outcome;

pub(crate) use view::{IpSetViewAction, IpSetViewState, view_ipset_content};

/// Authoritative IP-set list, selection, details, and entry-editor state.
pub(crate) type State = IpSetViewState;

/// Immutable root state needed while reducing IP-set messages.
pub(crate) struct Context {
    /// Whether another feature owns the global mutation slot.
    pub(crate) mutation_pending: bool,
    /// Localize typed validation failures without coupling to dialog state.
    pub(crate) localize_validation: fn(ValidationError) -> String,
}

/// IP-set-owned asynchronous work.
#[derive(Clone, Debug)]
pub(crate) enum Effect {
    /// Load the permanent IP-set list.
    List,
    /// Load the selected IP set.
    Details(String),
    /// Add one validated entry.
    AddEntry { ipset_name: String, entry: String },
    /// Remove one existing entry.
    RemoveEntry { ipset_name: String, entry: String },
    /// Create a permanent IP set.
    Create {
        name: String,
        ipset_type: String,
        entries: Vec<String>,
    },
    /// Delete a permanent IP set.
    Delete(String),
}

/// Root coordination requested by the IP-set reducer.
#[derive(Debug)]
pub(crate) enum Request {
    /// Reserve the global mutation slot before scheduling the effect.
    BeginMutation(Mutation),
    /// Open the destructive confirmation for this IP set.
    ConfirmDelete(String),
    /// Mark permanent configuration as requiring an explicit runtime reload.
    MarkRuntimeDirty,
    /// Finish the globally serialized mutation.
    FinishMutation(Result<(), BrokerError>),
    /// Reset the IP-set creation form.
    ResetCreateDialog,
    /// Close the root-owned context drawer.
    CloseDrawer,
}

/// Neutral mutation labels interpreted by the root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mutation {
    /// Add an IP-set entry.
    AddEntry,
    /// Remove an IP-set entry.
    RemoveEntry,
    /// Create a permanent IP set.
    Create,
    /// Delete a permanent IP set.
    Delete,
}

/// Messages owned by the IP-set feature slice.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Handle an action emitted by the IP-set view.
    View(IpSetViewAction),
    /// Request the permanent IP-set list.
    LoadList,
    /// Request creation after dialog validation.
    Create {
        name: String,
        ipset_type: String,
        entries: Vec<String>,
    },
    /// Request deletion after root confirmation.
    Delete(String),
    /// The permanent IP-set list finished loading.
    ListLoaded(Result<Vec<String>, BrokerError>),
    /// A selected IP-set detail request finished.
    DetailsLoaded {
        ipset_name: String,
        result: Result<IpSetDetails, BrokerError>,
    },
    /// Adding an entry to an IP set finished.
    EntryAdded {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    /// Removing an entry from an IP set finished.
    EntryRemoved {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    /// Creating an IP set finished.
    Created {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    /// Deleting an IP set finished.
    Deleted {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
}

/// Reduce every IP-set message synchronously.
pub(crate) fn update(
    state: &mut State,
    message: Message,
    context: Context,
) -> Outcome<Effect, Request> {
    match message {
        Message::View(action) => update_view(state, action, context),
        Message::LoadList => {
            state.list_loading = true;
            Outcome::effect(Effect::List)
        }
        Message::Create {
            name,
            ipset_type,
            entries,
        } => begin_effect(
            context.mutation_pending,
            Mutation::Create,
            Effect::Create {
                name,
                ipset_type,
                entries,
            },
        ),
        Message::Delete(ipset_name) => begin_effect(
            context.mutation_pending,
            Mutation::Delete,
            Effect::Delete(ipset_name),
        ),
        Message::ListLoaded(result) => finish_list(state, result),
        Message::DetailsLoaded { ipset_name, result } => finish_details(state, &ipset_name, result),
        Message::EntryAdded { ipset_name, result } => {
            finish_entry_change(state, ipset_name, result, true)
        }
        Message::EntryRemoved { ipset_name, result } => {
            finish_entry_change(state, ipset_name, result, false)
        }
        Message::Created { ipset_name, result } => finish_create(state, ipset_name, result),
        Message::Deleted { ipset_name, result } => finish_delete(state, &ipset_name, result),
    }
}

fn update_view(
    state: &mut State,
    action: IpSetViewAction,
    context: Context,
) -> Outcome<Effect, Request> {
    if context.mutation_pending
        && matches!(
            action,
            IpSetViewAction::AddEntry | IpSetViewAction::RemoveEntry(_)
        )
    {
        return Outcome::default();
    }

    match action {
        IpSetViewAction::Select(name) => {
            state.selected = Some(name.clone());
            state.details = None;
            state.entry_error = None;
            state.entry_input.clear();
            state.details_loading = true;
            Outcome::effect(Effect::Details(name))
        }
        IpSetViewAction::EntryInputChanged(value) => {
            state.entry_input = value;
            state.entry_error = state
                .details
                .as_ref()
                .and_then(|details| {
                    let input = state.entry_input.trim();
                    (!input.is_empty())
                        .then(|| validate_ipset_entry(input, &details.ipset_type).err())
                        .flatten()
                })
                .map(context.localize_validation);
            Outcome::default()
        }
        IpSetViewAction::AddEntry => {
            let (Some(ipset_name), Some(details)) = (state.selected.clone(), &state.details) else {
                return Outcome::default();
            };
            let entry = state.entry_input.trim();
            if let Err(error) = validate_ipset_entry(entry, &details.ipset_type) {
                state.entry_error = Some((context.localize_validation)(error));
                return Outcome::default();
            }
            begin_effect(
                false,
                Mutation::AddEntry,
                Effect::AddEntry {
                    ipset_name,
                    entry: entry.to_string(),
                },
            )
        }
        IpSetViewAction::RemoveEntry(entry) => state
            .selected
            .clone()
            .map(|ipset_name| {
                begin_effect(
                    false,
                    Mutation::RemoveEntry,
                    Effect::RemoveEntry { ipset_name, entry },
                )
            })
            .unwrap_or_default(),
        IpSetViewAction::DeleteSelected => state
            .selected
            .clone()
            .map(|name| Outcome::request(Request::ConfirmDelete(name)))
            .unwrap_or_default(),
    }
}

fn begin_effect(
    mutation_pending: bool,
    mutation: Mutation,
    effect: Effect,
) -> Outcome<Effect, Request> {
    if mutation_pending {
        Outcome::default()
    } else {
        Outcome {
            effects: vec![effect],
            requests: vec![Request::BeginMutation(mutation)],
        }
    }
}

fn finish_list(
    state: &mut State,
    result: Result<Vec<String>, BrokerError>,
) -> Outcome<Effect, Request> {
    state.list_loading = false;
    match result {
        Ok(ipsets) => {
            state.ipsets = ipsets;
            let Some(selected) = state.selected.clone() else {
                return Outcome::default();
            };
            if state.ipsets.iter().any(|item| item == &selected) {
                if state.details.is_none() {
                    state.details_loading = true;
                    return Outcome::effect(Effect::Details(selected));
                }
            } else {
                state.selected = None;
                state.details = None;
            }
        }
        Err(error) => {
            state.ipsets.clear();
            state.entry_error = Some(error.to_string());
            state.details = None;
        }
    }
    Outcome::default()
}

fn finish_details(
    state: &mut State,
    ipset_name: &str,
    result: Result<IpSetDetails, BrokerError>,
) -> Outcome<Effect, Request> {
    if state.selected.as_deref() != Some(ipset_name) {
        return Outcome::default();
    }
    state.details_loading = false;
    match result {
        Ok(details) => {
            state.details = Some(details);
            state.entry_error = None;
        }
        Err(error) => {
            state.details = None;
            state.entry_error = Some(error.to_string());
        }
    }
    Outcome::default()
}

fn finish_entry_change(
    state: &mut State,
    ipset_name: String,
    result: Result<(), BrokerError>,
    clear_input: bool,
) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => {
            if clear_input {
                state.entry_input.clear();
            }
            state.entry_error = None;
            state.details_loading = true;
            Outcome {
                effects: vec![Effect::Details(ipset_name)],
                requests: vec![Request::MarkRuntimeDirty, Request::FinishMutation(Ok(()))],
            }
        }
        Err(error) => {
            state.entry_error = Some(error.to_string());
            Outcome::request(Request::FinishMutation(Err(error)))
        }
    }
}

fn finish_create(
    state: &mut State,
    ipset_name: String,
    result: Result<(), BrokerError>,
) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => {
            state.selected = Some(ipset_name.clone());
            state.entry_input.clear();
            state.entry_error = None;
            state.list_loading = true;
            state.details_loading = true;
            Outcome {
                effects: vec![Effect::List, Effect::Details(ipset_name)],
                requests: vec![
                    Request::MarkRuntimeDirty,
                    Request::ResetCreateDialog,
                    Request::CloseDrawer,
                    Request::FinishMutation(Ok(())),
                ],
            }
        }
        Err(error) => {
            state.entry_error = Some(error.to_string());
            Outcome::request(Request::FinishMutation(Err(error)))
        }
    }
}

fn finish_delete(
    state: &mut State,
    ipset_name: &str,
    result: Result<(), BrokerError>,
) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => {
            if state.selected.as_deref() == Some(ipset_name) {
                state.selected = None;
                state.details = None;
                state.entry_input.clear();
                state.entry_error = None;
            }
            state.list_loading = true;
            Outcome {
                effects: vec![Effect::List],
                requests: vec![Request::MarkRuntimeDirty, Request::FinishMutation(Ok(()))],
            }
        }
        Err(error) => {
            state.entry_error = Some(error.to_string());
            Outcome::request(Request::FinishMutation(Err(error)))
        }
    }
}

/// Run IP-set work after root requests have been applied.
pub(crate) fn effects(effect: Effect) -> Task<Message> {
    match effect {
        Effect::List => Task::perform(load_list(), Message::ListLoaded),
        Effect::Details(ipset_name) => {
            let requested = ipset_name.clone();
            Task::perform(load_details(ipset_name), move |result| {
                Message::DetailsLoaded {
                    ipset_name: requested.clone(),
                    result,
                }
            })
        }
        Effect::AddEntry { ipset_name, entry } => {
            let mutated = ipset_name.clone();
            Task::perform(add_entry(ipset_name, entry), move |result| {
                Message::EntryAdded {
                    ipset_name: mutated.clone(),
                    result,
                }
            })
        }
        Effect::RemoveEntry { ipset_name, entry } => {
            let mutated = ipset_name.clone();
            Task::perform(remove_entry(ipset_name, entry), move |result| {
                Message::EntryRemoved {
                    ipset_name: mutated.clone(),
                    result,
                }
            })
        }
        Effect::Create {
            name,
            ipset_type,
            entries,
        } => {
            let requested = name.clone();
            Task::perform(create(name, ipset_type, entries), move |result| {
                Message::Created {
                    ipset_name: requested.clone(),
                    result,
                }
            })
        }
        Effect::Delete(ipset_name) => {
            let requested = ipset_name.clone();
            Task::perform(remove(ipset_name), move |result| Message::Deleted {
                ipset_name: requested.clone(),
                result,
            })
        }
    }
}

async fn load_list() -> Result<Vec<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_ipsets().await
}

async fn load_details(ipset_name: String) -> Result<IpSetDetails, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_ipset_details(&ipset_name).await
}

async fn add_entry(ipset_name: String, entry: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_ipset_entry(&ipset_name, &entry).await
}

async fn remove_entry(ipset_name: String, entry: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_ipset_entry(&ipset_name, &entry).await
}

async fn create(name: String, ipset_type: String, entries: Vec<String>) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.create_ipset(&name, &ipset_type, entries).await
}

async fn remove(ipset_name: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_ipset(&ipset_name).await
}

#[cfg(test)]
mod tests {
    use super::{Context, Effect, IpSetViewAction, Message, Request, State, update};
    use crate::models::IpSetDetails;
    use std::collections::HashMap;

    fn context() -> Context {
        Context {
            mutation_pending: false,
            localize_validation: |_| "invalid".into(),
        }
    }

    #[test]
    fn selection_resets_editor_and_requests_matching_details() {
        let mut state = State {
            entry_input: "old".into(),
            entry_error: Some("old error".into()),
            ..State::default()
        };

        let outcome = update(
            &mut state,
            Message::View(IpSetViewAction::Select("work".into())),
            context(),
        );

        assert_eq!(state.selected.as_deref(), Some("work"));
        assert!(state.entry_input.is_empty());
        assert!(state.entry_error.is_none());
        assert!(matches!(outcome.effects.as_slice(), [Effect::Details(name)] if name == "work"));
    }

    #[test]
    fn stale_detail_completion_is_ignored() {
        let mut state = State {
            selected: Some("current".into()),
            details_loading: true,
            ..State::default()
        };

        let outcome = update(
            &mut state,
            Message::DetailsLoaded {
                ipset_name: "stale".into(),
                result: Ok(IpSetDetails {
                    name: "stale".into(),
                    ipset_type: "hash:ip".into(),
                    entries: Vec::new(),
                    options: HashMap::new(),
                }),
            },
            context(),
        );

        assert!(outcome.effects.is_empty());
        assert!(outcome.requests.is_empty());
        assert!(state.details_loading);
        assert!(state.entry_error.is_none());
    }

    #[test]
    fn successful_delete_marks_dirty_then_finishes_before_refresh() {
        let mut state = State {
            selected: Some("work".into()),
            ..State::default()
        };

        let outcome = update(
            &mut state,
            Message::Deleted {
                ipset_name: "work".into(),
                result: Ok(()),
            },
            context(),
        );

        assert!(state.selected.is_none());
        assert!(matches!(outcome.effects.as_slice(), [Effect::List]));
        assert!(matches!(
            outcome.requests.as_slice(),
            [Request::MarkRuntimeDirty, Request::FinishMutation(Ok(()))]
        ));
    }
}
