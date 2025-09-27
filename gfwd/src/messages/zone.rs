use crate::models::{ZoneSettings, ZoneTarget};

#[derive(Debug)]
pub enum ZoneDialogRequest {
    SetName(String),
    SetDescription(String),
    SetTarget(ZoneTarget),
    ValidateName,
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum ZoneDialogResponse {
    ZoneSettings(ZoneSettings),
}

#[derive(Debug)]
pub enum ZoneViewRequest {
    SetZoneContent(String),
    ToggleFirewalld,
    UpdateZoneSettings(ZoneSettings),
    RemoveZone,
    SetFirewalldRunning(bool),
    ShowAddPortDialog,
    AddPort(String, String),
    AddForwardPort(String, String, String, String),
    RemovePort(String, String),
    RemoveForwardPort(String, String, String, String),
    ToggleMasquerading,
    ToggleIcmpBlockInversion,
    ToggleService(String, bool),
    LoadServices,
    UpdateAvailableServices(Vec<String>),
}

#[derive(Debug)]
pub enum ZoneViewResponse {
    ToggleSidebar,
    RemovedZoneSuccess(String),
}

#[derive(Debug)]
pub enum SidebarRequest {
    UpdateZones,
    ShowAddZoneDialog,
    SetDefaultZone,
    SetActiveZones,
    RemoveZone(String),
}

#[derive(Debug)]
pub enum SidebarResponse {
    ShowAddZoneDialog,
    SelectedZone(String),
}
