//! Zone and firewalld asynchronous effects.

use std::collections::HashSet;

use cosmic::Task;

use super::{Message as ZoneMessage, ZoneViewAction, ZoneViewState};
use crate::{
    app::{AppModel, Message},
    core::broker::{BrokerError, FirewalldStatus, FwdBroker},
    fl,
    models::zone::{ZoneDetails, ZoneTarget},
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

/// Start or stop firewalld after reserving the global mutation slot.
pub(crate) fn start_firewalld_control(
    app: &mut AppModel,
    start: bool,
) -> Task<cosmic::Action<Message>> {
    let operation = if start {
        fl!("operation-start-firewalld")
    } else {
        fl!("operation-stop-firewalld")
    };
    if !app.begin_mutation(operation) {
        return Task::none();
    }
    app.firewalld_status = FirewalldStatus::Loading;
    Task::perform(control_firewalld(start), |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::DaemonControlFinished(result)))
    })
}

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

/// Add a service to a permanent zone.
pub(crate) fn start_service_add(
    app: &mut AppModel,
    zone_name: String,
    service: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-service")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_service(zone_name, service), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Add a port to a permanent zone.
pub(crate) fn start_port_add(
    app: &mut AppModel,
    zone_name: String,
    port: String,
    protocol: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-port")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_port(zone_name, port, protocol), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Add a source port to a permanent zone.
pub(crate) fn start_source_port_add(
    app: &mut AppModel,
    zone_name: String,
    port: String,
    protocol: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-source-port")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_source_port(zone_name, port, protocol), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Add a forwarded port to a permanent zone.
pub(crate) fn start_forward_port_add(
    app: &mut AppModel,
    zone_name: String,
    port: String,
    protocol: String,
    to_port: String,
    to_addr: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-forward-port")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(
        add_forward_port(zone_name, port, protocol, to_port, to_addr),
        move |result| {
            cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
        },
    )
}

/// Add an interface to a permanent zone.
pub(crate) fn start_interface_add(
    app: &mut AppModel,
    zone_name: String,
    interface: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-interface")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_interface(zone_name, interface), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Add a source to a permanent zone.
pub(crate) fn start_source_add(
    app: &mut AppModel,
    zone_name: String,
    source: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-source")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_source(zone_name, source), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Add an ICMP block to a permanent zone.
pub(crate) fn start_icmp_add(
    app: &mut AppModel,
    zone_name: String,
    icmp: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-icmp")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_icmp_block(zone_name, icmp), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
            zone_name: zone_name_for_task.clone(),
            result,
        }))
    })
}

/// Add a rich rule to a permanent zone.
pub(crate) fn start_rich_rule_add(
    app: &mut AppModel,
    zone_name: String,
    rule: String,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-add-rich-rule")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    Task::perform(add_rich_rule(zone_name, rule), move |result| {
        cosmic::Action::from(Message::Zone(ZoneMessage::ItemAdded {
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

/// Remove the item selected by a zone view action.
pub(crate) fn start_zone_item_remove(
    app: &mut AppModel,
    zone_name: String,
    action: ZoneViewAction,
) -> Task<cosmic::Action<Message>> {
    if !app.begin_mutation(fl!("operation-remove-zone-item")) {
        return Task::none();
    }
    let zone_name_for_task = zone_name.clone();
    match action {
        ZoneViewAction::SetMasquerade(_)
        | ZoneViewAction::SetIcmpBlockInversion(_)
        | ZoneViewAction::StartFirewalld
        | ZoneViewAction::StopFirewalld
        | ZoneViewAction::Reconciliation(_) => Task::none(),
        ZoneViewAction::AddService
        | ZoneViewAction::AddInterface
        | ZoneViewAction::AddPort { .. }
        | ZoneViewAction::AddSource
        | ZoneViewAction::AddIcmpBlock
        | ZoneViewAction::AddRichRule => Task::none(),
        ZoneViewAction::RemoveService(service) => {
            Task::perform(remove_service(zone_name, service), move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            })
        }
        ZoneViewAction::RemoveInterface(interface) => {
            Task::perform(remove_interface(zone_name, interface), move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            })
        }
        ZoneViewAction::RemoveSource(source) => {
            Task::perform(remove_source(zone_name, source), move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            })
        }
        ZoneViewAction::RemovePort { port, protocol } => {
            Task::perform(remove_port(zone_name, port, protocol), move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            })
        }
        ZoneViewAction::RemoveForwardPort {
            port,
            protocol,
            to_port,
            to_addr,
        } => Task::perform(
            remove_forward_port(zone_name, port, protocol, to_port, to_addr),
            move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            },
        ),
        ZoneViewAction::RemoveSourcePort { port, protocol } => Task::perform(
            remove_source_port(zone_name, port, protocol),
            move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            },
        ),
        ZoneViewAction::RemoveIcmpBlock(icmp) => {
            Task::perform(remove_icmp_block(zone_name, icmp), move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            })
        }
        ZoneViewAction::RemoveRichRule(rule) => {
            Task::perform(remove_rich_rule(zone_name, rule), move |result| {
                cosmic::Action::from(Message::Zone(ZoneMessage::ItemRemoved {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            })
        }
    }
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

/// Start or stop firewalld through the broker.
pub(crate) async fn control_firewalld(start: bool) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    if start {
        broker.start_firewalld().await
    } else {
        broker.stop_firewalld().await
    }
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

/// Add a service to a permanent zone through the broker.
pub(crate) async fn add_service(zone_name: String, service: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_service(&zone_name, &service).await
}

/// Add a port to a permanent zone through the broker.
pub(crate) async fn add_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_port(&zone_name, &port, &protocol).await
}

/// Add a source port to a permanent zone through the broker.
pub(crate) async fn add_source_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_source_port(&zone_name, &port, &protocol).await
}

/// Add a forwarded port to a permanent zone through the broker.
pub(crate) async fn add_forward_port(
    zone_name: String,
    port: String,
    protocol: String,
    to_port: String,
    to_addr: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker
        .add_forward_port(&zone_name, &port, &protocol, &to_port, &to_addr)
        .await
}

/// Add an interface to a permanent zone through the broker.
pub(crate) async fn add_interface(zone_name: String, interface: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_interface(&zone_name, &interface).await
}

/// Add a source to a permanent zone through the broker.
pub(crate) async fn add_source(zone_name: String, source: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_source(&zone_name, &source).await
}

/// Add an ICMP block to a permanent zone through the broker.
pub(crate) async fn add_icmp_block(zone_name: String, icmp: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_icmp_block(&zone_name, &icmp).await
}

/// Add a rich rule to a permanent zone through the broker.
pub(crate) async fn add_rich_rule(zone_name: String, rule: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.add_rich_rule(&zone_name, &rule).await
}

/// Delete a permanent zone through the broker.
pub(crate) async fn remove_zone(zone_name: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_zone(&zone_name).await
}

/// Remove a service from a permanent zone through the broker.
pub(crate) async fn remove_service(zone_name: String, service: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_service(&zone_name, &service).await
}

/// Remove an interface from a permanent zone through the broker.
pub(crate) async fn remove_interface(
    zone_name: String,
    interface: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_interface(&zone_name, &interface).await
}

/// Remove a source from a permanent zone through the broker.
pub(crate) async fn remove_source(zone_name: String, source: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_source(&zone_name, &source).await
}

/// Remove a port from a permanent zone through the broker.
pub(crate) async fn remove_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_port(&zone_name, &port, &protocol).await
}

/// Remove a forwarded port from a permanent zone through the broker.
pub(crate) async fn remove_forward_port(
    zone_name: String,
    port: String,
    protocol: String,
    to_port: String,
    to_addr: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker
        .remove_forward_port(&zone_name, &port, &protocol, &to_port, &to_addr)
        .await
}

/// Remove a source port from a permanent zone through the broker.
pub(crate) async fn remove_source_port(
    zone_name: String,
    port: String,
    protocol: String,
) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker
        .remove_source_port(&zone_name, &port, &protocol)
        .await
}

/// Remove an ICMP block from a permanent zone through the broker.
pub(crate) async fn remove_icmp_block(zone_name: String, icmp: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_icmp_block(&zone_name, &icmp).await
}

/// Remove a rich rule from a permanent zone through the broker.
pub(crate) async fn remove_rich_rule(zone_name: String, rule: String) -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.remove_rich_rule(&zone_name, &rule).await
}
