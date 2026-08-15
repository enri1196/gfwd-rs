//! Permanent zone rule addition and removal effects.

use cosmic::Task;

use super::super::{Message as ZoneMessage, ZoneViewAction};
use crate::{
    app::{AppModel, Message},
    core::broker::{BrokerError, FwdBroker},
    fl,
};

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
