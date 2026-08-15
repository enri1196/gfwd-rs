//! Permanent zone and zone-level setting effects.

use cosmic::Task;

use super::super::Message as ZoneMessage;
use crate::{
    app::{AppModel, Message},
    core::broker::{BrokerError, FwdBroker},
    fl,
    models::zone::ZoneTarget,
};

/// Set permanent masquerading for the selected zone.
pub(crate) fn start_masquerade_set(
    app: &mut AppModel,
    zone_name: String,
    enabled: bool,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-set-masquerading")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(set_masquerade(zone_name, enabled), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Set permanent ICMP block inversion for the selected zone.
pub(crate) fn start_icmp_inversion_set(
    app: &mut AppModel,
    zone_name: String,
    enabled: bool,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-set-icmp-inversion")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(
        set_icmp_block_inversion(zone_name, enabled),
        move |result| {
            cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
        },
    )
}

/// Set the permanent default zone.
pub(crate) fn start_default_zone_set(
    app: &mut AppModel,
    zone_name: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-set-default-zone")) {
        return Task::none();
    }
    Task::perform(set_default_zone(zone_name), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::DefaultSet(result)))
    })
}

/// Create a permanent zone.
pub(crate) fn start_zone_create(
    app: &mut AppModel,
    name: String,
    description: String,
    target: ZoneTarget,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-create-zone")) {
        return Task::none();
    }
    let zone_name_for_task = name.clone();
    Task::perform(add_zone(name, description, target), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::Created {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Delete a permanent zone.
pub(crate) fn start_zone_delete(
    app: &mut AppModel,
    zone_name: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-delete-zone")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(remove_zone(zone_name), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::Deleted {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Set permanent masquerading through the broker.
pub(crate) async fn set_masquerade(zone_name: String, enabled: bool) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.set_masquerade(&zone_name, enabled).await
}

/// Set permanent ICMP block inversion through the broker.
pub(crate) async fn set_icmp_block_inversion(
    zone_name: String,
    enabled: bool,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.set_icmp_block_inversion(&zone_name, enabled).await
}

/// Set the permanent default zone through the broker.
pub(crate) async fn set_default_zone(zone_name: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.set_default_zone(&zone_name).await
}

/// Create a permanent zone through the broker.
pub(crate) async fn add_zone(
    name: String,
    description: String,
    target: ZoneTarget,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_zone(&name, &description, &target).await
}

/// Delete a permanent zone through the broker.
pub(crate) async fn remove_zone(zone_name: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_zone(&zone_name).await
}
