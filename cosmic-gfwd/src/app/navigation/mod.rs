use crate::config::Config;

use super::{ContextPage, NavMenuAction};

mod view;

pub(crate) use view::{Sidebar, SidebarItem};

/// Navigation item materialization and selection state.
pub(crate) type State = Sidebar;

/// Messages owned by the application shell and navigation slice.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Launch a URL through the desktop.
    LaunchUrl(String),
    /// Toggle the requested context drawer page.
    ToggleContextPage(ContextPage),
    /// Handle a navigation context-menu action.
    MenuAction(NavMenuAction),
    /// Replace the application configuration after a subscription update.
    UpdateConfig(Config),
}
