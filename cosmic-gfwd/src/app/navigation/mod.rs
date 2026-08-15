mod view;

use std::collections::HashSet;

use cosmic::widget::{menu, nav_bar};

use super::{ContextPage, outcome::Outcome as SliceOutcome};
use crate::config::Config;

pub(crate) use view::{Sidebar, SidebarItem};

/// Navigation item materialization and selection state.
pub(crate) type State = Sidebar;

/// Actions initiated by the navigation bar and navigation-owned projections.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Select the sidebar entry represented by the COSMIC navigation identifier.
    Select(nav_bar::Id),
    /// Open or close a root-owned context page.
    ToggleContextPage(ContextPage),
    /// Apply a context-menu action to a sidebar entry.
    MenuAction(MenuAction),
    FilterChanged(String),
    /// Record a successful permanent-zone list load.
    ZonesLoaded(Result<Vec<String>, String>),
    /// Record the permanent default-zone projection.
    DefaultZoneLoaded(Result<String, String>),
    /// Record the runtime-active-zone projection.
    ActiveZonesLoaded(Result<HashSet<String>, String>),
    /// Persist application configuration supplied by the navigation shell.
    UpdateConfig(Config),
    /// Open an external URL requested by the navigation shell.
    LaunchUrl(String),
}

/// Context-menu commands associated with a sidebar entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MenuAction {
    /// Select a zone and open its interface drawer.
    AssignInterface(nav_bar::Id),
    /// Make a zone the permanent default.
    SetDefault(nav_bar::Id),
    Rename(nav_bar::Id),
    /// Request confirmation before deleting a zone.
    Delete(nav_bar::Id),
}

/// Immutable root information used by the navigation reducer.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Context;

/// Consequences that navigation delegates to the application root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    /// Load the permanent details for the newly selected zone.
    LoadZone(String),
    /// Load the permanent IP-set list for the selected IP-set page.
    LoadIpSets,
    /// Open a root-owned context page without toggling an already-open drawer.
    OpenContextPage(ContextPage),
    /// Toggle a root-owned context page and preserve its existing dialog behavior.
    ToggleContextPage(ContextPage),
    /// Load the interface catalog needed by the add-interface context page.
    /// Start the serialized default-zone mutation through the zones slice.
    SetDefaultZone(String),
    /// Present the root-owned destructive confirmation for a zone.
    ConfirmDeleteZone(String),
    RenameZone(String),
    /// Ask the root to apply the current navigation label to the window title.
    RefreshTitle,
    /// Clear zone detail content and mark reconciliation unavailable.
    ClearSelectedZone,
    /// Load the default-zone sidebar projection after a zone-list refresh.
    LoadDefaultZone,
    /// Load the active-zone sidebar projection after a zone-list refresh.
    LoadActiveZones,
    /// Finish a pending configuration refresh after a non-zone selection.
    FinishConfigurationRefresh,
    /// Surface a zone-list failure in the root-owned zone-content area.
    ShowZoneListError(String),
    /// Apply persisted application configuration at the root.
    UpdateConfig(Config),
    /// Launch an external URL at the root.
    LaunchUrl(String),
}

/// Synchronous navigation result returned to the root request router.
pub(crate) type Outcome = SliceOutcome<(), Request>;

/// Reduce a navigation message and return root-owned consequences in causal order.
pub(crate) fn update(state: &mut State, message: Message, _context: Context) -> Outcome {
    match message {
        Message::Select(id) => select(state, id),
        Message::ToggleContextPage(page) => Outcome::request(Request::ToggleContextPage(page)),
        Message::MenuAction(action) => menu_action(state, action),
        Message::FilterChanged(filter) => {
            state.set_filter(filter);
            Outcome::default()
        }
        Message::ZonesLoaded(result) => zones_loaded(state, result),
        Message::DefaultZoneLoaded(result) => {
            state.set_default_zone(result.ok());
            Outcome::default()
        }
        Message::ActiveZonesLoaded(result) => {
            state.set_active_zones(result.unwrap_or_default());
            Outcome::default()
        }
        Message::UpdateConfig(config) => Outcome::request(Request::UpdateConfig(config)),
        Message::LaunchUrl(url) => Outcome::request(Request::LaunchUrl(url)),
    }
}

/// Apply a sidebar selection, then delegate its root-owned consequences.
fn select(state: &mut State, id: nav_bar::Id) -> Outcome {
    state.activate(id);
    selection_outcome(state)
}

/// Build the ordered consequences of the current sidebar selection.
fn selection_outcome(state: &State) -> Outcome {
    selection_outcome_for(state.active_item())
}

/// Build the ordered consequences for one resolved sidebar item.
fn selection_outcome_for(item: Option<&SidebarItem>) -> Outcome {
    let mut outcome = match item {
        Some(SidebarItem::Zone { name, .. }) => Outcome::request(Request::LoadZone(name.clone())),
        Some(SidebarItem::IpSets) => Outcome::request(Request::LoadIpSets),
        _ => Outcome::request(Request::ClearSelectedZone),
    };
    outcome.append(Outcome::request(Request::RefreshTitle));
    outcome
}

/// Reduce a navigation context-menu command without touching root state.
fn menu_action(state: &mut State, action: MenuAction) -> Outcome {
    match action {
        MenuAction::AssignInterface(id) => {
            let Some(zone_name) = state.zone_name_for_id(id) else {
                return Outcome::default();
            };
            let mut outcome = select(state, id);
            outcome.append(Outcome::request(Request::OpenContextPage(
                ContextPage::AddInterface,
            )));
            debug_assert!(matches!(
                state.active_item(),
                Some(SidebarItem::Zone { name, .. }) if name == &zone_name
            ));
            outcome
        }
        MenuAction::SetDefault(id) => state
            .zone_name_for_id(id)
            .map(Request::SetDefaultZone)
            .map(Outcome::request)
            .unwrap_or_default(),
        MenuAction::Rename(id) => state
            .zone_name_for_id(id)
            .map(Request::RenameZone)
            .map(Outcome::request)
            .unwrap_or_default(),
        MenuAction::Delete(id) => state
            .zone_name_for_id(id)
            .map(Request::ConfirmDeleteZone)
            .map(Outcome::request)
            .unwrap_or_default(),
    }
}

/// Apply a zone-list projection while retaining the current selected sidebar item.
fn zones_loaded(state: &mut State, result: Result<Vec<String>, String>) -> Outcome {
    match result {
        Ok(zones) => {
            state.set_zones(zones);
            let mut outcome = Outcome::request(Request::LoadDefaultZone);
            outcome.append(Outcome::request(Request::LoadActiveZones));
            match state.active_item() {
                Some(SidebarItem::Zone { name, .. }) => {
                    outcome.append(Outcome::request(Request::LoadZone(name.clone())));
                }
                _ => {
                    outcome.append(Outcome::request(Request::ClearSelectedZone));
                    outcome.append(Outcome::request(Request::FinishConfigurationRefresh));
                }
            }
            outcome.append(Outcome::request(Request::RefreshTitle));
            outcome
        }
        Err(message) => {
            state.set_error(message.clone());
            Outcome::request(Request::ShowZoneListError(message))
        }
    }
}

impl menu::action::MenuAction for MenuAction {
    type Message = cosmic::Action<crate::app::Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(crate::app::Message::Navigation(Message::MenuAction(*self)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_selection_loads_before_refreshing_title() {
        let mut state = State::new();
        state.set_zones(vec!["public".to_string()]);
        let id = state.zone_id("public").expect("zone is materialized");
        let outcome = select(&mut state, id);

        assert_eq!(
            outcome.requests,
            &[
                Request::LoadZone("public".to_string()),
                Request::RefreshTitle
            ]
        );
        assert!(
            matches!(state.active_item(), Some(SidebarItem::Zone { name, .. }) if name == "public")
        );
    }

    #[test]
    fn ipsets_selection_requests_list_load() {
        let mut state = State::new();
        let id = state.nav_model().active();

        assert_eq!(
            select(&mut state, id).requests,
            &[Request::LoadIpSets, Request::RefreshTitle]
        );
    }

    #[test]
    fn non_zone_selection_clears_reconciliation_before_title_refresh() {
        assert_eq!(
            selection_outcome_for(Some(&SidebarItem::Loading)).requests,
            &[Request::ClearSelectedZone, Request::RefreshTitle]
        );
    }

    #[test]
    fn zone_menu_actions_preserve_request_order() {
        let mut state = State::new();
        state.set_zones(vec!["public".to_string()]);
        let id = state.zone_id("public").expect("zone is materialized");

        assert_eq!(
            menu_action(&mut state, MenuAction::AssignInterface(id)).requests,
            &[
                Request::LoadZone("public".to_string()),
                Request::RefreshTitle,
                Request::OpenContextPage(ContextPage::AddInterface),
            ]
        );
        assert_eq!(
            menu_action(&mut state, MenuAction::SetDefault(id)).requests,
            &[Request::SetDefaultZone("public".to_string())]
        );
        assert_eq!(
            menu_action(&mut state, MenuAction::Rename(id)).requests,
            &[Request::RenameZone("public".to_string())]
        );
        assert_eq!(
            menu_action(&mut state, MenuAction::Delete(id)).requests,
            &[Request::ConfirmDeleteZone("public".to_string())]
        );
    }

    #[test]
    fn zone_projections_preserve_loading_error_default_and_active_indicators() {
        let mut state = State::new();
        assert!(state.is_loading());
        state.set_zones(vec!["public".to_string(), "home".to_string()]);
        state.set_default_zone(Some("home".to_string()));
        state.set_active_zones(HashSet::from(["public".to_string()]));

        assert!(matches!(state.active_item(), Some(SidebarItem::IpSets)));
        assert_eq!(state.zone_indicators("public"), Some((false, true)));
        assert_eq!(state.zone_indicators("home"), Some((true, false)));

        state.set_filter("pub".into());
        assert!(state.zone_exists("home"));
        assert_eq!(state.zone_indicators("public"), Some((false, true)));
        assert_eq!(state.zone_indicators("home"), None);

        let _ = zones_loaded(&mut state, Err("offline".to_string()));
        assert_eq!(state.error_message(), Some("offline"));
    }
}
