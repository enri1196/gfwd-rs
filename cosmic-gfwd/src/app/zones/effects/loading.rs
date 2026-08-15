//! Zone, selection, daemon-status, and projection loading.

use std::collections::HashSet;

use cosmic::Task;

use super::super::{Message as ZoneMessage, ZoneViewState};
use crate::{
    app::{AppModel, Message},
    core::broker::{BrokerError, FirewalldStatus, FwdBroker},
    models::zone::ZoneDetails,
};

/// Load the permanent zone list and mark navigation as loading.
pub(crate) fn start_zones_load(app: &mut AppModel) -> Task<cosmic::Action<Message>> {
    app.navigation.set_loading();
    Task::perform(load_zones(), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ListLoaded(result)))
    })
}

/// Load the configured default zone.
pub(crate) fn start_default_zone_load(_app: &mut AppModel) -> Task<cosmic::Action<Message>> {
    Task::perform(load_default_zone(), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::DefaultLoaded(result)))
    })
}

/// Load the runtime-active zone set.
pub(crate) fn start_active_zones_load(_app: &mut AppModel) -> Task<cosmic::Action<Message>> {
    Task::perform(load_active_zones(), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ActiveLoaded(result)))
    })
}

/// Load the current firewalld daemon status.
pub(crate) fn start_firewalld_status_load(app: &mut AppModel) -> Task<cosmic::Action<Message>> {
    app.firewalld_status = FirewalldStatus::Loading;
    Task::perform(load_firewalld_status(), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::FirewalldStatusLoaded(result)))
    })
}

/// Load details for a selected zone and update root selection state.
pub(crate) fn start_zone_load(
    app: &mut AppModel,
    zone_name: String,
) -> Task<cosmic::Action<Message>> {
    app.reconciliation
        .selection_changed(Some(zone_name.clone()));
    app.zones = ZoneViewState::Loading {
        zone: zone_name.clone(),
    };

    let zone_name_for_task = zone_name.clone();
    Task::perform(load_zone_details(zone_name), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::DetailsLoaded {
            zone_name: zone_name_for_task.clone(),
            result: Box::new(result),
        }))
    })
}

/// Load the permanent zone list through the broker.
pub(crate) async fn load_zones() -> Result<Vec<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_zones().await
}

/// Load permanent details for one zone through the broker.
pub(crate) async fn load_zone_details(zone_name: String) -> Result<ZoneDetails, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_zone_details(&zone_name).await
}

/// Load the configured default zone through the broker.
pub(crate) async fn load_default_zone() -> Result<String, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_default_zone().await
}

/// Load runtime-active zones through the broker.
pub(crate) async fn load_active_zones() -> Result<HashSet<String>, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.get_active_zones().await
}

/// Load the firewalld daemon status through the broker.
pub(crate) async fn load_firewalld_status() -> Result<FirewalldStatus, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.firewalld_status().await
}
