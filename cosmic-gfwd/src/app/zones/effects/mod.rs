//! Broker-backed asynchronous work for the zone feature.

use cosmic::Task;

use super::{Effect, Message};

mod daemon;
mod loading;
mod rules;
mod zone;

/// Build the asynchronous task for one zone effect.
pub(crate) fn effects(effect: Effect) -> Task<Message> {
    match effect {
        Effect::LoadZones => Task::perform(loading::load_zones(), Message::ListLoaded),
        Effect::LoadDetails(zone_name) => {
            let zone_name_for_task = zone_name.clone();
            Task::perform(loading::load_zone_details(zone_name), move |result| {
                Message::DetailsLoaded {
                    zone_name: zone_name_for_task.clone(),
                    result: Box::new(result),
                }
            })
        }
        Effect::LoadDefault => Task::perform(loading::load_default_zone(), Message::DefaultLoaded),
        Effect::LoadActive => Task::perform(loading::load_active_zones(), Message::ActiveLoaded),
        Effect::LoadStatus => Task::perform(
            loading::load_firewalld_status(),
            Message::FirewalldStatusLoaded,
        ),
        Effect::SetDefault(zone_name) => {
            Task::perform(zone::set_default_zone(zone_name), Message::DefaultSet)
        }
        Effect::CreateZone {
            name,
            description,
            target,
        } => {
            let zone_name = name.clone();
            Task::perform(zone::add_zone(name, description, target), move |result| {
                Message::Created {
                    zone_name: zone_name.clone(),
                    result,
                }
            })
        }
        Effect::DeleteZone(zone_name) => {
            let completed_zone = zone_name.clone();
            Task::perform(zone::remove_zone(zone_name), move |result| {
                Message::Deleted {
                    zone_name: completed_zone.clone(),
                    result,
                }
            })
        }
        Effect::AddService { zone, service } => {
            item_added(zone, move |zone| rules::add_service(zone, service))
        }
        Effect::AddPort {
            zone,
            port,
            protocol,
        } => item_added(zone, move |zone| rules::add_port(zone, port, protocol)),
        Effect::AddSourcePort {
            zone,
            port,
            protocol,
        } => item_added(zone, move |zone| {
            rules::add_source_port(zone, port, protocol)
        }),
        Effect::AddForwardPort {
            zone,
            port,
            protocol,
            to_port,
            to_addr,
        } => item_added(zone, move |zone| {
            rules::add_forward_port(zone, port, protocol, to_port, to_addr)
        }),
        Effect::AddInterface { zone, interface } => {
            item_added(zone, move |zone| rules::add_interface(zone, interface))
        }
        Effect::AddSource { zone, source } => {
            item_added(zone, move |zone| rules::add_source(zone, source))
        }
        Effect::AddIcmp { zone, icmp } => {
            item_added(zone, move |zone| rules::add_icmp_block(zone, icmp))
        }
        Effect::AddRichRule { zone, rule } => {
            item_added(zone, move |zone| rules::add_rich_rule(zone, rule))
        }
        Effect::RemoveService { zone, service } => {
            item_removed(zone, move |zone| rules::remove_service(zone, service))
        }
        Effect::RemoveInterface { zone, interface } => {
            item_removed(zone, move |zone| rules::remove_interface(zone, interface))
        }
        Effect::RemoveSource { zone, source } => {
            item_removed(zone, move |zone| rules::remove_source(zone, source))
        }
        Effect::RemovePort {
            zone,
            port,
            protocol,
        } => item_removed(zone, move |zone| rules::remove_port(zone, port, protocol)),
        Effect::RemoveForwardPort {
            zone,
            port,
            protocol,
            to_port,
            to_addr,
        } => item_removed(zone, move |zone| {
            rules::remove_forward_port(zone, port, protocol, to_port, to_addr)
        }),
        Effect::RemoveSourcePort {
            zone,
            port,
            protocol,
        } => item_removed(zone, move |zone| {
            rules::remove_source_port(zone, port, protocol)
        }),
        Effect::RemoveIcmp { zone, icmp } => {
            item_removed(zone, move |zone| rules::remove_icmp_block(zone, icmp))
        }
        Effect::RemoveRichRule { zone, rule } => {
            item_removed(zone, move |zone| rules::remove_rich_rule(zone, rule))
        }
        Effect::SetMasquerade { zone, enabled } => {
            item_added(zone, move |zone| zone::set_masquerade(zone, enabled))
        }
        Effect::SetIcmpBlockInversion { zone, enabled } => item_added(zone, move |zone| {
            zone::set_icmp_block_inversion(zone, enabled)
        }),
        Effect::ControlFirewalld(start) => Task::perform(
            daemon::control_firewalld(start),
            Message::DaemonControlFinished,
        ),
    }
}

fn item_added<F, Fut>(zone: String, operation: F) -> Task<Message>
where
    F: FnOnce(String) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), crate::core::BrokerError>> + Send + 'static,
{
    let completed_zone = zone.clone();
    Task::perform(operation(zone), move |result| Message::ItemAdded {
        zone_name: completed_zone.clone(),
        result,
    })
}

fn item_removed<F, Fut>(zone: String, operation: F) -> Task<Message>
where
    F: FnOnce(String) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), crate::core::BrokerError>> + Send + 'static,
{
    let completed_zone = zone.clone();
    Task::perform(operation(zone), move |result| Message::ItemRemoved {
        zone_name: completed_zone.clone(),
        result,
    })
}
