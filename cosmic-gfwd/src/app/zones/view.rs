//! Zone and firewalld status view.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, icon, settings};

use crate::core::{FirewalldStatus, ZoneReconciliationState};
use crate::fl;
use crate::models::zone::ZoneDetails;

use crate::app::dialogs::PortKind;
use crate::app::reconciliation::{
    ReconciliationAction, ReconciliationPresentation, ReconciliationPresentationStatus,
};

const MAX_LIST_ITEMS: usize = 5;
const LIST_ITEM_HEIGHT: f32 = 28.0;
const ADD_ICON: &str = "list-add-symbolic";
const REMOVE_ICON: &str = "user-trash-symbolic";
const REFRESH_ICON: &str = "view-refresh-symbolic";

#[derive(Debug, Clone)]
pub enum ZoneViewState {
    Empty,
    Loading { zone: String },
    Ready(Box<ZoneDetails>),
    Error { zone: String, message: String },
}

#[derive(Debug, Clone)]
pub enum ZoneViewAction {
    RetryLoad(String),
    /// Permanently enables or disables masquerading.
    SetMasquerade(bool),
    /// Permanently enables or disables ICMP block inversion.
    SetIcmpBlockInversion(bool),
    /// Starts the firewalld systemd unit.
    StartFirewalld,
    /// Requests confirmation before stopping the firewalld systemd unit.
    StopFirewalld,
    /// Performs an action on permanent/runtime reconciliation.
    Reconciliation(ReconciliationAction),
    /// Opens the configured-service picker for the selected zone.
    AddService,
    AddInterface,
    /// Opens the shared port form for the selected semantic port kind.
    AddPort {
        kind: PortKind,
    },
    AddSource,
    AddIcmpBlock,
    AddRichRule,
    RemoveService(String),
    RemoveInterface(String),
    RemoveSource(String),
    RemovePort {
        port: String,
        protocol: String,
    },
    RemoveForwardPort {
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    },
    RemoveSourcePort {
        port: String,
        protocol: String,
    },
    RemoveIcmpBlock(String),
    RemoveRichRule(String),
}

pub fn view_zone_content<'a, Message: 'static + Clone>(
    state: &'a ZoneViewState,
    firewalld_status: &'a FirewalldStatus,
    reconciliation: &'a ZoneReconciliationState,
    watch_warning: Option<&'a str>,
    last_checked_age_seconds: Option<u64>,
    pending_operation: Option<&'a str>,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    match state {
        ZoneViewState::Empty => centered_message(fl!("zone-select-prompt")),
        ZoneViewState::Loading { zone } => centered_message(fl!("zone-loading", zone = zone)),
        ZoneViewState::Error { zone, message } => error_message(zone, message, map),
        ZoneViewState::Ready(details) => widget::scrollable::scrollable(zone_details(
            details,
            firewalld_status,
            reconciliation,
            watch_warning,
            last_checked_age_seconds,
            pending_operation,
            map,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    }
}

fn zone_details<'a, Message: 'static + Clone>(
    details: &'a ZoneDetails,
    firewalld_status: &'a FirewalldStatus,
    reconciliation: &'a ZoneReconciliationState,
    watch_warning: Option<&'a str>,
    last_checked_age_seconds: Option<u64>,
    pending_operation: Option<&'a str>,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let mutation_pending = pending_operation.is_some();
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_m = spacing.space_m;
    let space_l = spacing.space_l;

    let mut header = widget::column::with_capacity(3)
        .push(widget::text::title1(details.name.as_str()))
        .push(zone_description(details))
        .spacing(space_s);
    if let Some(operation) = pending_operation {
        header = header.push(widget::text::caption(fl!(
            "operation-pending-target",
            operation = operation
        )));
    }
    let reconciliation_presentation =
        ReconciliationPresentation::from_state(reconciliation, mutation_pending);
    let reconciliation_section = reconciliation_banner(
        &reconciliation_presentation,
        watch_warning,
        last_checked_age_seconds,
        map,
    );

    let masquerade = if mutation_pending {
        widget::toggler(details.masquerade)
    } else {
        widget::toggler(details.masquerade)
            .on_toggle(move |enabled| map(ZoneViewAction::SetMasquerade(enabled)))
    };
    let icmp_inversion = if mutation_pending {
        widget::toggler(details.icmp_block_inversion)
    } else {
        widget::toggler(details.icmp_block_inversion)
            .on_toggle(move |enabled| map(ZoneViewAction::SetIcmpBlockInversion(enabled)))
    };

    let overview = settings::section()
        .title(fl!("zone-section-overview"))
        .add(
            settings::item::builder(fl!("zone-overview-target"))
                .control(widget::text(details.target.to_string())),
        )
        .add(settings::item::builder(fl!("zone-overview-masquerading")).control(masquerade))
        .add(settings::item::builder(fl!("zone-overview-icmp-inversion")).control(icmp_inversion))
        .add(
            settings::item::builder(fl!("zone-overview-protocols")).control(widget::text(
                list_or_none(&details.protocols, &fl!("zone-no-protocols")),
            )),
        );

    let service_action = match firewalld_status {
        FirewalldStatus::Active => button::destructive(fl!("firewalld-stop"))
            .on_press_maybe((!mutation_pending).then_some(map(ZoneViewAction::StopFirewalld))),
        FirewalldStatus::Inactive => button::suggested(fl!("firewalld-start"))
            .on_press_maybe((!mutation_pending).then_some(map(ZoneViewAction::StartFirewalld))),
        FirewalldStatus::Loading | FirewalldStatus::Error(_) => {
            button::standard(fl!("firewalld-unavailable"))
        }
    };
    let status_label = match firewalld_status {
        FirewalldStatus::Active => fl!("firewalld-status-active"),
        FirewalldStatus::Inactive => fl!("firewalld-status-inactive"),
        FirewalldStatus::Loading => fl!("firewalld-status-loading"),
        FirewalldStatus::Error(error) => {
            fl!("firewalld-status-error", error = error)
        }
    };
    let runtime_action = button::standard(fl!("firewalld-apply")).on_press_maybe(
        reconciliation_presentation
            .actions
            .can_apply_permanent
            .then_some(map(ZoneViewAction::Reconciliation(
                ReconciliationAction::ApplyPermanentToRuntime,
            ))),
    );
    let firewalld = settings::section()
        .title(fl!("firewalld-section"))
        .add(
            settings::item::builder(fl!("firewalld-service-status"))
                .description(status_label)
                .control(service_action),
        )
        .add(
            settings::item::builder(fl!("firewalld-runtime-status"))
                .description(fl!("firewalld-runtime-description"))
                .control(runtime_action),
        );

    let services = list_section(
        section_title(fl!("zone-section-services"), details.services.len()),
        details
            .services
            .iter()
            .cloned()
            .map(|service| (service.clone(), ZoneViewAction::RemoveService(service)))
            .collect(),
        fl!("zone-empty-services"),
        Some((ZoneViewAction::AddService, fl!("action-add-service"))),
        map,
    );

    let interfaces = list_section(
        section_title(fl!("zone-section-interfaces"), details.interfaces.len()),
        details
            .interfaces
            .iter()
            .cloned()
            .map(|interface| {
                (
                    interface.clone(),
                    ZoneViewAction::RemoveInterface(interface),
                )
            })
            .collect(),
        fl!("zone-empty-interfaces"),
        Some((ZoneViewAction::AddInterface, fl!("action-add-interface"))),
        map,
    );

    let sources = list_section(
        section_title(fl!("zone-section-sources"), details.sources.len()),
        details
            .sources
            .iter()
            .cloned()
            .map(|source| (source.clone(), ZoneViewAction::RemoveSource(source)))
            .collect(),
        fl!("zone-empty-sources"),
        Some((ZoneViewAction::AddSource, fl!("action-add-source"))),
        map,
    );

    let ports = list_section(
        section_title(fl!("zone-section-ports"), details.ports.len()),
        details
            .ports
            .iter()
            .cloned()
            .map(|(port, protocol)| {
                (
                    format!("{}/{}", port, protocol),
                    ZoneViewAction::RemovePort { port, protocol },
                )
            })
            .collect(),
        fl!("zone-empty-ports"),
        Some((
            port_add_action(PortKind::Destination),
            fl!("action-add-port"),
        )),
        map,
    );

    let forward_ports = list_section(
        section_title(
            fl!("zone-section-forward-ports"),
            details.forward_ports.len(),
        ),
        details
            .forward_ports
            .iter()
            .cloned()
            .map(|(port, protocol, to_port, to_addr)| {
                (
                    if to_addr.is_empty() {
                        format!("{}/{} -> {}", port, protocol, to_port)
                    } else {
                        format!("{}/{} -> {} ({})", port, protocol, to_port, to_addr)
                    },
                    ZoneViewAction::RemoveForwardPort {
                        port,
                        protocol,
                        to_port,
                        to_addr,
                    },
                )
            })
            .collect(),
        fl!("zone-empty-forward-ports"),
        Some((
            port_add_action(PortKind::Forward),
            fl!("action-add-forward-port"),
        )),
        map,
    );

    let source_ports = list_section(
        section_title(fl!("zone-section-source-ports"), details.source_ports.len()),
        source_port_rows(&details.source_ports),
        fl!("zone-empty-source-ports"),
        Some((
            port_add_action(PortKind::Source),
            fl!("action-add-source-port"),
        )),
        map,
    );

    let icmp_blocks = list_section(
        section_title(fl!("zone-section-icmp"), details.icmp_blocks.len()),
        details
            .icmp_blocks
            .iter()
            .cloned()
            .map(|icmp| (icmp.clone(), ZoneViewAction::RemoveIcmpBlock(icmp)))
            .collect(),
        fl!("zone-empty-icmp"),
        Some((ZoneViewAction::AddIcmpBlock, fl!("action-add-icmp"))),
        map,
    );

    let rich_rules = list_section(
        section_title(fl!("zone-section-rich-rules"), details.rich_rules.len()),
        details
            .rich_rules
            .iter()
            .cloned()
            .map(|rule| (rule.clone(), ZoneViewAction::RemoveRichRule(rule)))
            .collect(),
        fl!("zone-empty-rich-rules"),
        Some((ZoneViewAction::AddRichRule, fl!("action-add-rich-rule"))),
        map,
    );

    let sections = widget::column::with_capacity(10)
        .push(reconciliation_section)
        .push(overview)
        .push(firewalld)
        .push(interfaces)
        .push(sources)
        .push(services)
        .push(ports)
        .push(forward_ports)
        .push(source_ports)
        .push(icmp_blocks)
        .push(rich_rules)
        .spacing(space_m)
        .width(Length::Fill);

    widget::column::with_capacity(2)
        .push(header)
        .push(sections)
        .spacing(space_l)
        .width(Length::Fill)
        .into()
}

fn section_title(title: String, count: usize) -> String {
    format!("{title} ({count})")
}

fn reconciliation_banner<'a, Message: 'static + Clone>(
    presentation: &ReconciliationPresentation<'_>,
    watch_warning: Option<&'a str>,
    last_checked_age_seconds: Option<u64>,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let status = reconciliation_status(presentation.status);
    let actions: cosmic::Element<'_, Message> = if presentation.actions.can_review {
        button::standard(fl!("reconciliation-review"))
            .on_press(map(ZoneViewAction::Reconciliation(
                ReconciliationAction::Review,
            )))
            .into()
    } else {
        button::icon(icon::from_name(REFRESH_ICON))
            .tooltip(fl!("reconciliation-refresh"))
            .extra_small()
            .on_press_maybe(presentation.actions.can_refresh.then_some(map(
                ZoneViewAction::Reconciliation(ReconciliationAction::Refresh),
            )))
            .into()
    };

    let mut section = settings::section()
        .title(fl!("reconciliation-section"))
        .add(
            settings::item::builder(status)
                .description(fl!("reconciliation-banner-description"))
                .control(actions),
        );
    if let Some(seconds) = last_checked_age_seconds {
        section = section.add(widget::text::caption(fl!(
            "reconciliation-last-checked",
            seconds = seconds
        )));
    }
    if let Some(error) = watch_warning {
        section
            .add(
                settings::item::builder(fl!("reconciliation-watch-warning-title")).control(
                    widget::text::body(fl!("reconciliation-watch-warning", error = error)),
                ),
            )
            .into()
    } else {
        section.into()
    }
}

fn reconciliation_status(status: ReconciliationPresentationStatus) -> String {
    match status {
        ReconciliationPresentationStatus::Loading => fl!("reconciliation-status-loading"),
        ReconciliationPresentationStatus::InSync => fl!("reconciliation-status-in-sync"),
        ReconciliationPresentationStatus::Different { count } => {
            fl!("reconciliation-status-different", count = count)
        }
        ReconciliationPresentationStatus::Incomplete {
            known_difference_count,
        } => fl!(
            "reconciliation-status-incomplete",
            count = known_difference_count
        ),
        ReconciliationPresentationStatus::Unavailable => {
            fl!("reconciliation-status-unavailable")
        }
        ReconciliationPresentationStatus::Error => fl!("reconciliation-status-error-short"),
    }
}

fn zone_description<'a, Message: 'static>(
    details: &'a ZoneDetails,
) -> cosmic::Element<'a, Message> {
    if details.description.trim().is_empty() {
        widget::text::caption(fl!("zone-description-fallback")).into()
    } else {
        widget::text::caption(details.description.as_str()).into()
    }
}

fn list_section<'a, Message: 'static + Clone>(
    title: String,
    mut items: Vec<(String, ZoneViewAction)>,
    empty_label: String,
    add_action: Option<(ZoneViewAction, String)>,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    items.sort_by(|a, b| a.0.cmp(&b.0));
    let section = match add_action {
        Some((action, tooltip)) => {
            settings::section().header(section_header(title, action, tooltip, map))
        }
        None => settings::section().title(title),
    };
    if items.is_empty() {
        return section.add(widget::text::caption(empty_label)).into();
    }

    let items_len = items.len();
    let spacing = cosmic::theme::spacing().space_xxs;
    let list = widget::column::with_capacity(items_len)
        .spacing(spacing)
        .width(Length::Fill)
        .extend(
            items
                .into_iter()
                .map(|(label, action)| list_item_row(label, action, map)),
        );

    let list_element: cosmic::Element<'a, Message> = if items_len > MAX_LIST_ITEMS {
        let max_height = LIST_ITEM_HEIGHT * MAX_LIST_ITEMS as f32;
        widget::scrollable::scrollable(list)
            .height(Length::Fixed(max_height))
            .width(Length::Fill)
            .into()
    } else {
        list.into()
    };

    section.add(list_element).into()
}

fn section_header<'a, Message: 'static + Clone>(
    title: String,
    action: ZoneViewAction,
    tooltip: String,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let add = button::icon(icon::from_name(ADD_ICON))
        .tooltip(tooltip)
        .extra_small()
        .on_press(map(action));

    widget::row::with_capacity(2)
        .push(widget::text::heading(title).width(Length::Fill))
        .push(add)
        .align_y(Alignment::Center)
        .into()
}

fn list_item_row<'a, Message: 'static + Clone>(
    label: String,
    action: ZoneViewAction,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let remove = button::icon(icon::from_name(REMOVE_ICON))
        .tooltip(fl!("action-remove"))
        .extra_small()
        .on_press(map(action));

    widget::row::with_capacity(2)
        .push(widget::text::body(label).width(Length::Fill))
        .push(remove)
        .spacing(spacing.space_s)
        .align_y(Alignment::Center)
        .into()
}

fn list_or_none(items: &[String], empty_label: &str) -> String {
    if items.is_empty() {
        empty_label.to_string()
    } else {
        items.join(", ")
    }
}

/// Build the semantic add action shared by the three port sections.
fn port_add_action(kind: PortKind) -> ZoneViewAction {
    ZoneViewAction::AddPort { kind }
}

/// Build source-port rows while retaining the existing removal actions.
fn source_port_rows(source_ports: &[(String, String)]) -> Vec<(String, ZoneViewAction)> {
    source_ports
        .iter()
        .cloned()
        .map(|(port, protocol)| {
            (
                format!("{port}/{protocol}"),
                ZoneViewAction::RemoveSourcePort { port, protocol },
            )
        })
        .collect()
}

fn centered_message<'a, Message: 'static>(
    message: impl Into<String>,
) -> cosmic::Element<'a, Message> {
    let text = widget::text::title2(message.into());
    widget::container(text)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}

fn error_message<'a, Message: 'static + Clone>(
    zone: &str,
    message: &str,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let content = widget::column::with_capacity(3)
        .push(widget::text::title2(fl!("zone-load-error", zone = zone)))
        .push(widget::text::body(message.to_string()))
        .push(
            button::standard(fl!("action-retry"))
                .on_press(map(ZoneViewAction::RetryLoad(zone.to_string()))),
        )
        .spacing(spacing.space_s);

    widget::container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_section_add_actions_select_the_requested_kind() {
        for kind in [
            super::PortKind::Destination,
            super::PortKind::Source,
            super::PortKind::Forward,
        ] {
            assert!(matches!(
                port_add_action(kind),
                ZoneViewAction::AddPort { kind: actual } if actual == kind
            ));
        }
    }

    #[test]
    fn source_port_section_offers_add_and_existing_remove_actions() {
        assert!(matches!(
            port_add_action(super::PortKind::Source),
            ZoneViewAction::AddPort {
                kind: super::PortKind::Source
            }
        ));

        let rows = source_port_rows(&[("1024-2048".into(), "udp".into())]);
        assert!(matches!(
            rows.as_slice(),
            [(
                label,
                ZoneViewAction::RemoveSourcePort { port, protocol }
            )] if label == "1024-2048/udp" && port == "1024-2048" && protocol == "udp"
        ));
    }
}
