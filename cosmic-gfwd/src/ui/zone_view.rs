use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, settings};

use crate::models::zone::ZoneDetails;

#[derive(Debug, Clone)]
pub enum ZoneViewState {
    Empty,
    Loading { zone: String },
    Ready(ZoneDetails),
    Error { zone: String, message: String },
}

pub fn view_zone_content<'a, Message: 'static>(
    state: &'a ZoneViewState,
) -> cosmic::Element<'a, Message> {
    let content = match state {
        ZoneViewState::Empty => centered_message("Select a zone to view details"),
        ZoneViewState::Loading { zone } => {
            centered_message(format!("Loading {zone} settings..."))
        }
        ZoneViewState::Error { zone, message } => error_message(zone, message),
        ZoneViewState::Ready(details) => zone_details(details),
    };

    widget::scrollable::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn zone_details<'a, Message: 'static>(
    details: &'a ZoneDetails,
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
        .add(settings::item::builder("Masquerading").control(widget::text(bool_label(
            details.masquerade,
        ))))
        .add(
            settings::item::builder("ICMP Block Inversion")
                .control(widget::text(bool_label(details.icmp_block_inversion))),
        )
        .add(settings::item::builder("Protocols").control(widget::text(
            list_or_none(&details.protocols, "No protocols configured"),
        )));

    let services = list_section(
        "Services",
        details.services.iter().cloned().collect(),
        "No services configured",
    );

    let interfaces = list_section(
        "Interfaces",
        details.interfaces.iter().cloned().collect(),
        "No interfaces assigned",
    );

    let sources = list_section(
        "Sources",
        details.sources.iter().cloned().collect(),
        "All sources allowed",
    );

    let ports = list_section(
        "Ports",
        format_ports(&details.ports),
        "No ports configured",
    );

    let forward_ports = list_section(
        "Forwarded Ports",
        format_forward_ports(&details.forward_ports),
        "No forwarded ports configured",
    );

    let source_ports = list_section(
        "Source Ports",
        format_ports(&details.source_ports),
        "No source ports configured",
    );

    let icmp_blocks = list_section(
        "ICMP Blocks",
        details.icmp_blocks.iter().cloned().collect(),
        "No ICMP blocks configured",
    );

    let rich_rules = list_section(
        "Rich Rules",
        details.rich_rules.iter().cloned().collect(),
        "No rich rules configured",
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

fn zone_description<'a, Message: 'static>(details: &'a ZoneDetails) -> cosmic::Element<'a, Message> {
    if details.description.trim().is_empty() {
        widget::text::caption("Firewall zone configuration").into()
    } else {
        widget::text::caption(details.description.as_str()).into()
    }
}

fn list_section<'a, Message: 'static>(
    title: &'a str,
    mut items: Vec<String>,
    empty_label: &'a str,
) -> cosmic::Element<'a, Message> {
    items.sort();
    let section = settings::section().title(title);
    if items.is_empty() {
        section.add(widget::text::caption(empty_label)).into()
    } else {
        section
            .extend(items.into_iter().map(|item| widget::text::body(item)))
            .into()
    }
}

fn list_or_none(items: &[String], empty_label: &str) -> String {
    if items.is_empty() {
        empty_label.to_string()
    } else {
        items.join(", ")
    }
}

fn format_ports(ports: &[(String, String)]) -> Vec<String> {
    ports
        .iter()
        .map(|(port, protocol)| format!("{}/{}", port, protocol))
        .collect()
}

fn format_forward_ports(forward_ports: &[(String, String, String, String)]) -> Vec<String> {
    forward_ports
        .iter()
        .map(|(port, protocol, to_port, to_addr)| {
            if to_addr.is_empty() {
                format!("{}/{} -> {}", port, protocol, to_port)
            } else {
                format!("{}/{} -> {} ({})", port, protocol, to_port, to_addr)
            }
        })
        .collect()
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "Enabled"
    } else {
        "Disabled"
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

fn error_message<'a, Message: 'static>(
    zone: &str,
    message: &str,
) -> cosmic::Element<'a, Message> {
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
