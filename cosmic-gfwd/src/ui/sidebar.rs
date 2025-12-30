use std::borrow::Cow;

use cosmic::widget::nav_bar;

use crate::fl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidebarItem {
    IpSets,
    Zone(String),
    Loading,
    Empty,
    Error(String),
}

impl SidebarItem {
    fn label(&self) -> Cow<'static, str> {
        match self {
            SidebarItem::IpSets => Cow::Owned(fl!("sidebar-ipsets")),
            SidebarItem::Zone(name) => Cow::Owned(name.clone()),
            SidebarItem::Loading => Cow::Owned(fl!("sidebar-loading-zones")),
            SidebarItem::Empty => Cow::Owned(fl!("sidebar-empty-zones")),
            SidebarItem::Error(_) => Cow::Owned(fl!("sidebar-error-zones")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ActiveSidebarItem {
    IpSets,
    Zone(String),
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

    fn active_key(&self) -> Option<ActiveSidebarItem> {
        match self.active_item() {
            Some(SidebarItem::IpSets) => Some(ActiveSidebarItem::IpSets),
            Some(SidebarItem::Zone(name)) => Some(ActiveSidebarItem::Zone(name.clone())),
            _ => None,
        }
    }

    fn set_items(&mut self, mut items: Vec<SidebarItem>) {
        if items.is_empty() {
            items.push(SidebarItem::Empty);
        }

        let active_key = self.active_key();
        self.nav.clear();

        let mut first_id = None;
        let mut active_id = None;
        let mut all_items = Vec::with_capacity(items.len() + 1);
        all_items.push(SidebarItem::IpSets);
        all_items.extend(items);

        let mut added_separator = false;

        for item in all_items {
            let label = item.label();
            let is_active = match (&item, &active_key) {
                (SidebarItem::IpSets, Some(ActiveSidebarItem::IpSets)) => true,
                (SidebarItem::Zone(name), Some(ActiveSidebarItem::Zone(active))) => {
                    name == active
                }
                _ => false,
            };

            let needs_divider = !added_separator && !matches!(item, SidebarItem::IpSets);
            let mut entry = self.nav.insert().text(label).data(item);
            if needs_divider {
                entry = entry.divider_above(true);
                added_separator = true;
            }

            let id = entry.id();

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
