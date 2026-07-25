use crate::config::Config;

use super::{ContextPage, NavMenuAction};

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
