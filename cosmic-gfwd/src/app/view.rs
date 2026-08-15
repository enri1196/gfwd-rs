// SPDX-License-Identifier: MIT

use super::dialogs::{
    DialogKind, DialogMessage, drawer_cancel_footer, drawer_footer_with_submit, drawer_with_error,
    icmp_drawer, interface_drawer, ipset_drawer, port_drawer, rich_rule_drawer, service_drawer,
    source_drawer, zone_drawer,
};
use super::ipsets::view_ipset_content;
use super::navigation::SidebarItem;
use super::reconciliation::reconciliation_drawer;
use super::zones::view_zone_content;
use super::{AppModel, Confirmation, ContextPage, MenuAction, Message, NavMenuAction};
use crate::fl;
use cosmic::app::context_drawer as cosmic_context_drawer;
use cosmic::iced::Length;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::prelude::*;
use cosmic::widget::{self, menu};
use std::collections::HashMap;

/// Render the application's menu bar.
pub(crate) fn header_start(app: &AppModel) -> Vec<Element<'_, Message>> {
    let menu_bar = menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("menu-zones")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![menu::Item::Button(
                    fl!("action-add-zone"),
                    None,
                    MenuAction::AddZone,
                )],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-rules")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(fl!("action-add-port"), None, MenuAction::AddPort),
                    menu::Item::Button(fl!("action-add-rich-rule"), None, MenuAction::AddRichRule),
                    menu::Item::Button(fl!("action-add-icmp"), None, MenuAction::AddIcmp),
                    menu::Item::Button(fl!("action-add-source"), None, MenuAction::AddSource),
                    menu::Item::Button(fl!("action-add-interface"), None, MenuAction::AddInterface),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-objects")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![menu::Item::Button(
                    fl!("action-add-ipset"),
                    None,
                    MenuAction::AddIpSet,
                )],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("menu-help")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        ),
    ]);

    vec![menu_bar.into()]
}

/// Render the context menu for the currently selected navigation item.
pub(crate) fn nav_context_menu(app: &AppModel) -> Option<Vec<menu::Tree<cosmic::Action<Message>>>> {
    let context_id = app.navigation.nav_model().active();

    let Some(item) = app.navigation.item_for_id(context_id) else {
        return Some(Vec::new());
    };

    match item {
        SidebarItem::Zone { .. } => {
            let key_binds = HashMap::new();
            Some(menu::items(
                &key_binds,
                vec![
                    menu::Item::Button(
                        fl!("context-assign-interface"),
                        None,
                        NavMenuAction::AssignInterface(context_id),
                    ),
                    menu::Item::Button(
                        fl!("context-set-default-zone"),
                        None,
                        NavMenuAction::SetDefault(context_id),
                    ),
                    menu::Item::Button(
                        fl!("context-delete-zone"),
                        None,
                        NavMenuAction::Delete(context_id),
                    ),
                ],
            ))
        }
        _ => Some(Vec::new()),
    }
}

/// Render the active context drawer.
pub(crate) fn context_drawer(
    app: &AppModel,
) -> Option<cosmic_context_drawer::ContextDrawer<'_, Message>> {
    if !app.core.window.show_context {
        return None;
    }

    let interface_error = app
        .dialogs
        .interface
        .error
        .as_deref()
        .or(app.catalogs.interfaces.error());
    let can_submit_interface =
        app.dialogs.interface.error.is_none() && !app.dialogs.interface.interface.trim().is_empty();
    let can_submit = !app.mutation_pending();
    let error = app.dialogs.operation_error.as_deref();
    let enabled_services = app
        .zones
        .ready_detail()
        .map(|details| details.services.as_slice())
        .unwrap_or(&[]);
    let blocked_icmp = app
        .zones
        .ready_detail()
        .map(|details| details.icmp_blocks.as_slice())
        .unwrap_or(&[]);
    let descriptor = app.context_page.descriptor();

    let drawer = match app.context_page {
        ContextPage::About => cosmic_context_drawer::about(
            &app.about,
            |url| Message::Navigation(super::navigation::Message::LaunchUrl(url.to_string())),
            Message::Navigation(super::navigation::Message::ToggleContextPage(
                ContextPage::About,
            )),
        ),
        ContextPage::ReviewReconciliation => cosmic_context_drawer::context_drawer(
            reconciliation_drawer(
                app.reconciliation.state(),
                app.mutation_pending(),
                app.dialogs.operation_error.as_deref(),
                app.reconciliation.watch_warning(),
                |action| Message::Zone(super::zones::Message::View(action)),
            ),
            Message::Navigation(super::navigation::Message::ToggleContextPage(
                ContextPage::ReviewReconciliation,
            )),
        ),
        ContextPage::AddZone => cosmic_context_drawer::context_drawer(
            drawer_with_error(zone_drawer(&app.dialogs.zone), error).map(DialogMessage::Zone),
            DialogMessage::Cancel(DialogKind::Zone),
        )
        .map(Message::Dialog),
        ContextPage::AddService => cosmic_context_drawer::context_drawer(
            drawer_with_error(
                service_drawer(
                    &app.dialogs.service,
                    app.catalogs.services.items(),
                    enabled_services,
                    app.catalogs.services.is_loading(),
                    app.catalogs.services.error(),
                ),
                error,
            )
            .map(DialogMessage::Service),
            DialogMessage::Cancel(DialogKind::Service),
        )
        .map(Message::Dialog),
        ContextPage::AddPort => cosmic_context_drawer::context_drawer(
            drawer_with_error(port_drawer(&app.dialogs.port), error).map(DialogMessage::Port),
            DialogMessage::Cancel(DialogKind::Port),
        )
        .map(Message::Dialog),
        ContextPage::AddInterface => cosmic_context_drawer::context_drawer(
            drawer_with_error(
                interface_drawer(
                    &app.dialogs.interface,
                    app.catalogs.interfaces.items(),
                    app.catalogs.interfaces.is_loading(),
                    interface_error,
                ),
                error,
            )
            .map(DialogMessage::Interface),
            DialogMessage::Cancel(DialogKind::Interface),
        )
        .map(Message::Dialog),
        ContextPage::AddSource => cosmic_context_drawer::context_drawer(
            drawer_with_error(source_drawer(&app.dialogs.source), error).map(DialogMessage::Source),
            DialogMessage::Cancel(DialogKind::Source),
        )
        .map(Message::Dialog),
        ContextPage::AddIcmp => cosmic_context_drawer::context_drawer(
            drawer_with_error(
                icmp_drawer(
                    &app.dialogs.icmp,
                    app.catalogs.icmp_types.items(),
                    blocked_icmp,
                    app.catalogs.icmp_types.is_loading(),
                    app.catalogs.icmp_types.error(),
                ),
                error,
            )
            .map(DialogMessage::Icmp),
            DialogMessage::Cancel(DialogKind::Icmp),
        )
        .map(Message::Dialog),
        ContextPage::AddRichRule => cosmic_context_drawer::context_drawer(
            drawer_with_error(rich_rule_drawer(&app.dialogs.rich_rule), error)
                .map(DialogMessage::RichRule),
            DialogMessage::Cancel(DialogKind::RichRule),
        )
        .map(Message::Dialog),
        ContextPage::AddIpSet => cosmic_context_drawer::context_drawer(
            drawer_with_error(ipset_drawer(&app.dialogs.ipset), error).map(DialogMessage::IpSet),
            DialogMessage::Cancel(DialogKind::IpSet),
        )
        .map(Message::Dialog),
    };

    let drawer = match descriptor.title {
        super::ContextTitle::None => drawer,
        title => drawer.title(context_page_title(title, app.dialogs.port.kind)),
    };
    let drawer = match descriptor.footer {
        super::ContextFooter::None => drawer,
        super::ContextFooter::Cancel => drawer.footer(
            drawer_cancel_footer(
                descriptor
                    .dialog
                    .expect("a context page with a footer has a dialog"),
            )
            .map(Message::Dialog),
        ),
        super::ContextFooter::Submit => {
            let kind = descriptor
                .dialog
                .expect("a context page with a footer has a dialog");
            let valid = match kind {
                DialogKind::Zone => !app.dialogs.zone.name.trim().is_empty(),
                DialogKind::Port => app.dialogs.port.is_valid(),
                DialogKind::Interface => can_submit_interface,
                DialogKind::Source => app.dialogs.source.is_valid(),
                DialogKind::RichRule => app.dialogs.rich_rule.generated_rule().is_ok(),
                DialogKind::IpSet => app.dialogs.ipset.is_valid(),
                DialogKind::Service | DialogKind::Icmp => false,
            };
            drawer.footer(drawer_footer_with_submit(kind, can_submit && valid).map(Message::Dialog))
        }
    };

    Some(drawer)
}

/// Render the destructive-operation confirmation dialog.
pub(crate) fn dialog(app: &AppModel) -> Option<Element<'_, Message>> {
    let confirmation = app.operations.confirmation.as_ref()?;
    let (title, body, confirm_label) = match confirmation {
        Confirmation::DeleteZone(zone) => (
            fl!("confirm-delete-zone-title"),
            fl!("confirm-delete-zone-body", zone = zone),
            fl!("confirm-delete"),
        ),
        Confirmation::DeleteIpSet(ipset) => (
            fl!("confirm-delete-ipset-title"),
            fl!("confirm-delete-ipset-body", ipset = ipset),
            fl!("confirm-delete"),
        ),
        Confirmation::StopFirewalld => (
            fl!("confirm-stop-firewalld-title"),
            fl!("confirm-stop-firewalld-body"),
            fl!("firewalld-stop"),
        ),
        Confirmation::ApplyPermanentConfiguration => (
            fl!("confirm-apply-permanent-title"),
            fl!("confirm-apply-permanent-body"),
            fl!("confirm-apply-permanent-action"),
        ),
        Confirmation::SaveRuntimeConfiguration => (
            fl!("confirm-save-runtime-title"),
            fl!("confirm-save-runtime-body"),
            fl!("confirm-save-runtime-action"),
        ),
    };

    Some(
        widget::dialog()
            .title(title)
            .body(body)
            .primary_action(
                widget::button::destructive(confirm_label).on_press(Message::ConfirmDestructive),
            )
            .secondary_action(
                widget::button::text(fl!("dialog-cancel")).on_press(Message::CancelConfirmation),
            )
            .into(),
    )
}

/// Render the pending-operation footer.
pub(crate) fn footer(app: &AppModel) -> Option<Element<'_, Message>> {
    app.operations.pending.as_ref().map(|operation| {
        widget::container(widget::text::caption(fl!(
            "operation-pending",
            operation = operation
        )))
        .padding(cosmic::theme::spacing().space_xs)
        .width(Length::Fill)
        .into()
    })
}

/// Render the active application page.
pub(crate) fn view(app: &AppModel) -> Element<'_, Message> {
    let space_m = cosmic::theme::spacing().space_m;
    let content: Element<_> = match app.navigation.active_item() {
        Some(SidebarItem::IpSets) => {
            view_ipset_content(&app.ipsets, app.mutation_pending(), |action| {
                Message::IpSet(super::ipsets::Message::View(action))
            })
        }
        _ => view_zone_content(
            app.zones.detail(),
            app.zones.firewalld_status(),
            app.reconciliation.state(),
            app.reconciliation.watch_warning(),
            app.mutation_pending(),
            |action| Message::Zone(super::zones::Message::View(action)),
        ),
    };

    widget::toaster(
        &app.operations.toasts,
        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(space_m)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
}

/// Resolve the localized title selected by the shared context-page descriptor.
fn context_page_title(title: super::ContextTitle, port_kind: super::dialogs::PortKind) -> String {
    match title {
        super::ContextTitle::None => unreachable!("title-less pages skip title resolution"),
        super::ContextTitle::Reconciliation => fl!("reconciliation-review-title"),
        super::ContextTitle::Zone => fl!("drawer-title-zone"),
        super::ContextTitle::Service => fl!("dialog-service-title"),
        super::ContextTitle::Port => match port_kind {
            super::dialogs::PortKind::Destination => fl!("drawer-title-destination-port"),
            super::dialogs::PortKind::Source => fl!("drawer-title-source-port"),
            super::dialogs::PortKind::Forward => fl!("drawer-title-forward-port"),
        },
        super::ContextTitle::Interface => fl!("drawer-title-interface"),
        super::ContextTitle::Source => fl!("drawer-title-source"),
        super::ContextTitle::Icmp => fl!("drawer-title-icmp"),
        super::ContextTitle::RichRule => fl!("drawer-title-rich-rule"),
        super::ContextTitle::IpSet => fl!("drawer-title-ipset"),
    }
}
