use std::borrow::Cow;

use cosmic::widget::nav_bar;
// use cosmic::widget::icon;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidebarItem {
    Zone(String),
    Loading,
    Empty,
    Error(String),
}

impl SidebarItem {
    fn label(&self) -> Cow<'static, str> {
        match self {
            SidebarItem::Zone(name) => Cow::Owned(name.clone()),
            SidebarItem::Loading => Cow::Borrowed("Loading zones..."),
            SidebarItem::Empty => Cow::Borrowed("No zones found"),
            SidebarItem::Error(_) => Cow::Borrowed("Failed to load zones"),
        }
    }

    // fn icon_name(&self) -> &'static str {
    //     match self {
    //         SidebarItem::Zone(_) => "network-firewall-symbolic",
    //         SidebarItem::Loading => "view-refresh-symbolic",
    //         SidebarItem::Empty => "list-remove-symbolic",
    //         SidebarItem::Error(_) => "dialog-warning-symbolic",
    //     }
    // }
}

pub struct Sidebar {
    nav: nav_bar::Model,
}

impl Sidebar {
    pub fn new() -> Self {
        let mut sidebar = Self {
            nav: nav_bar::Model::default(),
        };
        sidebar.set_items(vec![SidebarItem::Loading]);
        sidebar
    }

    pub fn nav_model(&self) -> &nav_bar::Model {
        &self.nav
    }

    pub fn activate(&mut self, id: nav_bar::Id) {
        self.nav.activate(id);
    }

    pub fn active_item(&self) -> Option<&SidebarItem> {
        self.nav.active_data::<SidebarItem>()
    }

    pub fn active_label(&self) -> Option<&str> {
        self.nav.text(self.nav.active())
    }

    pub fn set_zones(&mut self, zones: Vec<String>) {
        if zones.is_empty() {
            self.set_items(vec![SidebarItem::Empty]);
            return;
        }

        let items = zones.into_iter().map(SidebarItem::Zone).collect();
        self.set_items(items);
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.set_items(vec![SidebarItem::Error(message.into())]);
    }

    fn active_zone(&self) -> Option<&str> {
        match self.active_item() {
            Some(SidebarItem::Zone(name)) => Some(name.as_str()),
            _ => None,
        }
    }

    fn set_items(&mut self, mut items: Vec<SidebarItem>) {
        if items.is_empty() {
            items.push(SidebarItem::Empty);
        }

        let active_zone = self.active_zone().map(str::to_string);
        self.nav.clear();

        let mut first_id = None;
        let mut active_id = None;

        for item in items {
            let label = item.label();
            // let icon_name = item.icon_name();
            let is_active = match (&item, &active_zone) {
                (SidebarItem::Zone(name), Some(active)) => name == active,
                _ => false,
            };

            let id = self
                .nav
                .insert()
                .text(label)
                .data(item)
                // .icon(icon::from_name(icon_name))
                .id();

            if first_id.is_none() {
                first_id = Some(id);
            }

            if is_active {
                active_id = Some(id);
            }
        }

        if let Some(id) = active_id.or(first_id) {
            self.nav.activate(id);
        }
    }
}
