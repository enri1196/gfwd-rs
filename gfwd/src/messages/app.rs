use crate::models::ZoneSettings;

#[derive(Debug)]
pub enum AppRequest {
    ToggleSidebar,
    ShowAddZoneDialog,
    ZoneAdded(ZoneSettings),
    ZoneRemoved(String),
    UpdateContentWithZoneName(String),
}


