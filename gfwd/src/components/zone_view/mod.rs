mod edit_mode;
mod export_mode;
mod port_item;
mod view_mode;

use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::components::zone_view::edit_mode::ZoneEditModeMsg;
use crate::fwd_broker::ZoneSettings;
use crate::components::zone_view::view_mode::{ZoneViewModeMsg, ZoneViewModeOut};
use crate::fwd_broker::FwdBroker;
use edit_mode::ZoneEditMode;
use export_mode::ZoneExportMode;
use view_mode::ZoneViewMode;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ActiveView {
    #[default]
    View,
    Edit,
    Export,
}

#[tracker::track]
pub struct ZoneView {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    // child components
    #[tracker::do_not_track]
    view_mode: Controller<ZoneViewMode>,
    #[tracker::do_not_track]
    edit_mode: Controller<ZoneEditMode>,
    #[tracker::do_not_track]
    export_mode: Controller<ZoneExportMode>,
    // props
    current_zone_name: String,
    active_view: ActiveView,
    firewalld_running: bool,
}

/// Requests that can be sent to the ZoneView component.
#[derive(Debug)]
pub enum ZoneViewRequest {
    SetZoneContent(String),
    SwitchTo(ActiveView),
    ToggleFirewalld,
    UpdateZoneSettings(ZoneSettings),
    RemoveZone,
    SetFirewalldRunning(bool),
    AddPort(String, String),
    RemovePort(String, String),
}

/// Responses that can be emitted from the ZoneView component.
#[derive(Debug)]
pub enum ZoneViewResponse {
    ToggleSidebar,
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(RemoveZoneAction, WindowActionGroup, "example");

#[relm4::component(async, pub)]
impl AsyncComponent for ZoneView {
    type Init = String;
    type Input = ZoneViewRequest;
    type Output = ZoneViewResponse;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                set_css_classes: &["flat"],

                pack_start = &gtk::Box {
                    set_spacing: 6,
                    gtk::Button {
                        set_icon_name: "sidebar-show-symbolic",
                        connect_clicked[sender] => move |_| {
                            sender.output(ZoneViewResponse::ToggleSidebar).unwrap();
                        },
                    },
                    gtk::Button {
                        #[track(model.changed(ZoneView::firewalld_running()))]
                        set_icon_name: if model.firewalld_running { "media-playback-stop-symbolic" } else { "media-playback-start-symbolic" },
                        #[track(model.changed(ZoneView::firewalld_running()))]
                        set_tooltip_text: Some(if model.firewalld_running { "Stop Firewalld" } else { "Start Firewalld" }),
                        connect_clicked[sender] => move |_| {
                            sender.input(ZoneViewRequest::ToggleFirewalld);
                        }
                    }
                },

                pack_end = &gtk::MenuButton {
                    #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
                },

                #[wrap(Some)]
                set_title_widget = &gtk::Box {
                    add_css_class: "linked",
                    #[name = "group"]
                    gtk::ToggleButton {
                        set_label: "View",
                        set_active: model.active_view == ActiveView::View,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(ZoneViewRequest::SwitchTo(ActiveView::View));
                            }
                        },
                    },
                    gtk::ToggleButton {
                        set_label: "Edit",
                        set_group: Some(&group),
                        set_active: model.active_view == ActiveView::Edit,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(ZoneViewRequest::SwitchTo(ActiveView::Edit));
                            }
                        },
                    },
                    gtk::ToggleButton {
                        set_label: "Export",
                        set_group: Some(&group),
                        set_active: model.active_view == ActiveView::Export,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(ZoneViewRequest::SwitchTo(ActiveView::Export));
                            }
                        },
                    },
                }
            },

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 12,
                    set_spacing: 12,

                    // Conditionally show the child components based on the active view
                    match model.active_view {
                        ActiveView::View => model.view_mode.widget().clone(),
                        ActiveView::Edit => model.edit_mode.widget().clone(),
                        ActiveView::Export => model.export_mode.widget().clone(),
                    }
                },
            },
        }
    }

    menu! {
        main_menu: {
            "Delete Zone" => RemoveZoneAction,
        }
    }

    async fn init(
        initial_zone_name: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        // Fetch initial zone settings
        let initial_settings = match broker.get_zone_settings(&initial_zone_name).await {
            Ok(settings) => Some(settings),
            Err(e) => {
                glib::g_log!(LogLevel::Message, "Failed to get initial zone settings: {}", e);
                None
            }
        };

        let view_mode = ZoneViewMode::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                ZoneViewModeOut::AddPort(port, protocol) => ZoneViewRequest::AddPort(port, protocol),
            ZoneViewModeOut::RemovePort(port, protocol) => ZoneViewRequest::RemovePort(port, protocol),
            });

        let edit_mode = ZoneEditMode::builder()
            .launch(())
            .forward(sender.input_sender(), |_| unimplemented!());

        // Only emit settings if they were successfully fetched
        if let Some(settings) = &initial_settings {
            view_mode.emit(ZoneViewModeMsg::SetSettings(settings.clone()));
            edit_mode.emit(ZoneEditModeMsg::SetSettings(settings.clone()));
        }

        let export_mode = ZoneExportMode::builder()
            .launch(initial_zone_name.clone())
            .forward(sender.input_sender(), |_| unimplemented!());

        let initial_firewalld_running = broker.is_firewalld_active().await.unwrap_or(false);

        let model = ZoneView {
            broker,
            view_mode,
            edit_mode,
            export_mode,
            current_zone_name: initial_zone_name,
            firewalld_running: initial_firewalld_running,
            active_view: ActiveView::default(),
            tracker: 0,
        };

        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<RemoveZoneAction>(&["<primary>D"]);

        let remove_zone_action: RelmAction<RemoveZoneAction> = {
            let action = RelmAction::new_stateless(move |_| {
                sender.input(ZoneViewRequest::RemoveZone);
            });
            action.set_enabled(true);
            action
        };

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(remove_zone_action);
        root.insert_action_group("win", Some(&group.into_action_group()));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();
        match msg {
            ZoneViewRequest::AddPort(port, protocol) => {
                // Persist add-port via broker, then refresh settings
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    // Lookup the zone proxy and add port
                    let _ = broker.add_port(zone_name.as_str(), port.as_str(), protocol.as_str()).await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::SetZoneContent(zone_name) => {
                self.set_current_zone_name(zone_name.clone());
                let sender = sender.clone();
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    match broker.get_zone_settings(&zone_name).await {
                        Ok(settings) => {
                            sender.input(ZoneViewRequest::UpdateZoneSettings(settings));
                        }
                        Err(e) => {
                            glib::g_log!(LogLevel::Error, "Failed to update zone content: {e}");
                            // You could also send a message to the UI to show an error
                        }
                    }
                });
            }
            ZoneViewRequest::UpdateZoneSettings(settings) => {
                self.view_mode
                    .emit(ZoneViewModeMsg::SetSettings(settings.clone()));
                self.edit_mode.emit(ZoneEditModeMsg::SetSettings(settings));
            }
            ZoneViewRequest::RemoveZone => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    match broker.remove_zone(zone_name.as_str()).await {
                        Ok(()) => {
                            glib::g_log!(LogLevel::Info, "Stateless action for deleting zone!")
                        }
                        Err(e) => glib::g_log!(LogLevel::Error, "Could not delete zone: {e}"),
                    };
                });
            }
            ZoneViewRequest::SetFirewalldRunning(is_running) => {
                self.set_firewalld_running(is_running);
            }
            ZoneViewRequest::SwitchTo(view) => {
                self.set_active_view(view);
            }
            ZoneViewRequest::ToggleFirewalld => {
                let desired_start = !self.get_firewalld_running();
                let broker = self.broker;
                let sender = sender.clone();
                // Optimistically reflect change; we'll verify after the call.
                self.set_firewalld_running(desired_start);
                relm4::spawn(async move {
                    let result = if desired_start {
                        broker.start_firewalld().await
                    } else {
                        broker.stop_firewalld().await
                    };
                    if let Err(e) = result {
                        glib::g_log!(LogLevel::Error, "Failed to toggle firewalld: {:?}", e);
                    }
                    let is_running = broker.is_firewalld_active().await.unwrap_or(false);
                    let _ = sender.input(ZoneViewRequest::SetFirewalldRunning(is_running));
                });
            }
            ZoneViewRequest::RemovePort(port, protocol) => {
                // Persist remove-port via broker, then refresh settings
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    // Lookup the zone proxy and remove port
                    let _ = broker.remove_port(zone_name.as_str(), port.as_str(), protocol.as_str()).await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
        }
    }
}
