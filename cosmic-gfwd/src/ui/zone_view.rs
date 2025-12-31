use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, icon, settings};

use crate::fl;
use crate::models::zone::ZoneDetails;

const MAX_LIST_ITEMS: usize = 5;
const LIST_ITEM_HEIGHT: f32 = 28.0;
const REMOVE_ICON: &str = "user-trash-symbolic";

#[derive(Debug, Clone)]
pub enum ZoneViewState {
    Empty,
    Loading { zone: String },
    Ready(ZoneDetails),
    Error { zone: String, message: String },
}

#[derive(Debug, Clone)]
pub enum ZoneViewAction {
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
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let content = match state {
        ZoneViewState::Empty => centered_message("Select a zone to view details"),
        ZoneViewState::Loading { zone } => centered_message(format!("Loading {zone} settings...")),
        ZoneViewState::Error { zone, message } => error_message(zone, message),
        ZoneViewState::Ready(details) => zone_details(details, map),
    };

    widget::scrollable::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn zone_details<'a, Message: 'static + Clone>(
    details: &'a ZoneDetails,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let space_s = spacing.space_s;
    let space_m = spacing.space_m;
    let space_l = spacing.space_l;

    let header = widget::column::with_capacity(2)
        .push(widget::text::title1(details.name.as_str()))
        .push(zone_description(details))
        .spacing(space_s);

    let overview = settings::section()
        .title("Overview")
        .add(settings::item::builder("Target").control(widget::text(details.target.to_string())))
        .add(
            settings::item::builder("Masquerading")
                .control(widget::text(bool_label(details.masquerade))),
        )
        .add(
            settings::item::builder("ICMP Block Inversion")
                .control(widget::text(bool_label(details.icmp_block_inversion))),
        )
        .add(
            settings::item::builder("Protocols").control(widget::text(list_or_none(
                &details.protocols,
                "No protocols configured",
            ))),
        );

    let services = list_section(
        "Services",
        details
            .services
            .iter()
            .cloned()
            .map(|service| (service.clone(), ZoneViewAction::RemoveService(service)))
            .collect(),
        "No services configured",
        map,
    );

    let interfaces = list_section(
        "Interfaces",
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
        "No interfaces assigned",
        map,
    );

    let sources = list_section(
        "Sources",
        details
            .sources
            .iter()
            .cloned()
            .map(|source| (source.clone(), ZoneViewAction::RemoveSource(source)))
            .collect(),
        "All sources allowed",
        map,
    );

    let ports = list_section(
        "Ports",
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
        "No ports configured",
        map,
    );

    let forward_ports = list_section(
        "Forwarded Ports",
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
        "No forwarded ports configured",
        map,
    );

    let source_ports = list_section(
        "Source Ports",
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
        "No source ports configured",
        map,
    );

    let icmp_blocks = list_section(
        "ICMP Blocks",
        details
            .icmp_blocks
            .iter()
            .cloned()
            .map(|icmp| (icmp.clone(), ZoneViewAction::RemoveIcmpBlock(icmp)))
            .collect(),
        "No ICMP blocks configured",
        map,
    );

    let rich_rules = list_section(
        "Rich Rules",
        details
            .rich_rules
            .iter()
            .cloned()
            .map(|rule| (rule.clone(), ZoneViewAction::RemoveRichRule(rule)))
            .collect(),
        "No rich rules configured",
        map,
    );

    let left_column = widget::column::with_capacity(3)
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
        widget::text::caption("Firewall zone configuration").into()
    } else {
        widget::text::caption(details.description.as_str()).into()
    }
}

fn list_section<'a, Message: 'static + Clone>(
    title: &'a str,
    mut items: Vec<(String, ZoneViewAction)>,
    empty_label: &'a str,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    items.sort_by(|a, b| a.0.cmp(&b.0));
    let section = settings::section().title(title);
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

fn bool_label(value: bool) -> &'static str {
    if value { "Enabled" } else { "Disabled" }
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
        .push(widget::text::title2(format!("Failed to load {zone}")))
        .push(widget::text::body(message.to_string()))
        .spacing(spacing.space_s);

    widget::container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}
