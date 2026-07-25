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

/// Immutable root state needed while reducing view actions.
pub(crate) struct Context {
    /// Whether another feature owns the global mutation slot.
    pub(crate) mutation_pending: bool,
    /// Localize typed validation failures without coupling to dialog state.
    pub(crate) localize_validation: fn(ValidationError) -> String,
}

/// IP-set-owned asynchronous work.
#[derive(Clone, Debug)]
pub(crate) enum Effect {
    /// Load the selected IP set.
    Details(String),
    /// Add one validated entry.
    AddEntry { ipset_name: String, entry: String },
    /// Remove one existing entry.
    RemoveEntry { ipset_name: String, entry: String },
}

/// Root coordination requested by the IP-set reducer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    /// Reserve the global mutation slot before scheduling the effect.
    BeginMutation(Mutation),
    /// Open the destructive confirmation for this IP set.
    ConfirmDelete(String),
}

/// Neutral mutation labels interpreted by the root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mutation {
    /// Add an IP-set entry.
    AddEntry,
    /// Remove an IP-set entry.
    RemoveEntry,
}

/// Reduce an IP-set view action synchronously.
pub(crate) fn update_view(
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
            Outcome {
                effects: vec![Effect::AddEntry {
                    ipset_name,
                    entry: entry.to_string(),
                }],
                requests: vec![Request::BeginMutation(Mutation::AddEntry)],
            }
        }
        IpSetViewAction::RemoveEntry(entry) => state
            .selected
            .clone()
            .map(|ipset_name| Outcome {
                effects: vec![Effect::RemoveEntry { ipset_name, entry }],
                requests: vec![Request::BeginMutation(Mutation::RemoveEntry)],
            })
            .unwrap_or_default(),
        IpSetViewAction::DeleteSelected => state
            .selected
            .clone()
            .map(|name| Outcome::request(Request::ConfirmDelete(name)))
            .unwrap_or_default(),
    }
}

/// Run IP-set work after root requests have been applied.
pub(crate) fn effects(effect: Effect) -> Task<Message> {
    match effect {
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
    }
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

/// Messages owned by the IP-set feature slice.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Handle an action emitted by the IP-set view.
    View(IpSetViewAction),
    /// The permanent IP-set list finished loading.
    ListLoaded(Result<Vec<String>, BrokerError>),
    /// A selected IP-set detail request finished.
    DetailsLoaded {
        /// IP set requested by the asynchronous task.
        ipset_name: String,
        /// Loaded details or the broker failure.
        result: Result<IpSetDetails, BrokerError>,
    },
    /// Adding an entry to an IP set finished.
    EntryAdded {
        /// Mutated IP set.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Removing an entry from an IP set finished.
    EntryRemoved {
        /// Mutated IP set.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Creating an IP set finished.
    Created {
        /// Requested IP-set name.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Deleting an IP set finished.
    Deleted {
        /// Deleted IP-set name.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
}
