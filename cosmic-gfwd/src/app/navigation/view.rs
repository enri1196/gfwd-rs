//! Navigation model materialization.

use std::borrow::Cow;
#[cfg(test)]
use std::collections::HashMap;
use std::collections::HashSet;

use cosmic::widget::nav_bar;

use crate::fl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidebarItem {
    IpSets,
    Zone {
        name: String,
        is_default: bool,
        is_active: bool,
    },
    Loading,
    Empty,
    Error(String),
}

impl SidebarItem {
    fn label(&self) -> Cow<'static, str> {
        match self {
            SidebarItem::IpSets => Cow::Owned(fl!("sidebar-ipsets")),
            SidebarItem::Zone {
                name,
                is_default,
                is_active,
            } => {
                if *is_default && *is_active {
                    Cow::Owned(fl!("sidebar-zone-default-active", name = name))
                } else if *is_default {
                    Cow::Owned(fl!("sidebar-zone-default", name = name))
                } else if *is_active {
                    Cow::Owned(fl!("sidebar-zone-active", name = name))
                } else {
                    Cow::Owned(name.clone())
                }
            }
            SidebarItem::Loading => Cow::Owned(fl!("sidebar-loading-zones")),
            SidebarItem::Empty => Cow::Owned(fl!("sidebar-empty-zones")),
            SidebarItem::Error(_) => Cow::Owned(fl!("sidebar-error-zones")),
        }
    }

    fn zone_name(&self) -> Option<&str> {
        match self {
            SidebarItem::Zone { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ActiveSidebarItem {
    IpSets,
    Zone(String),
}

#[derive(Clone, Debug)]
enum SidebarStatus {
    Loading,
    Ready,
    Empty,
    Error(String),
}

pub struct Sidebar {
    nav: nav_bar::Model,
    context_targets: Vec<Option<nav_bar::Id>>,
    #[cfg(test)]
    zone_ids: HashMap<String, nav_bar::Id>,
    zones: Vec<String>,
    filter: String,
    default_zone: Option<String>,
    active_zones: HashSet<String>,
    status: SidebarStatus,
    active_zone_override: Option<String>,
}

impl Sidebar {
    pub fn new() -> Self {
        let mut sidebar = Self {
            nav: nav_bar::Model::default(),
            context_targets: Vec::new(),
            #[cfg(test)]
            zone_ids: HashMap::new(),
            zones: Vec::new(),
            filter: String::new(),
            default_zone: None,
            active_zones: HashSet::new(),
            status: SidebarStatus::Loading,
            active_zone_override: None,
        };
        sidebar.rebuild_items();
        sidebar
    }

    pub fn nav_model(&self) -> &nav_bar::Model {
        &self.nav
    }

    pub(crate) fn context_targets(&self) -> &[Option<nav_bar::Id>] {
        &self.context_targets
    }

    pub fn activate(&mut self, id: nav_bar::Id) {
        self.nav.activate(id);
    }

    pub fn active_item(&self) -> Option<&SidebarItem> {
        self.nav.active_data::<SidebarItem>()
    }

    pub fn active_label(&self) -> Option<String> {
        self.active_item().map(|item| match item {
            SidebarItem::Zone { name, .. } => name.clone(),
            _ => item.label().into_owned(),
        })
    }

    pub fn item_for_id(&self, id: nav_bar::Id) -> Option<&SidebarItem> {
        self.nav.data::<SidebarItem>(id)
    }

    pub fn zone_name_for_id(&self, id: nav_bar::Id) -> Option<String> {
        self.item_for_id(id)
            .and_then(|item| item.zone_name().map(|name| name.to_string()))
    }

    /// Return the current navigation identifier for a materialized zone.
    #[cfg(test)]
    pub(crate) fn zone_id(&self, zone_name: &str) -> Option<nav_bar::Id> {
        self.zone_ids.get(zone_name).copied()
    }

    pub fn set_loading(&mut self) {
        self.status = SidebarStatus::Loading;
    }

    pub fn set_zones(&mut self, zones: Vec<String>) {
        self.zones = zones;
        if self.zones.is_empty() {
            self.status = SidebarStatus::Empty;
        } else {
            self.status = SidebarStatus::Ready;
        }
        self.rebuild_items();
    }

    pub(crate) fn filter(&self) -> &str {
        &self.filter
    }

    pub(crate) fn should_show_filter(&self) -> bool {
        self.zones.len() >= 8
    }

    pub(crate) fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.rebuild_items();
    }

    pub(crate) fn zone_exists(&self, name: &str) -> bool {
        self.zones.iter().any(|zone| zone == name)
    }

    /// Preserve selection across an externally initiated zone rename.
    pub fn preserve_zone_rename(&mut self, old_name: &str, new_name: &str) {
        if matches!(
            self.active_item(),
            Some(SidebarItem::Zone { name, .. }) if name == old_name
        ) {
            self.active_zone_override = Some(new_name.to_string());
        }
    }

    pub fn set_default_zone(&mut self, zone: Option<String>) {
        self.default_zone = zone;
        self.rebuild_items();
    }

    pub fn set_active_zones(&mut self, active_zones: HashSet<String>) {
        self.active_zones = active_zones;
        self.rebuild_items();
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.status = SidebarStatus::Error(message.into());
        self.rebuild_items();
    }

    /// Return the default and runtime-active indicators for a permanent zone.
    #[cfg(test)]
    pub(crate) fn zone_indicators(&self, zone_name: &str) -> Option<(bool, bool)> {
        self.build_zone_items()
            .into_iter()
            .find_map(|item| match item {
                SidebarItem::Zone {
                    name,
                    is_default,
                    is_active,
                } if name == zone_name => Some((is_default, is_active)),
                _ => None,
            })
    }

    /// Report whether the sidebar currently presents its zone-list loading state.
    #[cfg(test)]
    pub(crate) fn is_loading(&self) -> bool {
        matches!(self.status, SidebarStatus::Loading)
    }

    /// Return the zone-list error currently presented by the sidebar.
    #[cfg(test)]
    pub(crate) fn error_message(&self) -> Option<&str> {
        match &self.status {
            SidebarStatus::Error(message) => Some(message),
            _ => None,
        }
    }

    fn active_key(&self) -> Option<ActiveSidebarItem> {
        match self.active_item() {
            Some(SidebarItem::IpSets) => Some(ActiveSidebarItem::IpSets),
            Some(SidebarItem::Zone { name, .. }) => Some(ActiveSidebarItem::Zone(name.clone())),
            _ => None,
        }
    }

    fn build_zone_items(&self) -> Vec<SidebarItem> {
        if !matches!(self.status, SidebarStatus::Ready) {
            return Vec::new();
        }

        self.zones
            .iter()
            .filter(|name| {
                self.filter.trim().is_empty()
                    || name
                        .to_lowercase()
                        .contains(&self.filter.trim().to_lowercase())
            })
            .map(|name| SidebarItem::Zone {
                name: name.clone(),
                is_default: self.default_zone.as_deref() == Some(name.as_str()),
                is_active: self.active_zones.contains(name),
            })
            .collect()
    }

    fn rebuild_items(&mut self) {
        let items = match &self.status {
            SidebarStatus::Loading => vec![SidebarItem::Loading],
            SidebarStatus::Empty => vec![SidebarItem::Empty],
            SidebarStatus::Error(message) => vec![SidebarItem::Error(message.clone())],
            SidebarStatus::Ready => self.build_zone_items(),
        };

        self.set_items(items);
    }

    fn set_items(&mut self, mut items: Vec<SidebarItem>) {
        if items.is_empty() {
            items.push(SidebarItem::Empty);
        }

        let active_key = self
            .active_zone_override
            .take()
            .map(ActiveSidebarItem::Zone)
            .or_else(|| self.active_key());
        self.nav.clear();
        self.context_targets.clear();
        #[cfg(test)]
        self.zone_ids.clear();

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
                (SidebarItem::Zone { name, .. }, Some(ActiveSidebarItem::Zone(active))) => {
                    name == active
                }
                _ => false,
            };

            let needs_divider = !added_separator && !matches!(item, SidebarItem::IpSets);
            let mut entry = self.nav.insert().text(label).data(item.clone());
            if needs_divider {
                entry = entry.divider_above(true);
                added_separator = true;
            }

            let id = entry.id();
            self.context_targets
                .push(matches!(&item, SidebarItem::Zone { .. }).then_some(id));

            #[cfg(test)]
            if let SidebarItem::Zone { name, .. } = item {
                self.zone_ids.insert(name, id);
            }

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
