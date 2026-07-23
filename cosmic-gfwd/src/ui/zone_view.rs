use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, icon, settings};

use crate::core::FirewalldStatus;
use crate::fl;
use crate::models::zone::ZoneDetails;

const MAX_LIST_ITEMS: usize = 5;
const LIST_ITEM_HEIGHT: f32 = 28.0;
const ADD_ICON: &str = "list-add-symbolic";
const REMOVE_ICON: &str = "user-trash-symbolic";

#[derive(Debug, Clone)]
pub enum ZoneViewState {
    Empty,
    Loading { zone: String },
    Ready(Box<ZoneDetails>),
    Error { zone: String, message: String },
}

#[derive(Debug, Clone)]
pub enum ZoneViewAction {
    /// Permanently enables or disables masquerading.
    SetMasquerade(bool),
    /// Permanently enables or disables ICMP block inversion.
    SetIcmpBlockInversion(bool),
    /// Starts the firewalld systemd unit.
    StartFirewalld,
    /// Requests confirmation before stopping the firewalld systemd unit.
    StopFirewalld,
    /// Reloads firewalld to apply permanent configuration to runtime.
    ApplyPermanentConfiguration,
    /// Opens the configured-service picker for the selected zone.
    AddService,
    AddInterface,
    AddPort {
        forwarding: bool,
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
    mutation_pending: bool,
    runtime_reload_needed: bool,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    match state {
        ZoneViewState::Empty => centered_message(fl!("zone-select-prompt")),
        ZoneViewState::Loading { zone } => centered_message(fl!("zone-loading", zone = zone)),
        ZoneViewState::Error { zone, message } => error_message(zone, message),
        ZoneViewState::Ready(details) => widget::scrollable::scrollable(zone_details(
            details,
            firewalld_status,
            mutation_pending,
            runtime_reload_needed,
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
    mutation_pending: bool,
    runtime_reload_needed: bool,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_m = spacing.space_m;
    let space_l = spacing.space_l;

    let header = widget::column::with_capacity(3)
        .push(widget::text::title1(details.name.as_str()))
        .push(zone_description(details))
        .push(widget::text::caption(if runtime_reload_needed {
            fl!("zone-permanent-pending")
        } else {
            fl!("zone-permanent-notice")
        }))
        .spacing(space_s);

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
        (runtime_reload_needed && !mutation_pending)
            .then_some(map(ZoneViewAction::ApplyPermanentConfiguration)),
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
        fl!("zone-section-services"),
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
        fl!("zone-section-interfaces"),
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
        fl!("zone-section-sources"),
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
        fl!("zone-section-ports"),
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
            ZoneViewAction::AddPort { forwarding: false },
            fl!("action-add-port"),
        )),
        map,
    );

    let forward_ports = list_section(
        fl!("zone-section-forward-ports"),
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
            ZoneViewAction::AddPort { forwarding: true },
            fl!("action-add-forward-port"),
        )),
        map,
    );

    let source_ports = list_section(
        fl!("zone-section-source-ports"),
        details
            .source_ports
            .iter()
            .cloned()
            .map(|(port, protocol)| {
                (
                    format!("{}/{}", port, protocol),
                    ZoneViewAction::RemoveSourcePort { port, protocol },
                )
            })
            .collect(),
        fl!("zone-empty-source-ports"),
        None,
        map,
    );

    let icmp_blocks = list_section(
        fl!("zone-section-icmp"),
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
        fl!("zone-section-rich-rules"),
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

    let left_column = widget::column::with_capacity(3)
        .push(firewalld)
        .push(overview)
        .push(services)
        .push(interfaces)
        .spacing(space_m)
        .width(Length::Fill);

    let right_column = widget::column::with_capacity(5)
        .push(ports)
        .push(forward_ports)
        .push(source_ports)
        .push(sources)
        .push(icmp_blocks)
        .push(rich_rules)
        .spacing(space_m)
        .width(Length::Fill);

    let columns = widget::row::with_capacity(2)
        .push(left_column)
        .push(right_column)
        .spacing(space_l)
        .width(Length::Fill);

    widget::column::with_capacity(2)
        .push(header)
        .push(columns)
        .spacing(space_l)
        .width(Length::Fill)
        .into()
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

fn error_message<'a, Message: 'static>(zone: &str, message: &str) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let content = widget::column::with_capacity(2)
        .push(widget::text::title2(fl!("zone-load-error", zone = zone)))
        .push(widget::text::body(message.to_string()))
        .spacing(spacing.space_s);

    widget::container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}
