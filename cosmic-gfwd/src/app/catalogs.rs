//! Dialog option-catalog MVU slice.

/// Items, progress, and failure state for one asynchronously loaded catalog.
use crate::{
    core::{BrokerError, FwdBroker},
    models::IcmpTypeInfo,
};
use cosmic::Task;

use super::outcome::Outcome;

/// Completion messages for asynchronously loaded dialog catalogs.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Request network interface discovery.
    LoadInterfaces,
    /// Request permanent service discovery.
    LoadServices,
    /// Request ICMP-type discovery.
    LoadIcmpTypes,
    /// Network interface discovery completed.
    Interfaces(Result<Vec<String>, BrokerError>),
    /// Permanent service discovery completed.
    Services(Result<Vec<String>, BrokerError>),
    /// ICMP-type discovery completed.
    IcmpTypes(Result<Vec<IcmpTypeInfo>, BrokerError>),
}

/// All dialog option catalogs owned by this slice.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct State {
    /// Available network interfaces.
    pub(crate) interfaces: CatalogState<String>,
    /// Permanent firewalld service definitions.
    pub(crate) services: CatalogState<String>,
    /// Configured ICMP type definitions.
    pub(crate) icmp_types: CatalogState<IcmpTypeInfo>,
}

/// Immutable sibling data needed to decide root coordination.
pub(crate) struct Context<'a> {
    /// Interface currently selected by the open form.
    pub(crate) selected_interface: &'a str,
}

/// Catalog-owned asynchronous operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Effect {
    /// Discover network interfaces.
    Interfaces,
    /// Load permanent service definitions.
    Services,
    /// Load configured ICMP types.
    IcmpTypes,
}

/// Root coordination requested by catalog completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    /// Clear a form selection no longer present in the refreshed catalog.
    ClearInterfaceSelection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogState<T> {
    items: Vec<T>,
    loading: bool,
    error: Option<String>,
}

impl<T> Default for CatalogState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            loading: false,
            error: None,
        }
    }
}

impl<T> CatalogState<T> {
    /// Begin loading while retaining existing items and clearing an old error.
    pub(crate) fn begin_load(&mut self) {
        self.loading = true;
        self.error = None;
    }

    /// Replace items after a successful load.
    pub(crate) fn finish(&mut self, items: Vec<T>) {
        self.items = items;
        self.loading = false;
        self.error = None;
    }

    /// Finish loading with an error and discard stale items.
    pub(crate) fn fail(&mut self, error: String) {
        self.items.clear();
        self.loading = false;
        self.error = Some(error);
    }

    /// Return the currently available items.
    pub(crate) fn items(&self) -> &[T] {
        &self.items
    }

    /// Return whether a load is currently in progress.
    pub(crate) fn is_loading(&self) -> bool {
        self.loading
    }

    /// Return the most recent loading error.
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Reduce a catalog message synchronously.
pub(crate) fn update(
    state: &mut State,
    message: Message,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match message {
        Message::LoadInterfaces => {
            state.interfaces.finish(Vec::new());
            state.interfaces.begin_load();
            Outcome::effect(Effect::Interfaces)
        }
        Message::LoadServices => {
            state.services.begin_load();
            Outcome::effect(Effect::Services)
        }
        Message::LoadIcmpTypes => {
            state.icmp_types.begin_load();
            Outcome::effect(Effect::IcmpTypes)
        }
        Message::Interfaces(result) => match result {
            Ok(interfaces) => {
                state.interfaces.finish(interfaces);
                if !state.interfaces.items().is_empty()
                    && !state
                        .interfaces
                        .items()
                        .iter()
                        .any(|interface| interface == context.selected_interface)
                {
                    Outcome::request(Request::ClearInterfaceSelection)
                } else {
                    Outcome::default()
                }
            }
            Err(error) => {
                state.interfaces.fail(error.to_string());
                Outcome::default()
            }
        },
        Message::Services(result) => {
            match result {
                Ok(services) => state.services.finish(services),
                Err(error) => state.services.fail(error.to_string()),
            }
            Outcome::default()
        }
        Message::IcmpTypes(result) => {
            match result {
                Ok(types) => state.icmp_types.finish(types),
                Err(error) => state.icmp_types.fail(error.to_string()),
            }
            Outcome::default()
        }
    }
}

/// Run one catalog-owned effect.
pub(crate) fn effects(effect: Effect) -> Task<Message> {
    match effect {
        Effect::Interfaces => Task::perform(load_interfaces(), Message::Interfaces),
        Effect::Services => Task::perform(load_services(), Message::Services),
        Effect::IcmpTypes => Task::perform(load_icmp_types(), Message::IcmpTypes),
    }
}

async fn load_interfaces() -> Result<Vec<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_interfaces().await
}

async fn load_services() -> Result<Vec<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_services().await
}

async fn load_icmp_types() -> Result<Vec<IcmpTypeInfo>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_icmp_types().await
}

#[cfg(test)]
mod tests {
    use super::{CatalogState, Context, Message, Request, State, update};

    #[test]
    fn begin_marks_loading_and_retains_previous_items() {
        let mut catalog = CatalogState::default();
        catalog.finish(vec!["old"]);

        catalog.begin_load();

        assert!(catalog.is_loading());
        assert_eq!(catalog.items(), ["old"]);
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn success_replaces_items_and_clears_loading() {
        let mut catalog = CatalogState::default();
        catalog.begin_load();

        catalog.finish(vec!["new"]);

        assert!(!catalog.is_loading());
        assert_eq!(catalog.items(), ["new"]);
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn failure_discards_items_and_exposes_exact_error() {
        let mut catalog = CatalogState::default();
        catalog.finish(vec!["cached"]);
        catalog.begin_load();

        catalog.fail("permission denied".into());

        assert!(!catalog.is_loading());
        assert!(catalog.items().is_empty());
        assert_eq!(catalog.error(), Some("permission denied"));
    }

    #[test]
    fn retry_clears_previous_error_without_discarding_items() {
        let mut catalog = CatalogState::default();
        catalog.finish(vec!["cached"]);
        catalog.fail("temporary failure".into());

        catalog.begin_load();

        assert!(catalog.is_loading());
        assert!(catalog.items().is_empty());
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn failure_can_be_replaced_by_later_success() {
        let mut catalog = CatalogState::default();
        catalog.fail("temporary failure".into());

        catalog.begin_load();
        catalog.finish(vec!["recovered"]);

        assert_eq!(catalog.items(), ["recovered"]);
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn interface_refresh_requests_invalid_selection_clear() {
        let mut state = State::default();

        let outcome = update(
            &mut state,
            Message::Interfaces(Ok(vec!["eth0".into()])),
            Context {
                selected_interface: "wlan0",
            },
        );

        assert_eq!(outcome.requests, [Request::ClearInterfaceSelection]);
    }

    #[test]
    fn interface_refresh_preserves_valid_selection() {
        let mut state = State::default();

        let outcome = update(
            &mut state,
            Message::Interfaces(Ok(vec!["eth0".into()])),
            Context {
                selected_interface: "eth0",
            },
        );

        assert!(outcome.requests.is_empty());
    }
}
